import type { AddDeviceResult, Device } from '../types';

/**
 * 设备列表容量上限。
 *
 * §9 重构后从已删除的 `machines/device.ts` 迁移至此（设备列表逻辑的唯一家）。
 */
export const MAX_DEVICES = 5;

/**
 * In-browser device list manager for the mobile PWA.
 *
 * WebRTC 架构下（webrtc-scan-direct-design），设备通过扫码添加——QR 载荷
 * `dropvoice://pair?code=&device=&name=` 携带桌面 device_id + 配对码 + 名称。
 * 设备不再有 LAN URL/IP/port（旧 WS 数据模型）。
 *
 * §9 重构后 `Device` 仅含持久身份（`id/name/autoConnect`），无任何连接瞬态。
 * 连接状态由 `apps/mobile/src/hooks/useConnections.ts` 独占。
 */

export interface DeviceManagerOptions {
  maxDevices?: number;
}

/** 扫码结果（解析自 QR 载荷，§6.5）。 */
export interface ScannedDevice {
  /** 桌面 device_id（UUID v4，信令端点路径参数）。 */
  device: string;
  /** 6 位配对码（首次配对）。 */
  code: string;
  /** 设备名（URL encoded，可选）。 */
  name?: string;
}

export function createDeviceManager(options: DeviceManagerOptions = {}) {
  const maxDevices = options.maxDevices ?? MAX_DEVICES;

  return {
    maxDevices,

    /**
     * 添加设备（扫码）。返回：
     * - `'added'` 当 device_id 是新的并已加入列表
     * - `'switched'` 当 device_id 已存在（调用方应切换 active）
     * - `false` 当列表已满或 payload 无效
     */
    addDevice(devices: Device[], scanned: ScannedDevice): AddDeviceResult {
      if (!scanned.device || !scanned.code) {
        return false;
      }
      const existing = devices.find((device) => device.id === scanned.device);
      if (existing) {
        return 'switched';
      }
      if (devices.length >= maxDevices) {
        return false;
      }
      // §9：Device 仅含持久身份，无瞬态。
      const device: Device = {
        id: scanned.device,
        name: scanned.name ?? 'Desktop',
        autoConnect: true,
      };
      devices.push(device);
      return 'added';
    },

    removeDevice(devices: Device[], deviceId: string): Device[] {
      return devices.filter((device) => device.id !== deviceId);
    },

    setActive(devices: Device[], deviceId: string | null): string | null {
      if (deviceId === null) {
        return null;
      }
      return devices.some((device) => device.id === deviceId) ? deviceId : null;
    },
  };
}
