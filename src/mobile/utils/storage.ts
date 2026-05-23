const host = window.location.host;

function draftKey(deviceId: string): string {
  return `dropvoice_draft_${host}_${deviceId}`;
}

function lastSentKey(deviceId: string): string {
  return `dropvoice_last_sent_${host}_${deviceId}`;
}

export function saveDraft(text: string, deviceId: string): void {
  try {
    localStorage.setItem(draftKey(deviceId), text);
  } catch {}
}

export function loadDraft(deviceId: string): string | null {
  try {
    return localStorage.getItem(draftKey(deviceId));
  } catch {
    return null;
  }
}

export function clearDraft(deviceId: string): void {
  try {
    localStorage.removeItem(draftKey(deviceId));
  } catch {}
}

export function saveLastSent(text: string, deviceId: string): void {
  try {
    localStorage.setItem(lastSentKey(deviceId), text);
  } catch {}
}

export function loadLastSent(deviceId: string): string | null {
  try {
    return localStorage.getItem(lastSentKey(deviceId));
  } catch {
    return null;
  }
}

export function clearLastSent(deviceId: string): void {
  try {
    localStorage.removeItem(lastSentKey(deviceId));
  } catch {}
}

export function hasLastSent(deviceId: string): boolean {
  try {
    return localStorage.getItem(lastSentKey(deviceId)) !== null;
  } catch {
    return false;
  }
}
