# State Management

> How state is managed in this project.

---

## Overview

DropVoice uses a **lightweight state management** approach:
- **Local state** with `useState` for component-level state
- **Context API** for theme/settings shared across components
- **Tauri backend** as the source of truth for persisted settings
- **No global state library** (no Redux, Zustand, etc.)

---

## State Categories

### 1. Local Component State

Used for UI-only state that doesn't need to be shared:

```tsx
// src/components/Header.tsx:22-23
const [version, setVersion] = useState<string>("");
const [isSettingsOpen, setIsSettingsOpen] = useState(false);
```

### 2. Context-Based Global State

Used for theme and settings that need to be accessed from multiple components:

```tsx
// src/components/ThemeProvider.tsx
const ThemeProviderContext = createContext<ThemeProviderState>(initialState);

export function ThemeProvider({ children }: ThemeProviderProps) {
  // State managed here
  const [theme, setThemeState] = useState<Theme>(defaultTheme);

  // Context value
  const value = { theme, setTheme, toggleTheme, ... };

  return (
    <ThemeProviderContext.Provider value={value}>
      {children}
    </ThemeProviderContext.Provider>
  );
}

export const useTheme = () => useContext(ThemeProviderContext);
```

### 3. Backend-Synced State

State that persists across app restarts is managed by Tauri backend:

```tsx
// Save to backend
await invoke("set_theme", { theme: next });

// Load from backend
const saved = await invoke<string>("get_theme");
```

**Reference**: `src/components/ThemeProvider.tsx:68-70`

---

## When to Use Global State

In this project, use Context when:

1. **Multiple components** need the same state (theme, language)
2. **State needs to persist** beyond component lifecycle
3. **State changes trigger** effects in multiple places

### Current Global State

| Context | Provider | Consumers |
|---------|----------|-----------|
| Theme | `ThemeProvider` | Header, SettingsModal |
| i18n | `I18nextProvider` | All components using `useTranslation()` |

---

## State Persistence

### Desktop App (Tauri)

Settings are persisted via Tauri commands to app data directory:

| Setting | Backend Command | Storage |
|---------|-----------------|---------|
| Language | `set_language` / `get_language` | `language-preference.txt` |
| Theme | `set_theme` / `get_theme` | `theme-preference.txt` |
| LAN warning | `set_lan_warning_dismissed` | `lan-warning-dismissed.txt` |

**Reference**: `src-tauri/src/commands.rs:209-213`

### Mobile App (Browser)

Settings are persisted to localStorage:

```tsx
// src/mobile/hooks/useSettings.ts:7-8
const THEME_KEY = "dropvoice-mobile-theme";
const LANGUAGE_KEY = "dropvoice-mobile-lang";

function saveTheme(theme: Theme): void {
  try {
    localStorage.setItem(THEME_KEY, theme);
  } catch {}
}
```

---

## Server State

### Polling Pattern

For server connection info, the desktop app uses polling:

```tsx
// src/App.tsx:49-56
const interval = setInterval(async () => {
  try {
    const info = await invoke<ConnectionInfo>("get_connection_info");
    if (info.url) setServerUrl(info.url);
  } catch (err) {}
}, 1000);

return () => clearInterval(interval);
```

**Note**: This could be optimized to event-driven in the future.

### WebSocket State

Mobile app uses custom hook for WebSocket state:

```tsx
const { status, isSending, lastError, send } = useWebSocket();
```

The hook manages:
- Connection status
- Reconnection attempts
- Send queue
- Error state

---

## Common Mistakes

### 1. Not syncing with backend

```tsx
// Bad - only updates local state
setTheme(next);

// Good - persists to backend and updates local state
const setTheme = async (next: Theme) => {
  setThemeState(next);
  await invoke("set_theme", { theme: next });
};
```

**Reference**: `src/components/ThemeProvider.tsx:61-73`

### 2. Prop drilling instead of Context

```tsx
// Bad - passing through multiple layers
<App>
  <Header theme={theme} setTheme={setTheme}>
    <Navigation theme={theme} setTheme={setTheme}>

// Good - use Context
<App>
  <ThemeProvider>
    <Header>
      <Navigation>
```

### 3. Not handling loading state

```tsx
// Good pattern from ThemeProvider
const [isLoading, setIsLoading] = useState(true);

// Show loading spinner until state is loaded
if (isLoading) {
  return <LoadingSpinner />;
}
```

**Reference**: `src/App.tsx:59-70`
