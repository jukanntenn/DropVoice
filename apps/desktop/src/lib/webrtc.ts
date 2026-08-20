/**
 * WebRTC 连接管理（桌面 webview JS，webrtc-scan-direct-design §5.5/§5.6/§6.3）。
 *
 * 职责：
 * - 维护 Map<sessionId, RTCPeerConnection>（每个 incoming offer 一个 PC，§5.6）
 * - 收到 offer → setRemoteDescription → createAnswer → setLocalDescription
 *   → 等 ICE complete（2s 超时 fallback）→ 返回 answer SDP
 * - DataChannel 建立：onmessage 解析 {type:'text'} → invoke('inject_text')；回 {type:'ack'}
 * - 首次配对（credential 命中 code）+ DataChannel open → invoke('issue_connection_token')
 *   → 发 {type:'token',token}
 */

import { tauriInvoke } from './invoke';

/** DataChannel 应用层消息协议（§6.3，与 WS 逐字节兼容）。 */
type DataChannelMessage =
  | { type: 'text'; text: string }
  | { type: 'ack' }
  | { type: 'token'; token: string }
  | { type: 'error'; code: string; message?: string }
  // 手机上报稳定身份（客户端→桌面，见 wireDataChannel 的 hello 处理）。
  | { type: 'hello'; clientId: string };

/** ICE 收集超时 fallback（§5.5 2s）。 */
const ICE_GATHERING_TIMEOUT = 2000;

/**
 * 为一个 incoming offer 创建 RTCPeerConnection + 生成 answer。
 *
 * @param offerSdp 手机发来的 offer SDP
 * @param onChannel 收到 DataChannel 时回调（手机 offerer 创建的通道在
 *   setRemoteDescription 处理时即触发 datachannel 事件，回调必须在
 *   setRemoteDescription 前挂好，否则事件丢失——实测桌面显示"暂无设备连接"）
 * @returns answer SDP（ICE complete 或 2s 超时后）+ PC
 *
 * 注意：DataChannel 由**手机侧（offerer）创建**（m=application 行必须出现在
 * offer 中，answer 不能新增 m-line）。桌面（answerer）通过 PC 的
 * `datachannel` 事件接收通道。
 */
export async function createAnswerForOffer(
  offerSdp: string,
  onChannel: (channel: RTCDataChannel) => void
): Promise<{
  pc: RTCPeerConnection;
  answerSdp: string;
}> {
  // §5.5 空 iceServers：同 LAN 只收集 host candidate（mDNS .local），无需 STUN/TURN。
  const pc = new RTCPeerConnection({ iceServers: [], iceCandidatePoolSize: 1 });

  // 必须在 setRemoteDescription 之前注册：offer 的数据通道 m-line 在
  // setRemoteDescription 处理时即触发 datachannel 事件。
  pc.addEventListener('datachannel', (e) => onChannel(e.channel));

  // setRemoteDescription(offer) → createAnswer → setLocalDescription(answer)
  await pc.setRemoteDescription({ type: 'offer', sdp: offerSdp });
  const answer = await pc.createAnswer();
  await pc.setLocalDescription(answer);

  // §5.5 等 ICE complete（非 trickle），2s 超时 fallback。
  const answerSdp = await waitForIceComplete(pc);

  return { pc, answerSdp };
}

/**
 * 等 ICE candidate 收集完成（iceGatheringState === 'complete'），
 * 或 2s 超时后用已收集的 candidate 继续（§5.5 fallback）。
 */
function waitForIceComplete(pc: RTCPeerConnection): Promise<string> {
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
    // 超时 fallback（无论是否 complete）。
    setTimeout(finish, ICE_GATHERING_TIMEOUT);
  });
}

/**
 * DataChannel 消息处理器。
 *
 * 注册 onmessage/onopen/onclose。首次配对成功（credentialKind==='code'）+ open
 * 后签发 token 发给手机。
 *
 * 注册键（`reg.id`）：
 * - 初始为 `session_id`（每次 offer 全新）——立即注册，保证旧手机/计数即时可用。
 * - 收到手机 `hello.clientId` 后改为稳定 clientId 重新注册（先注销旧键），
 *   使活跃设备数以"设备"而非"会话"计（断连重连不虚增）。
 * - `connectionstatechange` 与 channel close 都以最新 `reg.id` 注销。
 *
 * @param channel DataChannel
 * @param credentialKind validate_credential 返回的命中类型（'code' 首次配对）
 * @param reg 可变的注册键（初始 session_id；hello 后改写为稳定 clientId）
 */
export function wireDataChannel(
  channel: RTCDataChannel,
  credentialKind: 'code' | 'token' | 'invalid',
  reg: { id: string }
): void {
  channel.addEventListener('open', async () => {
    console.debug('[webrtc] DataChannel open', reg.id);
    // 注册到 ConnectionManager（初始键 = session_id）。
    try {
      await tauriInvoke.registerClient(reg.id);
    } catch (err) {
      console.error('[webrtc] registerClient failed', err);
    }
    // 首次配对（code 命中）→ 签发 token 发给手机（§6.3）。
    if (credentialKind === 'code') {
      try {
        const token = await tauriInvoke.issueConnectionToken();
        sendMessage(channel, { type: 'token', token });
      } catch (err) {
        console.error('[webrtc] issueConnectionToken failed', err);
      }
    }
  });

  channel.addEventListener('message', async (e: MessageEvent) => {
    try {
      const msg = JSON.parse(e.data as string) as DataChannelMessage;
      if (msg.type === 'text') {
        // §5.4：invoke('inject_text') → Rust ConnectionManager → Enigo。
        try {
          await tauriInvoke.injectText(msg.text, reg.id);
          sendMessage(channel, { type: 'ack' });
        } catch (err) {
          console.error('[webrtc] injectText failed', err);
          sendMessage(channel, {
            type: 'error',
            code: 'INJECTION_FAILED',
            message: err instanceof Error ? err.message : String(err),
          });
        }
      } else if (msg.type === 'hello' && msg.clientId && msg.clientId !== reg.id) {
        // 稳定身份上报：把注册键从 session_id 换到客户端 clientId。
        const old = reg.id;
        reg.id = msg.clientId;
        console.debug('[webrtc] hello: re-keying registration', old, '->', reg.id);
        try {
          await tauriInvoke.unregisterClient(old);
          await tauriInvoke.registerClient(reg.id);
        } catch (err) {
          console.error('[webrtc] re-register under clientId failed', err);
        }
      }
    } catch (err) {
      console.error('[webrtc] DataChannel message parse failed', err);
    }
  });

  channel.addEventListener('close', async () => {
    console.debug('[webrtc] DataChannel close', reg.id);
    try {
      await tauriInvoke.unregisterClient(reg.id);
    } catch (err) {
      console.error('[webrtc] unregisterClient failed', err);
    }
  });

  channel.addEventListener('error', (e) => {
    console.error('[webrtc] DataChannel error', reg.id, e);
  });
}

/** 发送 DataChannel 消息。 */
export function sendMessage(channel: RTCDataChannel, msg: DataChannelMessage): void {
  if (channel.readyState === 'open') {
    channel.send(JSON.stringify(msg));
  }
}
