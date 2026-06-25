import { storageGet, storageSet, storageRemove } from "./createStorage";

const host = window.location.host;

function draftKey(deviceId: string): string {
  return `dropvoice_draft_${host}_${deviceId}`;
}

function lastSentKey(deviceId: string): string {
  return `dropvoice_last_sent_${host}_${deviceId}`;
}

export function saveDraft(text: string, deviceId: string): void {
  storageSet(draftKey(deviceId), text);
}

export function loadDraft(deviceId: string): string | null {
  return storageGet(draftKey(deviceId));
}

export function clearDraft(deviceId: string): void {
  storageRemove(draftKey(deviceId));
}

export function saveLastSent(text: string, deviceId: string): void {
  storageSet(lastSentKey(deviceId), text);
}

export function loadLastSent(deviceId: string): string | null {
  return storageGet(lastSentKey(deviceId));
}

export function clearLastSent(deviceId: string): void {
  storageRemove(lastSentKey(deviceId));
}

export function hasLastSent(deviceId: string): boolean {
  return storageGet(lastSentKey(deviceId)) !== null;
}
