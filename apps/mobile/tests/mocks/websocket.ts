/**
 * WebSocket mock for jsdom (spec 06 §2.4).
 *
 * jsdom's WebSocket doesn't exist in the Node test runner, and hook tests need
 * a controllable socket to drive open/message/close events. The mock auto-opens
 * on the next tick so existing `await waitFor(() => status === 'connected')`
 * patterns work; tests can also call `socket.emit(...)` directly.
 */
export class MockWebSocket {
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
  private openTimer: ReturnType<typeof setTimeout> | null = null;

  constructor(url: string | URL) {
    this.url = url.toString();
    // Auto-open on next tick so callers can wire addEventListener first.
    this.openTimer = setTimeout(() => {
      this.readyState = MockWebSocket.OPEN;
      this.onopen?.(new Event('open'));
      this.emit('open', new Event('open'));
    }, 0);
  }

  send(_data: string | ArrayBufferLike | Blob): void {
    // no-op
  }

  close(_code?: number, _reason?: string): void {
    if (this.openTimer) clearTimeout(this.openTimer);
    this.readyState = MockWebSocket.CLOSED;
    const ev = new CloseEvent('close');
    this.onclose?.(ev);
    this.emit('close', ev);
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

  /** Test-only helper: pretend the server sent a message. */
  emitMessage(data: unknown): void {
    const ev = new MessageEvent('message', { data });
    this.onmessage?.(ev);
    this.emit('message', ev);
  }

  private emit(type: string, event: Event): void {
    for (const listener of this.listeners[type] || []) {
      listener(event);
    }
  }
}

/** Installs MockWebSocket as the global WebSocket. Call once in setup. */
export function mockGlobalWebSocket(): void {
  (globalThis as unknown as { WebSocket: typeof MockWebSocket }).WebSocket = MockWebSocket;
}
