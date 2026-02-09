# Frontend Development Guidelines

> Best practices for frontend development in this project.

---

## Overview

This directory contains guidelines for frontend development in DropVoice. DropVoice uses React 18 + TypeScript with Vite, Tailwind CSS for styling, and Tauri for desktop integration.

---

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Directory Structure](./directory-structure.md) | Module organization and file layout | Filled |
| [Component Guidelines](./component-guidelines.md) | Component patterns, props, composition | Filled |
| [Hook Guidelines](./hook-guidelines.md) | Custom hooks, data fetching patterns | Filled |
| [State Management](./state-management.md) | Local state, global state, server state | Filled |
| [Quality Guidelines](./quality-guidelines.md) | Code standards, forbidden patterns | Filled |
| [Type Safety](./type-safety.md) | Type patterns, validation | Filled |

---

## Quick Reference

### Project Type
- **Framework**: React 18 + TypeScript
- **Build Tool**: Vite
- **Styling**: Tailwind CSS with shadcn/ui components
- **Desktop**: Tauri 2.x
- **i18n**: react-i18next

### Key Patterns

| Area | Pattern |
|------|---------|
| Components | Functional components with exported props interface |
| State | useState + Context API, no global state library |
| Styling | Tailwind CSS with `cn()` utility, dark mode inline |
| Backend | Tauri commands via `invoke<T>()` |
| Mobile | Separate React app, WebSocket-based |

### File Structure

```
src/
├── components/          # Desktop UI components
│   └── ui/              # shadcn/ui primitives
├── mobile/              # Mobile browser app
│   ├── components/      # Mobile UI components
│   └── hooks/           # Mobile-specific hooks
├── lib/                 # Utilities (cn)
└── locales/             # i18n translations
```

---

**Language**: All documentation is written in **English**.
