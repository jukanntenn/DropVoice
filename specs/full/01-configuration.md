# 配置机制规范

> 版本: 1.0.0
> 最后更新: 2026-08-15
> 状态: 已批准

## 1. 概述

本规范定义 DropVoice 全部配置面的机制：每个配置项的位置、默认值、优先级与
管辖边界。目标是最小心智负担的启动部署体验：

- **每个环境一条命令**（`dev:full` / `accept:up` / `deploy:*`，无需手工设 env）
- **每个配置一个真相源**（无重复表达，无"改了 A 处 B 处不生效"）
- **默认值覆盖 95% 场景**（环境间真实差异收敛到极少数变量）
- **配置面无假字段**（文件里出现的每个键都真实生效）

本规范描述的是现状（代码权威）；与历史 spec 冲突时以本文件 + 代码为准。

## 2. 设计原则

### 2.1 常量 vs 配置的界限

- **协议/时序参数是常量，不是配置**（SSE 心跳、长轮询 hold、token TTL 等，
  见 §6.3）。它们有测试钉死，改值即改协议语义，不应被运行时配置稀释。
- **运维会调的才是配置**（监听地址、数据库路径、限流、CORS、日志级别）。
- **调试开关归 env 域，不入 TOML**（`ENABLE_DOCS` / `RUST_LOG`）——与
  RUST_LOG 同类的东西进配置文件只会让模板间多一处无意义差异。

### 2.2 优先级

pairing-server 特意设计为 **env > TOML 文件 > 内置默认值**：env 是逃生舱，
常态不用；文件是主战场；默认值保证"零配置也能起"。桌面端反向——
**`config.toml` 是唯一真源**，env 仅一条开发编排覆盖（`PAIRING_SERVER_URL`，
由 `.vscode/tasks.json` 内置，打包应用不设）。

### 2.3 Fail-fast

配置文件**存在但读取/解析失败 → 启动即失败**，绝不静默回落默认值（限流/
CORS/数据库路径的悄然漂移比一次明确的启动失败昂贵得多）。文件**不存在**则
用默认值（或首跑生成，桌面端）。桌面端 `load()` 失败时进程降级为内存默认值
并打日志（配置坏了不应阻止窗口打开）。

### 2.4 无 migration

TOML 反序列化忽略未知字段。删除字段不需要迁移逻辑：旧文件里的已删键被
忽略，下次 save 时自然消失。桌面端因此没有 migration 模块；前提由测试
`unknown_and_removed_fields_are_ignored` 钉死。

## 3. 环境差异矩阵

所有环境摆开后，真实变化的值只有下表；**其余一切都是常量**。评估"要不要加
一个配置项"时，先看它能否让这张表更小，否则应该是常量或代码。

| # | 差异点 | dev（裸跑） | 容器验收 | staging | production | 自部署 |
|---|--------|-------------|----------|---------|------------|--------|
| 1 | 桌面指向的 pairing URL | `:38424`（tasks.json env） | `:8080`（tasks.json env） | 默认值即生产域名，dogfood 机器手改 `config.toml` | 同左 | 同左 |
| 2 | `rate_limit_per_sec` | 10000 | 10000 | 1 | 1 | 1 |
| 3 | `ENABLE_DOCS`（env-only） | true（task 内置） | true（compose 内置） | off | off | off |
| 4 | `cors_origins` | localhost:5173 + tauri 两条（手写模板） | 同 dev（提交的验收模板） | public_url + tauri 两条 + `cors_extra_origins`（j2 推导） | public_url + tauri 两条（j2 推导） | 手写（默认注释掉） |
| 5 | 镜像来源 + tag | — | 本地 build | LAN registry `main`（滚动） | Docker Hub `vX.Y.Z`（pin） | Docker Hub `latest` |
| 6 | TLS 形态 | 无（HTTP） | 无（HTTP :8080） | 内网隧道外部终结（Caddy HTTP） | Cloudflare Origin CA（Caddy :4443） | 部署者自理 |
| 7 | 数据持久化 | 平台数据目录 | 无卷（`down -v` 清空） | named volume | named volume | named volume |
| 8 | mobile API base | Vite proxy → `:38424` | Vite proxy → `:8080` | 同源（Caddy） | 同源（Caddy） | 同源（Caddy） |

## 4. 桌面端配置（apps/desktop）

### 4.1 config.toml

路径：`<config_dir>/dropvoice/config.toml`（Windows
`%APPDATA%\dropvoice\config.toml`；macOS `~/Library/Application
Support/dropvoice/`；Linux `~/.config/dropvoice/`）。首跑不存在则写入默认
文件；`save()` 由设置命令在变更时调用。

**全部字段（13 个静态项 + device 状态），无死字段**：

| 节 | 字段 | 默认 | 消费者 |
|----|------|------|--------|
| `[app]` | `language` | `"en"`（en/zh/zh-TW/ja） | 设置界面、托盘 i18n |
| | `theme` | `"system"` | 设置界面 |
| `[server]` | `port` | `38425` | heartbeat 注册载荷 |
| | `max_connections` | `10` | `ConnectionManager`（`MAX_DEVICES_REACHED`） |
| `[network]` | `pairing_server_url` | `https://api.dropvoice.app` | `resolve_base_url` → 心跳 + `get_signaling_url`（webview SSE） |
| `[injection]` | `delay_ms` | `10`（0–5000） | 注入队列逐字符间隔 |
| | `max_text_length` | `10000` | `inject_text` 命令 + `Injector::inject` 双层校验 |
| | `queue_size` | `100` | 注入队列上限（`QUEUE_FULL`） |
| `[security]` | `pairing_code_expiry_minutes` | `5` | 配对码 TTL + 自主轮换周期 |
| `[telemetry]` | `log_retention_days` | `30` | `cleanup_old_logs` |
| `[window]` | `minimize_to_tray` | `true` | 设置界面展示 |
| `[device]` | `device_id` / `device_name` | 首跑生成（UUIDv4 / hostname 派生） | 心跳注册、QR 载荷 |
| | `pairing_token` / `connected_tokens` | `None`（运行时写入） | 信令鉴权 / 重连免配对（FIFO 上限 100） |

历史删除字段（`meta.*`、`app.version`、`server.host`、`security.pairing_code_length`、
`security.rate_limit_*`、`[updater]`、`telemetry.enabled/output/log_dir`、
`window.width/height/resizable`、`[devices]`）**不得回加**——它们要么无消费者，
要么真源在别处。

### 4.2 tauri.conf.json 的管辖范围

窗口尺寸/最小尺寸、CSP、updater（endpoint + pubkey）归 `tauri.conf.json`，
**不进 config.toml**。桌面 webview 不持有任何构建期 URL：SSE 地址运行时经
`get_signaling_url` 命令从 Rust 获取，与心跳永远同源。

### 4.3 桌面端 env（仅三条，全部逃生舱）

| env | 作用 | 说明 |
|-----|------|------|
| `PAIRING_SERVER_URL` | 覆盖 `network.pairing_server_url` | 开发编排专用（tasks.json 内置），打包应用不设 |
| `PAIRING_SERVER_INSECURE` | `=1/true` 时跳过 TLS 校验 | 自签证书逃生舱；**只作用于 Rust reqwest 腿（心跳），webview SSE/fetch 走系统 TLS 栈不受影响**；生产永不设置 |
| `RUST_LOG` | tracing 过滤器 | 默认 `info` |

### 4.4 固定值（有意不配置化）

配对码恒 6 位数字；连接令牌 `dvct_` + 32 字节 base64url；连接令牌持久化上限
100（FIFO）；心跳周期 5min、退避上限 5min/16s；注入器默认上限
`DEFAULT_MAX_TEXT_LENGTH = 10000`（仅作 serde 默认值）。

## 5. pairing-server 配置（apps/pairing-server）

### 5.1 TOML 键（4 个）

| 键 | 默认 | 说明 |
|----|------|------|
| `listen_addr` | `"127.0.0.1:38424"` | 仅回环，Caddy 反代；部署形态固定勿改 |
| `database_url` | `sqlite://<平台数据目录>/pairing.db?mode=rwc` | 容器内固定 `sqlite:///app/data/pairing.db?mode=rwc` |
| `rate_limit_per_sec` | `1` | 每 IP 每秒；本地/e2e 提到 10000 |
| `cors_origins` | `[]`（= 不启用 CORS） | 桌面 webview 跨域直连需要；PWA 同域无感 |

探测顺序：`/app/config.toml`（容器挂载点）→ `apps/pairing-server/config.local.toml`
（裸跑，gitignored），取第一个**存在**的。

### 5.2 env（优先级最高）

- **逃生舱（覆盖同名 TOML 键，仅临时用）**：`LISTEN_ADDR` / `DATABASE_URL` /
  `RATE_LIMIT_PER_SEC`（解析失败打 warn 并保留原值）/ `CORS_ORIGINS`（逗号分隔）。
- **env-only（无 TOML 对应键）**：`ENABLE_DOCS`（Swagger UI，默认关）、
  `RUST_LOG`、`LOG_DIR`、`APP_VERSION`（镜像构建时由 `VERSION` build-arg 烘焙）。

### 5.3 时间常量（不可配，测试钉死）

`SIGNAL_SESSION_TTL=60s`、`LONG_POLL_HOLD=30s`、`SSE_PING_INTERVAL=15s`、
`SSE_MAX_LIFETIME=240s`、`MAX_SESSIONS_PER_DEVICE=10`、`SDP_MAX_BYTES=64KiB`、
`DEVICE_TOKEN_TTL=24h`、`DEVICE_STATUS_INTERVAL=5min`（离线阈值 2×=10min）、
`SQLITE_BUSY_TIMEOUT_MS=50`、`CLEANUP_INTERVAL_SECS=60`、
`BATCH_WRITE_INTERVAL_SECS=1`、`PAIRING_TOKEN_LEN=64`。见 `src/config.rs`，
由 `time_constants_match_spec` 断言。

## 6. 模板体系（4 份 TOML + 谁消费）

| 模板 | 消费者 | 差异来源 |
|------|--------|----------|
| `config.local.toml.example` | dev 裸跑（手动 `cp`） | rate 10000、cors localhost+tauri |
| `docker/config.acceptance.toml`（提交） | 验收 compose 挂载 | 同上；docs 经 compose env |
| `docker/config.example.toml` | 自部署用户拷贝源 | 全默认（键大多注释掉） |
| `devops/ansible/templates/config.toml.j2` | staging/production 渲染 | 见 §7 |

**cors 推导规则（j2）**：`cors_origins = [public_url] + tauri 固定两条
（http://tauri.localhost, tauri://localhost） + cors_extra_origins（默认 []）`。
环境不再手写 cors 列表；staging 唯一的 extra 是 `http://localhost:5173`
（dev 桌面直连 dogfood 场）。四份模板间**不允许出现 enable_docs**（env-only）。

## 7. 部署配置（ansible）

分层：`group_vars/all.yml`（拓扑常量：端口 38424/8080/4443、数据库路径、
rate=1）→ `group_vars/staging.yml`（LAN registry + `main` 滚动 tag + 隧道
8888 + `cors_extra_origins`）→ `group_vars/production.yml`（Docker Hub +
`expected_version` pin + origin TLS + Cloudflare CIDR）→ `host_vars/`（用户/
家目录）。渲染产物：`config.toml`、`docker-compose.yml`、Caddyfile。

部署经 `devops/deploy.py` 在容器化 runner 中跑 ansible（runner 的
ansible-core / collection 版本 pin 在 `devops/runner/`，requirements.yml 是
集合版本的单一真源）。前置条件：`~/.ansible-vault/dropvoice-<env>.pwd` 与
`~/.ssh`（`pnpm doctor` 可体检）。

## 8. mobile（零配置）

PWA 与 API **恒同源**（dev 走 Vite proxy，部署形态 Caddy 同机反代），请求
恒用相对路径——**不存在构建期 API base 变量**（历史上的 `VITE_API_BASE_URL`
已删除，不得回加）。唯一相关 env 是 `VITE_API_PROXY_TARGET`（dev proxy
目标，默认 `http://localhost:38424`，验收 `:8080` 由 task 内置）。

## 9. 域名策略

| 域名 | 角色 |
|------|------|
| `api.dropvoice.app` | 生产 API。**桌面默认值指向它**（发布应用的唯一正确终态） |
| `releases.dropvoice.app` | 更新器清单（tauri.conf.json 唯一真源） |
| `dropvoice.bytehome.fun` | 家庭内网域 = staging/dogfood 场（隧道终结 HTTPS），**永不作发布默认值** |

生产未上线期间，dogfood 打包桌面需手动把 `config.toml` 的
`network.pairing_server_url` 指回 staging 域名；dev 流程不受影响（env 覆盖）。

## 10. 编排与体检

- **本地开发**：VS Code task `dev:full`（env 全部内置于 tasks.json，含
  `PAIRING_SERVER_URL`、`ENABLE_DOCS`、验收的 `VITE_API_PROXY_TARGET`）。
  CLI 裸跑等价命令见根 `.env.example`（纯文档，不被任何进程加载）。
- **一键体检**：`pnpm doctor`（`scripts/doctor.py`）——工具链版本、prek
  hooks、`config.local.toml`、端口占用、docker daemon、vault 密码文件、
  `~/.ssh`；`[fail]` 非零退出，`[warn]` 只影响对应路径。
- **根 `.env.example`** 与 `docker/.env.example` 同名不同物：前者纯文档，
  后者仅被自部署 compose 插值（`PAIRING_PORT` / `RUST_LOG`）。

## 11. 变更流程（加/改一个配置项的 checklist）

1. 先问：能否是常量（§2.1）？能否让 §3 的差异表更小？
2. Rust/TS 结构体加字段 + serde 默认值 + 消费者（不允许无消费者的字段存在）。
3. pairing-server 项：同步 4 份模板（§6）与 env 逃生舱说明；桌面项：确认
   不与 `tauri.conf.json` 管辖重叠（§4.2）。
4. 若是新的环境差异：落 `group_vars/<env>.yml` + j2 模板，而非新增模板文件。
5. 文档同步：根 `.env.example`、AGENTS.md 配置架构表（+ `cp AGENTS.md
   CLAUDE.md`）、本规范。
6. `pnpm quality` 全绿（含 openapi drift / agents-sync / lockfile 门禁）。
