import { describe, it, expect } from 'vitest';

import {
  inputDelayAtom,
  languageAtom,
  selectedDeviceIdsAtom,
  sendModeAtom,
  themeAtom,
  type Theme,
} from './atoms';

/**
 * Atoms are jotai primitives — the value we want to lock down here is their
 * *initial* value, since changing a default is a behaviour change for every
 * consumer (spec 02 §2.2 lists the canonical defaults).
 *
 * §9 重构后 `devicesAtom`/`activeDeviceIdAtom`/`connectionModeAtom` 已删除
 * （只写不读的死镜像），故不再测试。
 */
describe('jotai atom defaults', () => {
  it('theme defaults to system', () => {
    expect((themeAtom as unknown as { init: Theme }).init).toBe('system');
  });

  it('language defaults to zh (spec 02 §2.2)', () => {
    expect((languageAtom as unknown as { init: string }).init).toBe('zh');
  });

  it('inputDelay defaults to 10ms (spec 14 §2.3)', () => {
    expect((inputDelayAtom as unknown as { init: number }).init).toBe(10);
  });

  it('sendMode defaults to active', () => {
    expect((sendModeAtom as unknown as { init: string }).init).toBe('active');
  });

  it('selectedDeviceIds defaults to empty array', () => {
    expect((selectedDeviceIdsAtom as unknown as { init: unknown[] }).init).toEqual([]);
  });
});
