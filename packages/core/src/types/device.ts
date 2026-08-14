/**
 * Device identity types shared between desktop and mobile surfaces.
 *
 * §9 重构后，`Device` 只承载**持久身份 + 配置**，不含任何连接瞬态状态。
 * 连接状态（idle/connecting/connected/reconnecting/offline）由连接 hook
 * 独占（`apps/mobile/src/hooks/useConnections.ts`），驱动 UI 的
 * `Record<deviceId, ConnectionState>`。这样刷新后瞬态总是回 idle，持久列表
 * 只含身份字段。
 *
 * WebRTC 架构下（webrtc-scan-direct-design），`Device.id` 是桌面 device_id
 * （UUID v4，从 QR 载荷提取），无 LAN URL/IP/port（旧 WS 数据模型已删）。
 */

/**
 * 持久身份（存盘）：仅桌面身份 + 配置。无任何连接状态。
 */
export interface Device {
  /** 桌面设备 ID（UUID v4，从 QR 载荷 `device` 字段提取，信令端点路径参数）。 */
  id: string;
  /** 设备显示名（QR 载荷 `name` 字段）。 */
  name: string;
  /** 是否在启动时自动连接，默认 true。 */
  autoConnect: boolean;
}

/**
 * 持久化形状 = Device 瘦身后一致（§9）。
 *
 * 存储键 `dropvoice:devices:v2` 保持不变；旧数据多出的 `status`/`lastConnected`
 * 等瞬态字段在水合时静默忽略（`hydrate` 只取 `id/name/autoConnect`）。
 * 不 bump v3、不强制重新配对。
 */
export interface StoredDevice {
  id: string;
  name: string;
  autoConnect: boolean;
}

export interface DeviceStorage {
  devices: StoredDevice[];
  lastActiveDeviceId: string | null;
}

export type AddDeviceResult = 'added' | 'switched' | false;

export type SendMode = 'active' | 'all' | 'selected';
