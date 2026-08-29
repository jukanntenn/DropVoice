# 架构

DropVoice 组成方式的现状事实。决策理由在 [DV-RFCs](../.agents/dv-rfcs/README.md)；操作步骤在 [development.md](development.zh.md)；各子树指令在各自的 `AGENTS.md`。

## 组成

DropVoice 经 WebRTC DataChannel（P2P；手机 = offerer，桌面 = answerer）把手机语音转文字送到 PC。公网 pairing-server 只中继信令——数据绝不经过它。

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

## 技术栈（精确版本）

- **React** 19.0.0、**TypeScript** 5.9.0、**Vite** 6.4.0
- **Tauri** 2（API ^2.10.0、CLI ^2.8.0、8 个插件）
- **Rust** edition 2021，MSRV **1.93**（workspace `Cargo.toml` 的 `rust-version`）
- **axum** 0.8（ws feature；**仅 pairing-server**——desktop 不依赖 axum）、**sqlx** 0.8（sqlite、migrate、macros、chrono；bundled libsqlite3）
- **tower-http** 0.7（trace、cors、fs；**仅 pairing-server**）、**tokio** 1（full）
- **Tailwind CSS** 4.2.0、**jotai** 2.18、**@tanstack/react-query** 5.90
- **enigo** 0.6（键盘注入）、**webrtc** 0.17（webrtc-rs：desktop 应答面 DataChannel）、**directories** 6（路径解析）
- **vitest** 4.1、**@playwright/test** 1.49+、**oxlint** 0.15、**prettier** 3.6
- **Node** >=22、**pnpm** >=11（`packageManager pnpm@11.0.0`）

Cargo 与 pnpm 两个 workspace 都把版本集中在根；成员经 `workspace = true`（Cargo）或 `workspace:*`（pnpm）继承。根已声明的版本绝不在成员内联固定。

## 配置单一源

env 是最后手段；每个旋钮恰好有一个事实源：

| 关注点                       | 单一源                                                                                                                          | 说明                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| ---------------------------- | ------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Desktop → pairing-server URL | `config.toml` `network.pairing_server_url`（用户文件 `<config_dir>/dropvoice/config.toml`，默认 `https://ps.dropvoice.online`） | 在 Rust 中解析（`pairing_client::resolve_base_url`）；webview 绝不碰 API（信令全在 Rust 进程）。`PAIRING_SERVER_URL` env 仅为开发编排覆盖，由 `.vscode/tasks.json` 注入。`PAIRING_SERVER_INSECURE=1` 仅在 Rust HTTP 腿跳过 TLS 校验（自签逃生口，绝不用于生产）。                                                                                                                                                                                |
| pairing-server 应用配置      | 依次探测的 TOML：`/app/config.toml`（容器）→ `apps/pairing-server/config.local.toml`（裸 `cargo run`，可选）                    | 模板：`config.local.toml.example`（裸跑便利，已提交）、`docker/config.acceptance.toml`（验收，已提交）、`docker/config.example.toml`（自部署复制源）、`devops/ansible/templates/config.toml.j2`（由 `group_vars` 渲染；cors_origins 留空——标准拓扑没有跨源调用方）。Dev 经任务 env 注入一切（`LISTEN_ADDR`/`DATABASE_URL`/`RATE_LIMIT_PER_SEC`/`ENABLE_DOCS`），优先级高于任何 TOML 文件。存在但损坏的配置文件直接启动失败（不静默回退默认值）。 |
| Caddy 路由                   | `docker/Caddyfile`（镜像默认/验收，单主机 HTTP 形态）+ `devops/ansible/templates/Caddyfile.j2`（部署环境）                      | j2 模板用同一三主机布局同时渲染 staging（HTTP，tunnel 终结 TLS）与 production（TLS，Cloudflare Origin CA）：`landing_domain` → landing-dist，`app_domain` → mobile-dist + `/api` 反代（PWA 同源），`api_domain` → 纯 API。真实客户端 IP：server 级 `trusted_proxies` + `trusted_proxies_strict` + `header_up X-Forwarded-For {client_ip}`——axum 的最左 XFF 解析于是总是得到真实客户端 IP，不受信的直连对端也被改写回真实地址。                   |
| 各环境部署值                 | `devops/ansible/group_vars/<env>.yml`                                                                                           | 由 `deploy.yml` 渲染进 `config.toml`、`docker-compose.yml` 与 Caddyfile。staging（`*.dropvoice.bytehome.fun`）与 production（`dropvoice.online` / `app.` / `ps.`）1:1 对应——翻转域名/TLS/受信代理值即可让 staging 原样搬进 Cloudflare。                                                                                                                                                                                                          |
| Mobile API base              | 无——PWA 永远与 API 同源（开发：Vite 代理；部署：app 主机反代）                                                                  | 开发代理目标经 `VITE_API_PROXY_TARGET`（默认 `localhost:7380`，`dev:mobile` 任务注入；验收为 `:8080`，见 `accept:mobile`）。不存在构建期 API-base 变量。                                                                                                                                                                                                                                                                                         |

完整细节见配置规范（`specs/full/01-configuration.md`）。

## 配对流程（WebRTC 信令）

1. 桌面 `start_server` 生成本地配对码（内存态）+ 启动心跳（向公网 pairing-server 注册 → `pairing_token`，分发进程内 token 总线）+ 信令监督任务（SSE 订阅）。注册是凭据受控的幂等 upsert：已存在设备必须携带当前 Bearer token（复用/轮换），裸重注册 401——token 只发给持有者，绝不复述（device_id 印在 QR 里）。桌面 401 时自动重置设备身份（需重扫码）。token 落库只存 SHA-256 哈希。
2. 手机扫二维码（`dropvoice://pair?code=&device=&name=`），向 pairing-server POST WebRTC offer（`POST /api/devices/{id}/webrtc/offer`）。
3. pairing-server 经桌面的 SSE 订阅中继 offer（`POST .../webrtc/subscribe` 用 Bearer 换一次性 60s 票据 → `GET /api/devices/{id}/webrtc/events?ticket=`；长效 token 不进 URL/访问日志），由 **Rust 信令监督任务**消费：校验 credential（本地 code/token 比对）→ webrtc-rs 应答器生成 answer → 回填。SSE 活性靠 reqwest chunk 级 read_timeout（45s）确定性检出半开连接，重建完全在 Rust 时钟内（tokio 定时器休眠唤醒后立即触发）。
4. 手机长轮询 answer（`GET .../answer/{session_id}`）；收到 `accepted` 后 `setRemoteDescription` → DataChannel 建立。
5. 文本经 DataChannel 流动；桌面收到 `{type:"text"}` → `invoke inject_text`（Enigo 键盘注入）。桌面在通道上发放连接 `token`；手机此后凭 `token` 重连。

桌面的两条腿（心跳注册 + SSE 订阅）都在 Rust 进程内，并经同一单一源解析服务器 URL，永不分叉。webview 是纯视图：绝不参与信令（WebView2 冻结隐藏页面的 JS 定时器/网络，托盘隐藏加系统休眠唤醒曾让 SSE 变死）。

## 桌面连接态事件总线

桌面后端推送 Tauri 事件，把"何时轮换/重连"（后端）与"如何渲染"（前端）解耦。1s 一次的 `get_connection_info` 轮询保留作 `active_connections`/`clients`/`queue_depth` 的兜底。

| 事件                   | Payload                | 触发                                     | 消费方                            |
| ---------------------- | ---------------------- | ---------------------------------------- | --------------------------------- |
| `pairing_code_rotated` | `{ code, qr_payload }` | 配对码轮换定时器                         | 桌面前端：立即更新 QR/链接/重连码 |
| `client_registered`    | `{ client_id }`        | Rust 应答面 DataChannel open（注册成功） | 桌面前端：关闭"添加设备"浮层      |

`client_unregistered` 没有消费者（YAGNI），不发射。

## 端口

| 服务                                                 | 端口        |
| ---------------------------------------------------- | ----------- |
| Desktop 前端（Vite dev）                             | 5173        |
| Mobile PWA（Vite dev）                               | 5174        |
| Landing 页（Vite dev）                               | 5175        |
| Pairing server，dev 裸跑（`dev:pairing-server` env） | 7380        |
| Pairing server，容器内（loopback，Caddy 之后）       | 38424       |
| 容器 Caddy，验收（HTTP）/ staging 主机               | 8080 / 8888 |
| 容器 Caddy，生产（TLS，Cloudflare Origin CA）        | 4443        |

> WebRTC 架构下，数据经 P2P DataChannel 直连，不经任何服务器端口。桌面不绑定本地 HTTP/WS 服务器或 UDP 发现 —— 均已删除。
