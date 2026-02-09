# Quality Guidelines

> Code quality standards for frontend development.

---

## Overview

DropVoice follows standard React/TypeScript best practices with emphasis on:
- Clean, readable code
- Proper error handling
- Dark mode support
- Internationalization (i18n)

---

## Forbidden Patterns

### 1. Direct DOM Manipulation

```tsx
// Bad
document.getElementById("root").style.color = "red";

// Good - use React state
const [color, setColor] = useState("red");
<div style={{ color }}>...</div>
```

### 2. Storing Derived State

```tsx
// Bad - duplicates state
const [items, setItems] = useState([]);
const [itemCount, setItemCount] = useState(0);

// Good - derive from source
const items = useState([]);
const itemCount = items.length;
```

### 3. Missing Dark Mode Support

```tsx
// Bad - no dark mode
<div className="bg-white text-slate-900">

// Good - include dark mode
<div className="bg-white dark:bg-slate-900 text-slate-900 dark:text-white">
```

### 4. Hardcoded Strings (No i18n)

```tsx
// Bad
<p>Loading...</p>

// Good
<p>{t("app.loading")}</p>
```

---

## Required Patterns

### 1. Async Cleanup

All async effects must handle unmount:

```tsx
useEffect(() => {
  let cancelled = false;

  async function load() {
    const data = await fetchData();
    if (!cancelled) setData(data);
  }

  load();
  return () => { cancelled = true; };
}, []);
```

### 2. Error Boundaries

Wrap root with ErrorBoundary:

```tsx
// src/main.tsx:9-11
<ErrorBoundary>
  <App />
</ErrorBoundary>
```

### 3. Accessibility Labels

All interactive elements need aria-labels:

```tsx
<Button aria-label={t("settings.title")}>
  <Settings />
</Button>
```

### 4. TypeScript Strict Mode

All code must pass strict TypeScript checks:
- No implicit `any`
- Proper null checks
- Correct return types

---

## Testing Requirements

### Current State

The project has minimal automated testing. Manual testing is done via:

1. **Desktop**: `pnpm tauri dev`
2. **Mobile**: Open browser on mobile device, connect to desktop

### Testing Checklist

When adding new features:

- [ ] Desktop UI renders correctly
- [ ] Dark mode styling is correct
- [ ] Language switching works
- [ ] Tauri commands execute successfully
- [ ] Mobile interface works in browser
- [ ] WebSocket connection/reconnection works

---

## Code Review Checklist

### Before Committing

- [ ] TypeScript compiles without errors
- [ ] No hardcoded strings (use i18n)
- [ ] Dark mode classes are complete
- [ ] Async operations have cleanup
- [ ] Interactive elements have aria-labels
- [ ] No console.log in production code

### Component Review

- [ ] Props interface is exported
- [ ] Default export for component
- [ ] Uses `cn()` for conditional classes
- [ ] Cleanup in useEffect return

### Hook Review

- [ ] Return type interface defined
- [ ] Refs used for non-reactive values
- [ ] Cleanup on unmount
- [ ] Stable function references with useCallback

---

## Common Mistakes

### 1. Missing i18n Keys

```tsx
// Bad - will show key name if missing
<p>{t("newFeature.title")}</p>

// Before adding, check if key exists in:
// - src/locales/en.json
// - src/locales/zh.json
```

### 2. Inconsistent Styling

Follow existing patterns:

```tsx
// Card styling pattern
<div className="rounded-2xl border border-slate-200 bg-white/80 backdrop-blur-sm dark:border-slate-600 dark:bg-slate-900/80">

// Button styling pattern
<Button variant="ghost" size="icon" className="rounded-xl">
```

### 3. Breaking Mobile/Desktop Separation

```tsx
// Bad - using Tauri API in mobile code
import { invoke } from "@tauri-apps/api/core";
// This will fail in mobile browser

// Mobile code should only use:
// - Browser APIs
// - WebSocket
// - localStorage
```
