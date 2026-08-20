# Changelog

本文件记录 DropVoice 的所有重要变更。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [未发布]

### 新增

- 配对服务器（Axum + SQLite）实现 LAN 地址会合（spec 11）
- 桌面端 device_id / device_name / pairing_token 配置字段（spec 14）
- 桌面端 i18n 模块 + 系统托盘菜单国际化（spec 15）
- 桌面端 T2 IP 变化检测任务（spec 11 §8）
- 桌面端 T3 心跳/状态上报任务（spec 11 §7.2）
- 配对服务器完整单元测试 + 集成测试（88 + 29 用例）
- 桌面端设备名长度校验（max 64 chars）
- 仓库根 README.md 快速开始文档

### 修复

- `hasExhaustedRetries` 误用 `MAX_DEVICES` 改为 `MAX_RETRIES`（spec 02 §5）
- `languageAtom` 默认值从 `'en'` 改为 `'zh'`（spec 02 §2.2）
- `useErrorHandler` 透出 suggestion 到 toast（spec 04 §4.4）
- `settings:maxTextLength` i18n key 补全（4 语言）

## [0.2.0] - 2026-07-15

### 新增

- 完整按规范重写为 monorepo 结构（apps/desktop + apps/mobile + packages/core、ui、i18n）
- 多设备管理（最多 5 台，Chip 形式选择器，IP 哈希颜色生成）
- 注入队列（多手机并发注入时先进先出）
- axum HTTP/WebSocket 服务器（替代 hyper）
- TOML 静态配置 + tauri-plugin-store 动态设置
- 配对码认证 + 速率限制 + 输入验证
- 结构化错误处理（thiserror + 错误码 + i18n 映射）
- tracing + tracing-appender 按天轮转日志
- OpenTelemetry 文件输出（JSON Lines）
- PWA 移动端（多语言、暗色模式、设备切换、QR 扫码）
- E2E 测试（Playwright）
- CI/CD 流水线（质量检查 + 多平台发布）

### 变更

- 升级到 React 19
- 升级到 Tauri v2
- UI 组件库迁移到 @base-ui/react
- CSS 框架升级到 Tailwind CSS v4
- 状态管理迁移到 jotai + @tanstack/react-query
- HTTP 服务器迁移到 axum
- 错误处理迁移到 thiserror
- 日志迁移到 tracing
- 国际化迁移到 i18next + react-i18next
- Lint 迁移到 oxlint
- Pre-commit 迁移到 prek
- 项目结构迁移到 pnpm workspace monorepo

## [0.1.0] - 2026-06-01

### 新增

- 初始版本发布
- QR 码扫描连接
- WebSocket 文本传输
- 多语言支持（中/英/日）
- 深色模式支持
