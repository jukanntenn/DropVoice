import { describe, it, expect } from 'vitest';

import { hydrate } from './useDeviceManager';
import type { Device, StoredDevice } from '../types';

/**
 * §8 + §9 + §0.4 数据契约测试（pure，node 环境可跑）：
 * - Device 瘦身为 {id,name,autoConnect}，无瞬态。
 * - hydrate 只取身份字段，旧数据多出的 status/lastConnected 等静默忽略。
 *
 * 完整的"删活跃设备刷新不复活"竞态测试（依赖 React effect 时序）见
 * apps/mobile 测试套件（jsdom + renderHook）。
 */
describe('hydrate (§8/§9 data contract)', () => {
  it('maps StoredDevice → Device（仅身份字段）', () => {
    const stored: StoredDevice[] = [
      { id: 'a', name: 'PC', autoConnect: true },
      { id: 'b', name: 'Mac', autoConnect: false },
    ];
    expect(hydrate(stored)).toEqual<Device[]>([
      { id: 'a', name: 'PC', autoConnect: true },
      { id: 'b', name: 'Mac', autoConnect: false },
    ]);
  });

  it('缺省 autoConnect 时默认 true', () => {
    // 旧数据可能没有 autoConnect 字段。
    const stored = [{ id: 'a', name: 'PC' }] as StoredDevice[];
    expect(hydrate(stored)[0].autoConnect).toBe(true);
  });

  it('静默忽略旧数据多出的瞬态字段（status/lastConnected/errorType 等）', () => {
    // 模拟 v2 键下残留的旧形状（§0.4：不 bump v3，旧字段水合时丢弃）。
    const legacy = [
      {
        id: 'a',
        name: 'PC',
        autoConnect: true,
        status: 'connected',
        lastConnected: 1234567890,
        errorType: 'timeout',
        reconnectAttempts: 3,
        hasExhaustedRetries: false,
      },
    ] as unknown as StoredDevice[];
    const result = hydrate(legacy);
    expect(result).toEqual<Device[]>([{ id: 'a', name: 'PC', autoConnect: true }]);
    // 关键：瞬态字段不带过来（刷新后总是干净身份）。
    expect(result[0]).not.toHaveProperty('status');
    expect(result[0]).not.toHaveProperty('lastConnected');
  });

  it('空列表水合为空', () => {
    expect(hydrate([])).toEqual([]);
  });
});
