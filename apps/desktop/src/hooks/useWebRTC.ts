/**
 * useWebRTC —— 桌面 WebRTC 信令客户端编排（webrtc-scan-direct-design §5.1）。
 *
 * 启动流程：
 * 1. start_server 返回 ConnectionInfo（含 qr_payload，内含 device_id）
 * 2. 监听 'pairing_token' 事件获取 pairing_token
 * 3. 用 device_id + token 启动 SignalingClient（SSE）
 * 4. 收到 offer → invoke('validate_credential') → createAnswerForOffer → wireDataChannel
 *
 * 多手机同时配对：每个 offer 创建独立 RTCPeerConnection（§5.6 Map<sessionId, PC>）。
 */

import { useCallback, useEffect, useRef } from 'react';

import { SignalingClient } from '../lib/signaling';
import { createAnswerForOffer, wireDataChannel } from '../lib/webrtc';
import { tauriInvoke } from '../lib/invoke';

interface UseWebRTCHandle {
  start: (deviceId: string, pairingToken: string) => void;
  stop: () => void;
}

/** 'disconnected' 后注销的宽限期：网络闪断可在这段时间内恢复。 */
const DISCONNECT_GRACE_MS = 10_000;

/**
 * @param deviceIdProvider 返回当前 device_id 的函数（从 config 读取）
 */
export function useWebRTC(): UseWebRTCHandle {
  const signalingRef = useRef<SignalingClient | null>(null);
  const pcsRef = useRef<Map<string, RTCPeerConnection>>(new Map());
  const disconnectTimersRef = useRef<Map<string, number>>(new Map());

  /** 清除某会话的"断连宽限"定时器（重连成功时调用）。 */
  const clearDisconnectTimer = useCallback((sessionId: string) => {
    const timer = disconnectTimersRef.current.get(sessionId);
    if (timer !== undefined) {
      window.clearTimeout(timer);
      disconnectTimersRef.current.delete(sessionId);
    }
  }, []);

  const start = useCallback(
    (deviceId: string, pairingToken: string) => {
      // 停止旧的。
      if (signalingRef.current) {
        signalingRef.current.stop();
      }
      pcsRef.current.forEach((pc) => pc.close());
      pcsRef.current.clear();
      disconnectTimersRef.current.forEach((t) => window.clearTimeout(t));
      disconnectTimersRef.current.clear();

      const client = new SignalingClient(deviceId);
      client.onOffer = async (event) => {
        // §5.1：invoke('validate_credential') 校验 code/token。
        const kind = (await tauriInvoke.validateCredential(event.credential)) as
          'code' | 'token' | 'invalid';
        if (kind === 'invalid') {
          // credential 错误 → rejected（§3.2 安全模型）。
          return { accepted: false };
        }

        // 生成 answer（数据通道由手机 offerer 创建，桌面经 ondatachannel 接收）。
        // onChannel 在 setRemoteDescription 之前挂载，避免 datachannel 事件丢失。
        const reg = { id: event.session_id };
        const { pc, answerSdp } = await createAnswerForOffer(event.sdp, (channel) => {
          // 绑定 DataChannel；reg.id 初始为 sessionId，手机 hello 后改写为稳定 clientId。
          wireDataChannel(channel, kind, reg);
        });
        // 注册 PC（sessionId → PC，§5.6）。
        pcsRef.current.set(event.session_id, pc);
        // PC 关闭时清理 + 可靠注销（此前仅删 PC 引用，僵尸客户端占满 max_connections）：
        // - failed / closed → 立即注销；
        // - disconnected → 10s 宽限（网络闪断可恢复），超时未恢复则注销。
        pc.addEventListener('connectionstatechange', () => {
          const state = pc.connectionState;
          if (state === 'failed' || state === 'closed') {
            clearDisconnectTimer(event.session_id);
            pcsRef.current.delete(event.session_id);
            tauriInvoke.unregisterClient(reg.id).catch((err) => {
              console.error('[useWebRTC] unregister (failed/closed) error', err);
            });
          } else if (state === 'disconnected') {
            const timer = window.setTimeout(() => {
              pcsRef.current.delete(event.session_id);
              disconnectTimersRef.current.delete(event.session_id);
              tauriInvoke.unregisterClient(reg.id).catch((err) => {
                console.error('[useWebRTC] unregister (disconnect grace) error', err);
              });
            }, DISCONNECT_GRACE_MS);
            disconnectTimersRef.current.set(event.session_id, timer);
          } else if (state === 'connected') {
            clearDisconnectTimer(event.session_id);
          }
        });

        return { accepted: true, sdp: answerSdp };
      };

      client.start(pairingToken).catch((err) => {
        console.error('[useWebRTC] SignalingClient start failed', err);
      });
      signalingRef.current = client;
    },
    [clearDisconnectTimer]
  );

  const stop = useCallback(() => {
    if (signalingRef.current) {
      signalingRef.current.stop();
      signalingRef.current = null;
    }
    pcsRef.current.forEach((pc) => pc.close());
    pcsRef.current.clear();
    disconnectTimersRef.current.forEach((t) => window.clearTimeout(t));
    disconnectTimersRef.current.clear();
  }, []);

  // 卸载时清理。
  useEffect(() => {
    return () => stop();
  }, [stop]);

  // 持有 WebLock（§5.2.2 belt-and-suspenders，实验证明非必需但无害）。
  useEffect(() => {
    if ('locks' in navigator) {
      navigator.locks.request('dropvoice-keepalive', () => {
        // 永不 resolve（持有 WebLock 直到页面卸载）。
        return new Promise<void>(() => {});
      });
    }
    return undefined;
  }, []);

  return { start, stop };
}
