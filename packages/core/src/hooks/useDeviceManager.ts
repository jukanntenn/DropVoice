import { useCallback, useEffect, useRef, useState } from 'react';

import type { AddDeviceResult, Device, DeviceStorage, StoredDevice } from '../types';
import { createDeviceManager, type ScannedDevice } from '../connection';

/** 存储键 v2（v1 是旧 WS 数据模型，不兼容；无历史数据，直接用新键）。 */
const STORAGE_KEY = 'dropvoice:devices:v2';

export interface UseDeviceManagerReturn {
  isInitialized: boolean;
  devices: Device[];
  activeDeviceId: string | null;
  setActiveDeviceId: (deviceId: string | null) => void;
  setDevices: (devices: Device[]) => void;
  addDevice: (scanned: ScannedDevice) => AddDeviceResult;
  removeDevice: (deviceId: string) => void;
}

const manager = createDeviceManager();

function loadStorage(): DeviceStorage {
  if (typeof window === 'undefined') {
    return { devices: [], lastActiveDeviceId: null };
  }
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      return { devices: [], lastActiveDeviceId: null };
    }
    const parsed = JSON.parse(raw) as DeviceStorage;
    return {
      devices: Array.isArray(parsed.devices) ? parsed.devices : [],
      lastActiveDeviceId: parsed.lastActiveDeviceId ?? null,
    };
  } catch {
    return { devices: [], lastActiveDeviceId: null };
  }
}

function saveStorage(storage: DeviceStorage): void {
  if (typeof window === 'undefined') {
    return;
  }
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(storage));
  } catch {
    // Quota or serialization issues: silently ignore so the UI keeps working.
  }
}

/** 水合：只取持久身份字段（§9 瘦身后无瞬态）。旧数据多出的字段静默忽略。 */
export function hydrate(stored: StoredDevice[]): Device[] {
  return stored.map((entry) => ({
    id: entry.id,
    name: entry.name,
    autoConnect: entry.autoConnect ?? true,
  }));
}

export function useDeviceManager(): UseDeviceManagerReturn {
  const [isInitialized, setIsInitialized] = useState(false);
  const [devices, setDevicesState] = useState<Device[]>([]);
  const [activeDeviceId, setActiveDeviceIdState] = useState<string | null>(null);
  // devicesRef 仅供 addDevice/removeDevice 同步读当前列表以计算返回值
  //（同步契约不变）；不再参与持久化。
  const devicesRef = useRef(devices);
  devicesRef.current = devices;

  useEffect(() => {
    const stored = loadStorage();
    const hydrated = hydrate(stored.devices);
    setDevicesState(hydrated);
    setActiveDeviceIdState(stored.lastActiveDeviceId);
    setIsInitialized(true);
  }, []);

  // §8：单一持久化副作用——所有 mutation 只更 state，唯一一个 effect 在渲染
  // 提交后用 committed 值写 localStorage。消除"删活跃设备被陈旧 ref 覆盖"的竞态。
  useEffect(() => {
    if (!isInitialized) return;
    const stored: StoredDevice[] = devices.map((d) => ({
      id: d.id,
      name: d.name,
      autoConnect: d.autoConnect,
    }));
    saveStorage({ devices: stored, lastActiveDeviceId: activeDeviceId });
  }, [isInitialized, devices, activeDeviceId]);

  const setActiveDeviceId = useCallback((deviceId: string | null) => {
    const next = manager.setActive(devicesRef.current, deviceId);
    setActiveDeviceIdState(next);
  }, []);

  const setDevices = useCallback((next: Device[]) => {
    setDevicesState(next);
  }, []);

  const addDevice = useCallback(
    (scanned: ScannedDevice): AddDeviceResult => {
      const next = [...devicesRef.current];
      const result = manager.addDevice(next, scanned);
      if (result === 'added' || result === 'switched') {
        setDevicesState(next);
        const newActive = result === 'added' ? (next[next.length - 1]?.id ?? null) : activeDeviceId;
        if (newActive !== activeDeviceId) {
          setActiveDeviceIdState(newActive);
        }
      }
      return result;
    },
    [activeDeviceId]
  );

  const removeDevice = useCallback(
    (deviceId: string) => {
      const next = manager.removeDevice(devicesRef.current, deviceId);
      setDevicesState(next);
      if (activeDeviceId === deviceId) {
        setActiveDeviceIdState(null);
      }
    },
    [activeDeviceId]
  );

  return {
    isInitialized,
    devices,
    activeDeviceId,
    setActiveDeviceId,
    setDevices,
    addDevice,
    removeDevice,
  };
}
