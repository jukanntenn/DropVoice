const host = window.location.host;
const DRAFT_KEY = `dropvoice_draft_${host}`;
const LAST_SENT_KEY = `dropvoice_last_sent_${host}`;

export function saveDraft(text: string): void {
  try {
    localStorage.setItem(DRAFT_KEY, text);
  } catch {}
}

export function loadDraft(): string | null {
  try {
    return localStorage.getItem(DRAFT_KEY);
  } catch {
    return null;
  }
}

export function clearDraft(): void {
  try {
    localStorage.removeItem(DRAFT_KEY);
  } catch {}
}

export function saveLastSent(text: string): void {
  try {
    localStorage.setItem(LAST_SENT_KEY, text);
  } catch {}
}

export function loadLastSent(): string | null {
  try {
    return localStorage.getItem(LAST_SENT_KEY);
  } catch {
    return null;
  }
}

export function clearLastSent(): void {
  try {
    localStorage.removeItem(LAST_SENT_KEY);
  } catch {}
}

export function hasLastSent(): boolean {
  try {
    return localStorage.getItem(LAST_SENT_KEY) !== null;
  } catch {
    return false;
  }
}
