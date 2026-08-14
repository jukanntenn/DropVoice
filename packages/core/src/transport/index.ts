/**
 * 传输适配层类型（webrtc-scan-direct-design §6.1）。
 *
 * §9 重构后传输实现唯一：`RtcTransport`（WebRTC DataChannel），WS 降级与运行时
 * 选择（旧 `selectTransport`/`wsFactory`）已作为死代码删除。
 *
 * 消息协议（§6.3）：
 * - 手机→桌面：`{"type":"text","text":"..."}`
 * - 桌面→手机：`{"type":"ack"}` / `{"type":"token","token":"dvct_xxx"}`
 *   / `{"type":"error","code":"...","message":"..."}`
 */

import type { Device } from '../types';

/** 传输连接的认证方式。 */
export type TransportAuth = { kind: 'code'; code: string } | { kind: 'token'; token: string };

/** 服务端→客户端的消息（§6.3）。 */
export type ServerMessage =
  | { type: 'token'; token: string }
  | { type: 'ack' }
  | { type: 'error'; code: string; message?: string };

/** 传输事件类型。 */
export type TransportEvent = 'open' | 'message' | 'close' | 'error';

/** 传输事件回调。 */
export type TransportEventHandler = (detail?: { data?: ServerMessage }) => void;

/**
 * 传输适配器接口（§6.1）。
 *
 * `useConnections` 通过此接口与传输层交互。实现需在 `open` 后通过 `on`
 * 注册的回调通知连接生命周期。
 */
export interface TransportAdapter {
  /** 打开连接（带认证）。 */
  open(device: Device, auth: TransportAuth): void;
  /** 发送文本。返回是否成功投递到传输层。 */
  send(text: string): boolean;
  /** 关闭连接。 */
  close(): void;
  /** 是否已连接且可发送。 */
  readonly isOpen: boolean;
  /** 注册事件回调。返回取消注册的函数。 */
  on(event: TransportEvent, handler: TransportEventHandler): () => void;
}
