/**
 * Connection-related types shared across the desktop and mobile apps.
 *
 * These are the single source of truth for the wire format exchanged with the
 * Rust backend. Field names match the Rust structs *exactly* (snake_case),
 * because Tauri v2 serializes command args and return values with serde's
 * default naming and the Rust side does not apply `#[serde(rename_all)]`.
 *
 * See apps/desktop/src-tauri/src/commands/mod.rs (ConnectionInfo, Settings).
 */

/**
 * Coarse failure classification. §9 重构后连接层使用 `OfflineReason`
 * （见 `machines/connection.ts`）作为权威原因枚举；本类型保留供未来复用。
 */
export type ConnectionError = 'unreachable' | 'refused' | 'timeout' | 'unknown';

/**
 * Connected client info from the desktop connection manager.
 */
export interface ClientInfo {
  id: string;
  connected_at: string;
}

/**
 * Result returned by `start_server` and `get_connection_info`.
 *
 * Mirrors `commands::ConnectionInfo` in the Rust backend (snake_case).
 * WebRTC 架构下：`qr_payload` 是扫码直连载荷（dropvoice://pair?...），
 * 不再有 `url`/`port`（旧 LAN-WS 字段）。
 */
export interface ConnectionInfo {
  running: boolean;
  /** QR 载荷（dropvoice://pair?code=&device=&name=，§6.5）。 */
  qr_payload?: string;
  active_connections: number;
  /** 当前 6 位配对码（§3.2）。 */
  pairing_code?: string;
  /** 已连接客户端详情。 */
  clients: ClientInfo[];
  /** 当前注入队列深度。 */
  queue_depth: number;
}

/**
 * Settings returned by `get_settings` and mutated via `set_language` /
 * `set_theme` / `set_input_delay`.
 *
 * Mirrors `commands::Settings` in the Rust backend (snake_case).
 */
export interface Settings {
  language: string;
  theme: string;
  delay_ms: number;
  port: number;
  max_text_length: number;
  minimize_to_tray: boolean;
  /** 开机自启动（tauri-plugin-autostart 实时状态）。 */
  autostart: boolean;
}
