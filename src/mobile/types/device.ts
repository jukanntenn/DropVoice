export type DeviceStatus = "connected" | "disconnected" | "connecting";

export type DeviceErrorType = "unreachable" | "refused" | "timeout" | "unknown";

export interface Device {
  id: string;
  name: string;
  url: string;
  status: DeviceStatus;
  lastConnected: number;
  hasExhaustedRetries?: boolean;
  errorType?: DeviceErrorType;
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
