import { invoke } from '@tauri-apps/api/core';

import type { ConnectionInfo, Settings } from '@dropvoice/core';

export type { ConnectionInfo, Settings };

/**
 * Typed Tauri invoke wrapper.
 *
 * Tauri v2：Rust 命令参数 snake_case，JS 侧 invoke 的 key 必须用 camelCase
 * （如 `delay_ms` ↔ `delayMs`）。此前误用 snake_case 导致
 * `set_input_delay` 报 "missing required key"（实测 CDP console 确认）。
 * 返回类型（`ConnectionInfo`、`Settings`）从 `@dropvoice/core` 导入。
 *
 * 信令/WebRTC 命令已随应答面迁入 Rust 常驻子系统（network/signaling、
 * webrtc/）——webview 只保留视图所需的查询与设置命令。
 */
export const tauriInvoke = {
  startServer: () => invoke<ConnectionInfo>('start_server'),
  stopServer: () => invoke<void>('stop_server'),
  getConnectionInfo: () => invoke<ConnectionInfo>('get_connection_info'),
  // 设置。
  getSettings: () => invoke<Settings>('get_settings'),
  setLanguage: (language: string) => invoke<void>('set_language', { language }),
  setTheme: (theme: string) => invoke<void>('set_theme', { theme }),
  setInputDelay: (delay_ms: number) => invoke<void>('set_input_delay', { delayMs: delay_ms }),
  setAutostart: (enabled: boolean) => invoke<void>('set_autostart', { enabled }),
  minimizeToTray: () => invoke<void>('minimize_to_tray'),
} as const;
