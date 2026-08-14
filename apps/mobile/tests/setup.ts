/**
 * Vitest 全局 setup（spec 06 §2.4）。
 *
 * 1. 引入 @testing-library/jest-dom 匹配器；
 * 2. 每个测试后清理 DOM 与 mock 状态；
 * 3. 全局 mock `@tauri-apps/api/core`（移动端组件在 jsdom 下无法访问 Tauri 桥）；
 * 4. 全局 mock `global.WebSocket`（让 hooks 能在无真实 WS 的环境下测试）。
 */
import '@testing-library/jest-dom/vitest';
import { cleanup } from '@testing-library/react';
import { afterEach, vi } from 'vitest';

import { mockInvoke } from './mocks/tauri';
import { mockGlobalWebSocket } from './mocks/websocket';

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

// Mock Tauri API - 所有组件测试通过 mockInvoke 断言调用。
vi.mock('@tauri-apps/api/core', () => ({ invoke: mockInvoke }));

// Mock WebSocket - 提供 readyState 常量与最小可观察的实现。
mockGlobalWebSocket();

// jsdom 不带 PointerEvent 构造器；framer-motion / @base-ui/react 的某些点击路径会调用
// `new ownerWindow().PointerEvent(...)`。在测试环境下补一个 polyfill。
class PointerEventPolyfill extends MouseEvent {
  pointerId: number;
  pointerType: string;
  isPrimary: boolean;
  width: number;
  height: number;
  pressure: number;
  tangentialPressure: number;
  tiltX: number;
  tiltY: number;
  twist: number;

  constructor(type: string, params: PointerEventInit = {}) {
    super(type, params);
    this.pointerId = params.pointerId ?? 0;
    this.pointerType = params.pointerType ?? '';
    this.isPrimary = params.isPrimary ?? false;
    this.width = params.width ?? 1;
    this.height = params.height ?? 1;
    this.pressure = params.pressure ?? 0;
    this.tangentialPressure = params.tangentialPressure ?? 0;
    this.tiltX = params.tiltX ?? 0;
    this.tiltY = params.tiltY ?? 0;
    this.twist = params.twist ?? 0;
  }
}

if (typeof globalThis.PointerEvent === 'undefined') {
  (globalThis as unknown as { PointerEvent: typeof PointerEventPolyfill }).PointerEvent =
    PointerEventPolyfill;
}
