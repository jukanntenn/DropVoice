# 第三方库选型规范

> 版本: 1.0.0
> 最后更新: 2026-07-16
> 状态: 已批准

## 1. 概述

本规范定义 DropVoice 项目使用的第三方库及其选型依据。

## 2. 前端依赖

### 2.1 核心框架

| 库 | 版本 | 用途 | 说明 |
|---|------|------|------|
| `react` | ^19.0.0 | UI 框架 | 升级到 React 19 |
| `react-dom` | ^19.0.0 | React DOM 渲染 | - |
| `@tauri-apps/api` | ^2.10.0 | Tauri IPC 调用 | - |
| `@tauri-apps/plugin-autostart` | ^2.5.0 | 自启动插件 | - |
| `@tauri-apps/plugin-clipboard-manager` | ^2.3.0 | 剪贴板插件 | - |

### 2.2 UI 组件库

| 库 | 版本 | 用途 | 说明 |
|---|------|------|------|
| `@base-ui/react` | ^1.0.0 | 无样式基础组件 | 替代 Radix UI |
| `tailwindcss` | ^4.2.0 | 原子化 CSS | v4 版本 |
| `tailwind-merge` | ^3.5.0 | Tailwind 类名合并 | 解决类名冲突 |
| `clsx` | ^2.1.1 | 条件类名拼接 | 极小（228 bytes） |
| `class-variance-authority` | ^0.7.1 | 组件变体管理 | 完整方案 |
| `lucide-react` | ^0.511.0 | 图标库 | - |

### 2.3 状态管理

| 库 | 版本 | 用途 | 说明 |
|---|------|------|------|
| `jotai` | ^2.18.0 | 客户端状态管理 | 原子化状态 |
| `@tanstack/react-query` | ^5.90.0 | 服务器状态管理 | 缓存 + 重试 |

### 2.4 表单与验证

| 库 | 版本 | 用途 | 说明 |
|---|------|------|------|
| `react-hook-form` | ^7.65.0 | 表单管理 | - |
| `@hookform/resolvers` | ^5.2.0 | 表单验证解析器 | - |
| `zod` | ^4.1.0 | 数据验证 | Schema 验证 |

### 2.5 通知与反馈

| 库 | 版本 | 用途 | 说明 |
|---|------|------|------|
| `sonner` | ^2.0.0 | Toast 通知 | 替代自实现 |

### 2.6 动效

| 库 | 版本 | 用途 | 说明 |
|---|------|------|------|
| `framer-motion` | ^12.0.0 | 动画库 | - |

### 2.7 国际化

| 库 | 版本 | 用途 | 说明 |
|---|------|------|------|
| `i18next` | ^25.8.0 | 国际化框架 | - |
| `react-i18next` | ^16.5.0 | React 集成 | - |

### 2.8 工具库

| 库 | 版本 | 用途 | 说明 |
|---|------|------|------|
| `qrcode.react` | ^4.2.0 | QR 码生成 | - |
| `html5-qrcode` | ^2.3.8 | QR 码扫描 | - |

## 3. Rust 依赖

### 3.1 核心框架

| 库 | 版本 | 用途 | 说明 |
|---|------|------|------|
| `tauri` | 2 | 桌面应用框架 | features: ["tray-icon"] |
| `axum` | 0.7 | HTTP/WebSocket 服务器 | 替代 hyper |
| `tokio` | 1 | 异步运行时 | features: ["full"] |

### 3.2 Tauri 插件

| 库 | 版本 | 用途 | 说明 |
|---|------|------|------|
| `tauri-plugin-shell` | 2 | Shell 命令 | - |
| `tauri-plugin-clipboard-manager` | 2 | 剪贴板管理 | - |
| `tauri-plugin-autostart` | 2 | 自启动 | - |
| `tauri-plugin-store` | 2 | 持久化存储 | 动态设置 |
| `tauri-plugin-updater` | 2 | 自动更新 | - |
| `tauri-plugin-dialog` | 2 | 原生对话框 | - |
| `tauri-plugin-process` | 2 | 进程管理 | - |
| `tauri-plugin-log` | 2 | 日志 | - |

### 3.3 网络

| 库 | 版本 | 用途 | 说明 |
|---|------|------|------|
| `tokio-tungstenite` | 0.24 | WebSocket 客户端 | - |
| `reqwest` | 0.12 | HTTP 客户端 | 可选，用于信令 |
| `local-ip-address` | 0.6 | 局域网 IP 检测 | - |

### 3.4 文本注入

| 库 | 版本 | 用途 | 说明 |
|---|------|------|------|
| `enigo` | 0.6 | 键盘模拟 | 跨平台文本注入 |

### 3.5 序列化与错误处理

| 库 | 版本 | 用途 | 说明 |
|---|------|------|------|
| `serde` | 1 | 序列化框架 | features: ["derive"] |
| `serde_json` | 1 | JSON 序列化 | - |
| `thiserror` | 2.0 | 错误类型派生 | 结构化错误 |
| `anyhow` | 1 | 通用错误处理 | 应用层错误传播 |

### 3.6 配置与日志

| 库 | 版本 | 用途 | 说明 |
|---|------|------|------|
| `toml` | 0.8 | TOML 解析 | 配置文件格式 |
| `tracing` | 0.1 | 结构化日志 | - |
| `tracing-subscriber` | 0.3 | 日志订阅者 | features: ["env-filter", "json"] |
| `tracing-appender` | 0.2 | 日志文件 appender | 按天轮转 |
| `dirs` | 5 | 系统目录 | - |

### 3.7 遥测

| 库 | 版本 | 用途 | 说明 |
|---|------|------|------|
| `opentelemetry` | 0.27 | OpenTelemetry SDK | - |
| `opentelemetry_sdk` | 0.27 | OTel SDK 实现 | features: ["rt-tokio"] |
| `opentelemetry-otlp` | 0.27 | OTLP 导出器 | features: ["tonic", "metrics", "logs"] |

### 3.8 安全

| 库 | 版本 | 用途 | 说明 |
|---|------|------|------|
| `aes-gcm` | 0.10 | AES 加密 | 本地存储加密 |
| `ed25519-dalek` | 2 | Ed25519 签名 | 更新包签名验证 |
| `uuid` | 1 | UUID 生成 | features: ["v4"] |
| `rand` | 0.8 | 随机数生成 | 配对码生成 |

### 3.9 平台特定

> **实现说明 (2026-07-21):** v0.2 中 `platform/` 模块整体移除（详见
> spec 08 §2.3），下列平台特定 crate 均未引入。当 v1.0 重新实现
> `PlatformExt` 时再按需恢复。

```toml
# v1.0 target — not present in v0.2
[target.'cfg(target_os = "windows")'.dependencies]
windows-sys = { version = "0.61", features = [
    "Win32_Globalization",
    "Win32_UI_Shell",
    "Win32_UI_WindowsAndMessaging",
] }

[target.'cfg(target_os = "macos")'.dependencies]
objc2 = "0.5"
objc2-app-kit = { version = "0.2", features = ["NSColor"] }

[target.'cfg(target_os = "linux")'.dependencies]
xdotool = "0.3"
```

## 4. 开发依赖

### 4.1 前端

| 库 | 版本 | 用途 |
|---|------|------|
| `typescript` | ^5.9.0 | TypeScript 编译器 |
| `vite` | ^6.4.0 | 构建工具 |
| `vitest` | ^4.1.0 | 测试框架 |
| `@vitejs/plugin-react` | ^4.7.0 | Vite React 插件 |
| `@tailwindcss/vite` | ^4.2.0 | Tailwind Vite 插件 |
| `oxlint` | ^0.15.0 | Lint 工具 |
| `prettier` | ^3.6.0 | 格式化工具 |
| `prettier-plugin-tailwindcss` | ^0.6.0 | Tailwind 类名排序 |
| `@testing-library/react` | ^16.0.0 | React 测试库 |
| `@testing-library/jest-dom` | ^6.6.0 | Jest DOM 匹配器 |
| `@testing-library/user-event` | ^14.5.0 | 用户事件模拟 |
| `msw` | ^2.11.0 | Mock Service Worker |
| `jsdom` | ^25.0.0 | DOM 环境 |
| `@google/design.md` | ^0.3.0 | DESIGN.md 工具 |
| `conventional-changelog-cli` | ^5.0.0 | CHANGELOG 生成 |

### 4.2 Rust

| 库 | 版本 | 用途 |
|---|------|------|
| `tauri-build` | 2 | Tauri 构建 |

## 5. 决策依据

### 5.1 UI 框架: @base-ui/react

**参考项目:**
- Yaak: 自建 UI 组件库
- CC-Switch: 使用 Radix UI（shadcn/ui）

**选择理由:**
- MUI 团队维护，专业可靠
- 无样式设计，与 Tailwind 完美配合
- API 设计现代，与 React 19 兼容

**否决方案:**
- ❌ Radix UI: @base-ui/react API 更现代，MUI 团队背书

### 5.2 CSS 框架: Tailwind CSS v4

**选择理由:**
- 业界标准，社区庞大
- 与 DESIGN.md Token 系统集成（css-tailwind 导出）
- v4 性能大幅提升

### 5.3 HTTP 服务器: axum 替代 hyper

**选择理由:**
- 声明式路由，代码简洁
- 内置 WebSocket 支持
- 与 tokio 深度集成
- hyper 过于底层，需要手动处理路由和 WebSocket 升级

**否决方案:**
- ❌ hyper: 过于底层，需要手动处理路由和 WebSocket 升级

### 5.4 错误处理: thiserror + anyhow

**选择理由:**
- thiserror 用于定义结构化错误类型
- anyhow 用于应用层错误传播
- 两者互补

### 5.5 Lint: oxlint 替代 ESLint

**参考项目:**
- Yaak: 使用 oxlint（通过 vp lint）

**选择理由:**
- 比 ESLint 快 100 倍
- 规则兼容 ESLint
- Rust 实现，启动快

**否决方案:**
- ❌ ESLint: 慢，配置复杂

### 5.6 格式化: Prettier（非 oxc formatter）

**选择理由:**
- 支持 CSS/HTML/JSON 格式化
- 有 prettier-plugin-tailwindcss 插件（自动排序类名）
- 社区标准

**否决方案:**
- ❌ oxc formatter: 不支持 CSS，无 Tailwind 插件

### 5.7 Pre-commit: prek

**参考项目:**
- CPython, FastAPI, Home Assistant 等大型项目

**选择理由:**
- Rust 实现，无依赖
- 兼容 pre-commit 配置格式
- 社区采用广泛

**否决方案:**
- ❌ lefthook: 私有配置格式，社区较小

## 6. 接口规范

### 6.1 包管理

```yaml
# pnpm-workspace.yaml
packages:
  - 'apps/*'
  - 'packages/*'
```

### 6.2 Cargo.toml workspace

```toml
[workspace]
members = [
    "apps/desktop/src-tauri",
]
resolver = "2"
```

## 7. 约束

1. **版本锁定**: 使用精确版本号，避免意外升级
2. **依赖审计**: 定期运行 `pnpm audit` 和 `cargo audit`
3. **最小依赖**: 只引入必要依赖，避免过度工程化
4. **React 19**: 所有 React 相关依赖必须兼容 React 19
