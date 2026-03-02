import { useCallback, useEffect, useMemo, useState } from "react";
import type { Device } from "../types";
import {
  deviceToStored,
  loadDeviceStorage,
  saveDeviceStorage,
  storedToDevice,
} from "../utils/deviceStorage";

interface UseDeviceManagerReturn {
  isInitialized: boolean;
  devices: Device[];
  activeDeviceId: string | null;
  setActiveDeviceId: (deviceId: string | null) => void;
  setDevices: (devices: Device[]) => void;
}

export function useDeviceManager(): UseDeviceManagerReturn {
  const [isInitialized, setIsInitialized] = useState(false);
  const [devices, setDevices] = useState<Device[]>([]);
  const [activeDeviceId, setActiveDeviceIdState] = useState<string | null>(null);

  useEffect(() => {
    const storage = loadDeviceStorage();
    const loadedDevices = storage.devices.map(storedToDevice);
    setDevices(loadedDevices);

    const hasStoredActive =
      typeof storage.lastActiveDeviceId === "string" &&
      loadedDevices.some((d) => d.id === storage.lastActiveDeviceId);

    setActiveDeviceIdState(
      hasStoredActive
        ? storage.lastActiveDeviceId
        : loadedDevices.length > 0
          ? loadedDevices[0].id
          : null,
    );

    setIsInitialized(true);
  }, []);

  useEffect(() => {
    if (!isInitialized) return;
    const storedDevices = devices.map(deviceToStored);
    const lastActiveDeviceId =
      activeDeviceId && devices.some((d) => d.id === activeDeviceId)
        ? activeDeviceId
        : null;
    saveDeviceStorage({ devices: storedDevices, lastActiveDeviceId });
  }, [activeDeviceId, devices, isInitialized]);

  const setActiveDeviceId = useCallback(
    (deviceId: string | null) => {
      if (deviceId === null) {
        setActiveDeviceIdState(null);
        return;
      }
      if (devices.some((d) => d.id === deviceId)) {
        setActiveDeviceIdState(deviceId);
      }
    },
    [devices],
  );

  const setDevicesWithActiveGuard = useCallback(
    (next: Device[]) => {
      setDevices(next);
      setActiveDeviceIdState((prev) => {
        if (prev && next.some((d) => d.id === prev)) return prev;
        return next.length > 0 ? next[0].id : null;
      });
    },
    [],
  );

  return useMemo(
    () => ({
      isInitialized,
      devices,
      activeDeviceId,
      setActiveDeviceId,
      setDevices: setDevicesWithActiveGuard,
    }),
    [activeDeviceId, devices, isInitialized, setActiveDeviceId, setDevicesWithActiveGuard],
  );
}
