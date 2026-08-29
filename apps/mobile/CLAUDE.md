# apps/mobile — standing orders

Standalone Vite + React 19 PWA: the phone-side typing surface. WebRTC DataChannel offerer — connects to the desktop via DataChannel, signaling relayed by the public pairing-server. System facts live in [docs/architecture.md](../../docs/architecture.md).

## Commands

- Dev: VS Code task `dev:mobile` (zero-config; Vite proxies `/api` to :7380), or `pnpm dev:mobile` with `VITE_API_PROXY_TARGET=http://localhost:7380`.
- Tests: mobile vitest via `pnpm test`. Lint/format/typecheck run through prek (`apps/mobile/prek.toml`).

## Layout

`src/`: `App.tsx` (multi-device manager, QR-scan pairing, send modes), `components/`, `hooks/` (`useDraft`, `useConnections`), `lib/` (`rtcTransport.ts`, `storage.ts`). PWA assets in `public/` (`sw.js`).

## Orders

- The PWA is always same-origin with the API (dev: Vite proxy; deployed: app host reverse proxy). No build-time API-base variable exists — never introduce one.
- The QR payload is `dropvoice://pair?code=&device=&name=`; reconnects use the connection `token` the desktop issued over the channel, not a re-pair.
- State machines are pure reducers from `@dropvoice/core` — never local ad-hoc connection state.
- UI copy comes from `@dropvoice/i18n` (all four locales); never hardcode user-facing strings.
- The service worker (`public/sw.js`) is hand-maintained: bump the cache version when shipped assets change.
