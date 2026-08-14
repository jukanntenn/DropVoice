/**
 * Error codes emitted by the Rust backend.
 *
 * Synchronized with `AppError::error_code()` in
 * `apps/desktop/src-tauri/src/error.rs`. Codes use `SCREAMING_SNAKE_CASE`
 * and are grouped by prefix per spec 04 section 2.2.
 */

export const ERROR_CODES = [
  'SERVER_ALREADY_RUNNING',
  'SERVER_START_FAILED',
  'PORT_IN_USE',
  'PORT_BIND_FAILED',
  'NETWORK_UNREACHABLE',
  'CONNECTION_REFUSED',
  'CONNECTION_TIMEOUT',
  'CONNECTION_LOST',
  'TEXT_INJECTION_FAILED',
  'TEXT_TOO_LONG',
  'TEXT_EMPTY',
  'INVALID_LANGUAGE',
  'INVALID_THEME',
  'SETTINGS_SAVE_FAILED',
  'DEVICE_NOT_FOUND',
  'DEVICE_ALREADY_CONNECTED',
  'MAX_DEVICES_REACHED',
  'MESSAGE_TOO_LARGE',
  'MESSAGE_INVALID',
  'QUEUE_FULL',
  'IO_ERROR',
  'INTERNAL_ERROR',
] as const;

export type ErrorCode = (typeof ERROR_CODES)[number];

export const ERROR_I18N_MAP: Record<ErrorCode, string> = {
  SERVER_ALREADY_RUNNING: 'errors:serverAlreadyRunning',
  SERVER_START_FAILED: 'errors:serverStartFailed',
  PORT_IN_USE: 'errors:portInUse',
  PORT_BIND_FAILED: 'errors:portBindFailed',
  NETWORK_UNREACHABLE: 'errors:networkUnreachable',
  CONNECTION_REFUSED: 'errors:connectionRefused',
  CONNECTION_TIMEOUT: 'errors:connectionTimeout',
  CONNECTION_LOST: 'errors:connectionLost',
  TEXT_INJECTION_FAILED: 'errors:textInjectionFailed',
  TEXT_TOO_LONG: 'errors:textTooLong',
  TEXT_EMPTY: 'errors:textEmpty',
  INVALID_LANGUAGE: 'errors:invalidLanguage',
  INVALID_THEME: 'errors:invalidTheme',
  SETTINGS_SAVE_FAILED: 'errors:settingsSaveFailed',
  DEVICE_NOT_FOUND: 'errors:deviceNotFound',
  DEVICE_ALREADY_CONNECTED: 'errors:deviceAlreadyConnected',
  MAX_DEVICES_REACHED: 'errors:maxDevicesReached',
  MESSAGE_TOO_LARGE: 'errors:messageTooLarge',
  MESSAGE_INVALID: 'errors:messageInvalid',
  QUEUE_FULL: 'errors:queueFull',
  IO_ERROR: 'errors:ioError',
  INTERNAL_ERROR: 'errors:internalError',
};

export const SUGGESTION_I18N_MAP: Record<string, string> = {
  check_network: 'errors:suggestion.checkNetwork',
  check_server_running: 'errors:suggestion.checkServerRunning',
  retry_later: 'errors:suggestion.retryLater',
  change_port: 'errors:suggestion.changePort',
  remove_device: 'errors:suggestion.removeDevice',
};

export function isErrorCode(value: unknown): value is ErrorCode {
  return typeof value === 'string' && (ERROR_CODES as readonly string[]).includes(value);
}
