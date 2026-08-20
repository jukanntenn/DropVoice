/**
 * WebRTC 信令客户端（桌面 webview JS，webrtc-scan-direct-design §5.1）。
 *
 * 职责：
 * 1. SSE 长连接（GET /api/devices/{id}/webrtc/events?token=）接收手机 offer
 * 2. 收到 offer → invoke('validate_credential') → 生成 answer → POST answer
 * 3. pairing_token 过期（SSE 401）→ invoke('refresh_pairing_token') → 重建 SSE
 *
 * 与 RTCPeerConnection 同一 JS 上下文（§5.1 理由 1）。
 */

import { listen, type UnlistenFn } from '@tauri-apps/api/event';

import { tauriInvoke } from './invoke';

/** SSE offer 事件 data 格式（§6.4）。 */
interface SseOfferEvent {
  session_id: string;
  credential: string;
  sdp: string;
}

/** offer 处理回调（由 webrtc.ts 注册：生成 answer，建立 DataChannel）。 */
type OfferHandler = (event: SseOfferEvent) => Promise<{ accepted: boolean; sdp?: string }>;

/** 每个 EventSource 的绝对回收定时器（45s 主动重建，避开中间层掐断窗口）。 */
const recycleTimers = new WeakMap<EventSource, ReturnType<typeof setTimeout>>();

/**
 * 信令客户端。维持 SSE 长连接，收到 offer 调用 handler，回填 answer。
 *
 * 用法：
 * ```ts
 * const client = new SignalingClient(deviceId);
 * client.onOffer = async (event) => { ... return { accepted: true, sdp } };
 * await client.start(initialToken);
 * ```
 */
export class SignalingClient {
  private deviceId: string;
  private eventSource: EventSource | null = null;
  private token: string;
  /** 信令服务器 base URL（start() 时从 Rust 解析结果获取，无构建时默认值）。 */
  private baseUrl = '';
  onOffer: OfferHandler | null = null;
  private pairingTokenUnlisten: UnlistenFn | null = null;
  private stopped = false;
  private lastEventAt = 0;
  private watchdog: ReturnType<typeof setInterval> | null = null;

  constructor(deviceId: string) {
    this.deviceId = deviceId;
    this.token = '';
  }

  /** 启动 SSE 长连接。token 由 Rust heartbeat emit 提供。 */
  async start(initialToken: string): Promise<void> {
    this.token = initialToken;

    // URL 唯一真源在 Rust（env PAIRING_SERVER_URL → config.toml
    // network.pairing_server_url），运行时经 get_signaling_url 下发 ——
    // 心跳注册与 SSE 永远指向同一台服务器。
    this.baseUrl = (await tauriInvoke.getSignalingUrl()).replace(/\/+$/, '');

    // 看门狗必须最先武装（不依赖 listen 是否 resolve）：
    // 90s 无任何事件（keepalive/ping 15s 一次）→ 连接已被中间层静默掐断
    // （实测：隧道掐断时 EventSource 可能收不到 error（半开连接）也不自动重连，
    // 且此前 await listen() 未完成导致看门狗从未装上 → SSE 永久死亡）→ 强制重建。
    this.lastEventAt = Date.now();
    if (this.watchdog) window.clearInterval(this.watchdog);
    this.watchdog = window.setInterval(() => {
      if (this.stopped) return;
      if (Date.now() - this.lastEventAt > 90_000) {
        console.warn('[signaling] SSE silent for 90s, forcing reconnect');
        this.reconnect();
      }
    }, 30_000);

    this.connectSse();

    // 监听 Rust heartbeat 的 pairing_token 事件（§5.1 新增链路）。
    // 收到新 token（注册/续期）→ 重建 SSE。
    this.pairingTokenUnlisten = await listen<string>('pairing_token', (event) => {
      const newToken = event.payload;
      if (newToken && newToken !== this.token) {
        console.debug('[signaling] pairing_token refreshed, reconnecting SSE');
        this.token = newToken;
        this.reconnect();
      }
    });
  }

  /** 停止 SSE + 移除监听。 */
  stop(): void {
    this.stopped = true;
    if (this.watchdog) {
      window.clearInterval(this.watchdog);
      this.watchdog = null;
    }
    if (this.eventSource) {
      const timer = recycleTimers.get(this.eventSource);
      if (timer !== undefined) window.clearTimeout(timer);
      recycleTimers.delete(this.eventSource);
      this.eventSource.close();
      this.eventSource = null;
    }
    if (this.pairingTokenUnlisten) {
      this.pairingTokenUnlisten();
      this.pairingTokenUnlisten = null;
    }
  }

  /** 建立/重建 SSE 连接。 */
  private connectSse(): void {
    if (this.stopped) return;
    const url = `${this.baseUrl}/api/devices/${this.deviceId}/webrtc/events?token=${encodeURIComponent(this.token)}`;
    const es = new EventSource(url);
    this.eventSource = es;
    this.lastEventAt = Date.now();

    // 连接级绝对回收：中间层（内网隧道实测 27~300s 不等）会在任意时刻掐断连接
    // 且不触发 error（半开假象，EventSource 不重连）。45s 主动重建，永远赶在
    // 隧道掐死之前。订阅由服务端 insert 覆盖，重建无缝。
    recycleTimers.set(
      es,
      window.setTimeout(() => {
        if (this.stopped || this.eventSource !== es) return;
        console.debug('[signaling] SSE recycled (45s lifetime)');
        this.reconnect();
      }, 45_000)
    );
    es.addEventListener('open', () => {
      this.lastEventAt = Date.now();
    });

    es.addEventListener('offer', (e: MessageEvent) => {
      this.lastEventAt = Date.now();
      this.handleOfferEvent(e).catch((err) => {
        console.error('[signaling] offer handler error', err);
      });
    });

    es.addEventListener('ping', () => {
      // 15s 具名 ping 事件（§4.6），连接活性监测。无需处理。
      this.lastEventAt = Date.now();
    });

    es.addEventListener('error', () => {
      // EventSource 自动重连（浏览器原生）。但 401（token 过期）会关闭连接。
      // 检测 readyState === CLOSED 表示认证失败。
      if (es.readyState === EventSource.CLOSED) {
        console.warn('[signaling] SSE closed (likely 401), refreshing pairing_token');
        this.handleAuthFailure().catch((err) => {
          console.error('[signaling] token refresh failed', err);
        });
      }
    });
  }

  /** 重建 SSE（token 刷新后 / 回收 / 看门狗）。 */
  private reconnect(): void {
    if (this.eventSource) {
      const timer = recycleTimers.get(this.eventSource);
      if (timer !== undefined) window.clearTimeout(timer);
      recycleTimers.delete(this.eventSource);
      this.eventSource.close();
      this.eventSource = null;
    }
    this.connectSse();
  }

  /** 处理 offer 事件：校验 credential → 生成 answer → POST answer。 */
  private async handleOfferEvent(e: MessageEvent): Promise<void> {
    const event = JSON.parse(e.data as string) as SseOfferEvent;
    if (!this.onOffer) {
      console.warn('[signaling] offer received but no handler registered');
      return;
    }

    const result = await this.onOffer(event);

    // POST answer 回填（§4.1 端点 4）。
    const body =
      result.accepted && result.sdp
        ? { session_id: event.session_id, sdp: result.sdp, status: 'accepted' }
        : { session_id: event.session_id, status: 'rejected', reason: 'invalid_credential' };

    await fetch(`${this.baseUrl}/api/devices/${this.deviceId}/webrtc/answer`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${this.token}`,
      },
      body: JSON.stringify(body),
    });
  }

  /** SSE 401 → refresh pairing_token → 重连。 */
  private async handleAuthFailure(): Promise<void> {
    try {
      const newToken = await tauriInvoke.refreshPairingToken();
      if (newToken) {
        this.token = newToken;
        this.reconnect();
      }
    } catch (err) {
      console.error('[signaling] refreshPairingToken failed', err);
    }
  }
}
