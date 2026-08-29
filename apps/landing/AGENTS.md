# apps/landing — standing orders

Static marketing site (Vite + React 19, dev on :5175). Built from the product's own design system — `@dropvoice/ui` (tokens, Card/QRCode/DeviceSelector, LiquidBackground) and `@dropvoice/i18n` (`landing` namespace) — never a parallel design system.

## Orders

- Site facts and version constants live in `src/lib/site.ts` only — repo/release/PWA links change there and nowhere else.
- Copy comes from the `landing` i18n namespace (all four locales); never hardcode user-facing strings.
- The Hero demo (`src/demo/`) is a pure-function timeline (`script.ts`, tested) + `DemoStage` — keep the timeline pure.
- Commands: `pnpm dev:landing` (dev), build via workspace `pnpm build:landing`. Lint/format run through prek (`apps/landing/prek.toml`).
