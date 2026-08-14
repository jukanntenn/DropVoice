import { describe, it, expect } from 'vitest';

import {
  detectEnvironment,
  detectPlatform,
  getPWAInstallStrategy,
  isCameraSupported,
  isLocalhost,
  isSecureContext,
  isServiceWorkerSupported,
} from './index';

describe('platform detection', () => {
  it('detectEnvironment returns expected fields', () => {
    const env = detectEnvironment();
    expect(env).toHaveProperty('protocol');
    expect(env).toHaveProperty('hostname');
    expect(env).toHaveProperty('isSecureContext');
    expect(env).toHaveProperty('isLocalhost');
    expect(env).toHaveProperty('isLAN');
    expect(env).toHaveProperty('serviceWorkerSupported');
    expect(env).toHaveProperty('cameraSupported');
  });

  it('detectPlatform returns a known value', () => {
    const platform = detectPlatform();
    expect(['ios', 'android', 'desktop', 'unknown']).toContain(platform);
  });

  it('isSecureContext returns a boolean', () => {
    expect(typeof isSecureContext()).toBe('boolean');
  });

  it('isLocalhost returns a boolean', () => {
    expect(typeof isLocalhost()).toBe('boolean');
  });

  it('isCameraSupported returns a boolean', () => {
    expect(typeof isCameraSupported()).toBe('boolean');
  });

  it('isServiceWorkerSupported returns a boolean', () => {
    expect(typeof isServiceWorkerSupported()).toBe('boolean');
  });

  it('getPWAInstallStrategy returns one of the known strategies', () => {
    const strategy = getPWAInstallStrategy();
    expect(['pwa', 'bookmark', 'none']).toContain(strategy);
  });
});
