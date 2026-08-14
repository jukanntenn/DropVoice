# Harness 基建蓝图（Opinionated 规范）

> 面向 Rust + TypeScript/React 多语言 monorepo 的从零构建规范。各章节自包含，按需取用。
>
> **指导思想**：本文档涉及的每一个工具都满足四个条件——(a) 社区广泛采用、(b) 持续维护、(c) 配置为出错即报错（fail loudly）、(d) 版本锁定避免静默漂移。不引入任何没有文档化理由的工具。

---

## 目录

1. [技术栈假设](#1-技术栈假设)
2. [Workspace 布局](#2-workspace-布局)
3. [格式化与 Lint](#3-格式化与-lint)
4. [测试](#4-测试)
5. [端到端测试](#5-端到端测试)
6. [CI/CD](#6-cicd)
7. [Pre-commit Hook（prek）](#7-pre-commit-hookprek)
8. [开发者体验（VSCode）](#8-开发者体验vscode)
9. [数据库迁移（sqlx）](#9-数据库迁移sqlx)
10. [AI Agent 指令文件（AGENTS.md）](#10-ai-agent-指令文件agentsmd)
11. [AI Agent Hook](#11-ai-agent-hook)
12. [提交规范](#12-提交规范)
13. [版本与发布](#13-版本与发布)
14. [搭建清单](#14-搭建清单)

---

## 1. 技术栈假设

本蓝图面向以下结构的 monorepo：

- **Rust** 后端——Cargo workspace，edition 2021，声明 MSRV。
- **TypeScript/React** 前端——pnpm workspace，Vite，React 19。
- **SQLite/Postgres**（可选，通过 sqlx 访问；仅在有数据库时适用）。
- **Tauri** 桌面应用（可选；若存在，前端 + Rust 同在一个 app 内）。

**本蓝图强制使用的工具**（替换会破坏集成点）：

| 关注点 | 工具 | 选择理由 |
|--------|------|----------|
| JS 包管理器 | **pnpm**（≥11） | workspace 支持、硬链接省磁盘、`only-allow pnpm` 强制 |
| JS linter | **oxlint** | 比 ESLint 快 50-100 倍，零配置起步，规则覆盖持续扩大 |
| JS 格式化 | **prettier** | 事实标准 |
| JS 测试框架 | **vitest** | Vite 原生、快、API 兼容 Jest |
| E2E | **Playwright** | 跨浏览器、自动等待、统一 API |
| Rust 格式化/lint | **rustfmt** + **clippy** | 官方出品，零配置 |
| Pre-commit | **prek** | pre-commit 的 Rust 重写，drop-in 兼容、更快、支持 monorepo workspace 模式 |
| CI 路径过滤 | **dorny/paths-filter**（锁定 SHA） | 在 `tj-actions/changed-files` 2025-03 遭供应链攻击后的安全选择 |

**禁用工具**（不要使用）：

- ❌ `tj-actions/changed-files`——2025 年 3 月遭遇供应链攻击，窃取了 CI secret。
- ❌ ESLint（当 oxlint 能覆盖需求时）——更慢、配置更重；仅在你需要 oxlint 缺失的插件时才补充 ESLint。
- ❌ `husky` + `lint-staged`——被 prek 取代（单一工具、原生支持 workspace）。

---

## 2. Workspace 布局

### 2.1 双 workspace

```
my-monorepo/
├── Cargo.toml              # Cargo workspace 根
├── package.json            # pnpm workspace 根
├── pnpm-workspace.yaml
├── apps/
│   ├── desktop/            # 例如 Tauri 应用
│   │   ├── src-tauri/      # Rust crate
│   │   └── src/            # React 前端
│   ├── web/                # 独立前端
│   └── api-server/         # 独立 Rust crate
├── packages/
│   ├── shared/             # 共享 TS
│   ├── ui/                 # 共享 React 组件
│   └── config/             # 共享配置
└── e2e/                    # Playwright 测试
```

### 2.2 Cargo workspace——版本集中化

**规则**：被多于一个 crate 使用的依赖，必须在根 `[workspace.dependencies]` 声明。成员 crate 通过 `dep.workspace = true` 继承。根已声明的依赖，**绝不**在成员 crate 内联版本号——版本漂移是静默且代价高昂的。

```toml
# 根 Cargo.toml
[workspace]
resolver = "2"
members = ["apps/*/src-tauri", "apps/api-server"]

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.80"        # 声明 MSRV；clippy 会自动读取此字段
license = "MIT"
authors = ["Your Org"]

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
axum = { version = "0.8", features = ["ws"] }
serde = { version = "1", features = ["derive"] }
# ... 所有共享依赖放这里
```

```toml
# apps/api-server/Cargo.toml——成员 crate
[package]
name = "my-api-server"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[dependencies]
tokio = { workspace = true }
axum = { workspace = true }
serde = { workspace = true }
```

**为什么把 `rust-version` 放在 `[workspace.package]`**：clippy 1.74+ 会自动读取此字段作为 MSRV，所以你可以删掉 `clippy.toml` 里的 `msrv = "..."`——单一事实源。MSRV 设得太低会抑制现代化 lint（例如 MSRV < 1.45 时 `manual_strip` 不会建议用 `strip_prefix`）；设成你真实的最低版本即可。

### 2.3 pnpm workspace

```yaml
# pnpm-workspace.yaml
packages:
  - 'apps/*'
  - 'packages/*'
```

```json
// 根 package.json——强制 pnpm，声明共享开发工具
{
  "private": true,
  "scripts": {
    "dev": "pnpm --filter <desktop-app> dev",
    "build": "pnpm -r --filter './apps/*' run build",
    "test": "pnpm -r run test",
    "lint": "pnpm -r run lint",
    "format": "pnpm -r run format",
    "typecheck": "pnpm -r run typecheck",
    "quality": "pnpm typecheck && pnpm lint && pnpm format:check && pnpm test"
  },
  "devDependencies": {
    "oxlint": "^0.15.0",
    "prettier": "^3.6.0",
    "vitest": "^4.1.0",
    "@playwright/test": "^1.49.0",
    "typescript": "^5.9.0"
  },
  "engines": { "node": ">=22", "pnpm": ">=11" },
  "packageManager": "pnpm@11.0.0",
  "scripts": {
    "preinstall": "npx only-allow pnpm"
  }
}
```

`preinstall` 钩子会在执行 `npm install` / `yarn install` 时以明确错误中止。

---

## 3. 格式化与 Lint

### 3.1 Rust——rustfmt + clippy（零配置）

```toml
# rustfmt.toml（根目录）
edition = "2021"
max_width = 100
tab_spaces = 4
```

这三项都是 stable 选项。除非 CI 承诺用 nightly rustfmt（在 stable 上会报错），否则不要启用 `unstable_features = true`。

**如果你已在 `Cargo.toml` 设置 `rust-version`，则不需要 `clippy.toml`**。若想要额外的 clippy 配置（如 `cognitive-complexity-threshold`），放进 `clippy.toml`——但那里的 `msrv` 与 `Cargo.toml` 重复，应删除。

**workspace 级 lint 命令**：
```bash
cargo fmt --check              # 格式检查
cargo fmt                      # 格式修复
cargo clippy --workspace -- -D warnings   # lint，拒绝 warning
```

`--workspace` 一次覆盖所有 crate。有多个 crate 时**绝不**只 lint 单个 `--manifest-path`——你一定会漏掉某个。

### 3.2 TypeScript——oxlint + prettier

**`.oxlintrc.json`**（根目录）：
```json
{
  "categories": { "correctness": "error", "perf": "warn" },
  "rules": {
    "no-unused-vars": "error",
    "no-explicit-any": "warn",
    "react-hooks/rules-of-hooks": "error",
    "react-hooks/exhaustive-deps": "warn",
    "no-console": "warn"
  }
}
```

oxlint 自动发现此文件。不需要 `.eslintrc`。若你需要某条 oxlint 尚未实现的规则，才在 oxlint 之外补充 ESLint（而非替换）。

**`.prettierrc`**（根目录）：
```json
{
  "semi": true,
  "singleQuote": true,
  "tabWidth": 2,
  "trailingComma": "es5",
  "printWidth": 100,
  "plugins": ["prettier-plugin-tailwindcss"]
}
```

不用 Tailwind 就去掉 `prettier-plugin-tailwindcss`。保持 prettier 配置精简——每一个选项都是潜在的风格争论点。

**`.prettierignore`**：
```
node_modules
dist
build
target
coverage
*.lock
```

**脚本**（根 `package.json`）：
```json
{
  "scripts": {
    "lint": "pnpm -r run lint",
    "lint:fix": "pnpm -r run lint:fix",
    "format": "pnpm -r run format",
    "format:check": "pnpm -r run format:check",
    "format:root": "prettier --write \"*.{json,md,yaml,yml}\" \".github/**/*\""
  }
}
```

---

## 4. 测试

### 4.1 Rust 测试

- 单元测试就地放在每个 `.rs` 文件内的 `#[cfg(test)] mod tests { ... }`。
- 集成测试放在各 crate 的 `tests/` 目录。
- 运行：`cargo test --workspace`。

### 4.2 TypeScript 测试——vitest

每个 package 一个 `vitest.config.ts`。通过共享结构保持 DRY：

```typescript
// packages/shared/vitest.config.ts
import path from 'node:path';
import { defineConfig } from 'vitest/config';

export default defineConfig({
  resolve: { alias: { '@': path.resolve(__dirname, './src') } },
  test: {
    globals: true,
    environment: 'node',           // React 组件用 'jsdom'
    include: ['src/**/*.{test,spec}.{ts,tsx}'],
    coverage: {
      reporter: ['text', 'lcov'],
      thresholds: { statements: 60, branches: 50, functions: 60, lines: 60 },
    },
  },
});
```

**覆盖率阈值**：全量起步 60%（地板，非天花板）。随 package 成熟度逐包上调。阈值设得过高过早会逼出为达标而写的垃圾测试。

**环境选择**：
- `'node'`：纯逻辑（状态机、工具函数、解析器）。
- `'jsdom'`：涉及 `window`、`document`、`localStorage` 的代码。
- `@vitejs/plugin-react` 仅在测试 React 处添加。

**测试文件放置**：就近放为 `*.test.ts(x)`，紧邻源码。不要用平行的 `__tests__/` 目录树——它会让导航成本翻倍。

### 4.3 测试命令

```json
{
  "scripts": {
    "test": "pnpm -r run test",
    "test:watch": "pnpm -r run test:watch",
    "test:coverage": "pnpm -r run test:coverage",
    "test:rust": "cargo test --workspace",
    "test:all": "pnpm test && pnpm test:rust"
  }
}
```

---

## 5. 端到端测试

### 5.1 Playwright 配置

仓库根目录一个 `playwright.config.ts`（若多个 app 有不同 URL，可每 app 一个）：

```typescript
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  reporter: process.env.CI ? 'github' : 'html',
  use: {
    baseURL: 'http://localhost:5173',
    trace: 'on-first-retry',
  },
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
  ],
  webServer: {
    command: 'pnpm dev',
    url: 'http://localhost:5173',
    reuseExistingServer: !process.env.CI,
    timeout: 60_000,
  },
});
```

**关键决策**：
- `webServer` 在 CI 自动启动 dev server，本地复用已运行的实例。
- **起步只用 chromium**。只有当你有确凿的跨浏览器需求时才加 firefox/webkit——它会让 CI 时间变为 3 倍。
- `trace: 'on-first-retry'`：仅在重试时给完整 trace，兼顾 flaky 排查与成本。

### 5.2 运行

```json
{
  "scripts": {
    "test:e2e": "playwright test",
    "test:e2e:ui": "playwright test --ui"
  }
}
```

---

## 6. CI/CD

### 6.1 原则

1. **没有变更就不跑。** 对 monorepo 而言路径过滤是必须的。
2. **第三方 action 锁定到 commit SHA**，不用浮动 tag（`@v3`）。tag 可变；被攻陷的维护者可以移动它。锁定 SHA 是抵御供应链攻击的唯一手段。
3. **同一 ref 新推送时取消进行中的运行**（`concurrency: cancel-in-progress: true`）。
4. **`push` 到 `main`（默认分支）跑全量质量**，即使有路径过滤——它是集成分支。

### 6.2 用 dorny/paths-filter 做路径过滤

架构：一个 `changes` job 计算布尔输出，下游每个 job 用 `if: needs.changes.outputs.<area> == 'true'` 门控。

```yaml
# .github/workflows/quality.yml
name: Quality

on:
  pull_request:
    branches: [main]
    paths-ignore:
      - '**.md'
      - 'docs/**'
      - 'LICENSE'
      - '.gitignore'
  push:
    branches: [main]
    paths-ignore:
      - '**.md'
      - 'docs/**'
      - 'LICENSE'
      - '.gitignore'

concurrency:
  group: quality-${{ github.ref }}
  cancel-in-progress: true

jobs:
  changes:
    runs-on: ubuntu-latest
    outputs:
      frontend: ${{ steps.filter.outputs.frontend }}
      rust: ${{ steps.filter.outputs.rust }}
    steps:
      - uses: actions/checkout@v4
      # 锁定 SHA——不要改成 @v3（供应链安全）。
      - uses: dorny/paths-filter@de90cc6fb66fc756703f280f72eb1f71e65ce00f
        id: filter
        with:
          filters: |
            frontend:
              - 'apps/*/src/**'
              - 'packages/**'
              - '*.json'
              - '*.ts'
            rust:
              - 'apps/*/src-tauri/**'
              - 'apps/api-server/**'
              - 'Cargo.toml'
              - 'Cargo.lock'

  frontend:
    needs: changes
    if: needs.changes.outputs.frontend == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with: { version: 11 }
      - uses: actions/setup-node@v4
        with: { node-version: 22, cache: pnpm }
      - run: pnpm install --frozen-lockfile
      - run: pnpm typecheck
      - run: pnpm lint
      - run: pnpm format:check
      - run: pnpm test:coverage

  rust:
    needs: changes
    if: needs.changes.outputs.rust == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: rustfmt, clippy }
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --check
      - run: cargo clippy --workspace -- -D warnings
      - run: cargo test --workspace
```

### 6.3 路径过滤设计规则

- **`paths-ignore`**（workflow 级）：只用于**绝不**影响代码的文件——`*.md`、`docs/`、`LICENSE`、`.gitignore`。绝不在这里忽略 `package.json` 或 `Cargo.toml`（它们影响构建）。
- **dorny 过滤器**（job 级）：按影响区域分组。把 lockfile 和 manifest 纳入每个依赖它的区域（Rust 区含 `Cargo.toml` + `Cargo.lock`；前端区含 `pnpm-lock.yaml`）。
- **`push` 到 main**：应用与 `pull_request` 相同的 `paths-ignore`（纯文档推送不应触发 CI），但 dorny 的 job 级过滤器仍让所有变更区域得以运行。

### 6.4 发布 workflow

由 tag push `v*` 触发。构建所有平台目标。发布**不做**路径过滤——每次发布都全量构建。

---

## 7. Pre-commit Hook（prek）

### 7.1 为什么用 prek 而非 pre-commit / husky

- **prek** 是 `pre-commit` 的 Rust 重写——单一二进制、无需 Python、完全兼容 `.pre-commit-config.yaml`，另有原生 `prek.toml` 格式。
- 有面向 monorepo 的 **workspace 模式**，以及 Rust 实现的**内置快速 hook**（空白、JSON/YAML/TOML 校验等）——无需 Python venv 引导。
- **`husky` + `lint-staged`** 只支持 JS，无法统一跑 Rust hook；prek 两者兼顾。

### 7.2 `prek.toml`

```toml
# prek.toml——pre-commit 配置。
# 安装：clone 后执行一次 `prek install`。
# 文档：https://prek.j178.dev/

minimum_prek_version = "0.2.0"
default_install_hook_types = ["pre-commit", "commit-msg"]

# --- 内置快速 hook（Rust 原生，零设置）---
[[repos]]
repo = "builtin"
hooks = [
  { id = "trailing-whitespace" },
  { id = "end-of-file-fixer" },
  { id = "check-yaml" },
  { id = "check-json" },
  { id = "check-toml" },
  { id = "check-merge-conflict" },
  { id = "check-added-large-files" },
  { id = "detect-private-key" },
]

# --- 本地 hook（项目工具）---
[[repos]]
repo = "local"

[[repos.hooks]]
id = "oxlint"
name = "oxlint"
entry = "pnpm oxlint"
language = "system"
types_or = ["javascript", "typescript"]
pass_filenames = false

[[repos.hooks]]
id = "prettier"
name = "Prettier"
entry = "pnpm prettier --write"
language = "system"
types_or = ["javascript", "typescript", "json", "css"]

[[repos.hooks]]
id = "cargo-fmt"
name = "cargo fmt"
entry = "cargo fmt --"
language = "system"
types = ["rust"]
pass_filenames = true

[[repos.hooks]]
id = "cargo-clippy"
name = "cargo clippy"
entry = "cargo clippy --workspace -- -D warnings"
language = "system"
types = ["rust"]
pass_filenames = false

[[repos.hooks]]
id = "commitlint"
name = "commitlint"
entry = "pnpm commitlint --edit"
language = "system"
stages = ["commit-msg"]
```

### 7.3 配置要点

- **`default_install_hook_types = ["pre-commit", "commit-msg"]`**：没有它，`prek install` 只装 `pre-commit` shim，你的 `commit-msg` hook（commitlint）会静默地永不触发。
- **`pass_filenames`**：对不接受文件参数的工具设 `false`（oxlint 扫全项目；clippy 需要 `--workspace`）。对格式化具体文件的工具设 `true`（prettier、cargo fmt）。
- **`cargo fmt --`**（结尾的 `--`）：告诉 cargo fmt 接受来自 prek 的文件名。没有 `--`，cargo fmt 会把它们当 flag 解析。
- **`detect-private-key`**：低成本保险，防止提交 `id_rsa` / `.pem` 文件。
- **`minimum_prek_version`**：防止队友用了过时的 prek 导致行为不一致。

### 7.4 首次安装（写入 README）

```bash
pip install prek          # 或：pipx install prek / cargo install prek
prek install              # 安装 git shim
prek run --all-files      # 跑一次确认全部通过
```

> **`prek` 在 PATH 中**：Windows 上 `pip install prek` 可能把二进制装到 Git Bash 的 PATH 之外。在脚本/CI 中用 `python -m prek` 作为可移植的回退方案。

---

## 8. 开发者体验（VSCode）

### 8.1 `.vscode/tasks.json`——一键启动开发

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "dev",
      "detail": "启动开发服务器",
      "type": "shell",
      "command": "pnpm dev",
      "isBackground": true,
      "problemMatcher": [{
        "owner": "dev-server",
        "pattern": { "regexp": "^$" },
        "background": {
          "activeOnStart": true,
          "beginsPattern": "VITE v|Starting",
          "endsPattern": "Local:|ready|listening"
        }
      }],
      "group": { "kind": "build", "isDefault": true },
      "presentation": { "reveal": "always", "panel": "dedicated", "clear": true }
    }
  ]
}
```

**要点**：
- `isBackground: true` 对长驻进程（dev server）**必须**。没有它，VSCode 会等进程退出，task 永远不"完成"。
- `problemMatcher` 的 `background.beginsPattern` / `endsPattern` 告诉 VSCode 服务器何时就绪（从而让依赖此 task 的后续 task 继续）。把 `endsPattern` 设为你的服务器就绪时打印的字符串（如 Vite 打印 `Local: http://localhost:5173/`）。
- `group.isDefault: true` 把 task 绑定到 `Ctrl+Shift+B`（标准的"运行构建任务"快捷键）。**不要**通过 `keybindings.json` 劫持 `Ctrl+F5`——它更不易发现，且与调试扩展冲突。
- 若需同时启动多个服务（前端 + API + DB），加一个复合 task（`dependsOn` + `dependsOrder: "parallel"`）。

### 8.2 `.vscode/extensions.json`——推荐扩展

```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "oxc.oxc-vscode",
    "esbenp.prettier-vscode",
    "ms-python.python"
  ]
}
```

VSCode 会在新贡献者首次打开时提示安装。

### 8.3 `.vscode/settings.json`

```json
{
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "esbenp.prettier-vscode",
  "editor.codeActionsOnSave": { "source.fixAll.oxlint": "explicit" },
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.check.extraArgs": [],
  "typescript.tsdk": "node_modules/typescript/lib"
}
```

- `rust-analyzer.check.extraArgs: []`——留空，让 rust-analyzer 检查整个 workspace（而非单个 `--manifest-path`）。仅当 workspace 极大需要加速检查时才收窄。
- `source.fixAll.oxlint: "explicit"`（不是 `true`）——现代 VSCode 设置值，要求显式保存动作才修复，避免每次按键都意外修复。

### 8.4 `.gitignore` 对 VSCode 的处理

```
.vscode/*
!.vscode/settings.json
!.vscode/extensions.json
!.vscode/tasks.json
```

追踪 settings/extensions/tasks（团队共享）。忽略其余（`launch.json` 是个人调试偏好——除非你有所有人都需要的调试配置，否则不要强推给团队）。

---

## 9. 数据库迁移（sqlx）

> 若项目无数据库，跳过本章。

### 9.1 嵌入式迁移——唯一推荐的模式

用 `sqlx::migrate!()` 把迁移嵌入二进制。启动时运行。**任何环境都绝不手动执行 `sqlx migrate run`。**

```rust
// 启动时：
let pool = sqlx::sqlite::SqlitePoolOptions::new()
    .connect(&database_url)
    .await?;

sqlx::migrate!("./migrations").run(&pool).await?;
```

这会把迁移 SQL 编译进二进制，并在构建期校验文件名。生产部署是自迁移的——容器启动、迁移应用、服务器就绪。

### 9.2 文件命名约定

```
migrations/
  YYYYMMDDHHMMSS_description.sql          (simple——仅 up)
  YYYYMMDDHHMMSS_description.up.sql       (reversible——应用)
  YYYYMMDDHHMMSS_description.down.sql     (reversible——回滚)
```

时间戳格式是 sqlx 的默认值，且排序正确。**绝不编辑已应用的迁移**——永远新增一个。

### 9.3 Simple vs Reversible

- **Simple**（默认）：用于无有意义回滚的增量变更（新表、带默认值的新列）。
- **Reversible**（`-r` flag）：用于有清晰、安全的 down-path 的变更（如重命名列后再改回）。
- **绝不**把破坏性变更做成 reversible（如果 down 迁移会丢数据）——把它当不可逆处理，写一个新的补偿迁移。

### 9.4 `sqlx.toml`

```toml
# sqlx.toml——`sqlx migrate add` 的默认值
[create]
default_migration_type = "simple"    # 或 "reversible"
default_versioning = "timestamp"     # 匹配 YYYYMMDDHHMMSS 命名
```

### 9.5 查询风格——运行时 vs 编译期

- **运行时查询**（`query_as` + `FromRow`）：编译期不需要 `DATABASE_URL`，没有 `cargo sqlx prepare` 步骤。CI 更简单。**优先使用**，除非你需要编译期检查的类型安全保证。
- **编译期宏**（`query!`）：在构建期捕获 SQL 错误，但 `cargo build` 时需要活数据库（或 `.sqlx/` 离线缓存）。若使用，每次查询变更后运行 `cargo sqlx prepare` 并提交 `.sqlx/` 目录。

### 9.6 一次性安装

```bash
cargo install sqlx-cli --no-default-features --features sqlite,rustls
# Postgres：--features postgres,rustls
```

始终用 `--no-default-features` + 显式 feature，避免拉入 native TLS（用 rustls 代替——它是纯 Rust，交叉编译干净）。

### 9.7 工作流命令

```bash
sqlx migrate add <description>       # simple 迁移
sqlx migrate add -r <description>    # reversible 迁移
sqlx migrate info                    # 显示已应用 / 待应用
sqlx database create                 # 从 DATABASE_URL 创建开发库
sqlx database drop -f                # 销毁开发库
```

---

## 10. AI Agent 指令文件（AGENTS.md）

### 10.1 为什么需要 AGENTS.md

`AGENTS.md` 是新兴的跨工具标准（Claude Code、Codex、Cursor、ZCode、Aider 都识别）。它给 AI agent 提供在仓库中准确工作所需的上下文。一份含糊的"做一个有用的助手"文件毫无用处；一份具体的文件能防止 agent 臆造架构。

### 10.2 六大必写章节

根据对 2500+ 仓库的研究，有效的 agent 指令文件涵盖：

1. **命令（Commands）**——精确的、可复制粘贴的带 flag 命令。放在最前（agent 最常引用它）。
2. **测试（Testing）**——如何运行测试、覆盖率门控是什么。
3. **项目结构（Project Structure）**——目录地图 + 精确技术栈含版本。
4. **代码风格（Code Style）**——真实的代码片段，而非散文描述。（"一个片段胜过三段描述。"）
5. **Git 工作流**——提交格式、分支策略、发布流程。
6. **边界（Boundaries）**——agent 绝不能碰的东西（生成文件、lockfile、规格文档）。

### 10.3 写作规则

- **对技术栈要具体**："React 19、TypeScript 5.9、Vite 6"而非"React 项目"。
- **赋予具体人设**："你是一名为 React 组件写 vitest 测试的测试工程师，且绝不修改源码"胜过"你是一个有用的助手"。
- **三层边界**："总是做" / "先问" / "绝不做"。
- **展示而非描述**：放一个 10 行代码片段演示你的错误处理模式，而非用文字描述它。
- **写明版本**：过时的版本引用（你在 React 19 却写 React 18）会导致 agent 写出废弃 API。

### 10.4 CLAUDE.md 同步

部分工具（Claude Code）读 `CLAUDE.md`；其他读 `AGENTS.md`。让 `AGENTS.md` 作为事实源，`CLAUDE.md` 是字节级副本。用 pre-commit hook 强制：

```python
# scripts/sync-agents.py——仅检查，绝不写入
import filecmp, sys
from pathlib import Path
root = Path(__file__).resolve().parent.parent
if not filecmp.cmp(root / "AGENTS.md", root / "CLAUDE.md", shallow=False):
    print("AGENTS.md 和 CLAUDE.md 不一致。执行：cp AGENTS.md CLAUDE.md", file=sys.stderr)
    sys.exit(1)
```

```toml
# prek.toml——添加此 hook
[[repos.hooks]]
id = "agents-sync"
name = "AGENTS.md/CLAUDE.md sync"
entry = "python scripts/sync-agents.py"
language = "system"
files = "^(AGENTS|CLAUDE)\\.md$"
pass_filenames = false
```

---

## 11. AI Agent Hook

### 11.1 Hook 解决什么问题

AI agent（Claude Code、Codex、ZCode）能自主编辑文件。没有 hook，它们会留下未格式化的代码、引入 lint 错误、编辑生成文件。Hook 闭环：

- **PostToolUse**（文件编辑后）：自动格式化该文件。
- **Stop**（agent 结束前）：跑 lint；有错误则阻止结束。
- **PreToolUse**（文件编辑前）：阻止编辑受保护/生成文件。

### 11.2 三 agent 的 Hook 契约

Claude Code、Codex、ZCode 都支持**兼容契约**的 hook：

| 方面 | 行为 |
|------|------|
| **输入** | stdin 上的 JSON（工具名、工具输入、会话信息） |
| **Exit 0** | 成功 / 放行 |
| **Exit 2** | 阻塞（PreToolUse 拒绝编辑；Stop 阻止结束；PostToolUse 把 stderr 反馈给 agent） |
| **其他非零** | 非阻塞错误（记日志，执行继续） |
| **stderr** | 作为阻塞原因反馈给 agent |
| **stdout（exit 0 时）** | 解析为 JSON 做高级控制，或被忽略 |

**关键**：exit code **2** 才是阻塞信号——不是 1。Unix 惯例中 1 是失败，但这些 agent 把 1 当非阻塞错误。你的 hook 若要阻塞，用 `exit 2`。

### 11.3 各 agent 的配置位置

| Agent | 文件 | 启用方式 | 项目根环境变量 |
|-------|------|----------|----------------|
| Claude Code | `.claude/settings.json` → `hooks` | 默认启用 | `${CLAUDE_PROJECT_DIR}` |
| Codex | `.codex/config.toml` → `[hooks]` | `[features] hooks = true` | （相对路径，基于 CWD） |
| ZCode | `.zcode/config.json` → `hooks.events` | `hooks.enabled: true` | `${ZCODE_PROJECT_DIR}` |

### 11.4 支持事件（交集）

| 事件 | Claude Code | Codex | ZCode |
|------|:-----------:|:-----:|:-----:|
| PreToolUse | ✅ | ✅ | ✅ |
| PostToolUse | ✅ | ✅ | ✅ |
| Stop | ✅ | ✅ | ✅（最多阻塞 3 次） |
| UserPromptSubmit | ✅ | ✅ | ✅ |
| SessionStart | ✅ | ✅ | ✅ |
| PermissionRequest | ✅ | ✅ | ✅ |
| PostToolUseFailure | ✅ | — | ✅ |
| PreCompact / PostCompact | ✅ | ✅ | — |
| SubagentStart / SubagentStop | ✅ | ✅ | — |

跨 agent 可移植时，只用前六个事件。

### 11.5 Matcher——工具名匹配

`matcher` 字段是大小写敏感的正则，对工具名测试。文件编辑 hook 要用全集，因为各 agent 对工具命名不同：

| Agent | 编辑工具名 |
|-------|------------|
| Claude Code | `Edit`、`Write` |
| Codex | `apply_patch`（matcher 接受 `Edit`/`Write` 别名） |
| ZCode | `Edit`、`Write`、`ApplyPatch` |

**通用 matcher**：`"Edit|Write|ApplyPatch|apply_patch"` 覆盖三者。

### 11.6 Hook 脚本设计（Python）

用 Python 写 hook（仅标准库——`json`、`subprocess`、`sys`、`os`）。无第三方依赖。

**PostToolUse（格式化）模式**：
```python
import json, os, subprocess, sys
from pathlib import Path

def repo_root():
    env = os.environ.get("CLAUDE_PROJECT_DIR") or os.environ.get("ZCODE_PROJECT_DIR")
    return Path(env) if env else Path.cwd()

data = json.load(sys.stdin)
file_path = data.get("tool_input", {}).get("file_path", "")
if file_path:
    ext = Path(file_path).suffix.lower()
    root = repo_root()
    if ext in {".ts", ".tsx", ".js", ".jsx", ".json", ".css"}:
        subprocess.run(["pnpm", "prettier", "--write", file_path], cwd=str(root), capture_output=True, timeout=30)
    elif ext == ".rs":
        subprocess.run(["cargo", "fmt"], cwd=str(root), capture_output=True, timeout=60)
sys.exit(0)   # 格式化永不阻塞
```

**Stop（lint，失败时阻塞）模式**：
```python
import subprocess, sys, os
from pathlib import Path

def repo_root():
    env = os.environ.get("CLAUDE_PROJECT_DIR") or os.environ.get("ZCODE_PROJECT_DIR")
    return Path(env) if env else Path.cwd()

root = repo_root()
failures = []
for cmd in [["pnpm", "oxlint"], ["cargo", "clippy", "--workspace", "--", "-D", "warnings"]]:
    r = subprocess.run(cmd, cwd=str(root), capture_output=True, text=True, timeout=240)
    if r.returncode != 0:
        failures.append((cmd[0], r.stdout + r.stderr))

if failures:
    for name, out in failures:
        print(f"--- {name} ---", file=sys.stderr)
        print(out[-3000:], file=sys.stderr)
    sys.exit(2)   # 阻塞：agent 必须修复后才能结束
sys.exit(0)
```

**PreToolUse（保护文件）模式**：
```python
import json, sys

PROTECTED = {"Cargo.lock": "不要手编 Cargo.lock", "path/to/generated.css": "生成文件"}

data = json.load(sys.stdin)
fp = data.get("tool_input", {}).get("file_path", "").replace("\\", "/")
for protected, msg in PROTECTED.items():
    if fp == protected or fp.endswith("/" + protected):
        print(f"已阻止：{msg}", file=sys.stderr)
        sys.exit(2)
sys.exit(0)
```

### 11.7 Timeout 单位（陷阱！）

| Agent | Hook 类型 | Timeout 单位 |
|-------|-----------|--------------|
| Claude Code | `command` | **秒**（默认 600） |
| Codex | `command` | **秒**（`timeout` 字段） |
| ZCode | `command` | **秒**（`timeout` 字段） |
| ZCode | `process` | **毫秒**（`timeoutMs` 字段） |

跑全量 clippy 的 Stop hook，设 `timeout: 240`（秒）允许最多 4 分钟。

### 11.8 Stop hook 限制（ZCode）

ZCode 对 Stop 事件的阻塞上限为 **3 次**。如果 agent 在 3 轮内修不完所有 lint 错误，仍会强制结束。据此设计 lint 范围（全量 lint 可行——3 轮通常足够 agent 修复格式/clippy 小问题）。pre-commit hook 是最终安全网。

### 11.9 跨平台 Python 调用

- Windows：`python`（无 `python3`）。
- macOS/Linux：`python3`。

Codex 通过 `command` + `command_windows` 字段原生处理：
```toml
[[hooks.PostToolUse.hooks]]
type = "command"
command = "python3 .codex/hooks/format.py"
command_windows = "python .codex/hooks/format.py"
```

对于 Claude Code / ZCode（无原生平台分流），hook 配置用 `python`（Windows 可用；macOS/Linux 上若脚本可执行则靠 shebang `#!/usr/bin/env python3`，或建一个 `python` 软链）。

### 11.10 完整配置示例

**`.claude/settings.json`**（Claude Code）：
```json
{
  "hooks": {
    "PreToolUse": [{ "matcher": "Edit|Write|ApplyPatch", "hooks": [
      { "type": "command", "command": "python \"${CLAUDE_PROJECT_DIR}/.claude/hooks/protect.py\"" }
    ]}],
    "PostToolUse": [{ "matcher": "Edit|Write|ApplyPatch", "hooks": [
      { "type": "command", "command": "python \"${CLAUDE_PROJECT_DIR}/.claude/hooks/format.py\"" }
    ]}],
    "Stop": [{ "hooks": [
      { "type": "command", "command": "python \"${CLAUDE_PROJECT_DIR}/.claude/hooks/lint.py\"", "timeout": 240 }
    ]}]
  }
}
```

**`.zcode/config.json`**（ZCode——注意 `enabled` 和 `events` 嵌套）：
```json
{
  "hooks": {
    "enabled": true,
    "events": {
      "PreToolUse": [{ "matcher": "Edit|Write|ApplyPatch", "hooks": [
        { "type": "command", "command": "python \"${ZCODE_PROJECT_DIR}/.zcode/hooks/protect.py\"" }
      ]}],
      "PostToolUse": [{ "matcher": "Edit|Write|ApplyPatch", "hooks": [
        { "type": "command", "command": "python \"${ZCODE_PROJECT_DIR}/.zcode/hooks/format.py\"" }
      ]}],
      "Stop": [{ "hooks": [
        { "type": "command", "command": "python \"${ZCODE_PROJECT_DIR}/.zcode/hooks/lint.py\"", "timeout": 240 }
      ]}]
    }
  }
}
```

**`.codex/config.toml`**（Codex——注意 `features.hooks` 和 `command_windows`）：
```toml
[features]
hooks = true

[[hooks.PreToolUse]]
matcher = "Edit|Write|ApplyPatch"
[[hooks.PreToolUse.hooks]]
type = "command"
command = "python3 .codex/hooks/protect.py"
command_windows = "python .codex/hooks/protect.py"

[[hooks.PostToolUse]]
matcher = "Edit|Write|ApplyPatch"
[[hooks.PostToolUse.hooks]]
type = "command"
command = "python3 .codex/hooks/format.py"
command_windows = "python .codex/hooks/format.py"

[[hooks.Stop]]
[[hooks.Stop.hooks]]
type = "command"
command = "python3 .codex/hooks/lint.py"
command_windows = "python .codex/hooks/lint.py"
timeout = 240
```

### 11.11 Hook 脚本放置

把 hook 脚本放进每个 agent 自己的目录（`.claude/hooks/`、`.codex/hooks/`、`.zcode/hooks/`），作为**内容相同的副本**。脚本很小（各 30-50 行）；维护三份副本的成本低于用共享目录加各 agent 路径变量的间接寻址成本。三份都纳入 git（它们是团队共享配置）。

---

## 12. 提交规范

### 12.1 Conventional Commits + commitlint

```json
// .commitlintrc.json
{
  "extends": ["@commitlint/config-conventional"],
  "rules": {
    "type-enum": [2, "always", [
      "feat", "fix", "docs", "style", "refactor", "perf", "test", "chore", "revert"
    ]],
    "subject-case": [0]
  }
}
```

```json
// package.json devDependencies
"@commitlint/cli": "^19.0.0",
"@commitlint/config-conventional": "^19.0.0"
```

通过 prek 的 `commit-msg` hook 强制（见[§7](#7-pre-commit-hookprek)）。

### 12.2 Scope 约定

使用与 monorepo 结构匹配的 scope：`feat(desktop):`、`fix(api-server):`、`refactor(ui):`、`chore(deps):`。这让 changelog 可过滤。

---

## 13. 版本与发布

### 13.1 单一版本源

在根 `package.json` 和 `Cargo.toml` 的 `[workspace.package]` 各保留一个版本。通过脚本同步到所有成员 package：

```json
{
  "scripts": {
    "version:patch": "pnpm version patch && pnpm version:sync",
    "version:sync": "node scripts/sync-version.js"
  }
}
```

`sync-version.js` 读取根版本并写入 workspace 内每个 `package.json` 和 `Cargo.toml`。

### 13.2 CHANGELOG

```json
{
  "devDependencies": { "conventional-changelog-cli": "^5.0.0" },
  "scripts": {
    "changelog": "conventional-changelog -p angular -i CHANGELOG.md -s"
  }
}
```

把 `changelog` 接入 `version:*` 脚本，使每次升版时重新生成。

### 13.3 发布触发

tag 驱动：`git tag v1.2.3 && git push --tags`。GitHub Actions 在 `push: tags: ['v*']` 时构建所有平台并创建 release。发布**不做**路径过滤。

---

## 14. 搭建清单

新仓库 bootstrap 时按序应用：

- [ ] **Workspace**：`Cargo.toml`（workspace deps 集中）+ `pnpm-workspace.yaml`
- [ ] **`rustfmt.toml`**：edition、max_width、tab_spaces
- [ ] **MSRV**：`[workspace.package]` 的 `rust-version`（删除任何 `clippy.toml` 的 msrv）
- [ ] **`.oxlintrc.json`** + **`.prettierrc`** + **`.prettierignore`**
- [ ] **`vitest.config.ts`** 每 package（覆盖率阈值 60% 起步）
- [ ] **`playwright.config.ts`**（仅 chromium、webServer 自动启动）
- [ ] **CI**：`quality.yml` 含 paths-ignore + dorny/paths-filter（锁定 SHA）
- [ ] **`prek.toml`** + `.commitlintrc.json` → `prek install`
- [ ] **`.vscode/`**：tasks.json（默认 build task）、extensions.json、settings.json
- [ ] **`.gitignore`**：白名单 `.vscode/{settings,extensions,tasks}.json`
- [ ] **`AGENTS.md`** + `CLAUDE.md`（副本）+ `scripts/sync-agents.py`（prek hook）
- [ ] **AI hook**（若用 AI agent）：`.claude/`、`.zcode/`、`.codex/` 配置 + Python hook 脚本
- [ ] **sqlx**（若有 DB）：`migrations/` + `sqlx.toml` + 启动时 `sqlx::migrate!()`
- [ ] **发布**：tag push 触发的 `release.yml`

---

*本蓝图是一份活文档。当工具被替换、默认值变更、或实践中发现新的失败模式时，更新它。*
