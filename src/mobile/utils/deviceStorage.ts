import type { Device, DeviceStorage, StoredDevice } from "../types";
import { storageGet, storageSet } from "./createStorage";

const STORAGE_KEY = "dropvoice:devices:v1";

export const MAX_DEVICES = 5;

export function loadDeviceStorage(): DeviceStorage {
  try {
    const raw = storageGet(STORAGE_KEY);
    if (!raw) return { devices: [], lastActiveDeviceId: null };
    const parsed = JSON.parse(raw) as DeviceStorage;
    if (!parsed || !Array.isArray(parsed.devices)) {
      return { devices: [], lastActiveDeviceId: null };
    }
    return {
      devices: parsed.devices,
      lastActiveDeviceId:
        typeof parsed.lastActiveDeviceId === "string"
          ? parsed.lastActiveDeviceId
          : null,
    };
  } catch {
    return { devices: [], lastActiveDeviceId: null };
  }
}

export function saveDeviceStorage(data: DeviceStorage): void {
  storageSet(STORAGE_KEY, JSON.stringify(data));
}

export function storedToDevice(stored: StoredDevice): Device {
  return {
    ...stored,
    status: "disconnected",
  };
}

export function deviceToStored(device: Device): StoredDevice {
  const { status: _status, ...rest } = device;
  return rest;
}

export function generateDeviceId(): string {
  try {
    return crypto.randomUUID();
  } catch {
    return `device_${Date.now()}_${Math.random().toString(36).slice(2, 10)}`;
  }
}

export function getDefaultDeviceName(url: string): string {
  try {
    const urlObj = new URL(url);
    const hostname = urlObj.hostname;
    if (/^\d+\.\d+\.\d+\.\d+$/.test(hostname)) {
      const lastOctet = hostname.split(".").pop();
      return `PC-${lastOctet ?? hostname}`;
    }
    return hostname ? `PC-${hostname}` : "Unknown Device";
  } catch {
    return "Unknown Device";
  }
}

export function isValidWebSocketUrl(url: string): boolean {
  try {
    const parsed = new URL(url);
    return parsed.protocol === "ws:" || parsed.protocol === "wss:";
  } catch {
    return false;
  }
}

export function normalizeToWebSocketUrl(input: string): string | null {
  const trimmed = input.trim();
  if (!trimmed) return null;

  try {
    const u = new URL(trimmed);
    if (u.protocol === "http:" || u.protocol === "https:") {
      const wsProtocol = u.protocol === "https:" ? "wss:" : "ws:";
      const base = `${wsProtocol}//${u.host}`;
      const path = u.pathname && u.pathname !== "/" ? u.pathname : "";
      const suffix = path.endsWith("/ws") ? "" : "/ws";
      return `${base}${path}${suffix}`;
    }

    if (u.protocol === "ws:" || u.protocol === "wss:") {
      const base = `${u.protocol}//${u.host}`;
      const path = u.pathname && u.pathname !== "/" ? u.pathname : "";
      if (path.endsWith("/ws")) return `${base}${path}${u.search ?? ""}`;
      return `${base}${path}/ws${u.search ?? ""}`;
    }

    return null;
  } catch {
    return null;
  }
}
