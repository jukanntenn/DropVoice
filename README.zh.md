# DropVoice

[English](README.md) | 中文

通过局域网把手机上的语音输入发送到 PC。

## 快速开始

### 前置条件

- [Node.js](https://nodejs.org/) ≥ 22
- [pnpm](https://pnpm.io/) ≥ 11
- [Rust](https://www.rust-lang.org/) ≥ 1.75

### 安装

```bash
git clone https://github.com/jukanntenn/DropVoice.git
cd DropVoice
pnpm install
```

### 开发

```bash
# Start the full Tauri app (frontend + backend)
pnpm tauri dev

# Frontend only (for UI development)
pnpm dev

# Mobile PWA (accessible on LAN)
pnpm dev:mobile
```

### 构建

```bash
pnpm build          # Build frontend
pnpm tauri build    # Build production installer
```

### 测试

```bash
pnpm test                    # Frontend tests
pnpm test:rust               # Rust backend tests
pnpm test:all                # All tests
cargo test -p dropvoice-pairing-server  # Pairing server tests
```

## 架构

DropVoice 是一个 Tauri 应用，包含：

- **桌面应用**（`apps/desktop/`）— Tauri v2 + React + Rust 后端
- **移动端 PWA**（`apps/mobile/`）— React + WebSocket 客户端
- **配对服务器**（`apps/pairing-server/`）— 用于设备发现的 Axum HTTP 服务
- **共享包**（`packages/`）— 核心逻辑、UI 组件、i18n

```
┌─────────────────┐         ┌──────────────────┐         ┌─────────────────┐
│  Mobile Browser │ ──WS──> │  Tauri Backend   │ ──API──> │  React Frontend │
│  (PWA)          │         │  (Rust/Tokio)    │         │  (src/*.tsx)    │
└─────────────────┘         └──────────────────┘         └─────────────────┘
                                     │
                                     └──> Keyboard Injection (enigo)
```

## 配置

配置保存在 `config.toml`（平台专属配置目录）。

完整配置参考见 [specs/full/01-configuration.md](specs/full/01-configuration.md)。

## 文档

- [配置规范](specs/full/01-configuration.md) — 配置机制
- [架构](docs/architecture.zh.md) / [开发文档](docs/development.zh.md) — 系统如何组合，以及如何参与开发
- [设计系统](DESIGN.md) — 视觉语言与设计 token
- [变更日志](CHANGELOG.md) — 版本历史

## 许可证

MIT
