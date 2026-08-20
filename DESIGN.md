---
version: alpha
name: DropVoice Design System
description: Visual language for DropVoice desktop (Tauri) and mobile (PWA) surfaces.
colors:
  # Primary — Teal. Used for primary actions, active device highlight, brand, connection success.
  # primary-600 (#0d9488) is the light-mode anchor; primary-500 (#14b8a6) the dark-mode anchor.
  primary: '#0d9488'
  primary-50: '#f0fdfa'
  primary-100: '#ccfbf1'
  primary-200: '#99f6e4'
  primary-300: '#5eead4'
  primary-400: '#2dd4bf'
  primary-500: '#14b8a6'
  primary-600: '#0d9488'
  primary-700: '#0f766e'
  primary-800: '#115e59'
  primary-900: '#134e4a'
  primary-950: '#042f2e'
  # Accent — Orange. The single CTA accent; used sparingly for emphasis.
  accent: '#f97316'
  accent-50: '#fff7ed'
  accent-100: '#ffedd5'
  accent-200: '#fed7aa'
  accent-300: '#fdba74'
  accent-400: '#fb923c'
  accent-500: '#f97316'
  accent-600: '#ea580c'
  accent-700: '#c2410c'
  accent-800: '#9a3412'
  accent-900: '#7c2d12'
  accent-950: '#431407'
  # Neutral — backgrounds, surfaces, text. Pairs with light/dark themes.
  neutral-0: '#ffffff'
  neutral-50: '#f9fafb'
  neutral-100: '#f3f4f6'
  neutral-200: '#e5e7eb'
  neutral-300: '#d1d5db'
  neutral-400: '#9ca3af'
  neutral-500: '#6b7280'
  neutral-600: '#4b5563'
  neutral-700: '#374151'
  neutral-800: '#1f2937'
  neutral-900: '#111827'
  neutral-950: '#030712'
  # Semantic
  success-500: '#10b981'
  success-600: '#059669'
  warning-500: '#f59e0b'
  warning-600: '#d97706'
  danger-500: '#ef4444'
  danger-600: '#dc2626'
  info-500: '#0d9488'
  info-600: '#0f766e'
typography:
  sans:
    fontFamily: Inter
    fontSize: 16px
    fontWeight: 400
    lineHeight: 1.6
  mono:
    fontFamily: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace
    fontSize: 16px
    fontWeight: 400
    lineHeight: 1.5
rounded:
  sm: 0.375rem
  md: 0.5rem
  lg: 0.75rem
  xl: 1rem
  2xl: 1.25rem
  3xl: 1.5rem
  full: 9999px
spacing:
  1: 4px
  2: 8px
  3: 12px
  4: 16px
  6: 24px
  8: 32px
  12: 48px
  16: 64px
components:
  # Liquid-glass surfaces. Token values are context for agents; the live CSS lives in
  # the hand-written effects layer (backdrop-blur / glass shadow / gradient) because
  # the DESIGN.md exporter cannot emit shadow or animation tokens.
  glass-card:
    backgroundColor: rgba(255, 255, 255, 0.8)
    textColor: '{colors.neutral-900}'
    rounded: '{rounded.2xl}'
    padding: 16px
  glass-card-elevated:
    backgroundColor: rgba(255, 255, 255, 0.7)
    textColor: '{colors.neutral-900}'
    rounded: '{rounded.3xl}'
    padding: 24px
  glass-pill:
    backgroundColor: rgba(255, 255, 255, 0.8)
    textColor: '{colors.neutral-900}'
    rounded: '{rounded.full}'
    padding: 8px
  button-primary:
    backgroundColor: '{colors.primary-600}'
    textColor: '#ffffff'
    rounded: '{rounded.lg}'
    height: 40px
    padding: 16px
  button-primary-hover:
    backgroundColor: '{colors.primary-700}'
  button-ghost:
    backgroundColor: transparent
    textColor: '{colors.neutral-600}'
    rounded: '{rounded.xl}'
  send-cta:
    backgroundColor: '{colors.primary-600}'
    textColor: '#ffffff'
    rounded: '{rounded.full}'
    size: 64px
---

# DropVoice Design System

> Version: 2.0.0
> Last Updated: 2026-08-11
> Owner: DropVoice Team

## Overview

DropVoice speaks in **Liquid Glass**: translucent, frosted surfaces floating over a calm
teal-to-cyan gradient, softened by ambient color orbs. The mood is light, premium, and
unhurried — a focused utility that feels calm rather than busy. Density is low; whitespace
is generous; the primary teal (`#0d9488`) carries the brand and all primary action.

The system spans a desktop console (Tauri, opened occasionally to pair/manage/glance) and a
mobile typing surface (PWA, the daily high-frequency surface). Both share the same glass
language; the mobile surface is thumb-first and breathing, the desktop surface is a focused
single column.

## Colors

### Primary — Teal

The single brand and action color. Scale 50–950 follows the standard Tailwind teal ramp.
`primary-600` (`#0d9488`) anchors light mode; `primary-500` (`#14b8a6`) brightens for dark
mode. Used for the brand tile gradient, the QR glow, focus rings, the Send CTA, switch-on,
and active device highlight.

### Accent — Orange

`accent-500` (`#f97316`) is the sole warm accent. Used sparingly — never as a default
button. Reserved for genuine emphasis (e.g. warning tints pair amber, not accent).

### Neutral

A cool gray ramp (`neutral-0` white → `neutral-950` near-black) for text, borders, and flat
surfaces. Light surfaces sit on `neutral-0`/`neutral-50`; dark surfaces on `neutral-900`/
`neutral-950`. Note: the page **background** is not a flat neutral — it is the liquid
gradient (see Elevation & Depth).

### Semantic

- `success` — connected state, send success
- `warning` — degraded connection, LAN-safety banner, near-limit char count
- `danger` — disconnected state, destructive actions, errors
- `info` — aliases primary teal for informational pills

## Typography

**Inter** is the single typeface, loaded via `@fontsource/inter` (weights 400/500/600/700,
`font-display: swap`). The full fallback stack (system-ui, Segoe UI, PingFang SC, Hiragino
Sans GB, Microsoft YaHei) guarantees legibility before load and on locale without Inter.
Body is set at 16px / 1.6 line-height. Hierarchy is carried by weight (semibold for
wordmark and titles) and modest size steps — never by color. Monospace (`ui-monospace` …)
is reserved for connection payloads, pairing codes, and device ids.

> Implementation note: the real font stack is authored in `effects.css` as
> `--font-sans: "Inter", system-ui, …` because the DESIGN.md exporter quote-wraps the whole
> `fontFamily` value (which would collapse the comma list into one family name). The
> `typography.sans.fontFamily: Inter` above is the canonical family name and prose context.

## Layout & Spacing

A 4px base grid (spacing 1–16). Desktop is a single centered column (`max-w-sm`, 384px),
content stacked with `gap-8` rhythm — focused, never a wall of cards. Mobile is a
`max-w-md` column with safe-area-aware padding (`env(safe-area-inset-*)`), the textarea as
the hero, and a three-button thumb-reachable action row anchored near the bottom.

## Elevation & Depth

Depth is the signature — **Liquid Glass**, not flat shadows.

- **Background:** a diagonal gradient — light mode `teal-50 → white → cyan-50`, dark mode
  `slate-950 → slate-900 → slate-950`. Never a flat neutral.
- **Ambient orbs:** two large `blur-3xl` color circles fixed behind content
  (`bg-primary/10` top-right; `bg-cyan-400/10` bottom-left, dimmed to `cyan-500/5` in dark).
- **Glass surfaces:** `rounded-2xl`/`rounded-3xl`, translucent fills
  (`bg-white/80` light, `bg-slate-900/70` dark), `backdrop-blur-md`/`xl`, thin translucent
  borders (`border-white/60` light, `border-white/10` dark).
- **Shadows (authored in effects.css):**
  - `glass`: `0 8px 32px 0 rgba(13, 148, 136, 0.12)` — teal-tinted, the default card shadow.
  - `glass-dark`: `0 8px 32px 0 rgba(0, 0, 0, 0.25)` — dark-mode card shadow.
  - `glow`: `0 0 40px rgba(13, 148, 136, 0.15)` — ambient halo behind the QR.
  - `glow-accent`: `0 0 40px rgba(249, 115, 22, 0.2)`.
- **Hover** intensifies border and shadow rather than shifting color.

## Motion

Restrained and purposeful. Authored as `@keyframes` + `--animate-*` tokens in effects.css.

- `slide-up`: `translateY(10px) → 0` on content entrance (cards, banners).
- `fade-in`, `scale-in`: subtle appear transitions.
- `shimmer`: background-position sweep for skeleton/loading.
- Press feedback: `active:scale-95` on tactile buttons (Send, chips).
- Connecting states pulse softly (`animate-pulse`). Respect `prefers-reduced-motion`.

## Shapes

Generous rounding reinforces the soft, organic feel. `rounded-lg` (0.75rem) for inputs and
buttons; `rounded-2xl`/`rounded-3xl` for glass cards and modals; `rounded-full` for pills,
dots, and the circular Send CTA. Mixed sharpness is avoided — corners stay rounded
throughout.

## Components

### Glass containers

The primary surface. `rounded-2xl`/`3xl`, `bg-white/80 backdrop-blur-md shadow-glass`,
translucent border. Elevated/modal variants raise blur to `xl` and opacity slightly. Every
glass surface pairs a soft border with the teal-tinted glass shadow to "float" over the
gradient without feeling heavy.

### QR presentation

The connection centerpiece. A 192px pure-white card (always white so the code scans in both
themes) with a `from-primary/20 to-cyan-400/20 blur-xl` glow behind it. Foreground
`#0f172a`, error-correction level M, transparent background.

### Action elements

Buttons use `rounded-lg`/`full`. The **Send CTA** on mobile is a 64px circular
`from-primary to-teal-500` gradient with `shadow-primary/30` and `active:scale-95` — the
single most prominent control on the screen, flanked by two 48px glassy circular secondary
buttons (Restore, Clear). Ghost icon buttons (`h-9 w-9 rounded-xl`) carry header controls.

### Inputs

Translucent (`bg-white/50` light, `bg-slate-800/50` dark), `rounded-2xl`, focus ring
`ring-4 ring-primary/10`. The mobile textarea is the hero input (200px min) with a floating
char counter that warms to amber near the limit.

## Do's and Don'ts

- **Do** keep the liquid gradient + orbs on every screen — flat neutrals are not the brand.
- **Do** use `primary` teal for the single primary action per screen.
- **Do** keep glass surfaces translucent with backdrop-blur; never opaque white cards.
- **Don't** mix flat shadows with glass shadows — pick the glass family.
- **Don't** present the 6-digit pairing code as a standalone pairing method (it cannot pair
  alone; the full QR URL is required). Show the code only as a small reconnect credential.
- **Don't** show empty/idle diagnostics (queue depth 0, no devices) — surface them only
  when they carry information.
- **Do** maintain WCAG AA contrast on translucent surfaces (increase opacity where needed).
