import { act, renderHook } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { ServerMessage, TransportEvent, TransportEventHandler } from '@dropvoice/core';

// ── FakeTransport：可控的 TransportAdapter，替代真实 RtcTransport（无 WebRTC）────
interface FakeTransport {
  handlers: Map<TransportEvent, Set<TransportEventHandler>>;
  openState: boolean;
  sent: string[];
  openArgs: { device: unknown; auth: unknown };
  on(event: TransportEvent, handler: TransportEventHandler): () => void;
  emit(event: TransportEvent, detail?: { data?: ServerMessage }): void;
  send(text: string): boolean;
  close(): void;
  open(device: unknown, auth: unknown): void;
  readonly isOpen: boolean;
  // 测试驱动 helper：
  simulateOpen(): void;
  simulateClose(): void;
  simulateError(code: string): void;
  simulateMessage(msg: ServerMessage): void;
}

// vi.hoisted 让 mock 工厂与测试共享 instances 引用。
const { instances, resetInstances } = vi.hoisted(() => ({
  instances: [] as FakeTransport[],
  resetInstances: () => {
    instances.length = 0;
  },
}));

vi.mock('../lib/rtcTransport', () => {
  class FakeTransportImpl {
    handlers = new Map();
    openState = false;
    sent: string[] = [];
    openArgs: { device: unknown; auth: unknown } = { device: null, auth: null };

    constructor() {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      instances.push(this as any);
    }

    on(event: TransportEvent, handler: TransportEventHandler) {
      let set = this.handlers.get(event);
      if (!set) {
        set = new Set();
        this.handlers.set(event, set);
      }
      set.add(handler);
      return () => set.delete(handler);
    }
    emit(event: TransportEvent, detail?: { data?: ServerMessage }) {
      this.handlers.get(event)?.forEach((h: TransportEventHandler) => h(detail));
    }
    send(text: string) {
      this.sent.push(text);
      return true;
    }
    close() {
      this.openState = false;
    }
    open(device: unknown, auth: unknown) {
      this.openArgs = { device, auth };
      this.openState = false; // 不自动 open；测试主动 simulateOpen
    }
    get isOpen() {
      return this.openState;
    }
    simulateOpen() {
      this.openState = true;
      this.emit('open');
    }
    simulateClose() {
      this.openState = false;
      this.emit('close');
    }
    simulateError(code: string) {
      this.emit('error', { data: { type: 'error', code } });
    }
    simulateMessage(msg: ServerMessage) {
      this.emit('message', { data: msg });
    }
  }
  return { RtcTransport: FakeTransportImpl };
});

import { useConnections } from './useConnections';
import { getPairingCode, getToken, setPairingCode, setToken } from '../lib/storage';

const DEVICE = { id: 'dev-1', name: 'PC', autoConnect: true };

/** 取最新创建的 FakeTransport（hook 内部 new 出来的）。 */
function lastTransport(): FakeTransport {
  const t = instances[instances.length - 1];
  if (!t) throw new Error('no transport created yet');
  return t;
}

describe('useConnections (§9 重写)', () => {
  beforeEach(() => {
    resetInstances();
    window.localStorage.clear();
  });

  it('connectDevice(token) → transport open → open 事件 → connected', () => {
    setToken('dev-1', 'dvct_tok');
    const { result } = renderHook(() => useConnections());
    act(() => result.current.connectDevice(DEVICE));
    expect(result.current.statuses['dev-1']?.status).toBe('connecting');
    const t = lastTransport();
    expect(t.openArgs.auth).toEqual({ kind: 'token', token: 'dvct_tok' });
    act(() => t.simulateOpen());
    expect(result.current.statuses['dev-1']).toEqual({ status: 'connected' });
  });

  it('无凭据（结构性失败）→ 直接 offline{auth}（不走退避，提示重新配对）', () => {
    const { result } = renderHook(() => useConnections());
    act(() => result.current.connectDevice(DEVICE));
    expect(result.current.statuses['dev-1']).toEqual({ status: 'offline', reason: 'auth' });
  });

  it('DESKTOP_OFFLINE error（§7 503）→ FAIL → 计入退避 reconnecting（自愈回归用例）', () => {
    setToken('dev-1', 'dvct_tok');
    const { result } = renderHook(() => useConnections());
    act(() => result.current.connectDevice(DEVICE));
    expect(result.current.statuses['dev-1']?.status).toBe('connecting');
    const t = lastTransport();
    // 桌面 SSE 重建窗口内 503：error 紧随 close（rtcTransport 真实行为）。
    act(() => t.simulateError('DESKTOP_OFFLINE'));
    // §7 验收"快速重试"：不停车在 offline，而是进入退避（r+1）。
    expect(result.current.statuses['dev-1']).toEqual({ status: 'reconnecting', retries: 1 });
    // error 处理器已 detach 该 transport：紧随的 close 不进 reducer（不双重计数）。
    act(() => t.simulateClose());
    expect(result.current.statuses['dev-1']).toEqual({ status: 'reconnecting', retries: 1 });
    // 退避重试（手动驱动，替代真实定时器）：CONNECT 保留 r → connecting{1}。
    act(() => result.current.retryDevice('dev-1'));
    expect(result.current.statuses['dev-1']).toEqual({ status: 'connecting', retries: 1 });
    // 桌面 SSE 恢复 → open → connected（自愈）。
    const t2 = lastTransport();
    act(() => t2.simulateOpen());
    expect(result.current.statuses['dev-1']).toEqual({ status: 'connected' });
    // 清理退避定时器，避免测试结束后触发。
    act(() => result.current.disconnectDevice('dev-1'));
  });

  /** 连续失败 n 轮（每轮：手动重试新建 transport + error），驱动退避计数直至耗尽。
   *  必须每轮先 retryDevice——error 处理会 detach 旧 transport，直接对旧实例
   *  simulateError 不会进 reducer。 */
  function exhaustRetries(
    result: { current: ReturnType<typeof useConnections> },
    code: string,
    rounds: number
  ): void {
    for (let i = 0; i < rounds; i++) {
      act(() => result.current.retryDevice('dev-1'));
      act(() => lastTransport().simulateError(code));
    }
  }

  it('auth 回退：token 被拒 + 有 code → 改用 code 重连 → open → connected', () => {
    setToken('dev-1', 'dvct_tok');
    setPairingCode('dev-1', '123456');
    const { result } = renderHook(() => useConnections());
    act(() => result.current.connectDevice(DEVICE));
    const tokenTransport = lastTransport();
    expect(tokenTransport.openArgs.auth).toEqual({ kind: 'token', token: 'dvct_tok' });

    // token 被拒（INVALID_CREDENTIAL）→ 回退到 code（新建 transport）。
    act(() => tokenTransport.simulateError('INVALID_CREDENTIAL'));
    const codeTransport = lastTransport();
    expect(codeTransport).not.toBe(tokenTransport);
    expect(codeTransport.openArgs.auth).toEqual({ kind: 'code', code: '123456' });
    act(() => codeTransport.simulateOpen());
    expect(result.current.statuses['dev-1']).toEqual({ status: 'connected' });
  });

  it('auth 回退穷尽（已是 code 仍被拒）→ 退避重试，耗尽 → offline{auth}', () => {
    setPairingCode('dev-1', '123456'); // 仅 code，无 token
    const { result } = renderHook(() => useConnections());
    act(() => result.current.connectDevice(DEVICE));
    const t = lastTransport();
    expect(t.openArgs.auth).toEqual({ kind: 'code', code: '123456' });
    // 第一次被拒：已是 code，无回退空间 → FAIL{auth} 计入退避（非立即 offline）。
    act(() => t.simulateError('INVALID_CREDENTIAL'));
    expect(result.current.statuses['dev-1']).toEqual({ status: 'reconnecting', retries: 1 });
    // 连续失败至耗尽（共 6 次 error：r=0..5）→ offline{auth} 提示重新配对。
    exhaustRetries(result, 'INVALID_CREDENTIAL', 5);
    expect(result.current.statuses['dev-1']).toEqual({ status: 'offline', reason: 'auth' });
  });

  it('发送忙锁（§9.4）：发送后第二次被挡；ack 释放', () => {
    setToken('dev-1', 'dvct_tok');
    const { result } = renderHook(() => useConnections());
    act(() => result.current.connectDevice(DEVICE));
    const t = lastTransport();
    act(() => t.simulateOpen());
    expect(result.current.sendToDevice('dev-1', 'hello')).toBe(true);
    // 忙：第二次发送被挡。
    expect(result.current.sendToDevice('dev-1', 'again')).toBe(false);
    // ack 释放锁。
    act(() => t.simulateMessage({ type: 'ack' }));
    expect(result.current.sendToDevice('dev-1', 'again2')).toBe(true);
  });

  it('未连接/不存在设备 sendToDevice 返回 false', () => {
    const { result } = renderHook(() => useConnections());
    expect(result.current.sendToDevice('dev-1', 'x')).toBe(false);
  });

  it('disconnectDevice 清除 token + code（§9.5）', () => {
    setToken('dev-1', 'dvct_tok');
    setPairingCode('dev-1', '123456');
    expect(getToken('dev-1')).toBe('dvct_tok');
    const { result } = renderHook(() => useConnections());
    act(() => result.current.connectDevice(DEVICE));
    act(() => result.current.disconnectDevice('dev-1'));
    expect(result.current.statuses['dev-1']).toBeUndefined();
    expect(getToken('dev-1')).toBeNull();
    expect(getPairingCode('dev-1')).toBeNull();
  });

  it('retryDevice：offline → connecting{0} + 重开 transport（用户手动重试）', () => {
    setPairingCode('dev-1', '123456');
    const { result } = renderHook(() => useConnections());
    act(() => result.current.connectDevice(DEVICE));
    // 制造 offline：连续 6 次连接超时耗尽退避。
    exhaustRetries(result, 'CONNECTION_TIMEOUT', 6);
    expect(result.current.statuses['dev-1']).toEqual({ status: 'offline', reason: 'timeout' });
    // 手动重试：offline → CONNECT 重置 connecting{0}。
    act(() => result.current.retryDevice('dev-1'));
    expect(result.current.statuses['dev-1']).toEqual({ status: 'connecting', retries: 0 });
    const t2 = lastTransport();
    act(() => t2.simulateOpen());
    expect(result.current.statuses['dev-1']).toEqual({ status: 'connected' });
  });

  // 前台对账（visibilitychange）：PWA 冻结悬挂在 connecting 的会话，回到前台
  // 必须重开 transport——这是 connecting 停留态唯一的自愈路径。
  it('前台对账：connecting 且 transport 未 open → 重建 transport（悬挂恢复）', () => {
    setToken('dev-1', 'dvct_tok');
    const { result } = renderHook(() => useConnections());
    act(() => result.current.connectDevice(DEVICE));
    expect(result.current.statuses['dev-1']).toEqual({ status: 'connecting', retries: 0 });
    const t1 = lastTransport();
    const created = instances.length;

    // 回到前台：connecting + transport 未 open → 重开。
    act(() => {
      Object.defineProperty(document, 'visibilityState', {
        configurable: true,
        get: () => 'visible',
      });
      document.dispatchEvent(new Event('visibilitychange'));
    });
    expect(instances.length).toBe(created + 1);
    expect(result.current.statuses['dev-1']).toEqual({ status: 'connecting', retries: 0 });
    expect(t1.openState).toBe(false);

    // 新 transport open → connected（恢复路径走通）。
    const t2 = lastTransport();
    act(() => t2.simulateOpen());
    expect(result.current.statuses['dev-1']).toEqual({ status: 'connected' });
  });

  it('前台对账：connected 会话与 open 中的 transport 不受影响', () => {
    setToken('dev-1', 'dvct_tok');
    const { result } = renderHook(() => useConnections());
    act(() => result.current.connectDevice(DEVICE));
    act(() => lastTransport().simulateOpen());
    const created = instances.length;

    act(() => {
      Object.defineProperty(document, 'visibilityState', {
        configurable: true,
        get: () => 'visible',
      });
      document.dispatchEvent(new Event('visibilitychange'));
    });
    expect(instances.length).toBe(created);
    expect(result.current.statuses['dev-1']).toEqual({ status: 'connected' });
  });

  it('connectDevice 幂等：已 connecting 再调用不新建 transport', () => {
    setToken('dev-1', 'dvct_tok');
    const { result } = renderHook(() => useConnections());
    act(() => result.current.connectDevice(DEVICE));
    const before = instances.length;
    act(() => result.current.connectDevice(DEVICE)); // 已 connecting
    expect(instances.length).toBe(before);
  });

  it('close（连接成功后掉线）→ reconnecting{0}，计入退避', () => {
    setToken('dev-1', 'dvct_tok');
    const { result } = renderHook(() => useConnections());
    act(() => result.current.connectDevice(DEVICE));
    const t = lastTransport();
    act(() => t.simulateOpen());
    expect(result.current.statuses['dev-1']).toEqual({ status: 'connected' });
    // 掉线（无具名 error，仅 close）→ reconnecting{0}（首次归零）。
    act(() => t.simulateClose());
    expect(result.current.statuses['dev-1']).toEqual({ status: 'reconnecting', retries: 0 });
  });
});
