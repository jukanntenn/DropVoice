type StorageBackend = Pick<Storage, "getItem" | "setItem" | "removeItem">;

const backends: StorageBackend[] = [];

function probe(backend: StorageBackend): boolean {
  try {
    const k = "__dv_probe__";
    backend.setItem(k, "1");
    backend.removeItem(k);
    return true;
  } catch {
    return false;
  }
}

function init() {
  if (backends.length > 0) return;
  if (probe(localStorage)) backends.push(localStorage);
  if (probe(sessionStorage)) backends.push(sessionStorage);
}

const memory = new Map<string, string>();
const memoryBackend: StorageBackend = {
  getItem: (k) => memory.get(k) ?? null,
  setItem: (k, v) => memory.set(k, v),
  removeItem: (k) => memory.delete(k),
};

export function storageGet(key: string): string | null {
  init();
  for (const b of backends) {
    try {
      const v = b.getItem(key);
      if (v !== null) return v;
    } catch {}
  }
  return memoryBackend.getItem(key);
}

export function storageSet(key: string, value: string): boolean {
  init();
  let ok = false;
  for (const b of backends) {
    try {
      b.setItem(key, value);
      ok = true;
    } catch {}
  }
  memoryBackend.setItem(key, value);
  return ok;
}

export function storageRemove(key: string): void {
  init();
  for (const b of backends) {
    try {
      b.removeItem(key);
    } catch {}
  }
  memoryBackend.removeItem(key);
}

export function storageAvailable(): boolean {
  init();
  return backends.length > 0;
}
