# Directory Structure

> How frontend code is organized in this project.

---

## Overview

DropVoice is a Tauri desktop application with two frontend entry points:
1. **Desktop UI** (`src/`) - React app for the desktop application
2. **Mobile UI** (`src/mobile/`) - React app served to mobile browsers via WebSocket

The project uses Vite + React + TypeScript with Tailwind CSS for styling.

---

## Directory Layout

```
src/
├── App.tsx                    # Desktop app root component
├── main.tsx                   # Desktop app entry point
├── i18n.ts                    # i18n initialization for desktop
├── index.css                  # Global styles (Tailwind)
├── components/
│   ├── ui/                    # shadcn/ui components (button, tooltip, select, switch)
│   ├── Header.tsx             # Desktop header with theme/language toggles
│   ├── LanWarning.tsx         # LAN-only security warning banner
│   ├── QRCodeSection.tsx      # QR code display component
│   ├── URLDisplay.tsx         # Connection URL display with copy
│   ├── ThemeProvider.tsx      # Theme context and hook
│   ├── SettingsModal.tsx      # Settings modal dialog
│   └── ErrorBoundary.tsx      # React error boundary
├── mobile/
│   ├── index.tsx              # Mobile app entry point
│   ├── App.tsx                # Mobile app root component
│   ├── i18n.ts                # Mobile i18n (standalone)
│   ├── components/            # Mobile-specific components
│   │   ├── SendButton.tsx
│   │   ├── ClearButton.tsx
│   │   ├── RestoreButton.tsx
│   │   ├── StatusIndicator.tsx
│   │   ├── Toast.tsx
│   │   ├── TextInput.tsx
│   │   ├── SettingsButton.tsx
│   │   └── SettingsPage.tsx
│   ├── hooks/
│   │   ├── useWebSocket.ts    # WebSocket connection management
│   │   └── useSettings.ts     # Theme/language settings hook
│   └── utils/
│       └── storage.ts         # localStorage utilities
├── lib/
│   └── utils.ts               # cn() utility for Tailwind class merging
└── locales/
    ├── en.json                # English translations
    └── zh.json                # Chinese translations
```

---

## Module Organization

### Desktop vs Mobile Separation

The desktop (`src/`) and mobile (`src/mobile/`) apps are separate React applications:
- Desktop uses Tauri APIs (`@tauri-apps/api/core`)
- Mobile uses browser APIs only (served via HTTP/WebSocket)
- They share no components directly but follow similar patterns

### Component Categories

1. **UI Components** (`components/ui/`) - Reusable primitives from shadcn/ui
   - Button, Tooltip, Select, Switch
   - Use `cn()` utility for class composition

2. **Feature Components** (`components/`) - Desktop-specific features
   - Business logic components
   - Tauri command invocations

3. **Mobile Components** (`mobile/components/`) - Mobile-specific features
   - Touch-optimized UI
   - WebSocket-based communication

---

## Naming Conventions

| Type | Convention | Example |
|------|------------|---------|
| Components | PascalCase | `QRCodeSection.tsx`, `LanWarning.tsx` |
| Hooks | camelCase with `use` prefix | `useWebSocket.ts`, `useSettings.ts` |
| Utilities | camelCase | `utils.ts`, `storage.ts` |
| CSS classes | kebab-case (Tailwind) | `bg-primary`, `dark:text-slate-400` |
| Type interfaces | PascalCase | `ConnectionInfo`, `QRCodeSectionProps` |

---

## Examples

### Well-organized module: `src/mobile/hooks/`

```
mobile/hooks/
├── useWebSocket.ts    # Encapsulates WebSocket connection logic
└── useSettings.ts     # Encapsulates theme/language persistence
```

Each hook:
- Exports a single hook function
- Defines return type interface
- Handles cleanup in useEffect
- Uses refs for mutable values that don't trigger re-renders
