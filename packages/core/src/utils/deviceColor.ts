import type { Device } from '../types';

/**
 * Deterministic device color generation.
 *
 * Uses the device id (UUID v4，桌面 device_id）hashed to a stable hue, then maps
 * to HSL via the golden angle (137.5°) so colors stay visually distinct as the
 * device list grows.
 */

const GOLDEN_ANGLE = 137.5;

function hashString(input: string): number {
  let hash = 0;
  for (let i = 0; i < input.length; i += 1) {
    hash = (hash << 5) - hash + input.charCodeAt(i);
    hash |= 0; // Force 32-bit int.
  }
  return Math.abs(hash);
}

/**
 * Compute a hue (0-360) for the given device. Deterministic for the same id.
 */
export function getDeviceHue(device: Pick<Device, 'id'>): number {
  const hash = hashString(device.id);
  return Math.round((hash * GOLDEN_ANGLE) % 360);
}

/**
 * Build an HSL color string for a device. Dark mode uses slightly desaturated
 * colors for readability on dark surfaces.
 */
export function getDeviceColor(device: Pick<Device, 'id'>, dark: boolean): string {
  const hue = getDeviceHue(device);
  const saturation = dark ? 65 : 70;
  const lightness = dark ? 60 : 45;
  return `hsl(${hue} ${saturation}% ${lightness}%)`;
}

/**
 * Soft background tint variant, useful for chip backgrounds.
 */
export function getDeviceColorSoft(device: Pick<Device, 'id'>, dark: boolean): string {
  const hue = getDeviceHue(device);
  const saturation = dark ? 50 : 60;
  const lightness = dark ? 35 : 80;
  return `hsl(${hue} ${saturation}% ${lightness}%)`;
}
