/**
 * 移动端 localStorage 工具：按 deviceId 命名空间持久化草稿 / 上次发送 / 配对码 / token。
 *
 * Key 约定（规范 10 section 2.6）：
 * - `dropvoice:draft:{deviceId}` → 当前草稿文本
 * - `dropvoice:last_sent:{deviceId}` → 上次成功发送的文本
 * - `dropvoice:pairing:{deviceId}` → 配对码（8 位数字）
 * - `dropvoice:token:{deviceId}` → 服务端下发的连接 token（UUID）
 */

const DRAFT_PREFIX = 'dropvoice:draft:';
const LAST_SENT_PREFIX = 'dropvoice:last_sent:';
const PAIRING_PREFIX = 'dropvoice:pairing:';
const TOKEN_PREFIX = 'dropvoice:token:';

function safeGet(key: string): string | null {
  if (typeof window === 'undefined') return null;
  try {
    return window.localStorage.getItem(key);
  } catch {
    return null;
  }
}

function safeSet(key: string, value: string): void {
  if (typeof window === 'undefined') return;
  try {
    window.localStorage.setItem(key, value);
  } catch {
    // Quota 或隐私模式：静默忽略，UI 继续工作。
  }
}

function safeRemove(key: string): void {
  if (typeof window === 'undefined') return;
  try {
    window.localStorage.removeItem(key);
  } catch {
    // 忽略。
  }
}

export function getDraft(deviceId: string): string {
  return safeGet(`${DRAFT_PREFIX}${deviceId}`) ?? '';
}

export function setDraft(deviceId: string, text: string): void {
  const key = `${DRAFT_PREFIX}${deviceId}`;
  if (!text) {
    safeRemove(key);
    return;
  }
  safeSet(key, text);
}

export function getLastSent(deviceId: string): string {
  return safeGet(`${LAST_SENT_PREFIX}${deviceId}`) ?? '';
}

export function setLastSent(deviceId: string, text: string): void {
  const key = `${LAST_SENT_PREFIX}${deviceId}`;
  if (!text) {
    safeRemove(key);
    return;
  }
  safeSet(key, text);
}

export function getPairingCode(deviceId: string): string | null {
  return safeGet(`${PAIRING_PREFIX}${deviceId}`);
}

export function setPairingCode(deviceId: string, code: string): void {
  safeSet(`${PAIRING_PREFIX}${deviceId}`, code);
}

export function clearPairingCode(deviceId: string): void {
  safeRemove(`${PAIRING_PREFIX}${deviceId}`);
}

export function getToken(deviceId: string): string | null {
  return safeGet(`${TOKEN_PREFIX}${deviceId}`);
}

export function setToken(deviceId: string, token: string): void {
  safeSet(`${TOKEN_PREFIX}${deviceId}`, token);
}

export function clearToken(deviceId: string): void {
  safeRemove(`${TOKEN_PREFIX}${deviceId}`);
}
