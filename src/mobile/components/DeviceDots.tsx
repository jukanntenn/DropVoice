import type { Device } from "../types";
import { getDeviceColor, getDeviceHostLabel } from "../utils/deviceColor";

interface DeviceDotsProps {
  devices: Device[];
  activeDeviceId: string | null;
  dark: boolean;
  onSelect: (deviceId: string) => void;
}

/**
 * Dot indicator row for switching devices. Hidden when <=1 device.
 * Active dot = solid + larger; inactive = hollow + smaller, in each device's IP color.
 * Inactive status is encoded as: connecting -> pulse, disconnected -> dimmed. Hue is preserved.
 */
export function DeviceDots({ devices, activeDeviceId, dark, onSelect }: DeviceDotsProps) {
  if (devices.length <= 1) return null;

  return (
    <div className="mt-3 flex items-center justify-center gap-3">
      {devices.map((device) => {
        const active = device.id === activeDeviceId;
        const color = getDeviceColor(device, dark);
        const connecting = device.status === "connecting";
        const dimmed = !active && device.status === "disconnected";
        const sizeCls = active ? "h-3.5 w-3.5" : "h-2.5 w-2.5";
        const dotStyle = active
          ? { backgroundColor: color }
          : { borderColor: color, backgroundColor: "transparent" };
        const stateCls = [
          connecting ? "animate-pulse" : "",
          dimmed ? "opacity-40" : "",
        ].join(" ");
        return (
          <button
            key={device.id}
            type="button"
            onClick={() => onSelect(device.id)}
            aria-label={getDeviceHostLabel(device)}
            aria-current={active ? "true" : undefined}
            className="flex h-9 w-9 items-center justify-center rounded-full"
          >
            <span
              style={dotStyle}
              className={[
                "rounded-full",
                sizeCls,
                active ? "" : "border-2",
                stateCls,
              ].join(" ")}
            />
          </button>
        );
      })}
    </div>
  );
}
