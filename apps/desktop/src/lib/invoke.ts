import { invoke } from '@tauri-apps/api/core';

import type { ConnectionInfo, Settings } from '@dropvoice/core';

export type { ConnectionInfo, Settings };

/**
 * Typed Tauri invoke wrapper.
 *
 * Tauri v2：Rust 命令参数 snake_case，JS 侧 invoke 的 key 必须用 camelCase
 * （如 `client_id` ↔ `clientId`）。此前误用 snake_case 导致
 * `register_client`/`inject_text`/`unregister_client`/`set_input_delay`
 * 全部报 "missing required key"（实测 CDP console 确认）。
 * 返回类型（`ConnectionInfo`、`Settings`）从 `@dropvoice/core` 导入。
 */
export const tauriInvoke = {
  startServer: () => invoke<ConnectionInfo>('start_server'),
  stopServer: () => invoke<void>('stop_server'),
  getConnectionInfo: () => invoke<ConnectionInfo>('get_connection_info'),
  // WebRTC 信令桥（§5.1）。
  injectText: (text: string, client_id: string) =>
    invoke<void>('inject_text', { text, clientId: client_id }),
  saveToken: (token: string) => invoke<void>('save_token', { token }),
  issueConnectionToken: () => invoke<string>('issue_connection_token'),
  validateCredential: (credential: string) => invoke<string>('validate_credential', { credential }),
  refreshPairingToken: () => invoke<string>('refresh_pairing_token'),
  getPairingToken: () => invoke<string | null>('get_pairing_token'),
  registerClient: (client_id: string) => invoke<void>('register_client', { clientId: client_id }),
  unregisterClient: (client_id: string) =>
    invoke<void>('unregister_client', { clientId: client_id }),
  // 设置。
  getSettings: () => invoke<Settings>('get_settings'),
  setLanguage: (language: string) => invoke<void>('set_language', { language }),
  setTheme: (theme: string) => invoke<void>('set_theme', { theme }),
  setInputDelay: (delay_ms: number) => invoke<void>('set_input_delay', { delayMs: delay_ms }),
  minimizeToTray: () => invoke<void>('minimize_to_tray'),
} as const;
