# Architecture

Current-state facts about how DropVoice is composed. Decision rationale lives in the [DV-RFCs](../.agents/dv-rfcs/README.md); procedures live in [development.md](development.md); per-subtree orders live in each `AGENTS.md`.

## Composition

DropVoice sends voice-to-text input from a mobile phone to a PC over a WebRTC DataChannel (P2P; phone = offerer, desktop = answerer). A public pairing-server relays signaling only — data never transits it.

```
apps/
  desktop/            Tauri 2 desktop app (Rust backend src-tauri/, React 19 frontend src/)
    src-tauri/src/
      commands/       Tauri commands: server.rs, settings.rs, window.rs
      server/         仅 auth（配对码 + 连接令牌生成/校验）
      connection.rs   ConnectionManager：WebRTC 客户端跟踪 + Enigo 注入队列
      config/         DropVoiceConfig (config.toml; no migration — unknown TOML fields are ignored)
      network/        heartbeat, pairing_client, signaling（SSE 订阅监督 + offer 分发）
      webrtc/         Rust WebRTC 应答面（webrtc-rs）：每 offer 一个 PC，DataChannel
                       协议（text/hello/token/ack）直连注入队列——与进程同生命周期，
                       托盘隐藏/休眠唤醒不中断
      text/           injector.rs (enigo keyboard injection, EnigoInjector + MockInjector)
      telemetry/      logging.rs, metrics.rs
      lib.rs          Tauri Builder setup, 8 plugins, system tray, invoke_handler
      main.rs         Entry stub
    src/              React 19 frontend
      App.tsx         QueryClientProvider + JotaiProvider, auto-starts server + event-bus 订阅
      hooks/          useServerState, useAppSettings, useAutoUpdate
      components/     HeaderBar, PairingView, PairingContent, ConnectedStatusCard,
                      LanWarningBanner, SettingsDialog
      lib/invoke.ts   Typed Tauri invoke wrapper
  mobile/             Standalone Vite + React 19 PWA (the phone-side typing surface)
    src/
      App.tsx         Multi-device manager, QR-scan pairing, send modes
      components/     MobileHeader, DeviceSelectorPanel, TextInputPanel, AddDeviceModal, ...
      hooks/          useDraft, useConnections
      lib/            rtcTransport.ts, storage.ts
  pairing-server/     Rust rendezvous (axum 0.8 + sqlx 0.8/SQLite) over public HTTPS
    src/
      api/            mod (router), auth (Bearer + token hash + bounded rate limiter),
                       devices, signaling (sessions + SSE tickets), signaling_handlers,
                       error, rate_limit, state
      store/          mod (open_pool), device_repo (token 只存哈希、不复述)
      domain/         device (pure data structs)
      batch/          BatchWriter (coalesced status writes)
      cache/          in-memory token cache
      config.rs       Time constants + runtime Config
      observability.rs Metrics, tracing init (access log 含 client_ip)
      main.rs         Startup: tracing -> config -> pool+migrate -> tasks -> serve
    migrations/       sqlx migrations (timestamp-prefixed: YYYYMMDDHHMMSS_name.sql)
    docker/           Multi-stage Dockerfile (cargo-chef + frontend builder:
                      mobile-dist + landing-dist), s6-overlay, Caddy; build.py
    devops/           deploy.py, ansible/ (playbook + group_vars + Caddyfile.j2
                      three-host template), runner/
  landing/            Vite + React 19 静态营销页（Liquid Glass，复用 @dropvoice/ui）
    src/
      sections/       Hero, HowItWorks, PrivacySection, MultiDeviceSection,
                      SelfHostSection, FaqSection, FinalCta, Footer, FactsLine
      demo/           Hero 活体演示：script.ts（纯函数时间线，有测试）+ DemoStage
      components/     Nav, QrPopover, ScenarioTabs, SectionKicker
      lib/site.ts     站点常量（repo/releases/PWA URL/version）— 改链接只动这里
packages/
  core/               Pure-TS: state machines (reducers, NOT xstate), jotai atoms, types, hooks
  i18n/               i18next: en, zh, zh-TW, ja x {common,devices,errors,landing,settings}
  ui/                 Design system: primitives, composite, layout; Tailwind v4 tokens
specs/                Approved design specs (read-only)
e2e/                  Playwright e2e tests + config
scripts/              Cross-platform (Python by default) tooling
```

## Tech stack (exact versions)

- **React** 19.0.0, **TypeScript** 5.9.0, **Vite** 6.4.0
- **Tauri** 2 (API ^2.10.0, CLI ^2.8.0, 8 plugins)
- **Rust** edition 2021, MSRV **1.93** (`rust-version` in workspace `Cargo.toml`)
- **axum** 0.8 (ws feature; **pairing-server only** — desktop does not depend on axum), **sqlx** 0.8 (sqlite, migrate, macros, chrono; bundled libsqlite3)
- **tower-http** 0.7 (trace, cors, fs; **pairing-server only**), **tokio** 1 (full)
- **Tailwind CSS** 4.2.0, **jotai** 2.18, **@tanstack/react-query** 5.90
- **enigo** 0.6 (keyboard injection), **webrtc** 0.17 (webrtc-rs：desktop 应答面 DataChannel), **directories** 6 (path resolution)
- **vitest** 4.1, **@playwright/test** 1.49+, **oxlint** 0.15, **prettier** 3.6
- **Node** >=22, **pnpm** >=11 (`packageManager pnpm@11.0.0`)

Both Cargo and pnpm workspaces centralize versions at the root; members inherit via `workspace = true` (Cargo) or `workspace:*` (pnpm). Never pin inline in a member what the root declares.

## Configuration single sources

Env vars are a last resort; every knob has exactly one source of truth:

| Concern                       | Single source                                                                                                                      | Notes                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| ----------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Desktop → pairing-server URL  | `config.toml` `network.pairing_server_url` (user file `<config_dir>/dropvoice/config.toml`, default `https://ps.dropvoice.online`) | Resolved in Rust (`pairing_client::resolve_base_url`); the webview never touches the API (signaling lives entirely in the Rust process). `PAIRING_SERVER_URL` env exists only as the dev-orchestration override, set by `.vscode/tasks.json`. `PAIRING_SERVER_INSECURE=1` skips TLS verification on the Rust HTTP leg only (self-signed escape hatch, never in prod).                                                                                                                                                                                                 |
| pairing-server app config     | TOML file probed at `/app/config.toml` (container) then `apps/pairing-server/config.local.toml` (bare `cargo run`, OPTIONAL)       | Templates: `config.local.toml.example` (bare-run convenience, committed), `docker/config.acceptance.toml` (acceptance, committed), `docker/config.example.toml` (self-deploy copy source), `devops/ansible/templates/config.toml.j2` (rendered from `group_vars`; cors_origins stays empty — the standard topology has no cross-origin caller). Dev injects everything via task env (`LISTEN_ADDR`/`DATABASE_URL`/`RATE_LIMIT_PER_SEC`/`ENABLE_DOCS`), which outranks any TOML file. A present-but-broken config file fails startup (no silent fallback to defaults). |
| Caddy routing                 | `docker/Caddyfile` (image default / acceptance, single-host HTTP form) + `devops/ansible/templates/Caddyfile.j2` (deployed envs)   | The j2 template renders BOTH staging (HTTP, tunnel terminates TLS) and production (TLS, Cloudflare Origin CA) from the same three-host layout: `landing_domain` → landing-dist, `app_domain` → mobile-dist + `/api` reverse proxy (PWA same-origin), `api_domain` → pure API. Real client IP: server-level `trusted_proxies` + `trusted_proxies_strict` + `header_up X-Forwarded-For {client_ip}` — axum's leftmost-XFF parse then always yields the true client IP, and untrusted direct peers are rewritten to their real address.                                  |
| Per-environment deploy values | `devops/ansible/group_vars/<env>.yml`                                                                                              | Rendered into `config.toml`, `docker-compose.yml` and the Caddyfile by `deploy.yml`. Staging (`*.dropvoice.bytehome.fun`) and production (`dropvoice.online` / `app.` / `ps.`) map 1:1 — flipping domain/TLS/trusted-proxy values moves staging behind Cloudflare unchanged.                                                                                                                                                                                                                                                                                          |
| Mobile API base               | none — PWA is always same-origin with the API (dev: Vite proxy; deployed: app host reverse proxy)                                  | Dev proxy target via `VITE_API_PROXY_TARGET` (default `localhost:7380`, injected by the `dev:mobile` task; acceptance `:8080` by `accept:mobile`). No build-time API-base variable exists.                                                                                                                                                                                                                                                                                                                                                                            |

Full details in the configuration spec (`specs/full/01-configuration.md`).

## Pairing flow (WebRTC signaling)

1. Desktop `start_server` issues a local pairing code (in-memory) + starts heartbeat (registers with public pairing-server → `pairing_token`, 分发进程内 token 总线) + 信令监督任务（SSE 订阅）。注册是凭据受控的幂等 upsert：已存在设备必须携带当前 Bearer token（复用/轮换），裸重注册 401——token 只发给持有者，绝不复述（device_id 印在 QR 里）。桌面 401 时自动重置设备身份（需重扫码）。token 落库只存 SHA-256 哈希。
2. Mobile scans QR (`dropvoice://pair?code=&device=&name=`), POSTs a WebRTC offer (`POST /api/devices/{id}/webrtc/offer`) to the pairing-server.
3. Pairing-server relays the offer to the desktop over its SSE subscription（`POST .../webrtc/subscribe` Bearer 换一次性 60s 票据 → `GET /api/devices/{id}/webrtc/events?ticket=`；长效 token 不进 URL/访问日志），由 **Rust 信令监督任务**消费：校验 credential（本地 code/token 比对）→ webrtc-rs 应答器生成 answer → 回填。SSE 活性靠 reqwest chunk 级 read_timeout（45s）确定性检出半开连接，重建完全在 Rust 时钟内（tokio 定时器休眠唤醒后立即触发）。
4. Mobile long-polls the answer (`GET .../answer/{session_id}`); on `accepted`, `setRemoteDescription` → DataChannel established.
5. Text flows over the DataChannel; desktop receives `{type:"text"}` → `invoke inject_text` (Enigo keyboard injection). Desktop issues a connection `token` over the channel; mobile reconnects with `token` thereafter.

Both desktop legs (heartbeat registration + SSE subscription) live in the Rust process and resolve the server URL through the same single source, so they can never diverge. The webview is a pure view: it never participates in signaling (WebView2 freezes hidden-page JS timers/network, which previously left SSE dead after tray-hide + system sleep/wake).

## Desktop connection-state event bus

The desktop backend pushes Tauri events to decouple "when to rotate/reconnect" (backend) from "how to render" (frontend). A 1s `get_connection_info` poll remains as fallback for `active_connections`/`clients`/`queue_depth`.

| Event                  | Payload                | Trigger                                  | Consumer                                                    |
| ---------------------- | ---------------------- | ---------------------------------------- | ----------------------------------------------------------- |
| `pairing_code_rotated` | `{ code, qr_payload }` | Pairing-code rotation timer              | Desktop frontend: update QR/link/reconnect-code immediately |
| `client_registered`    | `{ client_id }`        | Rust 应答面 DataChannel open（注册成功） | Desktop frontend: close the "add device" overlay            |

`client_unregistered` has no consumer (YAGNI) and is not emitted.

## Ports

| Service                                                 | Port        |
| ------------------------------------------------------- | ----------- |
| Desktop frontend (Vite dev)                             | 5173        |
| Mobile PWA (Vite dev)                                   | 5174        |
| Landing page (Vite dev)                                 | 5175        |
| Pairing server, dev bare run (`dev:pairing-server` env) | 7380        |
| Pairing server, in-container (loopback, behind Caddy)   | 38424       |
| Container Caddy, acceptance (HTTP) / staging host       | 8080 / 8888 |
| Container Caddy, production (TLS, Cloudflare Origin CA) | 4443        |

> WebRTC 架构下，数据经 P2P DataChannel 直连，不经任何服务器端口。桌面不绑定本地 HTTP/WS 服务器或 UDP 发现 —— 均已删除。
