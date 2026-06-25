import type { Device } from "../types";

const GOLDEN_ANGLE = 137.508;

function hostnameOf(url: string): string | null {
  try {
    return new URL(url).hostname;
  } catch {
    return null;
  }
}

function isIpv4(host: string): boolean {
  return /^\d+\.\d+\.\d+\.\d+$/.test(host);
}

function lastOctet(host: string): number {
  const part = host.split(".")[3];
  const n = Number(part);
  return Number.isFinite(n) ? n : 0;
}

function hashString(s: string): number {
  let h = 0;
  for (let i = 0; i < s.length; i++) {
    h = (h * 31 + s.charCodeAt(i)) >>> 0;
  }
  return h;
}

/** Hue [0,360) for a device, derived from its last IPv4 octet (golden angle) or hostname hash. */
export function getDeviceHue(device: Device): number {
  const host = hostnameOf(device.url);
  if (host && isIpv4(host)) {
    return Math.round((lastOctet(host) * GOLDEN_ANGLE) % 360);
  }
  return hashString(host ?? device.url) % 360;
}

/** CSS hsl() color for a device, with saturation/lightness chosen for the theme. */
export function getDeviceColor(device: Device, dark: boolean): string {
  const hue = getDeviceHue(device);
  return dark ? `hsl(${hue} 70% 62%)` : `hsl(${hue} 60% 52%)`;
}

/** Hostname for display (document.title, aria-label, confirm dialog). IPv4 literal if IP, else hostname. */
export function getDeviceHostLabel(device: Device): string {
  return hostnameOf(device.url) ?? device.url;
}
