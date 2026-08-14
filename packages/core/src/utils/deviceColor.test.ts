import { describe, it, expect } from 'vitest';

import { getDeviceColor, getDeviceColorSoft, getDeviceHue } from './deviceColor';

describe('deviceColor', () => {
  it('returns a hue in 0-360 range', () => {
    const hue = getDeviceHue({ id: '550e8400-e29b-41d4-a716-446655440000' });
    expect(hue).toBeGreaterThanOrEqual(0);
    expect(hue).toBeLessThan(360);
  });

  it('is deterministic for the same id', () => {
    const device = { id: '550e8400-e29b-41d4-a716-446655440000' };
    expect(getDeviceHue(device)).toBe(getDeviceHue(device));
  });

  it('produces different hues for different ids', () => {
    const a = getDeviceHue({ id: 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa' });
    const b = getDeviceHue({ id: 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb' });
    expect(a).not.toBe(b);
  });

  it('getDeviceColor returns an hsl() string', () => {
    const color = getDeviceColor({ id: 'device-1' }, false);
    expect(color).toMatch(/^hsl\(/);
  });

  it('getDeviceColorSoft returns an hsl() string', () => {
    const color = getDeviceColorSoft({ id: 'device-2' }, true);
    expect(color).toMatch(/^hsl\(/);
  });
});
