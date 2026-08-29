# apps/desktop — standing orders

Tauri 2 desktop app: Rust backend in `src-tauri/` (crate `dropvoice-desktop`), React 19 frontend in `src/`. The WebRTC DataChannel answerer — receives text and injects it via keyboard simulation. Commands, flows and ports live in [docs/architecture.md](../../docs/architecture.md); procedures in [docs/development.md](../../docs/development.md).

## Commands

- Dev: VS Code task `dev:desktop` (zero-config; sets `PAIRING_SERVER_URL`), or `pnpm dev:tauri` with `PAIRING_SERVER_URL=http://localhost:7380` exported.
- Build: `pnpm build:tauri` (installer) / `pnpm build` (frontend only).
- Tests: `pnpm test:rust` (cargo, manifest `src-tauri/Cargo.toml`) + desktop vitest via `pnpm test`. Lint/format/clippy run through prek (`apps/desktop/prek.toml`).

## Layout

`src-tauri/src/`: `commands/` (Tauri commands: server, settings, window), `server/` (仅 auth：配对码 + 连接令牌), `connection.rs` (ConnectionManager：客户端跟踪 + Enigo 注入队列), `config/` (`config.toml`; unknown TOML fields ignored, no migration), `network/` (heartbeat, pairing_client, signaling SSE 监督), `webrtc/` (webrtc-rs 应答面：每 offer 一个 PC，DataChannel 协议 text/hello/token/ack 直连注入队列), `text/injector.rs` (EnigoInjector + MockInjector), `telemetry/`, `lib.rs` (8 plugins, system tray, invoke_handler).

`src/`: `App.tsx` (QueryClientProvider + JotaiProvider, auto-start server + event-bus 订阅), `hooks/`, `components/`, `lib/invoke.ts` (typed Tauri invoke wrapper).

## Orders

- The webview is a pure view: it never touches the API or participates in signaling — signaling lives entirely in the Rust process (WebView2 freezes hidden-page JS timers/network).
- Both Rust legs (heartbeat, SSE) resolve the server URL through the single config source `network.pairing_server_url`; `PAIRING_SERVER_URL` is the dev-orchestration override only; `PAIRING_SERVER_INSECURE=1` skips TLS on the Rust HTTP leg only, never in prod.
- Connection-state updates flow over the Tauri event bus (`pairing_code_rotated`, `client_registered`); a 1s `get_connection_info` poll remains as data fallback.
- Rust errors are typed via thiserror (`AppError` + `error_code()` + `AppResult`); Tauri commands are `snake_case`, invoked as `invoke::<T>("snake_case_name")`.
- Injection goes through the Enigo queue (configurable delay), never straight to the OS — ordering with multi-chunk text depends on it.
