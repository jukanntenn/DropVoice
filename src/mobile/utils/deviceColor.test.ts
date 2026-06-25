import { describe, expect, it } from "vitest";
import type { Device } from "../types";
import { getDeviceColor, getDeviceHostLabel, getDeviceHue } from "./deviceColor";

function dev(url: string): Device {
  return { id: "x", name: "n", url, status: "connected", lastConnected: 0 };
}

describe("getDeviceHue", () => {
  it("derives hue from the last IPv4 octet via the golden angle", () => {
    expect(getDeviceHue(dev("ws://192.168.1.0:38425/ws"))).toBe(0);
    expect(getDeviceHue(dev("ws://192.168.1.100:38425/ws"))).toBe(71);
    expect(getDeviceHue(dev("ws://192.168.1.101:38425/ws"))).toBe(208);
    expect(getDeviceHue(dev("ws://192.168.1.102:38425/ws"))).toBe(346);
  });

  it("spreads consecutive octets widely (>=90 degrees apart)", () => {
    const h = (octet: number) =>
      getDeviceHue(dev(`ws://10.0.0.${octet}:1/ws`));
    for (let i = 0; i < 20; i++) {
      const d = Math.abs(h(i) - h(i + 1));
      const spread = Math.min(d, 360 - d);
      expect(spread).toBeGreaterThanOrEqual(90);
    }
  });

  it("is deterministic for the same device", () => {
    const a = getDeviceHue(dev("ws://192.168.0.42:1/ws"));
    const b = getDeviceHue(dev("ws://192.168.0.42:1/ws"));
    expect(a).toBe(b);
  });

  it("falls back to a hostname hash for non-IP hosts (stable, in range)", () => {
    const hue = getDeviceHue(dev("ws://my-pc.local:9000/ws"));
    expect(Number.isInteger(hue)).toBe(true);
    expect(hue).toBeGreaterThanOrEqual(0);
    expect(hue).toBeLessThan(360);
    expect(getDeviceHue(dev("ws://my-pc.local:9000/ws"))).toBe(hue);
  });

  it("falls back to hashing the raw url when the URL is malformed", () => {
    const hue = getDeviceHue(dev("not-a-url"));
    expect(hue).toBeGreaterThanOrEqual(0);
    expect(hue).toBeLessThan(360);
  });

  it("different hostnames usually map to different hues", () => {
    const a = getDeviceHue(dev("ws://alpha.local:1/ws"));
    const b = getDeviceHue(dev("ws://beta.local:1/ws"));
    expect(a).not.toBe(b);
  });
});

describe("getDeviceColor", () => {
  it("returns an hsl string with theme-aware saturation/lightness", () => {
    expect(getDeviceColor(dev("ws://192.168.1.100:1/ws"), false)).toBe("hsl(71 60% 52%)");
    expect(getDeviceColor(dev("ws://192.168.1.100:1/ws"), true)).toBe("hsl(71 70% 62%)");
  });
});

describe("getDeviceHostLabel", () => {
  it("returns the IPv4 hostname without port", () => {
    expect(getDeviceHostLabel(dev("ws://192.168.1.100:38425/ws"))).toBe("192.168.1.100");
  });

  it("returns a non-IP hostname as-is", () => {
    expect(getDeviceHostLabel(dev("ws://my-pc.local:9000/ws"))).toBe("my-pc.local");
  });

  it("falls back to the raw url when parsing fails", () => {
    expect(getDeviceHostLabel(dev("garbage"))).toBe("garbage");
  });
});
