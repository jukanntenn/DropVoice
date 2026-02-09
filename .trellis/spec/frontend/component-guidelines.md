# Component Guidelines

> How components are built in this project.

---

## Overview

DropVoice uses React functional components with TypeScript. The project follows a simple, flat component structure without complex patterns like HOCs or render props.

---

## Component Structure

### Standard Pattern

```tsx
// 1. Imports (React, external libs, internal modules)
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "./ui/button";

// 2. Types/Interfaces (inline for component-specific, exported for shared)
interface QRCodeSectionProps {
  url: string;
}

// 3. Component function (default export)
export default function QRCodeSection({ url }: QRCodeSectionProps) {
  const { t } = useTranslation();

  return (
    <div className="...">
      {/* JSX */}
    </div>
  );
}
```

### Key Files to Reference

- `src/components/QRCodeSection.tsx` - Simple presentational component
- `src/components/Header.tsx` - Component with state and async effects
- `src/components/LanWarning.tsx` - Component with Tauri integration

---

## Props Conventions

### Pattern

```tsx
// Export props interface for external use
export interface QRCodeSectionProps {
  url: string;
}

// Destructure props directly in function signature
export default function QRCodeSection({ url }: QRCodeSectionProps) {
  // ...
}
```

### Rules

1. **Props interfaces** are defined above the component and exported
2. **Optional props** use `?` with default values in destructuring
3. **Event handlers** are passed as props with `on` prefix: `onClick`, `onClose`
4. **Children** use React.ReactNode type

### Example with Multiple Props

```tsx
// src/components/ui/button.tsx
export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean;
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, asChild = false, ...props }, ref) => {
    // ...
  },
);
```

---

## Styling Patterns

### Tailwind CSS with Class Merging

All styling uses Tailwind CSS with the `cn()` utility for conditional classes:

```tsx
import { cn } from "../../lib/utils";

<div className={cn(
  "base-classes",
  condition && "conditional-classes",
  className
)} />
```

### Dark Mode

Dark mode classes are applied inline using `dark:` prefix:

```tsx
<div className="bg-white dark:bg-slate-900 text-slate-900 dark:text-white">
```

### Common Patterns

| Pattern | Classes |
|---------|---------|
| Card container | `rounded-2xl border bg-white/80 backdrop-blur-sm dark:bg-slate-900/80` |
| Button base | `rounded-xl transition-colors hover:bg-secondary` |
| Gradient background | `bg-gradient-to-br from-teal-50 via-white to-cyan-50` |

---

## Accessibility

### Required Patterns

1. **Interactive elements** must have `aria-label`:
   ```tsx
   <Button aria-label={t("settings.title")}>
   ```

2. **Form inputs** must have associated labels

3. **Tooltips** wrap triggers with `TooltipTrigger asChild`:
   ```tsx
   <Tooltip>
     <TooltipTrigger asChild>
       <Button aria-label={t("theme.light")}>
   ```

---

## Common Mistakes

### 1. Forgetting to handle async cleanup

```tsx
// Bad - race condition on unmount
useEffect(() => {
  fetchData().then(setData);
}, []);

// Good - cleanup with cancelled flag
useEffect(() => {
  let cancelled = false;
  fetchData().then(data => {
    if (!cancelled) setData(data);
  });
  return () => { cancelled = true; };
}, []);
```

**Reference**: `src/components/LanWarning.tsx:12-33`

### 2. Not using `cn()` for conditional classes

```tsx
// Bad
<div className={`base ${isActive ? 'active' : ''}`}>

// Good
<div className={cn("base", isActive && "active")}>
```

### 3. Catching errors without handling

```tsx
// Acceptable in this project - silent failures for non-critical operations
try {
  await invoke("set_language", { lang });
} catch {
  // Fallback to just changing UI language
  await i18n.changeLanguage(lang);
}
```

**Reference**: `src/components/Header.tsx:43-50`
