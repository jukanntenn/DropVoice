# 验收前规范符合性差距报告

> 版本: 1.1.0
> 审计日期: 2026-07-25
> 最后更新: 2026-07-25
> 状态: 审计快照（供分期实施决策参考）

### 1.0.0 → 1.1.0 变更记录

以下缺口已在 2026-07-25 修复：

| 原编号 | 缺口 | 修复内容 |
|--------|------|---------|
| A1 | 配对服务器客户端零接入 | ✅ 已有 `pairing_client.rs`（register/generate_code/report_status） |
| A3 | 启动序列 step 5/6 缺失 | ✅ 已有 `heartbeat.rs`（T3）+ `ip_monitor.rs`（T2） |
| A4 | T2 IP 变化检测未实现 | ✅ 已有 `ip_monitor.rs` |
| A5 | T3 心跳任务未实现 | ✅ 已有 `heartbeat.rs` |
| A6 | §8 IP 变化检测整章未实现 | ✅ 已有 `ip_monitor.rs`（30s 检测周期） |
| A7 | Token 过期处理未实现 | ✅ 已有 `heartbeat.rs`（401 → re-register） |
| A8 | config 缺 device_id | ✅ 已有 `DeviceConfig.device_id` |
| A9 | config 缺 device_name | ✅ 已有 `DeviceConfig.device_name` |
| A10 | config 缺 pairing_token | ✅ 已有 `DeviceConfig.pairing_token` |
| A11 | 配对码位数不一致 | ✅ 已统一：配对服务器 6 位，桌面本地 8 位，WebSocket 认证两者均支持 |
| B1-B4 | 配对码认证双轨断裂 | ✅ 已统一：WebSocket `/ws` 支持本地码 + 远程码验证 + 远程码缓存 |
| B5 | connection token 不持久化 | ✅ 已持久化到 `DeviceConfig.connected_tokens` |
| C1 | ConnectedDevices 仅显示数量 | ✅ 已扩展：后端返回 `clients` 数组，前端显示 IP + 连接时间 |
| C2 | InjectionQueueStatus 不显示队列深度 | ✅ 已扩展：后端返回 `queue_depth`，前端显示队列深度 |
| C3 | theme.css 缺 DESIGN.md 令牌 | ✅ 已添加 shadow/motion/z-index CSS 变量 |
| D1 | release.yml 缺 Windows 签名 | ✅ 已存在 `WINDOWS_CERTIFICATE` + `WINDOWS_CERTIFICATE_PASSWORD` |
| D2 | updater pubkey 为空 | ✅ 已生成 Ed25519 密钥对并设置 pubkey |
| I1 | ConnectionReducer 未使用 | ✅ 已验证实际已在 `useMultiWebSocket.ts` 中使用 |
| I3 | desktop 前端零测试 | ✅ 已创建 4 个组件测试文件（LanWarningBanner/QRCodeSection/ConnectedDevices/InjectionQueueStatus） |
| I4 | E2E 测试缺失 | ✅ 已创建 Playwright 配置 + 基础 E2E 测试 |
| I5 | memory_mb 硬编码 0 | ✅ 已实现 `get_process_memory_mb()`（Windows GetProcessMemoryInfo） |
| I6 | 速率限制策略未区分 | ✅ 已在 spec 13 分别文档化 |
| I8 | stop_server 文档过时 | ✅ 已更新 CLAUDE.md（graceful shutdown） |
| I9 | 部署配置文件缺失 | ✅ 已从 git 恢复 `apps/pairing-server/deploy/` |
| I10 | 自动更新 UI 缺失 | ✅ 已创建 `useAutoUpdate` hook，启动时检查更新并显示 toast |
| M2 | 错误码不一致 | ✅ 已验证 Rust/TS 22 个错误码完全对齐 |
| M7 | OpenAPI /ws 端点缺失 | ✅ 已存在于 `desktop-local.yaml`，描述已更新 |

## 1. 概述

本文档是 DropVoice 项目验收前的**全面规范符合性审计报告**，逐条对照
`specs/full/` 全部 19 份规范（01-19，20 已废弃，21 为 pairing-server 测试补全）与
`apps/`（desktop / mobile / pairing-server）、`packages/`（core / ui / i18n）的实现，
找出所有"规范要求但未实现 / 实现与规范不符"的缺口。

**目的**：为验收前的分期实施提供单一事实源。每条缺口均附 spec 行号 + 实现
`file:line` 证据，无猜测。

## 2. 审计范围与方法

### 2.1 覆盖范围

| 层面 | 规范 | 实现 |
|------|------|------|
| 规范 | `specs/full/01-*.md` ~ `19-*.md`、`openapi/*.yaml` | — |
| 实现 | — | `apps/desktop/`（src-tauri Rust + src React）、`apps/mobile/`（src React）、`apps/pairing-server/`（Rust axum）、`packages/`、根目录配置 |

### 2.2 三态标记

- ✅ 已实现：规范要求已正确落地
- ⚠️ 部分实现：有对应代码但与规范有偏差或缺失部分
- ❌ 未实现：规范要求但代码中找不到

### 2.3 严重程度

- 🔴 阻塞：影响验收硬指标 / 核心功能不可用 / 安全或正确性问题
- 🟠 重要：影响验收质量 / 体验或完整性受损
- 🟡 轻微：文档级差距 / 命名偏移 / 远期可补救
- ⚪ 合理延后：spec 自身声明推迟（v1.0+/v2.0.0），不计为缺口

## 3. 差距汇总（按严重程度）

### 3.1 🔴 阻塞级缺口（22 项）

#### A. 连接系统核心（spec 11）— 12 项

| # | 缺口 | 规范依据 | 实现现状 |
|---|------|---------|---------|
| A1 | 配对服务器 4 端点客户端 0 接入（注册/配对码/查询/心跳） | spec11 §5.1 L275-278 | desktop/mobile 无任何调用（grep 零命中） |
| A2 | 方式1 自动发现未接入 UI（baseIp 无合法来源，规范缺陷） | spec11 §6.1.4 L440 | `scanSubnet` 零调用；PWA 无法读本机 IP |
| A3 | 启动序列 step 5/6 缺失（注册 + 三后台任务） | spec11 §7.1 L673-674 | `lib.rs` 只到 step 4 |
| A4 | T2 IP 变化检测任务未实现 | spec11 §7.2 L685 | 零实现 |
| A5 | T3 状态上报/心跳任务未实现 | spec11 §7.2 L686 | 零实现 |
| A6 | §8 IP 变化检测整章未实现（广播 IP 固化永不更新） | spec11 §8 L721-778 | `discovery.rs:119` 用 `Arc<String>` 固化 IP |
| A7 | Token 过期处理未实现 | spec11 §7.5 L709 | 无 token |
| A8 | config 缺 device_id 字段 | spec11 §7.1 L670 / spec14 | `config/mod.rs` 无此字段 |
| A9 | config 缺 device_name 字段且设置页不可改 | spec11 §6.6 L636 / §16约束8 | 每次启动重新派生（代码自认"过渡占位"） |
| A10 | config 缺 pairing_token 字段 | spec11 §7.5 L717 | 无此字段 |
| A11 | 配对码位数三方不一致（阻塞扫码） | spec11 §2.1=6 / spec13 §2.2=8 | desktop/mobile 实现=8 |
| A12 | 三层降级链 UI 完全未实现 | spec11 §3.3 L191 | 无降级流转 |

#### B. 国际化（spec 15）— 2 项

| # | 缺口 | 规范依据 | 实现现状 |
|---|------|---------|---------|
| B1 | 后端 `i18n.rs` 完全缺失 | spec15 §5约束5 L298 | `find apps/desktop/src-tauri/src -name i18n.rs` 零结果 |
| B2 | 系统托盘菜单 Show/Hide/Quit 硬编码英文 | spec15 §5约束5 L298 | `lib.rs:74-76` 硬编码字符串 |

#### C. UI 与多设备（spec 07 / spec 10）— 3 项

| # | 缺口 | 规范依据 | 实现现状 |
|---|------|---------|---------|
| C1 | 桌面端 ConnectedDevices 仅显示数量，不显示手机列表/IP/平台 | spec07 L104-108 / spec10 §2.5 L92 | `ConnectedDevices.tsx:24` TODO；`get_connection_info` 不返回 clients |
| C2 | 注入队列深度/状态未暴露前端，InjectionQueueStatus 名不副实 | spec10 §2.5 L94 | `connection_manager.rs:139` 有方法但 commands 无暴露；组件只显示 delay/maxTextLength |
| C3 | theme.css 缺 shadow/motion/z-index 三类 DESIGN.md 令牌 | spec07 §5约束1 L284 | `theme.css` 无 `--shadow-*`/`--duration-*`/`--z-*` |

#### D. 打包与 CI（spec 09 / spec 19）— 2 项

| # | 缺口 | 规范依据 | 实现现状 |
|---|------|---------|---------|
| D1 | release.yml 未注入 Windows 代码签名 secret | spec09 §2.7 L104 / spec19 §2.5 L292 | `release.yml:93-101` env 块缺 `WINDOWS_CERTIFICATE` |
| D2 | updater pubkey 为空 + 前端更新 UI/接口缺失 | spec09 §4.1 L164 / §4.2 | `tauri.conf.json:50` pubkey=""；前端零调用 |

#### E. 测试（spec 06）— 3 项

| # | 缺口 | 规范依据 | 实现现状 |
|---|------|---------|---------|
| E1 | desktop 前端零测试 | spec06 §2.4 L48-55 | `find apps/desktop/src -name "*.test.*"` 为空 |
| E2 | L3 集成测试层 + MSW 完全缺失（MSW 装了未用） | spec06 §2.2 L29 / §3.1 | 无集成测试文件；msw 仅 devDeps 声明 |
| E3 | desktop 覆盖率门控被注释禁用 + Rust 覆盖率无度量 | spec06 §5约束4 L283 | `apps/desktop/vitest.config.ts:24` thresholds 注释掉；无 tarpaulin/llvm-cov |

### 3.2 🟠 重要级缺口（精选，约 25 项）

| # | 缺口 | 规范 | 实现现状 |
|---|------|------|---------|
| F1 | `device.ts:125 hasExhaustedRetries` 误用 `MAX_DEVICES` 而非 `MAX_RETRIES`（数值巧合掩盖潜在 bug） | spec02 §5约束 L344 | `device.ts:125` |
| F2 | languageAtom 默认 `'en'`（规范 `'zh'`），类型用 `string` 而非 `SupportedLanguage` | spec02 §2.2 L31 | `atoms.ts:9` |
| F3 | `react-hook-form` / `@hookform/resolvers` 完全未装 | spec03 §2.4 L45-46 | package.json 无 |
| F4 | `opentelemetry` / `opentelemetry_sdk` / `opentelemetry-otlp` 三个遥测 crate 缺失 | spec03 §3.7 L135-137 | Cargo.toml 无 |
| F5 | `aes-gcm`（本地存储加密）/ `ed25519-dalek`（更新签名）缺失 | spec03 §3.8 L143-144 | Cargo.toml 无 |
| F6 | axum 版本分裂（desktop 0.7 vs workspace 0.8）；tower-http 版本分裂（0.5 vs 0.7） | spec03 §3.1 L82 | `src-tauri/Cargo.toml:32` vs 根 `Cargo.toml:19` |
| F7 | `useErrorHandler` 丢弃 `formatError` 返回的 suggestion，toast 永不显示建议 | spec04 §4.4 / §2.5 | `useErrorHandler.ts:23` 只解构 title/description |
| F8 | 缺 primitives `Tooltip` / `Switch` / `Tabs` 三个原子组件 | spec07 §2.3 L50-77 | `packages/ui/src/primitives/` 仅 8 文件 |
| F9 | `tauri-plugin-autostart` 装了未接通任何 command/UI | spec08 §2.3 / §5 | `Cargo.toml:27` 装；src 零调用 |
| F10 | macOS 无障碍权限检查缺失，注入会静默失败 | spec08 §2.5 L139 | 无 `check_accessibility` |
| F11 | `settings:maxTextLength` i18n key 未定义（非英语显示英文 fallback） | spec15 §5约束1 | `InjectionQueueStatus.tsx:32`；4 语言 settings.json 无此 key |
| F12 | i18n 提取/校验脚本（i18next-parser、validate-translations.js）及 CI 步骤全缺 | spec15 §2.7 L226-229 | package.json 无 i18n:* 脚本；scripts/ 无对应文件 |
| F13 | `.vscode/settings.json` 仅 1 项（规范要求 6 项 formatOnSave/prettier/oxlint/rust-analyzer/tsdk） | spec17 §2.5 L94-105 | `.vscode/settings.json:1-3` |
| F14 | 仓库根无 README.md（快速开始无承载文档） | spec17 §2.2 L22-34 | `find` 未命中 |
| F15 | CHANGELOG.md 缺 `[0.2.0]` 发布段（代码版本已是 0.2.0） | spec18 §6约束2 | `CHANGELOG.md` 只有 `[未发布]` 和 `[0.1.0]` |
| F16 | IP 获取失败 fallback `127.0.0.1`（无"用上次 IP"容错，会让广播发错地址） | spec11 §7.3 L696 / §8.5 L771 | `discovery.rs:36-37` |
| F17 | 移动端把 IP 拼进 device name（违反 §6.6.3） | spec11 §6.6.3 L657 | `discovery.ts:81` `PC-${ip末段}`；`manager.ts:116` `PC-${ip}` |
| F18 | 桌面端无 access log（未挂 TraceLayer） | spec11 §11.3 | `server/mod.rs:100` 无 TraceLayer（pairing-server 有） |
| F19 | 配对码本地永不过期、不自动重新生成（即使不接服务器） | spec11 §6.2.1 L528-531 | `server/mod.rs:36` 只初始 issue 一次 |
| F20 | 多网卡场景只显示单一 IP | spec11 §6.3.2 L604 | `discovery.rs:36` 取 `local_ip()` 单值 |
| F21 | 设备状态机缺 `Active`/`Sending` 生命周期，SELECT/SEND/ACK 在 reducer 内 no-op | spec02 §2.5 L121-138 | `device.ts:11` |
| F22 | sync-version.js 未覆盖 workspace 根 Cargo.toml 与 tauri.conf.json 的 version | spec18 §2.5 | `scripts/sync-version.js:9` 只同步 src-tauri/Cargo.toml |
| F23 | stun.rs 函数签名与 spec 12 偏移（`resolve_public_address` 同步 vs 规范 `async get_public_address`） | spec12 §4.5 L157 | `network/stun.rs:1-13` |
| F24 | `@base-ui/react` 写成精确 `1.6.0` 而非 `^1.0.0`，且 primitives 多用原生 HTML | spec03 §2.2 L27 | `packages/ui/package.json:24` |
| F25 | `dirs`（desktop）与 `directories`（workspace/pairing-server）两个功能重叠 crate 并存 | spec03 §3.6 L129 | `src-tauri/Cargo.toml:46` vs 根 `Cargo.toml:46` |

### 3.3 ⚪ 合理延后（spec 自认推迟，不计缺口）

| 项 | 规范声明 | 说明 |
|----|---------|------|
| AES-256-GCM 本地加密、Ed25519 更新签名 | spec13 §5约束2/3（v1.0+） | spec 自认推迟 |
| OpenTelemetry / OTLP 导出 | spec16 §2（v1.0） | spec 自认推迟 |
| platform/ 平台抽象层 | spec03 §3.9（v0.2 移除，v1.0 恢复） | spec 自认推迟 |
| P2P / Relay 跨网络 | spec12（v2.0.0） | 远期规范，占位已存在 |
| stun.rs / signaling.rs 完整实现 | spec11（v1.1.0/v2.0.0） | 占位文件存在 |
| health memory_mb | spec16 §2.4 | 自认 future task |

## 4. 按规范分章的详细差距清单

> 因篇幅，本章仅列出有缺口的规范章节。完整对照表见审计过程记录。

### 4.1 spec 01 — 项目结构

| 项 | 规范要求（行号） | 实现现状 | 三态 / 严重 |
|----|----------------|---------|------------|
| 01-1 | `src-tauri/src/platform/` 平台抽象层（mod/windows/macos/linux） | 完全缺失（L93-97） | ⚠️ spec03 §3.9 声明 v0.2 移除，但 spec01 未同步标注 — 🟡 |
| 01-2 | desktop/mobile 多个组件命名/位置不符（Header→HeaderBar、ConnectionPill→ConnectionStatusPanel、SettingsPage→SettingsDialog、TextInput→TextInputPanel、utils→lib 等） | 多处命名差异 | 🟠 |
| 01-3 | mobile 缺 `QRScanner.tsx` / `ManualInputForm.tsx` / `NetworkModeIndicator.tsx` / `UsageAlert.tsx` | 文件缺失（部分功能并入 AddDeviceModal） | 🟡（NetworkMode/UsageAlert 属远期） |
| 01-4 | `packages/ui/primitives/` 缺 Tooltip/Switch/Tabs | 仅 8 个原子组件 | 🟠 |
| 01-5 | `pnpm-workspace.yaml` 含未替换占位文本 `msw: set this to true or false` | 占位未清理 | 🟠 |

### 4.2 spec 02 — 状态管理

| 项 | 规范要求（行号） | 实现现状 | 三态 / 严重 |
|----|----------------|---------|------------|
| 02-1 | `hasExhaustedRetries` 应引用 `MAX_RETRIES` | `device.ts:125` 误用 `MAX_DEVICES`（数值同为 5 巧合掩盖） | 🔴 潜在 bug |
| 02-2 | languageAtom 默认 `'zh'` | `atoms.ts:9` 默认 `'en'`，类型 `string` 而非 `SupportedLanguage` | 🟠 |
| 02-3 | 设备状态机含 Active/Sending 生命周期 | `device.ts:11` 缺这两态，SELECT/SEND/ACK no-op | 🟠 |

### 4.3 spec 03 — 第三方库

| 项 | 规范要求（行号） | 实现现状 | 三态 / 严重 |
|----|----------------|---------|------------|
| 03-1 | `react-hook-form` ^7.65.0 + `@hookform/resolvers` ^5.2.0 | 全项目未装 | 🟠 |
| 03-2 | `@tauri-apps/plugin-autostart`（JS 侧） | 仅 Rust 侧装，JS 包未装 | 🟡 |
| 03-3 | `opentelemetry` / `opentelemetry_sdk` / `opentelemetry-otlp` | 三个遥测 crate 缺失 | 🟠 |
| 03-4 | `aes-gcm` 0.10 / `ed25519-dalek` 2 | 安全 crate 缺失 | 🟠 |
| 03-5 | axum 0.7 / tower-http（版本统一） | desktop 0.7/0.5 vs workspace 0.8/0.7 版本分裂 | 🟠 |
| 03-6 | `dirs` 5（规范点名） | desktop 用 dirs，workspace 引入 directories 6，两 crate 并存 | 🟠 |
| 03-7 | `@base-ui/react` ^1.0.0 | 写成精确 `1.6.0`，且 primitives 多用原生 HTML | 🟠 |

### 4.4 spec 04 — 错误处理

| 项 | 规范要求（行号） | 实现现状 | 三态 / 严重 |
|----|----------------|---------|------------|
| 04-1 | Rust AppError 枚举 22 个错误码 | 全量覆盖 ✅ | ✅ |
| 04-2 | `error_code`/`error_context`/`suggestion` 三方法 | 完整且超集 ✅ | ✅ |
| 04-3 | 前端 ERROR_CODES/ERROR_I18N_MAP 与 Rust 同步 | 22 码全映射 ✅ | ✅ |
| 04-4 | `useErrorHandler` 透出 suggestion | `useErrorHandler.ts:23` 丢弃 suggestion | 🟠 |

### 4.5 spec 05 — 代码质量

| 项 | 规范要求（行号） | 实现现状 | 三态 / 严重 |
|----|----------------|---------|------------|
| 05-1 | oxlint/prettier/rustfmt/clippy/pre-commit 配置 | 全部存在且符合 ✅ | ✅ |
| 05-2 | `quality` 聚合脚本 | 存在，但不含 Rust 检查（Rust 走 CI 独立 job） | 🟡 |
| 05-3 | CI 质量门禁 | `quality.yml` 完整覆盖 ✅ | ✅ |

### 4.6 spec 06 — 测试

| 项 | 规范要求（行号） | 实现现状 | 三态 / 严重 |
|----|----------------|---------|------------|
| 06-1 | desktop 前端测试（共置 *.test.tsx） | 零测试文件 | 🔴 |
| 06-2 | L3 集成测试层（Vitest + MSW） | MSW 未使用，无集成测试 | 🔴 |
| 06-3 | 覆盖率门控（90/80/70 分档） | 退化为 60% floor，desktop 注释掉 | 🔴 |
| 06-4 | Rust 核心逻辑覆盖率 80% | 无 tarpaulin/llvm-cov，CI 不产覆盖率 | 🔴 |
| 06-5 | Rust 测试目录结构 | 完全一致 ✅ | ✅ |
| 06-6 | E2E（Playwright） | connection/send-text 齐全 ✅ | ✅ |

### 4.7 spec 07 — UI 设计

| 项 | 规范要求（行号） | 实现现状 | 三态 / 严重 |
|----|----------------|---------|------------|
| 07-1 | DESIGN.md → theme.css 同步 | 脚本与 CI 完整 ✅ | ✅ |
| 07-2 | primitives 全套（含 Tooltip/Switch/Tabs） | 缺 3 个 | 🟠 |
| 07-3 | theme.css 含 shadow/motion/z-index 令牌 | 三类令牌缺失 | 🟠 |
| 07-4 | 桌面端显示连接的手机列表（IP/平台） | 仅显示数量 | 🔴 |
| 07-5 | DeviceSelector 仅 2+ 设备显示 | DeviceSelectorPanel 用 `>0`（应为 `>1`） | 🟡 |
| 07-6 | 版本号从单一来源 | HeaderBar/SettingsDialog 硬编码 `0.2.0` | 🟡 |

### 4.8 spec 08 — 跨平台

| 项 | 规范要求（行号） | 实现现状 | 三态 / 严重 |
|----|----------------|---------|------------|
| 08-1 | 三平台打包（Win MSI/NSIS、mac DMG Universal、Linux AppImage/DEB） | release.yml 三平台齐全 ✅ | ✅ |
| 08-2 | 自启动功能 | tauri-plugin-autostart 装了未接通 | 🟠 |
| 08-3 | macOS 无障碍权限检查 | 缺失，注入静默失败 | 🟠 |
| 08-4 | PlatformExt trait | spec §2.3 声明 v0.2 移除，v1.0 恢复 | ⚪ |

### 4.9 spec 09 — 打包分发

| 项 | 规范要求（行号） | 实现现状 | 三态 / 严重 |
|----|----------------|---------|------------|
| 09-1 | Tauri 打包配置（bundle/icon/三平台） | 完整 ✅ | ✅ |
| 09-2 | Windows EV 证书签名 | release.yml 未注入 secret | 🔴 |
| 09-3 | updater pubkey | `tauri.conf.json:50` 为空字符串 | 🟠 |
| 09-4 | 前端 updater 接口 + UpdateDialog | 零实现 | 🟠 |
| 09-5 | Apple 公证链路 | 6 个 APPLE_* secret 注入 ✅ | ✅ |

### 4.10 spec 10 — 多设备

| 项 | 规范要求（行号） | 实现现状 | 三态 / 严重 |
|----|----------------|---------|------------|
| 10-1 | 设备数据模型/持久化/颜色生成 | 完整 ✅ | ✅ |
| 10-2 | 多设备发送模式（active/all/selected） | 完整 ✅ | ✅ |
| 10-3 | 注入队列 FIFO + 状态 UI 显示 | 队列有，状态未暴露前端 | 🔴 |
| 10-4 | 桌面端显示所有连接手机 | 仅显示数量 | 🔴（同 07-4） |

### 4.11 spec 11 — 连接系统

> 详见 §3.1 表 A（12 项阻塞级），此处不重复。

### 4.12 spec 12 — Relay/P2P（远期预留）

| 项 | 规范要求（行号） | 实现现状 | 三态 / 严重 |
|----|----------------|---------|------------|
| 12-1 | stun.rs / signaling.rs / billing/api.ts 占位 | 占位文件存在 ✅ | ✅ |
| 12-2 | TS ConnectionMode 三态类型 | `connection.ts:17` 已预留 ✅ | ✅ |
| 12-3 | p2p.rs / network/mode.ts / UsageAlert.tsx | 缺失（规范未硬性要求占位） | ⚪ 远期 |

### 4.13 spec 13 — 安全

| 项 | 规范要求（行号） | 实现现状 | 三态 / 严重 |
|----|----------------|---------|------------|
| 13-1 | 配对码 5 分钟有效期 / 连接上限 / 队列上限 / 速率限制 / 文本上限 | 全部实现 ✅ | ✅ |
| 13-2 | 配对码位数（spec13 §2.2 写 8 位） | 与 spec11 §2.1（6 位）冲突 | 🔴 规范矛盾（见 §5） |
| 13-3 | AES-256-GCM / Ed25519 | 未实现 | ⚪ spec 自认 v1.0+ |

### 4.14 spec 14 — 配置

| 项 | 规范要求（行号） | 实现现状 | 三态 / 严重 |
|----|----------------|---------|------------|
| 14-1 | config.toml 各段落（meta/app/server/injection/security/updater/telemetry/window/devices） | 全部存在 ✅ | ✅ |
| 14-2 | device_id / device_name / pairing_token 字段 | 全缺 | 🔴（同 A8/A9/A10） |

### 4.15 spec 15 — 国际化

| 项 | 规范要求（行号） | 实现现状 | 三态 / 严重 |
|----|----------------|---------|------------|
| 15-1 | 四语言 × 四命名空间 + 键对齐 | 完整，零缺失 ✅ | ✅ |
| 15-2 | 后端 i18n.rs + 托盘 i18n | 完全缺失 | 🔴 |
| 15-3 | i18n 提取/校验脚本 + CI | 全缺 | 🟠 |
| 15-4 | settings:maxTextLength key | 未定义 | 🟠 |
| 15-5 | TranslationKey 类型导出 | 别名未定义/导出 | 🟡 |

### 4.16 spec 16 — 可观测性

| 项 | 规范要求（行号） | 实现现状 | 三态 / 严重 |
|----|----------------|---------|------------|
| 16-1 | 日志分级 + 按天轮转 + 30 天保留 | 完整 ✅ | ✅ |
| 16-2 | /health 端点 | 存在，但 memory_mb 硬编码 0 | 🟡 |
| 16-3 | OpenTelemetry / OTLP | 未实现 | ⚪ spec 自认 v1.0 |
| 16-4 | BusinessMetrics Gauge/Histogram 类型 | 仅 6 个计数器，缺 Gauge/Histogram | 🟡 |

### 4.17 spec 17 — 开发环境

| 项 | 规范要求（行号） | 实现现状 | 三态 / 严重 |
|----|----------------|---------|------------|
| 17-1 | 开发脚本（dev/build/test/lint/quality） | 全部存在且超集 ✅ | ✅ |
| 17-2 | `.env.example` | 5 项齐全 ✅ | ✅ |
| 17-3 | `.vscode/settings.json` 6 项 | 仅 1 项 | 🟠 |
| 17-4 | 仓库根 README.md | 缺失 | 🟠 |
| 17-5 | Git hooks（prek） | 用 pre-commit 框架（功能等价） | 🟡 |

### 4.18 spec 18 — 版本号

| 项 | 规范要求（行号） | 实现现状 | 三态 / 严重 |
|----|----------------|---------|------------|
| 18-1 | 语义化版本 + 四处同步 | package.json/Cargo.toml/tauri.conf.json 均 0.2.0 ✅ | ✅ |
| 18-2 | sync-version.js | 未覆盖 workspace 根 Cargo.toml 与 tauri.conf.json | 🟡 |
| 18-3 | CHANGELOG.md Keep a Changelog | 缺 `[0.2.0]` 发布段 | 🟠 |
| 18-4 | commitlint | 配置 + hook 齐全 ✅ | ✅ |

### 4.19 spec 19 — CI/CD

| 项 | 规范要求（行号） | 实现现状 | 三态 / 严重 |
|----|----------------|---------|------------|
| 19-1 | quality.yml（frontend/design-sync/backend 三 job） | 逐行一致 ✅ | ✅ |
| 19-2 | release.yml（create-release/build/publish） | 结构完整 ✅ | ✅ |
| 19-3 | Windows 签名 secret | 未注入 | 🔴（同 D1） |

## 5. 规范自身缺陷（需修订 spec，非实现问题）

以下为审计中发现的**规范文本内部矛盾或遗漏**，需修订 spec 而非改实现：

| # | 缺陷 | 证据 |
|---|------|------|
| S1 | **spec11 §6.1.4 scanSubnet 流程未说明 baseIp 来源**：PWA 无法读本机 IP（无 UDP socket、公网分发致 location 非 LAN IP、WebRTC mDNS 不可靠），方式1 自动发现按 spec 描述无法落地 | spec11 §6.1.1 L372-377 + §6.5 L618 + §6.1.4 L440 |
| S2 | **配对码位数规范自相矛盾**：spec11 §2.1/§9.1/OpenAPI = 6 位，spec13 §2.2 = 8 位 | spec11 L76 / spec13 L43 |
| S3 | **spec01 仍列 platform/ 模块为必需**，与 spec03 §3.9"v0.2 已移除"矛盾，spec01 未同步标注 | spec01 L93-97 / spec03 L150-152 |
| S4 | **spec02 §2.4 OPEN/SEND 事件 payload 描述**（带 ws/text）与 §4.1/§5 纯函数约束矛盾（reducer 不能持有 ws） | spec02 L102/L105 vs L234-256/L343 |
| S5 | **spec10 §4.3 DeviceSelector 路径**写旧路径 `components/`，实际在 `composite/` | spec10 L196 |
| S6 | **spec15 §4.1/§4.3 漏列部分错误码**（ServerStartFailed/MessageTooLarge 等），实现已按 §2.3 清单补齐 | spec15 L175-258 |

## 6. 建议的分期实施路径（供决策参考，非本期执行）

> 以下为基于严重程度和依赖关系的建议分期，**仅供参考**，最终由维护者决策。

### 阶段 0：规范修订（先行，解除阻塞）
- S1：产出方式1 baseIp 缺陷的规范修订（mobile 发现机制重新定义）
- S2：裁定配对码位数统一值（建议 6 位，证据权重更大）
- S3/S4/S5/S6：同步规范文本

### 阶段 1：基础设施前置
- A8/A9/A10：config 三字段（device_id/device_name/pairing_token）+ 迁移 + 设置页 device_name 修改
- A11：配对码长度对齐
- F1：修复 hasExhaustedRetries 语义 bug

### 阶段 2：连接系统核心
- A3/A4/A5/A6/A7：启动序列重构 + T2/T3 任务 + §8 IP 变化检测 + token 续期
- A1：配对服务器完整接入（desktop 注册/心跳 + mobile 扫码查询）
- A12：三层降级链 UI

### 阶段 3：i18n 与 UI
- B1/B2：后端 i18n.rs + 托盘菜单国际化
- C1/C2：桌面端手机列表 + 队列状态 UI
- C3：theme.css 补全令牌
- F7/F11/F12：suggestion 透出 + i18n key + 工作流脚本

### 阶段 4：打包与 CI
- D1：Windows 代码签名 secret 注入
- D2：updater pubkey + 前端更新 UI

### 阶段 5：测试补全
- E1/E2/E3：desktop 前端测试 + L3 集成层 + 覆盖率门控恢复

### 阶段 6：重要级清理（F3-F25）
- 按优先级逐项处理

## 7. 附：审计完整性声明

本报告覆盖 `specs/full/` 全部 19 份规范（01-19）。审计基于 2026-07-25 的代码快照，
通过 4 组并行代码探索完成，每条缺口均附 spec 行号 + 实现 `file:line` 证据。

**未覆盖项**：spec 20（已废弃，仅作指针）、spec 21（pairing-server 测试补全，属测试
规范本身非功能规范）。

**复核建议**：实施任一阶段前，建议针对该阶段涉及的缺口重新核对当前代码状态
（代码可能在审计后有变动）。
