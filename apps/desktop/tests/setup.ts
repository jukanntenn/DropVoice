/**
 * Vitest 全局 setup（spec 06 §2.4）。
 *
 * 1. 引入 @testing-library/jest-dom 匹配器；
 * 2. 每个测试后清理 DOM 与 mock 状态；
 * 3. 全局 mock `@tauri-apps/api/core`（桌面端组件在 jsdom 下无法访问 Tauri 桥）；
 * 4. 全局 mock `global.WebSocket`（让 hooks 能在无真实 WS 的环境下测试）；
 * 5. 初始化 i18n（固定 en），让组件渲染真实文案而非 i18n key。
 */
import '@testing-library/jest-dom/vitest';
import { cleanup } from '@testing-library/react';
import { afterEach, vi } from 'vitest';

import { initI18n } from '@dropvoice/i18n';

// 同步完成 i18next 初始化（i18next 无异步资源时 init 是同步的），保证
// useTranslation 在各测试用例中直接返回真实翻译文案。
await initI18n({ language: 'en' });

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

// Mock Tauri API - 所有组件测试通过 vi.mocked(invoke) 断言调用。
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

// Mock WebSocket - 提供 readyState 常量与最小可观察的实现。
class MockWebSocket {
  static CONNECTING = 0;
  static OPEN = 1;
  static CLOSING = 2;
  static CLOSED = 3;

  readonly url: string;
  readyState: number = MockWebSocket.CONNECTING;
  onopen: ((ev: Event) => void) | null = null;
  onmessage: ((ev: MessageEvent) => void) | null = null;
  onclose: ((ev: CloseEvent) => void) | null = null;
  onerror: ((ev: Event) => void) | null = null;

  private listeners: Record<string, EventListener[]> = {};

  constructor(url: string | URL) {
    this.url = url.toString();
    // 异步触发 open 事件，便于测试断言。
    setTimeout(() => {
      this.readyState = MockWebSocket.OPEN;
      this.onopen?.(new Event('open'));
      this.emit('open', new Event('open'));
    }, 0);
  }

  send(_data: string | ArrayBufferLike | Blob): void {
    // no-op
  }

  close(_code?: number, _reason?: string): void {
    this.readyState = MockWebSocket.CLOSED;
    this.onclose?.(new CloseEvent('close'));
    this.emit('close', new CloseEvent('close'));
  }

  addEventListener(type: string, listener: EventListener): void {
    (this.listeners[type] ||= []).push(listener);
  }

  removeEventListener(type: string, listener: EventListener): void {
    this.listeners[type] = (this.listeners[type] || []).filter((l) => l !== listener);
  }

  dispatchEvent(event: Event): boolean {
    this.emit(event.type, event);
    return true;
  }

  private emit(type: string, event: Event): void {
    for (const listener of this.listeners[type] || []) {
      listener(event);
    }
  }
}

// @ts-expect-error - 测试环境下替换全局 WebSocket。
global.WebSocket = MockWebSocket;
