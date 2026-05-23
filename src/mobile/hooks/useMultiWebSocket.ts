import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { Device, DeviceErrorType } from "../types";
import { generateDeviceId, getDefaultDeviceName, MAX_DEVICES } from "../utils/deviceStorage";

const MAX_RECONNECT_ATTEMPTS = 5;
const SEND_TIMEOUT_MS = 3000;
const QUICK_FAIL_THRESHOLD_MS = 1000;
const SLOW_FAIL_THRESHOLD_MS = 5000;

type ConnectionEntry = {
  ws: WebSocket;
  url: string;
  reconnectAttempts: number;
  reconnectTimer: number | null;
  openTimestamp: number;
};

interface UseMultiWebSocketReturn {
  devices: Device[];
  activeDeviceId: string | null;
  isSending: boolean;
  lastError: string | null;
  clearError: () => void;
  addDevice: (url: string, name?: string) => boolean;
  removeDevice: (deviceId: string) => void;
  renameDevice: (deviceId: string, newName: string) => void;
  setActiveDevice: (deviceId: string) => void;
  retryDevice: (deviceId: string) => void;
  sendToActive: (text: string) => boolean;
}

export function useMultiWebSocket(
  devices: Device[],
  setDevices: (devices: Device[]) => void,
  activeDeviceId: string | null,
  setActiveDeviceId: (deviceId: string | null) => void,
): UseMultiWebSocketReturn {
  const connectionsRef = useRef<Map<string, ConnectionEntry>>(new Map());
  const devicesRef = useRef<Device[]>(devices);
  const activeDeviceIdRef = useRef<string | null>(activeDeviceId);

  const [isSending, setIsSending] = useState(false);
  const [lastError, setLastError] = useState<string | null>(null);
  const sendTimeoutRef = useRef<number | null>(null);

  useEffect(() => {
    devicesRef.current = devices;
  }, [devices]);

  useEffect(() => {
    activeDeviceIdRef.current = activeDeviceId;
  }, [activeDeviceId]);

  const clearSendTimer = useCallback(() => {
    if (sendTimeoutRef.current !== null) {
      window.clearTimeout(sendTimeoutRef.current);
      sendTimeoutRef.current = null;
    }
  }, []);

  const clearError = useCallback(() => {
    setLastError(null);
  }, []);

  const updateDevice = useCallback(
    (deviceId: string, patch: Partial<Device>) => {
      setDevices(
        devicesRef.current.map((d) => (d.id === deviceId ? { ...d, ...patch } : d)),
      );
    },
    [setDevices],
  );

  const scheduleReconnect = useCallback(
    (device: Device, prev: ConnectionEntry) => {
      if (prev.reconnectAttempts >= MAX_RECONNECT_ATTEMPTS) {
        updateDevice(device.id, { hasExhaustedRetries: true });
        return;
      }
      const nextAttempts = prev.reconnectAttempts + 1;
      const delay = Math.min(1000 * 2 ** (nextAttempts - 1), 10000);

      if (prev.reconnectTimer !== null) {
        window.clearTimeout(prev.reconnectTimer);
      }

      updateDevice(device.id, { status: "connecting" });

      const timer = window.setTimeout(() => {
        const stillExists = devicesRef.current.some((d) => d.id === device.id);
        if (!stillExists) return;
        connect(device, nextAttempts);
      }, delay);

      connectionsRef.current.set(device.id, {
        ws: prev.ws,
        url: prev.url,
        reconnectAttempts: nextAttempts,
        reconnectTimer: timer,
        openTimestamp: prev.openTimestamp,
      });
    },
    [updateDevice],
  );

  const connect = useCallback(
    (device: Device, reconnectAttempts = 0) => {
      const existing = connectionsRef.current.get(device.id);
      if (existing) {
        try {
          existing.ws.close();
        } catch {}
        if (existing.reconnectTimer !== null) {
          window.clearTimeout(existing.reconnectTimer);
        }
        connectionsRef.current.delete(device.id);
      }

      updateDevice(device.id, { status: "connecting", errorType: undefined });

      try {
        const ws = new WebSocket(device.url);
        const openTimestamp = Date.now();
        connectionsRef.current.set(device.id, {
          ws,
          url: device.url,
          reconnectAttempts,
          reconnectTimer: null,
          openTimestamp,
        });

        ws.onopen = () => {
          connectionsRef.current.set(device.id, {
            ws,
            url: device.url,
            reconnectAttempts: 0,
            reconnectTimer: null,
            openTimestamp: Date.now(),
          });
          updateDevice(device.id, {
            status: "connected",
            lastConnected: Date.now(),
            hasExhaustedRetries: false,
            errorType: undefined,
          });
          clearError();
          setIsSending(false);
          clearSendTimer();
        };

        ws.onmessage = (event) => {
          try {
            const data = JSON.parse(event.data);
            if (data?.type === "confirm") {
              clearSendTimer();
              setIsSending(false);
            } else if (data?.type === "error") {
              clearSendTimer();
              setLastError(typeof data.message === "string" ? data.message : "Error");
              setIsSending(false);
            }
          } catch {}
        };

        ws.onerror = () => {};

        ws.onclose = () => {
          const current = connectionsRef.current.get(device.id);
          if (!current || current.ws !== ws) return;
          connectionsRef.current.delete(device.id);

          const elapsed = Date.now() - current.openTimestamp;
          let errorType: DeviceErrorType = "unreachable";
          if (elapsed < QUICK_FAIL_THRESHOLD_MS) {
            errorType = "refused";
          } else if (elapsed > SLOW_FAIL_THRESHOLD_MS) {
            errorType = "timeout";
          }

          updateDevice(device.id, { status: "disconnected", errorType });
          clearSendTimer();
          setIsSending(false);
          scheduleReconnect(device, { ...current, ws });
        };
      } catch {
        updateDevice(device.id, { status: "disconnected", errorType: "unreachable" });
      }
    },
    [clearError, clearSendTimer, scheduleReconnect, updateDevice],
  );

  useEffect(() => {
    const currentDevices = devices;
    const currentIds = new Set(currentDevices.map((d) => d.id));

    currentDevices.forEach((device) => {
      const existing = connectionsRef.current.get(device.id);
      if (!existing) {
        connect(device, 0);
        return;
      }
      if (existing.url !== device.url) {
        connect(device, existing.reconnectAttempts);
      }
    });

    Array.from(connectionsRef.current.entries()).forEach(([deviceId, conn]) => {
      if (currentIds.has(deviceId)) return;
      try {
        conn.ws.close();
      } catch {}
      if (conn.reconnectTimer !== null) {
        window.clearTimeout(conn.reconnectTimer);
      }
      connectionsRef.current.delete(deviceId);
    });
  }, [connect, devices]);

  useEffect(() => {
    return () => {
      clearSendTimer();
      connectionsRef.current.forEach((conn) => {
        if (conn.reconnectTimer !== null) {
          window.clearTimeout(conn.reconnectTimer);
        }
        try {
          conn.ws.close();
        } catch {}
      });
      connectionsRef.current.clear();
    };
  }, [clearSendTimer]);

  const addDevice = useCallback(
    (url: string, name?: string) => {
      if (devicesRef.current.length >= MAX_DEVICES) {
        setLastError("devices.maxLimit");
        return false;
      }

      if (devicesRef.current.some((d) => d.url === url)) {
        setLastError("devices.alreadyExists");
        return false;
      }

      const device: Device = {
        id: generateDeviceId(),
        name: name?.trim() ? name.trim() : getDefaultDeviceName(url),
        url,
        status: "connecting",
        lastConnected: 0,
      };

      setDevices([...devicesRef.current, device]);
      setActiveDeviceId(device.id);
      return true;
    },
    [setActiveDeviceId, setDevices],
  );

  const removeDevice = useCallback(
    (deviceId: string) => {
      const existing = connectionsRef.current.get(deviceId);
      if (existing) {
        if (existing.reconnectTimer !== null) {
          window.clearTimeout(existing.reconnectTimer);
        }
        try {
          existing.ws.close();
        } catch {}
        connectionsRef.current.delete(deviceId);
      }

      const next = devicesRef.current.filter((d) => d.id !== deviceId);
      setDevices(next);

      if (activeDeviceIdRef.current === deviceId) {
        setActiveDeviceId(next.length > 0 ? next[0].id : null);
      }
    },
    [setActiveDeviceId, setDevices],
  );

  const renameDevice = useCallback(
    (deviceId: string, newName: string) => {
      const trimmed = newName.trim();
      if (!trimmed) return;
      setDevices(
        devicesRef.current.map((d) => (d.id === deviceId ? { ...d, name: trimmed } : d)),
      );
    },
    [setDevices],
  );

  const setActiveDevice = useCallback(
    (deviceId: string) => {
      if (!devicesRef.current.some((d) => d.id === deviceId)) return;
      setActiveDeviceId(deviceId);
    },
    [setActiveDeviceId],
  );

  const retryDevice = useCallback(
    (deviceId: string) => {
      const device = devicesRef.current.find((d) => d.id === deviceId);
      if (!device) return;
      updateDevice(deviceId, { hasExhaustedRetries: false, errorType: undefined });
      connect(device, 0);
    },
    [connect, updateDevice],
  );

  const sendToActive = useCallback(
    (text: string) => {
      const deviceId = activeDeviceIdRef.current;
      if (!deviceId) return false;
      const conn = connectionsRef.current.get(deviceId);
      if (!conn || conn.ws.readyState !== WebSocket.OPEN) return false;
      if (isSending) return false;

      setIsSending(true);
      clearError();
      clearSendTimer();

      try {
        conn.ws.send(JSON.stringify({ type: "text", content: text }));
        sendTimeoutRef.current = window.setTimeout(() => {
          setIsSending(false);
          setLastError("devices.sendTimeout");
        }, SEND_TIMEOUT_MS);
        return true;
      } catch {
        clearSendTimer();
        setIsSending(false);
        return false;
      }
    },
    [clearError, clearSendTimer, isSending],
  );

  return useMemo(
    () => ({
      devices,
      activeDeviceId,
      isSending,
      lastError,
      clearError,
      addDevice,
      removeDevice,
      renameDevice,
      setActiveDevice,
      retryDevice,
      sendToActive,
    }),
    [
      activeDeviceId,
      addDevice,
      clearError,
      devices,
      isSending,
      lastError,
      removeDevice,
      renameDevice,
      sendToActive,
      setActiveDevice,
      retryDevice,
    ],
  );
}
