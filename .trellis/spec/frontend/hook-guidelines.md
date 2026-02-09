# Hook Guidelines

> How hooks are used in this project.

---

## Overview

DropVoice uses custom hooks to encapsulate complex stateful logic. Hooks are the primary pattern for:
- WebSocket connection management
- Theme and settings persistence
- Tauri command invocations

---

## Custom Hook Patterns

### Return Type Interface

All custom hooks export a return type interface:

```tsx
// src/mobile/hooks/useWebSocket.ts
export type ConnectionStatus = "connected" | "disconnected" | "connecting";

interface UseWebSocketReturn {
  status: ConnectionStatus;
  isSending: boolean;
  lastError: string | null;
  send: (text: string) => boolean;
  reconnectAttempts: number;
}

export function useWebSocket(): UseWebSocketReturn {
  // ...
}
```

### Using Refs for Mutable State

Use `useRef` for values that:
- Don't need to trigger re-renders
- Need to persist across renders
- Are accessed by async callbacks

```tsx
// src/mobile/hooks/useWebSocket.ts:21-25
const wsRef = useRef<WebSocket | null>(null);
const reconnectAttemptsRef = useRef(0);
const reconnectTimeoutRef = useRef<number | null>(null);
const sendTimeoutRef = useRef<number | null>(null);
const wasConnectedRef = useRef(false);
```

### Cleanup Pattern

All hooks with side effects must clean up:

```tsx
// src/mobile/hooks/useWebSocket.ts:121-132
useEffect(() => {
  connect();

  return () => {
    clearReconnectTimer();
    clearSendTimer();
    try {
      wsRef.current?.close();
    } catch {}
    wsRef.current = null;
  };
}, [connect]);
```

---

## Custom Hooks in This Project

| Hook | Purpose | File |
|------|---------|------|
| `useWebSocket` | WebSocket connection with auto-reconnect | `src/mobile/hooks/useWebSocket.ts` |
| `useSettings` | Theme/language persistence (mobile) | `src/mobile/hooks/useSettings.ts` |
| `useTheme` | Theme context consumer | `src/components/ThemeProvider.tsx` |

---

## Data Fetching

### Tauri Command Invocation

For desktop app, use Tauri's `invoke` API:

```tsx
import { invoke } from "@tauri-apps/api/core";

// Async invocation with type parameter
const info = await invoke<ConnectionInfo>("get_connection_info");

// With error handling
try {
  const url = await invoke<string>("start_server");
  setServerUrl(url);
} catch (err) {
  const message = typeof err === "string" ? err : "";
  setError(message);
}
```

**Reference**: `src/App.tsx:35-45`

### WebSocket Data

For mobile app, use the custom `useWebSocket` hook:

```tsx
const { status, isSending, lastError, send } = useWebSocket();

// Send data
const success = send(text);
```

---

## Naming Conventions

| Pattern | Convention | Example |
|---------|------------|---------|
| Hook name | `use` prefix | `useWebSocket`, `useSettings` |
| Return type | `HookNameReturn` | `UseWebSocketReturn` |
| Status types | Union type | `"connected" \| "disconnected" \| "connecting"` |
| Boolean state | `is` prefix | `isSending`, `isLoading` |

---

## Common Mistakes

### 1. Stale closures in async callbacks

```tsx
// Bad - uses stale state value
const send = () => {
  ws.send(JSON.stringify({ text: currentText }));
};

// Good - uses ref for mutable value
const textRef = useRef(currentText);
textRef.current = currentText;
const send = () => {
  ws.send(JSON.stringify({ text: textRef.current }));
};
```

### 2. Not handling cleanup

```tsx
// Bad - timer continues after unmount
useEffect(() => {
  const timer = setInterval(poll, 1000);
}, []);

// Good - clear on unmount
useEffect(() => {
  const timer = setInterval(poll, 1000);
  return () => clearInterval(timer);
}, []);
```

**Reference**: `src/App.tsx:49-56`

### 3. Not using `useCallback` for stable references

```tsx
// Bad - function recreated every render
const connect = () => { /* ... */ };

// Good - stable reference
const connect = useCallback(() => {
  // ...
}, [dependencies]);
```

**Reference**: `src/mobile/hooks/useWebSocket.ts:55`
