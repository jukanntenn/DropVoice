/**
 * RtcTransport —— WebRTC DataChannel 传输实现（webrtc-scan-direct-design §6.2）。
 *
 * 职责：
 * 1. 信令客户端：POST offer(带 code/token) → 长轮询 answer(30s 超时)
 * 2. RTCPeerConnection({iceServers:[]}) + 非 trickle ICE（§5.5）
 * 3. DataChannel(label dropvoice, ordered) 收发消息（§6.3 协议）
 * 4. 断线检测：iceconnectionstatechange → disconnected/failed → 触发重连
 */

import type {
  Device,
  ServerMessage,
  TransportAdapter,
  TransportAuth,
  TransportEvent,
  TransportEventHandler,
} from '@dropvoice/core';

/** 信令服务器 API 基础 URL。 */
function apiBaseUrl(): string {
  // dev: Vite proxy /api → http://localhost:38424，PWA 同源。
  // prod: VITE_API_BASE_URL 注入（同域 https://dropvoice.bytehome.fun）。
  const env = import.meta.env.VITE_API_BASE_URL as string | undefined;
  return env && env.length > 0 ? env : '';
}

/** 长轮询 hold 时长（§4.1 30s）。 */
const LONG_POLL_TIMEOUT = 30_000;

/** ICE 收集超时 fallback（§5.5 2s）。 */
const ICE_GATHERING_TIMEOUT = 2000;

/** offer 请求体（§4.1）。 */
interface OfferRequestBody {
  code?: string;
  token?: string;
  sdp: string;
}

/** offer 响应体（§4.1）。 */
interface OfferResponseBody {
  session_id: string;
  device: { id: string; name?: string };
}

/** answer 响应体（§4.1）。 */
interface AnswerResponseBody {
  sdp?: string;
  status: 'accepted' | 'rejected';
  reason?: string;
}

/**
 * RtcTransport：实现 TransportAdapter，用 WebRTC DataChannel。
 *
 * 生命周期：
 * - open() → 创建 RTCPeerConnection + offer → POST offer → 长轮询 answer
 * - answer accepted → setRemoteDescription → DataChannel open → on('open')
 * - DataChannel message → on('message')
 * - close/error → on('close')/on('error')
 */
export class RtcTransport implements TransportAdapter {
  private pc: RTCPeerConnection | null = null;
  private channel: RTCDataChannel | null = null;
  private device: Device | null = null;
  private auth: TransportAuth | null = null;
  private pollAbort: AbortController | null = null;
  private handlers: Map<TransportEvent, Set<TransportEventHandler>> = new Map();

  get isOpen(): boolean {
    return this.channel?.readyState === 'open';
  }

  open(device: Device, auth: TransportAuth): void {
    this.device = device;
    this.auth = auth;
    this.connect().catch((err) => {
      console.error('[rtc] open failed', err);
      this.emit('error');
      this.emit('close');
    });
  }

  send(text: string): boolean {
    if (!this.isOpen || !this.channel) return false;
    try {
      this.channel.send(JSON.stringify({ type: 'text', text }));
      return true;
    } catch (err) {
      console.error('[rtc] send failed', err);
      return false;
    }
  }

  close(): void {
    this.pollAbort?.abort();
    if (this.channel) {
      try {
        this.channel.close();
      } catch {
        // ignore
      }
      this.channel = null;
    }
    if (this.pc) {
      try {
        this.pc.close();
      } catch {
        // ignore
      }
      this.pc = null;
    }
  }

  on(event: TransportEvent, handler: TransportEventHandler): () => void {
    let set = this.handlers.get(event);
    if (!set) {
      set = new Set();
      this.handlers.set(event, set);
    }
    set.add(handler);
    return () => set?.delete(handler);
  }

  private emit(event: TransportEvent, detail?: { data?: ServerMessage }): void {
    this.handlers.get(event)?.forEach((h) => h(detail));
  }

  /** 完整信令 + ICE 协商流程。 */
  private async connect(): Promise<void> {
    if (!this.device || !this.auth) return;

    // 解析 device.id（device_id 从 QR 载荷提取，见 AddDeviceModal）。
    const deviceId = this.device.id;
    const baseUrl = apiBaseUrl();

    // §5.5 空 iceServers。
    const pc = new RTCPeerConnection({ iceServers: [], iceCandidatePoolSize: 1 });
    this.pc = pc;

    // 手机是 offerer：必须在本侧 createDataChannel，m=application 行才会出现在
    // offer SDP 中（answer 不能新增 m-line——此前桌面侧创建导致 offer/answer
    // 都无数据通道行，DataChannel 永不建立，实测"连接中"卡死）。
    // 桌面侧（answerer）通过 ondatachannel 收到本通道。
    const channel = pc.createDataChannel('dropvoice', { ordered: true });
    this.channel = channel;
    this.wireChannel(channel);
    // 兼容兜底：若未来桌面侧也在 answer 里建通道，仍能收到。
    pc.addEventListener('datachannel', (e) => {
      this.channel = e.channel;
      this.wireChannel(e.channel);
    });

    // ICE 失败检测（§7.2）。
    pc.addEventListener('iceconnectionstatechange', () => {
      if (pc.iceConnectionState === 'failed' || pc.iceConnectionState === 'disconnected') {
        this.emit('close');
      }
    });

    // 生成 offer SDP（等 ICE complete，§5.5 非 trickle）。
    // 超时兜底：PC 在 createOffer in-flight 时被 close()，Chromium 的 promise
    // 可能永不 settle（实测复现），用 10s 上限让连接失败而非永久卡"连接中"。
    const offer = await this.withTimeout(pc.createOffer(), 10_000, 'createOffer');
    await this.withTimeout(pc.setLocalDescription(offer), 10_000, 'setLocalDescription');
    const offerSdp = await this.waitForIceComplete(pc);

    // POST offer（带 code 或 token，§4.1）。
    const body: OfferRequestBody = { sdp: offerSdp };
    if (this.auth.kind === 'code') {
      body.code = this.auth.code;
    } else {
      body.token = this.auth.token;
    }

    const offerResp = await fetch(`${baseUrl}/api/devices/${deviceId}/webrtc/offer`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    });

    // HTTP 错误状态 → 具名 error code（§7 含 503 DESKTOP_OFFLINE fast-fail）。
    const errorCode = offerStatusToErrorCode(offerResp.status);
    if (errorCode) {
      this.emit('error', { data: { type: 'error', code: errorCode } });
      this.emit('close');
      return;
    }
    if (!offerResp.ok) {
      this.emit('error');
      this.emit('close');
      return;
    }

    const offerResult = (await offerResp.json()) as OfferResponseBody;

    // 长轮询 answer（§4.1，hold 30s）。
    const answer = await this.pollAnswer(deviceId, offerResult.session_id);
    if (!answer) {
      // 30s 超时无 answer（桌面离线或 SSE 未就绪，§7.1）。
      this.emit('error', { data: { type: 'error', code: 'CONNECTION_TIMEOUT' } });
      this.emit('close');
      return;
    }
    if (answer.status === 'rejected') {
      // credential 错误（§3.2）。
      this.emit('error', {
        data: { type: 'error', code: 'INVALID_CREDENTIAL', message: answer.reason },
      });
      this.emit('close');
      return;
    }
    if (!answer.sdp) {
      this.emit('error');
      this.emit('close');
      return;
    }

    // setRemoteDescription(answer) → ICE 协商完成 → DataChannel open。
    await pc.setRemoteDescription({ type: 'answer', sdp: answer.sdp });
    // DataChannel open 由 wireChannel 中的 onopen 触发 emit('open')。
  }

  /** 长轮询 answer，hold 30s。返回 null 表示超时。 */
  private async pollAnswer(
    deviceId: string,
    sessionId: string
  ): Promise<AnswerResponseBody | null> {
    this.pollAbort = new AbortController();
    const baseUrl = apiBaseUrl();
    const deadline = Date.now() + LONG_POLL_TIMEOUT;

    while (Date.now() < deadline) {
      try {
        const resp = await fetch(`${baseUrl}/api/devices/${deviceId}/webrtc/answer/${sessionId}`, {
          signal: this.pollAbort.signal,
        });
        if (resp.status === 204) {
          // answer 未就绪，继续轮询。
          continue;
        }
        if (resp.status === 404) {
          // session 过期。
          return null;
        }
        if (resp.ok) {
          return (await resp.json()) as AnswerResponseBody;
        }
      } catch (err) {
        if ((err as Error).name === 'AbortError') return null;
        // 网络错误，短暂等待后重试。
        await new Promise((r) => setTimeout(r, 1000));
      }
    }
    return null;
  }

  /** 超时包装：PC 被 close 时 Chromium 的 WebRTC promise 可能永不 settle。 */
  private withTimeout<T>(promise: Promise<T>, ms: number, what: string): Promise<T> {
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error(`${what} timed out after ${ms}ms`)), ms);
      promise.then(
        (v) => {
          clearTimeout(timer);
          resolve(v);
        },
        (e) => {
          clearTimeout(timer);
          reject(e);
        }
      );
    });
  }

  /** 等 ICE complete（2s 超时 fallback，§5.5）。 */
  private waitForIceComplete(pc: RTCPeerConnection): Promise<string> {
    return new Promise((resolve) => {
      if (pc.iceGatheringState === 'complete') {
        resolve(pc.localDescription?.sdp ?? '');
        return;
      }
      let done = false;
      const finish = () => {
        if (done) return;
        done = true;
        resolve(pc.localDescription?.sdp ?? '');
      };
      pc.addEventListener('icegatheringstatechange', () => {
        if (pc.iceGatheringState === 'complete') finish();
      });
      setTimeout(finish, ICE_GATHERING_TIMEOUT);
    });
  }

  /** 绑定 DataChannel 事件。 */
  private wireChannel(channel: RTCDataChannel): void {
    channel.addEventListener('open', () => {
      this.emit('open');
    });

    channel.addEventListener('message', (e: MessageEvent) => {
      try {
        const msg = JSON.parse(e.data as string) as ServerMessage;
        this.emit('message', { data: msg });
      } catch (err) {
        console.error('[rtc] message parse failed', err);
      }
    });

    channel.addEventListener('close', () => {
      this.emit('close');
    });

    channel.addEventListener('error', (e) => {
      console.error('[rtc] channel error', e);
      this.emit('error');
    });
  }
}

/** QR 载荷解析（§6.5：dropvoice://pair?code=&device=&name=）。 */
export interface ParsedQrPayload {
  code: string;
  device: string;
  name?: string;
}

/**
 * POST offer 响应状态 → 具名 error code 映射（§7 含 503 DESKTOP_OFFLINE）。
 * 返回 null 表示该状态不映射到具名 code（2xx 成功，或交给泛化错误处理）。
 * 抽成纯函数便于单测（无需 mock WebRTC/fetch）。
 */
export function offerStatusToErrorCode(status: number): string | null {
  switch (status) {
    case 404:
      return 'DEVICE_NOT_FOUND';
    case 429:
      return 'RATE_LIMITED';
    case 503:
      return 'DESKTOP_OFFLINE';
    default:
      return null;
  }
}

/** 解析 QR 载荷。返回 null 表示格式无效。 */
export function parseQrPayload(payload: string): ParsedQrPayload | null {
  try {
    // dropvoice:// 是自定义 scheme，URL 能解析。
    const url = new URL(payload);
    if (url.protocol !== 'dropvoice:') return null;
    const code = url.searchParams.get('code');
    const device = url.searchParams.get('device');
    const name = url.searchParams.get('name');
    if (!code || !device) return null;
    return { code, device, name: name ?? undefined };
  } catch {
    return null;
  }
}
