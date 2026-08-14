import { useCallback, useEffect, useRef, useState } from 'react';

import i18next from 'i18next';
import {
  type ConnectionState,
  type ConnectionEvent,
  type Device,
  type OfflineReason,
  type ServerMessage,
  type TransportAdapter,
  type TransportAuth,
  connectionReducer,
  nextReconnectDelay,
} from '@dropvoice/core';
import { toast } from '@dropvoice/ui';

import { RtcTransport } from '../lib/rtcTransport';
import { clearPairingCode, clearToken, getPairingCode, getToken, setToken } from '../lib/storage';

/**
 * useConnections —— 多设备 WebRTC 连接管理（§9.3 重写）。
 *
 * 单一真相源：`statuses: Record<deviceId, ConnectionState>` 驱动 UI。所有瞬态
 * 连接状态只活在此 hook + 内存 Session；`Device`（持久身份）不含任何瞬态。
 *
 * 纯 {@link connectionReducer} 计算下一状态；本 hook 持有命令式资源（transport、
 * 重连定时器、auth 元数据）并对 reducer 输出做反应。
 *
 * 关键不变量（见 §9.2/§9.5 原文 + §7 验收"503 快速重试"）：
 * - `retries` 在 CLOSE/FAIL（connecting/reconnecting 来源）时 +1；OPEN 隐式归零。
 * - `FAIL{reason}` **计入退避**（瞬态失败如 §7 的 503 DESKTOP_OFFLINE），耗尽转
 *   offline{reason}。曾实现"立即 offline"（方案 A），实测桌面 SSE 重建窗口内的
 *   503 会把手机永久停车（连接回归），故回归规范原文。
 * - error 后紧随的 close 不双重计数：hook 在 FAIL 后立即 detach 死 transport
 *   （退订处理器），该 close 不再进入 reducer。
 * - auth 类错误触发凭据回退（token → code）；穷尽后 FAIL{auth} 同样计入退避，
 *   耗尽 offline{auth} 提示重新配对。无凭据（结构性）直接 offline{auth}。
 */

/** transport error code 中属于"鉴权失败"的集合（触发凭据回退）。 */
const AUTH_CODES = new Set(['INVALID_CREDENTIAL', 'INVALID_TOKEN', 'UNAUTHORIZED']);

/** transport error code → FAIL 原因映射（§9.5）。返回 null 表示不触发 FAIL。 */
function failReason(code: string): OfflineReason | null {
  switch (code) {
    case 'DESKTOP_OFFLINE':
      return 'offline';
    case 'CONNECTION_TIMEOUT':
      return 'timeout';
    case 'DEVICE_NOT_FOUND':
      return 'unreachable';
    case 'RATE_LIMITED':
      return 'timeout';
    default:
      return null;
  }
}

/** 命令式资源（§9.3 Session 的超集：补 device 供 transport.open 无损复用 + unsubscribers）。 */
interface Session {
  /** 设备身份（connectDevice 传入；transport.open 复用，name 不参与传输）。 */
  device: Device;
  state: ConnectionState;
  transport: TransportAdapter | null;
  /** 发送忙锁（§9.4）：ack/close/新连接释放。 */
  sending: boolean;
  /** 最近一次使用的凭据（重连/回退沿用）。 */
  lastAuth: TransportAuth | null;
  retryTimer: ReturnType<typeof setTimeout> | null;
  unsubscribers: Array<() => void>;
}

/** 模块级纯 helper：清重连定时器。 */
function clearRetryTimer(session: Session): void {
  if (session.retryTimer) {
    clearTimeout(session.retryTimer);
    session.retryTimer = null;
  }
}

/** 模块级纯 helper：取消所有 transport 事件订阅 + 关闭 transport。 */
function closeTransport(session: Session): void {
  session.unsubscribers.forEach((unsub) => unsub());
  session.unsubscribers = [];
  if (session.transport) {
    session.transport.close();
    session.transport = null;
  }
}

export interface UseConnectionsReturn {
  /** 每设备连接状态（驱动 UI）。 */
  statuses: Record<string, ConnectionState>;
  /** 连接到指定设备（幂等：已 connecting/connected 则跳过）。 */
  connectDevice: (device: Device) => void;
  /** 删设备路径：清 session/transport/timer + 清凭据。 */
  disconnectDevice: (deviceId: string) => void;
  /** 手动重试：offline→connecting{0} + 重开 transport。 */
  retryDevice: (deviceId: string) => void;
  /** 发送文本到指定设备（发送忙锁，§9.4）。 */
  sendToDevice: (deviceId: string, text: string) => boolean;
  /** 发送到所有已连接设备，返回投递数。 */
  sendAll: (text: string) => number;
}

export function useConnections(): UseConnectionsReturn {
  const sessionsRef = useRef<Map<string, Session>>(new Map());
  const [statuses, setStatuses] = useState<Record<string, ConnectionState>>({});

  /** 派发事件到某设备的 reducer；返回新状态。引用相等则 no-op（不触发 UI）。 */
  const dispatch = useCallback((deviceId: string, event: ConnectionEvent): ConnectionState => {
    const s = sessionsRef.current.get(deviceId);
    if (!s) return { status: 'idle' };
    const next = connectionReducer(s.state, event);
    if (next === s.state) return next; // no-op（reducer 约定：返回同引用）
    s.state = next;
    setStatuses((prev) => ({ ...prev, [deviceId]: next }));
    return next;
  }, []);

  // openTransport 与 scheduleRetry 互引用，用 ref 打破环；triggerAuthFallback
  // 定义在 scheduleRetry 之前（TDZ），同样经 ref 调用。
  const openTransportRef = useRef<(id: string, auth: TransportAuth) => void>(() => {});
  const scheduleRetryRef = useRef<(id: string, retries: number) => void>(() => {});

  /** 鉴权失败回退：token→code；穷尽则 FAIL{auth} 计入退避，耗尽 offline{auth}。 */
  const triggerAuthFallback = useCallback(
    (id: string) => {
      const s = sessionsRef.current.get(id);
      if (!s) return;
      if (s.lastAuth?.kind === 'token') {
        const code = getPairingCode(id);
        if (code) {
          s.lastAuth = { kind: 'code', code };
          // CONNECT 从 connecting 是 no-op；从 offline 则重置为 connecting{0}。
          dispatch(id, { type: 'CONNECT' });
          openTransportRef.current(id, s.lastAuth);
          return;
        }
      }
      // 已是 code 或无 code：凭据回退穷尽。FAIL{auth} 计入退避（§9.5），
      // 耗尽后 offline{auth} 提示重新配对；期间重试可能等到桌面码轮换后成功。
      const n = dispatch(id, { type: 'FAIL', reason: 'auth' });
      if (n.status === 'reconnecting') {
        scheduleRetryRef.current(id, n.retries);
      }
      closeTransport(s);
    },
    [dispatch]
  );

  /** 指数退避重连（仅在 reducer 转入 reconnecting 时被调用）。通过 ref 调用
   *  openTransport 以打破互引用环（openTransport 反向引用 scheduleRetry）。 */
  const scheduleRetry = useCallback((id: string, retries: number) => {
    const s = sessionsRef.current.get(id);
    if (!s) return;
    clearRetryTimer(s);
    s.retryTimer = setTimeout(() => {
      const cur = sessionsRef.current.get(id);
      if (!cur || !cur.lastAuth) return;
      openTransportRef.current(id, cur.lastAuth);
    }, nextReconnectDelay(retries));
  }, []);

  /** 打开 transport 并接线生命周期事件 → reducer。 */
  const openTransport = useCallback(
    (id: string, auth: TransportAuth) => {
      const s = sessionsRef.current.get(id);
      if (!s) return;
      closeTransport(s);
      dispatch(id, { type: 'CONNECT' });
      const transport = new RtcTransport();
      s.transport = transport;
      s.lastAuth = auth;

      const unsubOpen = transport.on('open', () => {
        const cur = sessionsRef.current.get(id);
        if (!cur || cur.transport !== transport) return; // stale 守卫
        cur.sending = false;
        dispatch(id, { type: 'OPEN' });
      });

      const unsubMessage = transport.on('message', (detail) => {
        const cur = sessionsRef.current.get(id);
        if (!cur || cur.transport !== transport) return;
        const msg = detail?.data as ServerMessage | undefined;
        if (!msg) return;
        if (msg.type === 'ack') {
          cur.sending = false;
        } else if (msg.type === 'token') {
          setToken(id, msg.token);
          cur.lastAuth = { kind: 'token', token: msg.token };
        } else if (msg.type === 'error') {
          if (AUTH_CODES.has(msg.code)) {
            triggerAuthFallback(id);
          } else if (msg.code === 'INJECTION_FAILED') {
            // 注入失败：UI toast，不计入连接状态（不 FAIL）。
            toast.error(i18next.t('errors:injectionFailed'));
          }
        }
      });

      const unsubClose = transport.on('close', () => {
        const cur = sessionsRef.current.get(id);
        if (!cur || cur.transport !== transport) return; // stale 守卫
        cur.sending = false;
        // 主动断开（DISCONNECT→idle）或已删除 → 不重连。
        if (cur.state.status === 'idle') return;
        const n = dispatch(id, { type: 'CLOSE' });
        if (n.status === 'reconnecting') {
          scheduleRetry(id, n.retries);
        }
      });

      const unsubError = transport.on('error', (detail) => {
        const cur = sessionsRef.current.get(id);
        if (!cur || cur.transport !== transport) return; // stale 守卫
        const data = detail?.data;
        const code = data?.type === 'error' ? data.code : undefined;
        if (!code) return; // 无 code 的泛化错误：让 close 走退避
        if (AUTH_CODES.has(code)) {
          triggerAuthFallback(id);
          return;
        }
        const reason = failReason(code);
        if (!reason) return;
        // §7 验收 + §9.5：瞬态失败（503 DESKTOP_OFFLINE 等）计入退避快速重试，
        // 耗尽转 offline{reason}。桌面 SSE 重建窗口内的 503 由此自愈。
        const n = dispatch(id, { type: 'FAIL', reason });
        if (n.status === 'reconnecting') {
          scheduleRetry(id, n.retries);
        }
        // 提前 detach 已死 transport：rtcTransport 失败路径 error 后紧随 close，
        // 退订后该 close 不再进入 reducer，避免退避双重计数。
        closeTransport(cur);
      });

      s.unsubscribers = [unsubOpen, unsubMessage, unsubClose, unsubError];
      transport.open(s.device, auth);
    },
    [dispatch, scheduleRetry, triggerAuthFallback]
  );

  // 打破 openTransport↔scheduleRetry 互引用环（及 triggerAuthFallback 的前向引用）。
  useEffect(() => {
    openTransportRef.current = openTransport;
    scheduleRetryRef.current = scheduleRetry;
  }, [openTransport, scheduleRetry]);

  const connectDevice = useCallback(
    (device: Device) => {
      const id = device.id;
      const existing = sessionsRef.current.get(id);
      // 幂等：已 connecting/connected 则跳过（防重复）。
      if (
        existing &&
        (existing.state.status === 'connecting' || existing.state.status === 'connected')
      ) {
        return;
      }
      if (existing) {
        clearRetryTimer(existing);
        closeTransport(existing);
      }
      sessionsRef.current.set(id, {
        device,
        state: { status: 'connecting', retries: 0 },
        transport: null,
        sending: false,
        lastAuth: null,
        retryTimer: null,
        unsubscribers: [],
      });
      setStatuses((prev) => ({ ...prev, [id]: { status: 'connecting', retries: 0 } }));

      // 凭据选择：优先 token，回退 code。
      const token = getToken(id);
      const code = getPairingCode(id);
      const auth: TransportAuth | null = token
        ? { kind: 'token', token }
        : code
          ? { kind: 'code', code }
          : null;
      if (!auth) {
        // 无凭据是结构性失败（非瞬态）：直接落 offline{auth}，UI 提示重新配对。
        // 不 dispatch FAIL——FAIL 语义计入退避，而无凭据重试无意义。
        const fresh = sessionsRef.current.get(id);
        if (fresh) {
          fresh.state = { status: 'offline', reason: 'auth' };
          setStatuses((prev) => ({ ...prev, [id]: fresh.state }));
        }
        return;
      }
      openTransport(id, auth);
    },
    [dispatch, openTransport]
  );

  const disconnectDevice = useCallback((deviceId: string) => {
    const s = sessionsRef.current.get(deviceId);
    if (!s) return;
    clearRetryTimer(s);
    s.transport?.close();
    sessionsRef.current.delete(deviceId);
    setStatuses((prev) => {
      const next = { ...prev };
      delete next[deviceId];
      return next;
    });
    // §9.5：删设备清除凭据（首次接线）。
    clearToken(deviceId);
    clearPairingCode(deviceId);
  }, []);

  const retryDevice = useCallback(
    (deviceId: string) => {
      const s = sessionsRef.current.get(deviceId);
      if (!s) return;
      clearRetryTimer(s);
      dispatch(deviceId, { type: 'CONNECT' }); // offline→connecting{0}
      const auth: TransportAuth | null =
        s.lastAuth ??
        (getToken(deviceId)
          ? { kind: 'token', token: getToken(deviceId) as string }
          : getPairingCode(deviceId)
            ? { kind: 'code', code: getPairingCode(deviceId) as string }
            : null);
      if (auth) openTransport(deviceId, auth);
      else dispatch(deviceId, { type: 'FAIL', reason: 'auth' });
    },
    [dispatch, openTransport]
  );

  const sendToDevice = useCallback((deviceId: string, text: string): boolean => {
    const s = sessionsRef.current.get(deviceId);
    if (!s || !s.transport?.isOpen || s.state.status !== 'connected' || s.sending) return false;
    s.sending = true;
    return s.transport.send(text);
  }, []);

  const sendAll = useCallback((text: string): number => {
    let count = 0;
    for (const [, s] of sessionsRef.current.entries()) {
      if (s.transport?.isOpen && s.state.status === 'connected' && !s.sending) {
        s.sending = true;
        if (s.transport.send(text)) count += 1;
      }
    }
    return count;
  }, []);

  // visibilitychange → 前台时立即重试所有 reconnecting/offline 设备（不等退避）。
  useEffect(() => {
    const onVisibility = () => {
      if (document.visibilityState !== 'visible') return;
      for (const [id, s] of sessionsRef.current.entries()) {
        if (s.state.status === 'reconnecting' || s.state.status === 'offline') {
          retryDevice(id);
        }
      }
    };
    document.addEventListener('visibilitychange', onVisibility);
    return () => document.removeEventListener('visibilitychange', onVisibility);
  }, [retryDevice]);

  // 卸载时关闭所有 session。
  useEffect(() => {
    return () => {
      for (const s of sessionsRef.current.values()) {
        clearRetryTimer(s);
        closeTransport(s);
      }
      sessionsRef.current.clear();
    };
  }, []);

  return {
    statuses,
    connectDevice,
    disconnectDevice,
    retryDevice,
    sendToDevice,
    sendAll,
  };
}
