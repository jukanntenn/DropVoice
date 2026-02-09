# Type Safety

> Type safety patterns in this project.

---

## Overview

DropVoice uses TypeScript with strict type checking. The project follows a pragmatic approach:
- Types are defined inline for component-specific interfaces
- Shared types are exported from their defining module
- No separate `types/` directory - types live with their usage

---

## Type Organization

### Inline Types (Component-Specific)

Define types directly above the component:

```tsx
// src/components/QRCodeSection.tsx:4-6
export interface QRCodeSectionProps {
  url: string;
}
```

### Module-Level Types (Shared)

Export types from the module that "owns" them:

```tsx
// src/mobile/hooks/useWebSocket.ts:3-11
export type ConnectionStatus = "connected" | "disconnected" | "connecting";

interface UseWebSocketReturn {
  status: ConnectionStatus;
  isSending: boolean;
  lastError: string | null;
  send: (text: string) => boolean;
  reconnectAttempts: number;
}
```

### Backend Response Types

Define types matching Tauri command responses:

```tsx
// src/App.tsx:12-16
interface ConnectionInfo {
  isRunning: boolean;
  url: string;
  connectionCount: number;
}
```

---

## Type Patterns

### Union Types for Status

```tsx
export type ConnectionStatus = "connected" | "disconnected" | "connecting";
export type Theme = "light" | "dark" | "system";
export type Language = "en" | "zh";
```

### Return Type Interfaces

```tsx
interface UseWebSocketReturn {
  status: ConnectionStatus;
  isSending: boolean;
  // ...
}

export function useWebSocket(): UseWebSocketReturn {
  // ...
}
```

### Props Extending HTML Elements

```tsx
// src/components/ui/button.tsx:36-40
export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean;
}
```

---

## Validation

### Type Guards for API Responses

When receiving data from Tauri backend:

```tsx
// src/components/ThemeProvider.tsx:91-95
if (saved === "light" || saved === "dark" || saved === "system") {
  initialTheme = saved;
} else {
  initialTheme = "system";
}
```

### Runtime Validation Pattern

For critical data, validate at runtime:

```tsx
// src/mobile/hooks/useSettings.ts:24-28
function loadTheme(): Theme {
  try {
    const saved = localStorage.getItem(THEME_KEY);
    if (saved === "light" || saved === "dark" || saved === "system") return saved;
  } catch {}
  return "system";
}
```

---

## Generics and Type Inference

### Generic Type Parameters

Use with Tauri invoke for type-safe responses:

```tsx
// Type parameter specifies return type
const url = await invoke<string>("start_server");
const info = await invoke<ConnectionInfo>("get_connection_info");
```

### Type Inference

Let TypeScript infer types when obvious:

```tsx
// Good - TypeScript infers the type
const [version, setVersion] = useState<string>("");
const [theme, setThemeState] = useState<Theme>(defaultTheme);

// Avoid over-specifying
const [count, setCount] = useState<number>(0); // Redundant, could be useState(0)
```

---

## Forbidden Patterns

### 1. Avoid `any`

```tsx
// Bad
const data: any = JSON.parse(text);

// Good - use unknown with type guard
const data: unknown = JSON.parse(text);
if (isExpectedShape(data)) {
  // ...
}
```

### 2. Avoid Non-Null Assertions

```tsx
// Bad
const element = document.getElementById("root")!;

// Good - handle null case
const element = document.getElementById("root") as HTMLElement;
```

### 3. Avoid Type Assertions for Data

```tsx
// Bad - assumes shape without validation
const info = response as ConnectionInfo;

// Good - validate or use type guard
const info: ConnectionInfo = await invoke("get_connection_info");
```

---

## Common Patterns

### Optional Props with Defaults

```tsx
interface ThemeProviderProps {
  children: React.ReactNode;
  defaultTheme?: Theme;
  storageKey?: string;
}

function ThemeProvider({
  children,
  defaultTheme = "system",
  ...props
}: ThemeProviderProps) {
  // ...
}
```

### Event Handler Types

```tsx
// Use React's built-in types
const handleClick = (e: React.MouseEvent<HTMLButtonElement>) => {};
const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {};
```
