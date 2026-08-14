# 本地开发环境规范

> 版本: 1.0.0
> 最后更新: 2026-07-16
> 状态: 已批准

## 1. 概述

本规范定义 DropVoice 的本地开发环境配置和开发流程。

## 2. 决策

### 2.1 环境要求

| 工具 | 版本 | 说明 |
|------|------|------|
| Node.js | ≥ 22 | 前端运行时 |
| pnpm | ≥ 11 | 包管理器 |
| Rust | ≥ 1.75 | 后端编译 |
| Git | ≥ 2.30 | 版本控制 |

### 2.2 快速开始

```bash
# 1. 克隆仓库
git clone https://github.com/jukanntenn/DropVoice.git
cd DropVoice

# 2. 安装依赖
pnpm install

# 3. 启动开发服务器
pnpm tauri dev
```

### 2.3 环境配置文件

#### .env.example

```bash
# 信令服务器（可选）
VITE_SIGNALING_SERVER=https://dropvoice.app

# 日志级别
RUST_LOG=info
RUST_LOG_STYLE=always

# 开发端口
VITE_DEV_PORT=5173
TAURI_DEV_PORT=38425
```

#### .env.local（本地开发，不提交）

```bash
# 本地开发配置
VITE_SIGNALING_SERVER=http://localhost:3000
RUST_LOG=debug
```

### 2.4 开发脚本

```json
{
  "scripts": {
    "dev": "vite",
    "dev:mobile": "vite --host",
    "dev:tauri": "pnpm tauri dev",
    "build": "tsc && vite build",
    "build:tauri": "pnpm tauri build",
    "preview": "vite preview",
    "test": "vitest run",
    "test:watch": "vitest",
    "test:coverage": "vitest run --coverage",
    "test:rust": "cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml",
    "test:all": "pnpm test && pnpm test:rust",
    "lint": "oxlint src/",
    "lint:fix": "oxlint src/ --fix",
    "format": "prettier --write \"src/**/*.{ts,tsx,json,css}\"",
    "format:check": "prettier --check \"src/**/*.{ts,tsx,json,css}\"",
    "typecheck": "tsc --noEmit",
    "design:lint": "npx @google/design.md lint DESIGN.md",
    "design:export:css": "npx @google/design.md export --format css-tailwind DESIGN.md > packages/ui/src/tokens/theme.css",
    "design:sync": "pnpm design:lint && pnpm design:export:css",
    "quality": "pnpm typecheck && pnpm lint && pnpm format:check && pnpm test"
  }
}
```

### 2.5 IDE 配置

#### VS Code (.vscode/settings.json)

```json
{
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "esbenp.prettier-vscode",
  "editor.codeActionsOnSave": {
    "source.fixAll.oxlint": "explicit"
  },
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.check.extraArgs": ["--manifest-path", "apps/desktop/src-tauri/Cargo.toml"],
  "typescript.tsdk": "node_modules/typescript/lib"
}
```

#### VS Code 扩展 (.vscode/extensions.json)

```json
{
  "recommendations": [
    "esbenp.prettier-vscode",
    "rust-lang.rust-analyzer",
    "bradlc.vscode-tailwindcss"
  ]
}
```

### 2.6 移动端开发

```bash
# 启动移动端开发服务器（局域网可访问）
pnpm dev:mobile

# 手机访问：http://<你的IP>:5173
```

### 2.7 Git Hooks

```bash
# 安装 prek
npm install -g @j178/prek

# 安装 git hooks
prek install
```

## 3. 决策依据

### 3.1 选择 Node.js ≥ 22

**选择理由:**
- LTS 版本，稳定可靠
- 支持最新 ES 特性
- 性能好

### 3.2 选择 pnpm ≥ 11

**选择理由:**
- 原生 workspace 支持
- 速度快
- 磁盘效率高

### 3.3 选择 VS Code

**选择理由:**
- 业界最流行的 IDE
- 扩展生态丰富
- 免费开源

## 4. 接口规范

### 4.1 依赖安装

```bash
pnpm install --frozen-lockfile
```

### 4.2 开发服务器

```bash
# 前端开发
pnpm dev

# Tauri 开发
pnpm tauri dev

# 移动端开发
pnpm dev:mobile
```

### 4.3 测试

```bash
# 前端测试
pnpm test

# Rust 测试
pnpm test:rust

# 全部测试
pnpm test:all
```

### 4.4 代码质量

```bash
# Lint
pnpm lint

# 格式化
pnpm format

# 类型检查
pnpm typecheck

# 全部检查
pnpm quality
```

## 5. 约束

1. **版本要求**: 必须满足最低版本要求
2. **依赖锁定**: 使用 `pnpm install --frozen-lockfile`
3. **环境变量**: 敏感配置使用 `.env.local`（不提交）
4. **Git Hooks**: 必须安装 prek 并启用 hooks
5. **IDE 配置**: 推荐使用 VS Code 并安装推荐扩展
