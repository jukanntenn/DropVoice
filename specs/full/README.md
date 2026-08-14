# DropVoice 技术规范文档

> 版本: 1.0.0
> 最后更新: 2026-07-16
> 状态: 已批准

## 概述

本文档是 DropVoice 项目的技术规范索引，包含所有设计决策、接口规范和约束条件。

## 规范清单

| # | 规范 | 文件 | 状态 | 行数 |
|---|------|------|------|------|
| 1 | 项目结构与代码架构 | [01-project-structure.md](./01-project-structure.md) | ✅ 已批准 | ~250 |
| 2 | 状态管理与数据流 | [02-state-management.md](./02-state-management.md) | ✅ 已批准 | ~300 |
| 3 | 第三方库选型 | [03-third-party-libraries.md](./03-third-party-libraries.md) | ✅ 已批准 | ~300 |
| 4 | 错误处理 | [04-error-handling.md](./04-error-handling.md) | ✅ 已批准 | ~350 |
| 5 | 代码质量门控 | [05-code-quality.md](./05-code-quality.md) | ✅ 已批准 | ~200 |
| 6 | 测试策略 | [06-testing.md](./06-testing.md) | ✅ 已批准 | ~300 |
| 7 | UI 设计规范 | [07-ui-design.md](./07-ui-design.md) | ✅ 已批准 | ~350 |
| 8 | 跨平台支持 | [08-cross-platform.md](./08-cross-platform.md) | ✅ 已批准 | ~200 |
| 9 | 打包分发与自动更新 | [09-packaging-distribution.md](./09-packaging-distribution.md) | ✅ 已批准 | ~250 |
| 10 | 多设备管理 | [10-multi-device.md](./10-multi-device.md) | ✅ 已批准 | ~300 |
| 11 | 连接系统 | [11-connection-strategy.md](./11-connection-strategy.md) | ✅ 已批准 | ~1310 |
| 12 | Relay/P2P 跨网络 | [12-relay-p2p.md](./12-relay-p2p.md) | ✅ 已批准（预留） | ~200 |
| 13 | 安全 | [13-security.md](./13-security.md) | ✅ 已批准 | ~200 |
| 14 | 配置管理 | [14-configuration.md](./14-configuration.md) | ✅ 已批准 | ~250 |
| 15 | 国际化 | [15-internationalization.md](./15-internationalization.md) | ✅ 已批准 | ~250 |
| 16 | 监控日志可观测性 | [16-observability.md](./16-observability.md) | ✅ 已批准 | ~300 |
| 17 | 本地开发环境 | [17-development.md](./17-development.md) | ✅ 已批准 | ~200 |
| 18 | 版本号与变更日志 | [18-versioning.md](./18-versioning.md) | ✅ 已批准 | ~250 |
| 19 | CI/CD 流水线 | [19-ci-cd.md](./19-ci-cd.md) | ✅ 已批准 | ~250 |
| 20 | 连接系统设计（已废弃） | [20-connection-system.md](./20-connection-system.md) | ⛔ 已合并至 11 | ~43 |
| 21 | 配对服务器测试补全 | [21-pairing-server-test-completeness.md](./21-pairing-server-test-completeness.md) | ✅ 已批准 | ~700 |
| 22 | （预留） | — | — | — |
| 23 | 验收前规范差距报告 | [23-acceptance-gap-audit.md](./23-acceptance-gap-audit.md) | 📋 审计快照 | ~450 |

## 版本规划

| 版本 | 功能 | 时间线 |
|------|------|--------|
| v0.1.0 | 基础功能（局域网直连、QR码、文本注入） | 当前 |
| v0.2.0 | 多设备管理、局域网发现、DESIGN.md | 近期 |
| v1.0.0 | 正式发布（完整测试、代码签名、自动更新） | 目标 |
| v1.1.0 | 连接系统（配对服务器 + 完整连接策略） | 中期 |
| v2.0.0 | P2P/Relay 跨网络、计费系统 | 远期 |

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri v2 |
| 前端框架 | React 19 |
| 构建工具 | Vite |
| CSS 框架 | Tailwind CSS v4 |
| UI 组件库 | @base-ui/react |
| 状态管理 | jotai + @tanstack/react-query |
| HTTP 服务器 | axum |
| 错误处理 | thiserror |
| 日志 | tracing + OpenTelemetry |
| 包管理 | pnpm workspace |
| 代码质量 | oxlint + Prettier + prek |

## 配套文档

- **DESIGN.md** - 设计规范（Google Labs 格式）
- **CHANGELOG.md** - 变更日志（Keep a Changelog 格式）
- **.pre-commit-config.yaml** - prek 配置
- **.prettierrc** - Prettier 配置
- **.oxlintrc.json** - oxlint 配置
- **rustfmt.toml** - Rust 格式化配置
- **clippy.toml** - Clippy 配置
