import { describe, it, expect } from 'vitest';

import { MAX_DEVICES, createDeviceManager } from './manager';
import type { Device } from '../types';

function makeDevice(id: string, name = 'PC'): Device {
  return { id, name, autoConnect: true };
}

describe('connection manager', () => {
  it('adds a new device from QR scan and returns "added"', () => {
    const manager = createDeviceManager();
    const devices: Device[] = [];
    const result = manager.addDevice(devices, {
      device: 'device-1',
      code: '123456',
      name: 'My PC',
    });
    expect(result).toBe('added');
    expect(devices).toHaveLength(1);
    expect(devices[0]).toEqual({ id: 'device-1', name: 'My PC', autoConnect: true });
  });

  it('returns "switched" when device_id already exists', () => {
    const manager = createDeviceManager();
    const devices: Device[] = [makeDevice('device-1')];
    const result = manager.addDevice(devices, {
      device: 'device-1',
      code: '123456',
    });
    expect(result).toBe('switched');
    expect(devices).toHaveLength(1);
  });

  it('refuses to add when at max devices', () => {
    const manager = createDeviceManager({ maxDevices: 1 });
    const devices: Device[] = [makeDevice('device-1')];
    const result = manager.addDevice(devices, {
      device: 'device-2',
      code: '123456',
    });
    expect(result).toBe(false);
  });

  it('default MAX_DEVICES = 5', () => {
    expect(MAX_DEVICES).toBe(5);
    const manager = createDeviceManager();
    expect(manager.maxDevices).toBe(5);
  });

  it('returns false for invalid scan (missing device_id)', () => {
    const manager = createDeviceManager();
    const devices: Device[] = [];
    const result = manager.addDevice(devices, { device: '', code: '123456' });
    expect(result).toBe(false);
  });

  it('returns false for invalid scan (missing code)', () => {
    const manager = createDeviceManager();
    const devices: Device[] = [];
    const result = manager.addDevice(devices, { device: 'device-1', code: '' });
    expect(result).toBe(false);
  });

  it('uses default name when scan has no name', () => {
    const manager = createDeviceManager();
    const devices: Device[] = [];
    manager.addDevice(devices, { device: 'device-1', code: '123456' });
    expect(devices[0].name).toBe('Desktop');
  });

  it('removes a device by id', () => {
    const manager = createDeviceManager();
    const devices: Device[] = [makeDevice('a'), makeDevice('b')];
    const next = manager.removeDevice(devices, 'a');
    expect(next).toHaveLength(1);
    expect(next[0].id).toBe('b');
  });

  it('setActive returns the id when it exists', () => {
    const manager = createDeviceManager();
    const devices: Device[] = [makeDevice('a')];
    expect(manager.setActive(devices, 'a')).toBe('a');
    expect(manager.setActive(devices, 'nonexistent')).toBeNull();
    expect(manager.setActive(devices, null)).toBeNull();
  });
});
