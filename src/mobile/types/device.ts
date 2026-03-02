export type DeviceStatus = "connected" | "disconnected" | "connecting";

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
