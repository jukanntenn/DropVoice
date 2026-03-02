# Multi-Device Connection and Switching Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enable mobile phone to connect to 3-5 PCs simultaneously via multiple WebSocket connections with card/tab UI for device switching.

**Architecture:** Mobile-side only changes. New `MultiWebSocketManager` class manages multiple WebSocket connections. Device list persisted to localStorage with auto-reconnect. Card/tab UI at top displays device status integrated.

**Tech Stack:** React 18, TypeScript, Vite, html5-qrcode (QR scanning), localStorage, WebSocket API

---

## Overview

This plan implements multi-device connection feature in **12 tasks**:

1. Task 1: Install QR scanning dependency
2. Task 2: Define core data types
3. Task 3: Create device storage utilities
4. Task 4: Create multi-WebSocket manager hook
5. Task 5: Create device manager hook
6. Task 6: Create DeviceCard component
7. Task 7: Create DeviceTabs component
8. Task 8: Create AddDeviceModal component
9. Task 9: Create QRScanner component
10. Task 10: Create DeviceNameEditor component
11. Task 11: Update App.tsx with device tabs
12. Task 12: Add internationalization strings

**No backend changes required.**

---

## Task 1: Install QR Scanning Dependency

**Files:**
- Modify: `package.json`

**Step 1: Install html5-qrcode package**

Run:
```bash
cd C:/Users/Administrator/Workspace/dropvoice
pnpm add html5-qrcode
```

Expected: Package added to dependencies

**Step 2: Verify installation**

Run:
```bash
cat package.json | grep "html5-qrcode"
```

Expected: `"html5-qrcode": "^2.3.8"` in dependencies

**Step 3: Commit**

```bash
git add package.json pnpm-lock.yaml
git commit -m "feat(mobile): add html5-qrcode for QR scanning"
```

---

## Task 2: Define Core Data Types

**Files:**
- Create: `src/mobile/types/device.ts`

**Step 1: Create device types file**

```typescript
// src/mobile/types/device.ts

export type DeviceStatus = 'connected' | 'disconnected' | 'connecting';

export interface Device {
  id: string;
  name: string;
  url: string;
  status: DeviceStatus;
  lastConnected: number;
}

export interface StoredDevice {
  id: string;
  name: string;
  url: string;
  lastConnected: number;
}

export interface DeviceStorage {
  devices: StoredDevice[];
  lastActiveDeviceId: string | null;
}

export interface WebSocketConnection {
  ws: WebSocket | null;
  device: Device;
  status: DeviceStatus;
  reconnectAttempts: number;
}

export interface MultiWebSocketState {
  devices: Device[];
  activeDeviceId: string | null;
}
```

**Step 2: Create barrel export**

Create: `src/mobile/types/index.ts`

```typescript
export * from './device';
```

**Step 3: Commit**

```bash
git add src/mobile/types/
git commit -m "feat(mobile): define core device data types"
```

---

## Task 3: Create Device Storage Utilities

**Files:**
- Create: `src/mobile/utils/deviceStorage.ts`

**Step 1: Write storage utilities**

```typescript
// src/mobile/utils/deviceStorage.ts

import { Device, DeviceStorage, StoredDevice } from '../types';

const STORAGE_KEY = 'dropvoice_devices';

export function loadDeviceStorage(): DeviceStorage {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      return { devices: [], lastActiveDeviceId: null };
    }
    return JSON.parse(raw) as DeviceStorage;
  } catch {
    return { devices: [], lastActiveDeviceId: null };
  }
}

export function saveDeviceStorage(storage: DeviceStorage): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(storage));
  } catch (error) {
    console.error('Failed to save device storage:', error);
  }
}

export function storedToDevice(stored: StoredDevice): Device {
  return {
    ...stored,
    status: 'disconnected',
  };
}

export function generateDeviceId(): string {
  return `device_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
}

export function getDefaultDeviceName(url: string): string {
  try {
    const urlObj = new URL(url);
    const hostname = urlObj.hostname;
    const lastOctet = hostname.split('.').pop();
    return `PC-${lastOctet}`;
  } catch {
    return 'Unknown Device';
  }
}

export function isValidWebSocketUrl(url: string): boolean {
  try {
    const parsed = new URL(url);
    return parsed.protocol === 'ws:' || parsed.protocol === 'wss:';
  } catch {
    return false;
  }
}

export function MAX_DEVICES(): number {
  return 5;
}
```

**Step 2: Create test file**

Create: `src/mobile/utils/__tests__/deviceStorage.test.ts`

```typescript
import { generateDeviceId, getDefaultDeviceName, isValidWebSocketUrl, loadDeviceStorage, saveDeviceStorage, MAX_DEVICES } from '../deviceStorage';

describe('deviceStorage', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  describe('generateDeviceId', () => {
    it('should generate unique IDs', () => {
      const id1 = generateDeviceId();
      const id2 = generateDeviceId();
      expect(id1).not.toBe(id2);
      expect(id1).toMatch(/^device_\d+_[a-z0-9]+$/);
    });
  });

  describe('getDefaultDeviceName', () => {
    it('should extract last IP octet', () => {
      expect(getDefaultDeviceName('ws://192.168.1.100:38425/ws')).toBe('PC-100');
      expect(getDefaultDeviceName('ws://192.168.1.5:38425/ws')).toBe('PC-5');
    });

    it('should handle invalid URLs', () => {
      expect(getDefaultDeviceName('invalid-url')).toBe('Unknown Device');
    });
  });

  describe('isValidWebSocketUrl', () => {
    it('should validate WebSocket URLs', () => {
      expect(isValidWebSocketUrl('ws://192.168.1.100:38425/ws')).toBe(true);
      expect(isValidWebSocketUrl('wss://example.com/ws')).toBe(true);
      expect(isValidWebSocketUrl('http://example.com')).toBe(false);
      expect(isValidWebSocketUrl('invalid')).toBe(false);
    });
  });

  describe('load/save DeviceStorage', () => {
    it('should save and load device storage', () => {
      const storage = {
        devices: [{ id: '1', name: 'PC-100', url: 'ws://192.168.1.100:38425/ws', lastConnected: 123 }],
        lastActiveDeviceId: '1'
      };
      saveDeviceStorage(storage);
      expect(loadDeviceStorage()).toEqual(storage);
    });

    it('should return empty storage when nothing saved', () => {
      expect(loadDeviceStorage()).toEqual({ devices: [], lastActiveDeviceId: null });
    });
  });

  describe('MAX_DEVICES', () => {
    it('should return 5', () => {
      expect(MAX_DEVICES()).toBe(5);
    });
  });
});
```

**Step 3: Run tests**

Run:
```bash
cd C:/Users/Administrator/Workspace/dropvoice
pnpm test src/mobile/utils/__tests__/deviceStorage.test.ts
```

Expected: All tests pass

**Step 4: Commit**

```bash
git add src/mobile/utils/
git commit -m "feat(mobile): add device storage utilities with tests"
```

---

## Task 4: Create Multi-WebSocket Manager Hook

**Files:**
- Create: `src/mobile/hooks/useMultiWebSocket.ts`

**Step 1: Create the hook**

```typescript
// src/mobile/hooks/useMultiWebSocket.ts

import { useCallback, useEffect, useRef, useState } from 'react';
import { Device, DeviceStatus, WebSocketConnection } from '../types';

const MAX_RECONNECT_ATTEMPTS = 5;
const SEND_TIMEOUT = 3000;

export interface UseMultiWebSocketReturn {
  devices: Device[];
  activeDeviceId: string | null;
  addDevice: (url: string, name?: string) => Promise<boolean>;
  removeDevice: (deviceId: string) => void;
  setActiveDevice: (deviceId: string) => void;
  renameDevice: (deviceId: string, newName: string) => void;
  sendToActive: (text: string) => Promise<boolean>;
  isSending: boolean;
  lastError: string | null;
  clearError: () => void;
}

export function useMultiWebSocket(
  initialDevices: Device[],
  onDevicesChange: (devices: Device[]) => void,
  initialActiveDeviceId: string | null
): UseMultiWebSocketReturn {
  const [devices, setDevices] = useState<Device[]>(initialDevices);
  const [activeDeviceId, setActiveDeviceId] = useState<string | null>(initialActiveDeviceId);
  const [isSending, setIsSending] = useState(false);
  const [lastError, setLastError] = useState<string | null>(null);

  const connectionsRef = useRef<Map<string, WebSocketConnection>>(new Map());
  const sendTimeoutRef = useRef<number | null>(null);

  // Clear send timeout
  const clearSendTimer = useCallback(() => {
    if (sendTimeoutRef.current !== null) {
      window.clearTimeout(sendTimeoutRef.current);
      sendTimeoutRef.current = null;
    }
  }, []);

  // Update device status
  const updateDeviceStatus = useCallback((deviceId: string, status: DeviceStatus) => {
    setDevices(prev => {
      const updated = prev.map(d =>
        d.id === deviceId ? { ...d, status } : d
      );
      onDevicesChange(updated);
      return updated;
    });
  }, [onDevicesChange]);

  // Connect to a device
  const connectToDevice = useCallback((device: Device) => {
    if (connectionsRef.current.has(device.id)) {
      return;
    }

    try {
      const ws = new WebSocket(device.url);
      const conn: WebSocketConnection = {
        ws,
        device,
        status: 'connecting',
        reconnectAttempts: 0,
      };
      connectionsRef.current.set(device.id, conn);

      ws.onopen = () => {
        const conn = connectionsRef.current.get(device.id);
        if (conn) {
          conn.status = 'connected';
          conn.reconnectAttempts = 0;
          updateDeviceStatus(device.id, 'connected');
        }
      };

      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          if (data?.type === 'confirm') {
            clearSendTimer();
            setIsSending(false);
          } else if (data?.type === 'error') {
            clearSendTimer();
            setLastError(typeof data.message === 'string' ? data.message : 'Error');
            setIsSending(false);
          }
        } catch {}
      };

      ws.onerror = () => {
        // Let onclose handle errors
      };

      ws.onclose = () => {
        const conn = connectionsRef.current.get(device.id);
        if (!conn) return;

        connectionsRef.current.delete(device.id);
        clearSendTimer();
        setIsSending(false);
        updateDeviceStatus(device.id, 'disconnected');

        // Auto-reconnect with exponential backoff
        if (conn.reconnectAttempts < MAX_RECONNECT_ATTEMPTS) {
          conn.reconnectAttempts++;
          const delay = Math.min(1000 * 2 ** (conn.reconnectAttempts - 1), 10000);
          setTimeout(() => {
            if (devices.find(d => d.id === device.id)) {
              connectToDevice(device);
            }
          }, delay);
        }
      };
    } catch (error) {
      updateDeviceStatus(device.id, 'disconnected');
    }
  }, [devices, updateDeviceStatus, clearSendTimer]);

  // Connect to initial devices on mount
  useEffect(() => {
    initialDevices.forEach(device => {
      connectToDevice(device);
    });

    return () => {
      connectionsRef.current.forEach(conn => {
        try {
          conn.ws?.close();
        } catch {}
      });
      connectionsRef.current.clear();
    };
  }, []); // Run once on mount

  // Add new device
  const addDevice = useCallback(async (url: string, name?: string): Promise<boolean> => {
    // Check max devices
    if (devices.length >= 5) {
      setLastError('最多支持 5 台设备');
      return false;
    }

    // Check for duplicate
    const exists = devices.some(d => d.url === url);
    if (exists) {
      setLastError('此设备已存在');
      return false;
    }

    // Create new device
    const newDevice: Device = {
      id: `device_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`,
      name: name || getDefaultDeviceNameFromUrl(url),
      url,
      status: 'connecting',
      lastConnected: Date.now(),
    };

    setDevices(prev => {
      const updated = [...prev, newDevice];
      onDevicesChange(updated);
      return updated;
    });

    // Connect
    connectToDevice(newDevice);

    // Set as active if first device
    if (devices.length === 0) {
      setActiveDeviceId(newDevice.id);
    }

    return true;
  }, [devices, onDevicesChange, connectToDevice]);

  // Remove device
  const removeDevice = useCallback((deviceId: string) => {
    const conn = connectionsRef.current.get(deviceId);
    if (conn) {
      try {
        conn.ws?.close();
      } catch {}
      connectionsRef.current.delete(deviceId);
    }

    setDevices(prev => {
      const updated = prev.filter(d => d.id !== deviceId);
      onDevicesChange(updated);

      // Set new active if removed was active
      if (activeDeviceId === deviceId && updated.length > 0) {
        setActiveDeviceId(updated[0].id);
      } else if (updated.length === 0) {
        setActiveDeviceId(null);
      }

      return updated;
    });
  }, [activeDeviceId, onDevicesChange]);

  // Set active device
  const setActiveDevice = useCallback((deviceId: string) => {
    if (devices.find(d => d.id === deviceId)) {
      setActiveDeviceId(deviceId);
    }
  }, [devices]);

  // Rename device
  const renameDevice = useCallback((deviceId: string, newName: string) => {
    setDevices(prev => {
      const updated = prev.map(d =>
        d.id === deviceId ? { ...d, name: newName } : d
      );
      onDevicesChange(updated);
      return updated;
    });
  }, [onDevicesChange]);

  // Send to active device
  const sendToActive = useCallback(async (text: string): Promise<boolean> => {
    if (!activeDeviceId) {
      setLastError('请先选择设备');
      return false;
    }

    const conn = connectionsRef.current.get(activeDeviceId);
    if (!conn || !conn.ws || conn.ws.readyState !== WebSocket.OPEN) {
      setLastError('设备未连接');
      return false;
    }

    if (isSending) {
      return false;
    }

    setIsSending(true);
    setLastError(null);

    try {
      conn.ws.send(JSON.stringify({ type: 'text', content: text }));
      sendTimeoutRef.current = window.setTimeout(() => {
        setIsSending(false);
        setLastError('发送超时');
      }, SEND_TIMEOUT);
      return true;
    } catch (error) {
      clearSendTimer();
      setLastError('发送失败');
      setIsSending(false);
      return false;
    }
  }, [activeDeviceId, isSending, clearSendTimer]);

  // Clear error
  const clearError = useCallback(() => {
    setLastError(null);
  }, []);

  return {
    devices,
    activeDeviceId,
    addDevice,
    removeDevice,
    setActiveDevice,
    renameDevice,
    sendToActive,
    isSending,
    lastError,
    clearError,
  };
}

function getDefaultDeviceNameFromUrl(url: string): string {
  try {
    const urlObj = new URL(url);
    const hostname = urlObj.hostname;
    const lastOctet = hostname.split('.').pop();
    return `PC-${lastOctet}`;
  } catch {
    return 'Unknown Device';
  }
}
```

**Step 2: Commit**

```bash
git add src/mobile/hooks/useMultiWebSocket.ts
git commit -m "feat(mobile): create multi-WebSocket manager hook"
```

---

## Task 5: Create Device Manager Hook

**Files:**
- Create: `src/mobile/hooks/useDeviceManager.ts`

**Step 1: Create the hook**

```typescript
// src/mobile/hooks/useDeviceManager.ts

import { useCallback, useEffect, useState } from 'react';
import { Device } from '../types';
import {
  loadDeviceStorage,
  saveDeviceStorage,
  storedToDevice,
  generateDeviceId,
  getDefaultDeviceName,
  isValidWebSocketUrl,
  MAX_DEVICES,
} from '../utils/deviceStorage';

export function useDeviceManager() {
  const [isInitialized, setIsInitialized] = useState(false);
  const [devices, setDevices] = useState<Device[]>([]);
  const [activeDeviceId, setActiveDeviceId] = useState<string | null>(null);

  // Load devices from storage on mount
  useEffect(() => {
    const storage = loadDeviceStorage();
    const loadedDevices = storage.devices.map(storedToDevice);
    setDevices(loadedDevices);
    setActiveDeviceId(storage.lastActiveDeviceId);
    setIsInitialized(true);
  }, []);

  // Save devices to storage when they change
  useEffect(() => {
    if (!isInitialized) return;

    const storage = {
      devices: devices.map(({ status, ...rest }) => rest),
      lastActiveDeviceId: activeDeviceId,
    };
    saveDeviceStorage(storage);
  }, [devices, activeDeviceId, isInitialized]);

  const updateDevices = useCallback((newDevices: Device[]) => {
    setDevices(newDevices);
  }, []);

  return {
    devices,
    activeDeviceId,
    setActiveDeviceId,
    updateDevices,
    isInitialized,
  };
}
```

**Step 2: Commit**

```bash
git add src/mobile/hooks/useDeviceManager.ts
git commit -m "feat(mobile): create device manager hook"
```

---

## Task 6: Create DeviceCard Component

**Files:**
- Create: `src/mobile/components/DeviceCard.tsx`

**Step 1: Create DeviceCard component**

```typescript
// src/mobile/components/DeviceCard.tsx

import type React from 'react';
import { DeviceStatus } from '../types';

interface DeviceCardProps {
  name: string;
  status: DeviceStatus;
  isActive: boolean;
  onClick: () => void;
  onRemove: () => void;
  onRename?: () => void;
}

const statusIcons: Record<DeviceStatus, string> = {
  connected: '●',
  disconnected: '○',
  connecting: '⟳',
};

const statusColors: Record<DeviceStatus, string> = {
  connected: 'text-green-500',
  disconnected: 'text-gray-400',
  connecting: 'text-yellow-500 animate-spin',
};

export function DeviceCard({
  name,
  status,
  isActive,
  onClick,
  onRemove,
  onRename,
}: DeviceCardProps) {
  const handleLongPress = () => {
    if (onRename) {
      onRename();
    }
  };

  let longPressTimer: number | null = null;
  let longPressTriggered = false;

  const onTouchStart = () => {
    longPressTriggered = false;
    longPressTimer = window.setTimeout(() => {
      longPressTriggered = true;
      handleLongPress();
    }, 500);
  };

  const onTouchEnd = () => {
    if (longPressTimer !== null) {
      window.clearTimeout(longPressTimer);
      longPressTimer = null;
    }
  };

  const handleClick = () => {
    if (!longPressTriggered) {
      onClick();
    }
  };

  const activeClasses = isActive
    ? 'bg-primary/20 border-primary shadow-md'
    : 'bg-white/60 dark:bg-slate-800/60 border-gray-200/80 dark:border-slate-700/80';

  const statusIcon = statusIcons[status];
  const statusColor = statusColors[status];

  return (
    <div
      className={`
        relative flex items-center gap-2 px-3 py-2 rounded-xl border-2
        transition-all duration-200 cursor-pointer
        ${activeClasses}
        ${status === 'connecting' ? 'animate-pulse' : ''}
      `}
      onClick={handleClick}
      onTouchStart={onTouchStart}
      onTouchEnd={onTouchEnd}
    >
      {/* Status icon */}
      <span className={`text-sm ${statusColor} ${status === 'connecting' ? 'animate-spin' : ''}`}>
        {statusIcon}
      </span>

      {/* Device name */}
      <span className="text-sm font-medium truncate max-w-[120px] dark:text-white">
        {name}
      </span>

      {/* Remove button */}
      <button
        onClick={(e) => {
          e.stopPropagation();
          onRemove();
        }}
        className="ml-1 text-gray-400 hover:text-red-500 dark:text-gray-500 dark:hover:text-red-400 transition-colors"
      >
        <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
          <path fillRule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clipRule="evenodd" />
        </svg>
      </button>
    </div>
  );
}
```

**Step 2: Commit**

```bash
git add src/mobile/components/DeviceCard.tsx
git commit -m "feat(mobile): create DeviceCard component"
```

---

## Task 7: Create DeviceTabs Component

**Files:**
- Create: `src/mobile/components/DeviceTabs.tsx`

**Step 1: Create DeviceTabs component**

```typescript
// src/mobile/components/DeviceTabs.tsx

import type React from 'react';
import { DeviceCard } from './DeviceCard';
import { Device, DeviceStatus } from '../types';

interface DeviceTabsProps {
  devices: Device[];
  activeDeviceId: string | null;
  onDeviceSelect: (deviceId: string) => void;
  onAddDevice: () => void;
  onRemoveDevice: (deviceId: string) => void;
  onRenameDevice: (deviceId: string) => void;
}

export function DeviceTabs({
  devices,
  activeDeviceId,
  onDeviceSelect,
  onAddDevice,
  onRemoveDevice,
  onRenameDevice,
}: DeviceTabsProps) {
  return (
    <div className="sticky top-0 z-10 bg-gradient-to-br from-teal-50 via-white to-cyan-50/95 dark:from-slate-950 dark:via-slate-900 dark:to-slate-950/95 backdrop-blur-sm px-2 pt-2 pb-4">
      {/* Horizontal scrollable container */}
      <div className="flex items-center gap-2 overflow-x-auto scrollbar-hide">
        {/* Device cards */}
        {devices.map((device) => (
          <DeviceCard
            key={device.id}
            name={device.name}
            status={device.status}
            isActive={device.id === activeDeviceId}
            onClick={() => onDeviceSelect(device.id)}
            onRemove={() => onRemoveDevice(device.id)}
            onRename={() => onRenameDevice(device.id)}
          />
        ))}

        {/* Add device button */}
        <button
          onClick={onAddDevice}
          className="
            flex items-center gap-1 px-3 py-2 rounded-xl border-2 border-dashed
            border-gray-300/80 dark:border-slate-700/80
            bg-white/40 dark:bg-slate-800/40
            text-gray-500 dark:text-gray-400
            hover:border-primary/50 hover:bg-primary/10
            transition-all duration-200
            text-sm font-medium
          "
        >
          <svg xmlns="http://www.w3.org/2000/svg" className="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
            <path fillRule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clipRule="evenodd" />
          </svg>
          <span>添加</span>
        </button>
      </div>

      {/* Empty state */}
      {devices.length === 0 && (
        <div className="text-center py-8">
          <p className="text-sm text-muted-foreground dark:text-slate-400">
            暂无设备，点击右上角添加
          </p>
        </div>
      )}
    </div>
  );
}
```

**Step 2: Add scrollbar-hide utility**

Create: `src/mobile/index.css` (if not exists, add to existing)

```css
.scrollbar-hide {
  -ms-overflow-style: none;
  scrollbar-width: none;
}

.scrollbar-hide::-webkit-scrollbar {
  display: none;
}
```

**Step 3: Import in mobile index**

Modify: `src/mobile/index.tsx`

```typescript
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "../index.css";
import "./index.css"; // Add this line

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
```

**Step 4: Commit**

```bash
git add src/mobile/components/DeviceTabs.tsx src/mobile/index.css src/mobile/index.tsx
git commit -m "feat(mobile): create DeviceTabs component with horizontal scroll"
```

---

## Task 8: Create AddDeviceModal Component

**Files:**
- Create: `src/mobile/components/AddDeviceModal.tsx`

**Step 1: Create AddDeviceModal component**

```typescript
// src/mobile/components/AddDeviceModal.tsx

import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Html5QrcodeScanner } from 'html5-qrcode';
import { isValidWebSocketUrl, getDefaultDeviceName } from '../utils/deviceStorage';

interface AddDeviceModalProps {
  isOpen: boolean;
  onClose: () => void;
  onAdd: (url: string, name?: string) => Promise<boolean>;
}

type Tab = 'scan' | 'input';

export function AddDeviceModal({ isOpen, onClose, onAdd }: AddDeviceModalProps) {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState<Tab>('scan');
  const [urlInput, setUrlInput] = useState('');
  const [nameInput, setNameInput] = useState('');
  const [isScanning, setIsScanning] = useState(false);
  const [error, setError] = useState<string | null>(null);

  if (!isOpen) return null;

  const handleUrlSubmit = async () => {
    setError(null);

    if (!urlInput.trim()) {
      setError('请输入连接地址');
      return;
    }

    if (!isValidWebSocketUrl(urlInput)) {
      setError('无效的连接地址，格式应为 ws:// 或 wss://');
      return;
    }

    const success = await onAdd(urlInput, nameInput.trim() || undefined);
    if (success) {
      setUrlInput('');
      setNameInput('');
      onClose();
    } else {
      setError('添加设备失败');
    }
  };

  const handleScanSuccess = async (decodedText: string) => {
    setError(null);
    setIsScanning(false);

    // QR code contains HTTP URL, convert to WebSocket
    let wsUrl = decodedText;
    if (decodedText.startsWith('http://')) {
      wsUrl = decodedText.replace('http://', 'ws://') + '/ws';
    } else if (decodedText.startsWith('https://')) {
      wsUrl = decodedText.replace('https://', 'wss://') + '/ws';
    }

    const success = await onAdd(wsUrl);
    if (!success) {
      setError('添加设备失败');
    }
  };

  const handleScanError = (errorMessage: string) => {
    // Ignore scan errors during normal operation
    console.warn('Scan error:', errorMessage);
  };

  const startScan = () => {
    setIsScanning(true);
    setError(null);

    const scanner = new Html5QrcodeScanner('qr-reader', {
      fps: 10,
      qrbox: { width: 250, height: 250 },
      aspectRatio: 1.0,
    });

    scanner.render(handleScanSuccess, handleScanError);
  };

  const stopScan = () => {
    setIsScanning(false);
    // Scanner cleanup happens when component unmounts
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm px-4">
      <div className="bg-white dark:bg-slate-900 rounded-2xl shadow-xl w-full max-w-md max-h-[80vh] overflow-hidden flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-slate-800">
          <h2 className="text-lg font-semibold dark:text-white">添加设备</h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
          >
            <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        {/* Tabs */}
        <div className="flex border-b border-gray-200 dark:border-slate-800">
          <button
            onClick={() => setActiveTab('scan')}
            className={`flex-1 py-3 text-sm font-medium transition-colors ${
              activeTab === 'scan'
                ? 'text-primary border-b-2 border-primary'
                : 'text-gray-500 dark:text-gray-400'
            }`}
          >
            扫描二维码
          </button>
          <button
            onClick={() => setActiveTab('input')}
            className={`flex-1 py-3 text-sm font-medium transition-colors ${
              activeTab === 'input'
                ? 'text-primary border-b-2 border-primary'
                : 'text-gray-500 dark:text-gray-400'
            }`}
          >
            输入地址
          </button>
        </div>

        {/* Content */}
        <div className="p-4 overflow-y-auto">
          {activeTab === 'scan' ? (
            <div className="space-y-4">
              {!isScanning ? (
                <>
                  <div className="text-center py-8">
                    <svg xmlns="http://www.w3.org/2000/svg" className="h-16 w-16 mx-auto text-gray-400 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 4v1m6 11h2m-6 0h-2v4m0-11v3m0 0h.01M12 12h4.01M16 20h4M4 12h4m12 0h.01M5 8h2a1 1 0 001-1V5a1 1 0 00-1-1H5a1 1 0 00-1 1v2a1 1 0 001 1zm12 0h2a1 1 0 001-1V5a1 1 0 00-1-1h-2a1 1 0 00-1 1v2a1 1 0 001 1zM5 20h2a1 1 0 001-1v-2a1 1 0 00-1-1H5a1 1 0 00-1 1v2a1 1 0 001 1z" />
                    </svg>
                    <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
                      点击下方按钮开始扫描
                    </p>
                  </div>
                  <button
                    onClick={startScan}
                    className="w-full py-3 bg-primary text-white rounded-xl font-medium hover:bg-primary/90 transition-colors"
                  >
                    开始扫描
                  </button>
                </>
              ) : (
                <>
                  <div id="qr-reader" className="rounded-xl overflow-hidden" />
                  <button
                    onClick={stopScan}
                    className="w-full py-3 bg-gray-200 dark:bg-slate-800 text-gray-700 dark:text-gray-300 rounded-xl font-medium hover:bg-gray-300 dark:hover:bg-slate-700 transition-colors"
                  >
                    停止扫描
                  </button>
                </>
              )}
            </div>
          ) : (
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  连接地址
                </label>
                <input
                  type="text"
                  value={urlInput}
                  onChange={(e) => setUrlInput(e.target.value)}
                  placeholder="ws://192.168.1.xxx:38425/ws"
                  className="w-full px-4 py-3 rounded-xl border border-gray-300 dark:border-slate-700 bg-white dark:bg-slate-800 dark:text-white focus:outline-none focus:ring-2 focus:ring-primary/50"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  设备名称（可选）
                </label>
                <input
                  type="text"
                  value={nameInput}
                  onChange={(e) => setNameInput(e.target.value)}
                  placeholder={`默认: PC-xxx`}
                  className="w-full px-4 py-3 rounded-xl border border-gray-300 dark:border-slate-700 bg-white dark:bg-slate-800 dark:text-white focus:outline-none focus:ring-2 focus:ring-primary/50"
                />
              </div>
              <button
                onClick={handleUrlSubmit}
                className="w-full py-3 bg-primary text-white rounded-xl font-medium hover:bg-primary/90 transition-colors"
              >
                添加设备
              </button>
            </div>
          )}

          {/* Error message */}
          {error && (
            <div className="mt-4 p-3 bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300 rounded-xl text-sm">
              {error}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
```

**Step 2: Commit**

```bash
git add src/mobile/components/AddDeviceModal.tsx
git commit -m "feat(mobile): create AddDeviceModal with QR scan and URL input"
```

---

## Task 9: Create QRScanner Component (Simplified)

**Note:** QR scanning is integrated into AddDeviceModal, so this is a lightweight wrapper for future use.

**Files:**
- Create: `src/mobile/components/QRScanner.tsx`

**Step 1: Create lightweight QRScanner component**

```typescript
// src/mobile/components/QRScanner.tsx

import { useEffect, useRef } from 'react';
import { Html5Qrcode } from 'html5-qrcode';

interface QRScannerProps {
  onScan: (text: string) => void;
  onError?: (error: string) => void;
}

export function QRScanner({ onScan, onError }: QRScannerProps) {
  const scannerRef = useRef<Html5Qrcode | null>(null);

  useEffect(() => {
    const scanner = new Html5Qrcode('qr-scanner');
    scannerRef.current = scanner;

    const config = {
      fps: 10,
      qrbox: { width: 250, height: 250 },
    };

    scanner.start(
      { facingMode: 'environment' },
      config,
      (decodedText) => {
        onScan(decodedText);
        scanner.stop();
      },
      (error) => {
        onError?.(error);
      }
    );

    return () => {
      scanner.stop().catch(console.error);
    };
  }, [onScan, onError]);

  return (
    <div
      id="qr-scanner"
      className="w-full aspect-square bg-black rounded-xl overflow-hidden"
    />
  );
}
```

**Step 2: Commit**

```bash
git add src/mobile/components/QRScanner.tsx
git commit -m "feat(mobile): create lightweight QRScanner component"
```

---

## Task 10: Create DeviceNameEditor Component

**Files:**
- Create: `src/mobile/components/DeviceNameEditor.tsx`

**Step 1: Create DeviceNameEditor component**

```typescript
// src/mobile/components/DeviceNameEditor.tsx

import { useState, useEffect } from 'react';

interface DeviceNameEditorProps {
  isOpen: boolean;
  currentName: string;
  onSave: (newName: string) => void;
  onClose: () => void;
}

export function DeviceNameEditor({ isOpen, currentName, onSave, onClose }: DeviceNameEditorProps) {
  const [name, setName] = useState(currentName);

  useEffect(() => {
    setName(currentName);
  }, [currentName]);

  if (!isOpen) return null;

  const handleSave = () => {
    const trimmed = name.trim();
    if (trimmed) {
      onSave(trimmed);
      onClose();
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      handleSave();
    } else if (e.key === 'Escape') {
      onClose();
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm px-4">
      <div className="bg-white dark:bg-slate-900 rounded-2xl shadow-xl w-full max-w-sm p-6">
        <h2 className="text-lg font-semibold mb-4 dark:text-white">重命名设备</h2>

        <input
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="设备名称"
          autoFocus
          className="w-full px-4 py-3 rounded-xl border border-gray-300 dark:border-slate-700 bg-white dark:bg-slate-800 dark:text-white focus:outline-none focus:ring-2 focus:ring-primary/50 mb-4"
        />

        <div className="flex gap-3">
          <button
            onClick={onClose}
            className="flex-1 py-3 bg-gray-200 dark:bg-slate-800 text-gray-700 dark:text-gray-300 rounded-xl font-medium hover:bg-gray-300 dark:hover:bg-slate-700 transition-colors"
          >
            取消
          </button>
          <button
            onClick={handleSave}
            disabled={!name.trim()}
            className="flex-1 py-3 bg-primary text-white rounded-xl font-medium hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            保存
          </button>
        </div>
      </div>
    </div>
  );
}
```

**Step 2: Commit**

```bash
git add src/mobile/components/DeviceNameEditor.tsx
git commit -m "feat(mobile): create DeviceNameEditor component"
```

---

## Task 11: Update App.tsx with Device Tabs

**Files:**
- Modify: `src/mobile/App.tsx`

**Step 1: Update App.tsx to integrate device tabs**

```typescript
// src/mobile/App.tsx

import type React from "react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { DeviceTabs } from "./components/DeviceTabs";
import { AddDeviceModal } from "./components/AddDeviceModal";
import { DeviceNameEditor } from "./components/DeviceNameEditor";
import { TextInput } from "./components/TextInput";
import { SendButton } from "./components/SendButton";
import { RestoreButton } from "./components/RestoreButton";
import { ClearButton } from "./components/ClearButton";
import { SettingsPage } from "./components/SettingsPage";
import { Toast, useToast } from "./components/Toast";
import { useMultiWebSocket } from "./hooks/useMultiWebSocket";
import { useDeviceManager } from "./hooks/useDeviceManager";
import {
  clearDraft,
  hasLastSent,
  loadDraft,
  loadLastSent,
  saveDraft,
  saveLastSent,
} from "./utils/storage";
import { initMobileI18n } from "./i18n";

type Page = "main" | "settings";

export default function App() {
  const [page, setPage] = useState<Page>("main");

  const [i18nReady, setI18nReady] = useState(false);
  const [text, setText] = useState("");
  const [showRestore, setShowRestore] = useState(false);

  const [showAddDevice, setShowAddDevice] = useState(false);
  const [showRenameEditor, setShowRenameEditor] = useState(false);
  const [renameDeviceId, setRenameDeviceId] = useState<string | null>(null);

  const { t } = useTranslation();
  const { toasts, showToast, removeToast } = useToast();

  // Device manager
  const {
    devices: managedDevices,
    activeDeviceId: managedActiveId,
    setActiveDeviceId,
    updateDevices,
    isInitialized,
  } = useDeviceManager();

  // Multi WebSocket manager
  const {
    devices,
    activeDeviceId,
    addDevice,
    removeDevice,
    setActiveDevice,
    renameDevice,
    sendToActive,
    isSending,
    lastError,
    clearError,
  } = useMultiWebSocket(managedDevices, updateDevices, managedActiveId);

  useEffect(() => {
    initMobileI18n().then(() => setI18nReady(true));
  }, []);

  useEffect(() => {
    const draft = loadDraft();
    if (draft) setText(draft);
    setShowRestore(hasLastSent());
  }, []);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      saveDraft(text);
    }, 500);
    return () => window.clearTimeout(timer);
  }, [text]);

  useEffect(() => {
    if (!lastError) return;
    showToast(lastError, "error");
    clearError();
  }, [lastError, showToast, clearError]);

  const canSend = useMemo(() => {
    return (
      activeDeviceId !== null &&
      devices.some(d => d.id === activeDeviceId && d.status === 'connected') &&
      !isSending &&
      text.trim().length > 0 &&
      text.length <= 10000
    );
  }, [activeDeviceId, devices, isSending, text]);

  const canRestore = useMemo(() => {
    return hasLastSent();
  }, [showRestore]);

  const canClear = useMemo(() => {
    return text.trim().length > 0;
  }, [text]);

  const handleSend = useCallback(async () => {
    const trimmed = text.trim();
    if (!trimmed) return;
    if (!canSend) return;

    if (trimmed.length > 10000) {
      showToast(t("input.textTooLong"), "error");
      return;
    }

    saveLastSent(trimmed);
    clearDraft();

    const ok = await sendToActive(trimmed);
    if (!ok) {
      showToast(t("notifications.sendFailed"), "error");
      return;
    }

    setText("");
    setShowRestore(true);
  }, [canSend, sendToActive, showToast, t, text]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key === "Enter" && !e.ctrlKey && !e.shiftKey) {
        e.preventDefault();
        handleSend();
      }
    },
    [handleSend],
  );

  const handleRestore = useCallback(() => {
    const last = loadLastSent();
    if (!last) return;
    setText(last);
    showToast(t("notifications.lastMessageRestored"), "success");
  }, [showToast, t]);

  const handleClearInput = useCallback(() => {
    setText("");
    clearDraft();
  }, []);

  const handleOpenSettings = useCallback(() => {
    setPage("settings");
  }, []);

  const handleBackFromSettings = useCallback(() => {
    setPage("main");
  }, []);

  const handleAddDevice = useCallback(async (url: string, name?: string) => {
    return await addDevice(url, name);
  }, [addDevice]);

  const handleRemoveDevice = useCallback((deviceId: string) => {
    removeDevice(deviceId);
  }, [removeDevice]);

  const handleRenameDevice = useCallback((deviceId: string) => {
    setRenameDeviceId(deviceId);
    setShowRenameEditor(true);
  }, []);

  const handleSaveRename = useCallback((newName: string) => {
    if (renameDeviceId) {
      renameDevice(renameDeviceId, newName);
    }
  }, [renameDeviceId, renameDevice]);

  const isAnyDeviceConnected = useMemo(() => {
    const activeDevice = devices.find(d => d.id === activeDeviceId);
    return activeDevice?.status === 'connected';
  }, [devices, activeDeviceId]);

  if (page === "settings") {
    return <SettingsPage onBack={handleBackFromSettings} />;
  }

  if (!i18nReady || !isInitialized) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-gradient-to-br from-teal-50 via-white to-cyan-50 p-4 dark:from-slate-950 dark:via-slate-900 dark:to-slate-950">
        <div className="flex flex-col items-center gap-3">
          <div className="h-8 w-8 animate-spin rounded-full border-2 border-primary border-t-transparent" />
          <p className="text-sm text-muted-foreground dark:text-slate-400">
            Loading...
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-teal-50 via-white to-cyan-50 px-5 pt-[calc(env(safe-area-inset-top)+1.25rem)] pb-[calc(env(safe-area-inset-bottom)+1.75rem)] dark:from-slate-950 dark:via-slate-900 dark:to-slate-950">
      {/* Decorative background elements */}
      <div className="pointer-events-none fixed inset-0 overflow-hidden">
        <div className="absolute -top-20 -right-20 h-64 w-64 rounded-full bg-primary/10 blur-3xl" />
        <div className="absolute -bottom-20 -left-20 h-64 w-64 rounded-full bg-cyan-400/10 blur-3xl dark:bg-cyan-500/5" />
      </div>

      <div className="relative mx-auto max-w-md">
        {/* Device Tabs (replaces StatusIndicator) */}
        <DeviceTabs
          devices={devices}
          activeDeviceId={activeDeviceId}
          onDeviceSelect={setActiveDevice}
          onAddDevice={() => setShowAddDevice(true)}
          onRemoveDevice={handleRemoveDevice}
          onRenameDevice={handleRenameDevice}
        />

        {/* Main glass card */}
        <div className="animate-slide-up mt-4 p-0 rounded-3xl">
          <TextInput
            value={text}
            onChange={setText}
            onKeyDown={handleKeyDown}
            disabled={!isAnyDeviceConnected}
            onOpenSettings={handleOpenSettings}
          />

          <div className="mt-6 flex items-center justify-between px-1">
            <RestoreButton onClick={handleRestore} disabled={!canRestore} />
            <SendButton
              onClick={handleSend}
              disabled={!canSend}
              isSending={isSending}
            />
            <ClearButton onClick={handleClearInput} disabled={!canClear} />
          </div>
        </div>
      </div>

      {/* Modals */}
      <AddDeviceModal
        isOpen={showAddDevice}
        onClose={() => setShowAddDevice(false)}
        onAdd={handleAddDevice}
      />

      <DeviceNameEditor
        isOpen={showRenameEditor}
        currentName={devices.find(d => d.id === renameDeviceId)?.name || ''}
        onSave={handleSaveRename}
        onClose={() => {
          setShowRenameEditor(false);
          setRenameDeviceId(null);
        }}
      />

      {toasts.map((toast) => (
        <Toast
          key={toast.id}
          message={toast.message}
          type={toast.type}
          onClose={() => removeToast(toast.id)}
        />
      ))}
    </div>
  );
}
```

**Step 2: Commit**

```bash
git add src/mobile/App.tsx
git commit -m "feat(mobile): integrate device tabs into App, replace StatusIndicator"
```

---

## Task 12: Add Internationalization Strings

**Files:**
- Modify: `src/mobile/locales/zh.json`
- Modify: `src/mobile/locales/en.json`

**Step 1: Add Chinese translations**

Modify: `src/mobile/locales/zh.json`

Add to existing translations:

```json
{
  "devices": {
    "add": "添加",
    "rename": "重命名",
    "renameDevice": "重命名设备",
    "disconnect": "断开连接",
    "maxLimit": "最多支持 5 台设备",
    "alreadyExists": "此设备已存在",
    "invalidUrl": "无效的连接地址",
    "scanQR": "扫描二维码",
    "inputUrl": "输入地址",
    "urlPlaceholder": "ws://192.168.1.xxx:38425/ws",
    "namePlaceholder": "默认: PC-xxx",
    "emptyState": "暂无设备，请添加",
    "startScan": "开始扫描",
    "stopScan": "停止扫描",
    "addDevice": "添加设备",
    "connectionAddress": "连接地址",
    "deviceName": "设备名称",
    "deviceNameOptional": "设备名称（可选）",
    "cancel": "取消",
    "save": "保存",
    "selectDevice": "请先选择设备",
    "deviceNotConnected": "设备未连接",
    "sendFailed": "发送失败，请检查连接"
  }
}
```

**Step 2: Add English translations**

Modify: `src/mobile/locales/en.json`

Add to existing translations:

```json
{
  "devices": {
    "add": "Add",
    "rename": "Rename",
    "renameDevice": "Rename Device",
    "disconnect": "Disconnect",
    "maxLimit": "Maximum 5 devices supported",
    "alreadyExists": "Device already exists",
    "invalidUrl": "Invalid connection address",
    "scanQR": "Scan QR Code",
    "inputUrl": "Input URL",
    "urlPlaceholder": "ws://192.168.1.xxx:38425/ws",
    "namePlaceholder": "Default: PC-xxx",
    "emptyState": "No devices, please add one",
    "startScan": "Start Scan",
    "stopScan": "Stop Scan",
    "addDevice": "Add Device",
    "connectionAddress": "Connection Address",
    "deviceName": "Device Name",
    "deviceNameOptional": "Device Name (Optional)",
    "cancel": "Cancel",
    "save": "Save",
    "selectDevice": "Please select a device first",
    "deviceNotConnected": "Device not connected",
    "sendFailed": "Send failed, please check connection"
  }
}
```

**Step 3: Update components to use translations**

The components created use hardcoded Chinese strings. For production, replace them with `t('devices.key')` calls.

**Step 4: Commit**

```bash
git add src/mobile/locales/
git commit -m "feat(mobile): add device-related i18n translations"
```

---

## Post-Implementation Tasks

### Testing Checklist

Run these tests manually after implementation:

```bash
# 1. Start dev server
pnpm tauri dev

# 2. On mobile browser:
# - Scan QR code from PC
# - Verify device card appears with green dot
# - Send text, verify it works

# 3. Add second device via URL input
# - Verify second device card appears
# - Click cards to switch
# - Verify text sends to correct device

# 4. Test rename
# - Long press device card
# - Enter new name
# - Verify name changed

# 5. Test disconnect
# - Click X on device card
# - Verify device removed
# - Verify connection closed

# 6. Test page reload
# - Refresh mobile page
# - Verify devices reconnect automatically

# 7. Test error scenarios
# - Try to add 6th device (should show error)
# - Disconnect network, try to send (should show error)
# - Enter invalid URL (should show error)
```

### Performance Checks

```bash
# Build production bundle
pnpm build

# Check bundle size
# Expected: < 500KB additional (html5-qrcode ~ 150KB gzipped)
```

### Final Commit

```bash
git add .
git commit -m "feat(mobile): complete multi-device connection feature implementation

- Add multi-WebSocket connection management
- Add device tabs UI with status indicators
- Add QR scanning for device discovery
- Add device rename/disconnect functionality
- Add persistent device storage with auto-reconnect
- Support up to 5 simultaneous devices

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
```

---

## Summary

This plan implements the multi-device connection feature in **12 bite-sized tasks**:

1. Install QR scanning dependency
2. Define core data types
3. Create device storage utilities
4. Create multi-WebSocket manager hook
5. Create device manager hook
6. Create DeviceCard component
7. Create DeviceTabs component
8. Create AddDeviceModal component
9. Create QRScanner component
10. Create DeviceNameEditor component
11. Update App.tsx with device tabs
12. Add internationalization strings

**Estimated completion time:** 2-3 hours

**Key changes:**
- New `useMultiWebSocket` hook replaces `useWebSocket`
- Device tabs at top of screen with integrated status
- QR scan + URL input for adding devices
- Persistent storage with auto-reconnect
- Max 5 devices limit

**No backend changes required.**
