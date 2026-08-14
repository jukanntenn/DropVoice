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

/**
 * @param deviceIdProvider 返回当前 device_id 的函数（从 config 读取）
 */
export function useWebRTC(): UseWebRTCHandle {
  const signalingRef = useRef<SignalingClient | null>(null);
  const pcsRef = useRef<Map<string, RTCPeerConnection>>(new Map());

  const start = useCallback((deviceId: string, pairingToken: string) => {
    // 停止旧的。
    if (signalingRef.current) {
      signalingRef.current.stop();
    }
    pcsRef.current.forEach((pc) => pc.close());
    pcsRef.current.clear();

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
      const { pc, answerSdp } = await createAnswerForOffer(event.sdp, (channel) => {
        // 绑定 DataChannel（clientId 用 sessionId）。
        wireDataChannel(channel, kind, event.session_id);
      });
      // 注册 PC（sessionId → PC，§5.6）。
      pcsRef.current.set(event.session_id, pc);
      // PC 关闭时清理。
      pc.addEventListener('connectionstatechange', () => {
        if (
          pc.connectionState === 'failed' ||
          pc.connectionState === 'closed' ||
          pc.connectionState === 'disconnected'
        ) {
          pcsRef.current.delete(event.session_id);
        }
      });

      return { accepted: true, sdp: answerSdp };
    };

    client.start(pairingToken).catch((err) => {
      console.error('[useWebRTC] SignalingClient start failed', err);
    });
    signalingRef.current = client;
  }, []);

  const stop = useCallback(() => {
    if (signalingRef.current) {
      signalingRef.current.stop();
      signalingRef.current = null;
    }
    pcsRef.current.forEach((pc) => pc.close());
    pcsRef.current.clear();
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
