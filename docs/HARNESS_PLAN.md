# DropVoice Harness 梳理 — 执行计划（开发者落地版）

> **阅读须知**：本文档是逐文件、逐命令的落地清单。每个步骤都标注了【精确操作】和【预期结果】。
> **严格按顺序执行，不要跳步、不要"优化"、不要自行判断**。遇到【预期结果】不符时，立即停止并报告，不要自行修复。
>
> **环境前提**：Windows + Git Bash + Python 3.14（`python` 命令可用）+ pnpm 11 + cargo（rustc 1.93）+ git（`core.autocrlf=true`）。
>
> **工作目录**：所有命令均在仓库根目录 `C:\Users\Administrator\Workspace\dropvoice` 执行，除非该步骤另注 `cd`。

---

## 目录

- [阶段 A：功能点1 收尾验证](#阶段-a功能点1-收尾验证)
- [阶段 B：功能点5 AGENTS.md / CLAUDE.md](#阶段-b功能点5agentsmd--claudemd)
- [阶段 C：功能点6 prek.toml](#阶段-c功能点6prektoml)
- [阶段 D：功能点7 AI agent hooks](#阶段-d功能点7ai-agent-hooks)
- [阶段 E：功能点2 CI/CD](#阶段-efunction点2cicd)
- [阶段 F：功能点3 VSCode](#阶段-ffunction点3vscode)
- [阶段 G：功能点4 数据库迁移](#阶段-gfunction点4数据库迁移)
- [阶段 H：.gitignore](#阶段-hgitignore)
- [阶段 I：全量验证](#阶段-i全量验证)
- [附录：变更清单](#附录变更清单)

---

## 阶段 A：功能点1 收尾验证

功能点1 已在前期实施（desktop Cargo.toml workspace 化、dirs→directories、axum 0.8 适配、clippy msrv 1.93、删除 clippy.toml、修复 manual_strip/redundant_closure）。本阶段**仅验证，不改动任何文件**。

### A1. 验证 workspace 编译

【精确操作】
```bash
cargo check --workspace 2>&1 | tail -3
```

【预期结果】最后一行包含 `Finished`，且无 `error` 字样。例如：
```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in X.XXs
```

【失败处理】如果有 `error`，停止全部流程并报告错误输出。不要自行修复。

### A2. 验证 clippy

【精确操作】
```bash
cargo clippy --workspace -- -D warnings 2>&1 | tail -3
```

【预期结果】最后一行 `Finished`，无 `warning`、无 `error`。

### A3. 验证 desktop 单元测试

【精确操作】
```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --lib 2>&1 | tail -5
```

【预期结果】包含 `test result: ok. 85 passed; 0 failed`（passed 数量 ≥ 85 即可）。

---

## 阶段 B：功能点5（AGENTS.md / CLAUDE.md）

本阶段创建 3 个文件：`scripts/sync-agents.py`、`AGENTS.md`、`CLAUDE.md`。

### B1. 创建 `scripts/sync-agents.py`

【精确操作】创建文件 `scripts/sync-agents.py`，内容**完整复制**下方代码块（从 `#!/usr/bin/env` 到文件末尾，一字不改）：

```python
#!/usr/bin/env python3
"""Check that AGENTS.md and CLAUDE.md are byte-for-byte identical.

AGENTS.md is the source of truth. CLAUDE.md must be an exact copy.
This script only CHECKS; it never writes. If the two files differ,
exit 1 with a message telling the developer to copy AGENTS.md to CLAUDE.md.

Usage:
    python scripts/sync-agents.py
"""
import filecmp
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
AGENTS = REPO_ROOT / "AGENTS.md"
CLAUDE = REPO_ROOT / "CLAUDE.md"


def main() -> int:
    if not AGENTS.exists():
        print("ERROR: AGENTS.md not found", file=sys.stderr)
        return 1
    if not CLAUDE.exists():
        print("ERROR: CLAUDE.md not found", file=sys.stderr)
        return 1
    if not filecmp.cmp(AGENTS, CLAUDE, shallow=False):
        print(
            "ERROR: AGENTS.md and CLAUDE.md are out of sync.\n"
            "AGENTS.md is the source of truth.\n"
            "Fix: copy AGENTS.md to CLAUDE.md (they must be byte-for-byte identical):\n"
            "    cp AGENTS.md CLAUDE.md",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
```

【验证】
```bash
ls -la scripts/sync-agents.py
```
【预期】文件存在，大小约 1KB。

### B2. 创建 `AGENTS.md`

【精确操作】创建文件 `AGENTS.md`（仓库根目录），内容**完整复制**下方代码块。

> **重要**：从 `` ```markdown `` 之后的第一行（即 `# AGENTS.md`）开始复制，直到最后的 `| Pairing server | 38424 |` 那一行结束。**不要**包含外层的 `` ```markdown `` 和 `` ``` `` 标记。

```markdown
# AGENTS.md

Guidance for AI coding agents (Claude Code, Codex, ZCode, Cursor) working in this repository.
This file is the single source of truth; `CLAUDE.md` is a byte-for-byte copy kept in sync by a precommit hook.

## Project Description

DropVoice sends voice-to-text input from a mobile phone to a PC over LAN. It is a monorepo with three applications:

- **`apps/desktop`** — Tauri 2 desktop app (Rust backend in `src-tauri/`, React 19 frontend in `src/`). Receives text and injects it via keyboard simulation.
- **`apps/mobile`** — Standalone Vite + React 19 PWA. The phone-side typing surface; connects to desktop over LAN WebSocket.
- **`apps/pairing-server`** — Rust (axum 0.8 + sqlx 0.8/SQLite) rendezvous server for LAN address lookup over public HTTPS.

Shared packages: `packages/core` (state machines, atoms, types), `packages/i18n` (4 locales), `packages/ui` (design system primitives).

## Commands

All commands run from the repo root unless noted. The project uses **pnpm** (>=11) and **Cargo workspaces**.

### Development

```bash
pnpm dev:tauri          # Full desktop app (Tauri + auto-starts frontend Vite on :5173)
pnpm dev:mobile         # Mobile PWA only (Vite on :5174)
cargo run --manifest-path apps/pairing-server/Cargo.toml   # Pairing server (default :38424)
```

### Quality gate (run before every commit)

```bash
pnpm quality            # typecheck + lint + format:check + test (frontend)
pnpm test:rust          # cargo test (desktop crate)
pnpm test:all           # pnpm test + pnpm test:rust
pnpm test:e2e           # Playwright e2e (root e2e/ dir)
```

### Build

```bash
pnpm build:tauri        # Production desktop installer (MSI/NSIS/dmg/deb)
pnpm ci:build           # Same as build:tauri (CI alias)
```

### Lint / Format (individual)

```bash
pnpm lint               # oxlint (all @dropvoice/* packages)
pnpm lint:fix           # oxlint --fix
pnpm format             # prettier --write
pnpm format:check       # prettier --check
cargo fmt               # Rust format (workspace root covers all crates)
cargo clippy --workspace -- -D warnings   # Rust lint, all crates
```

### Database migrations (pairing-server)

```bash
# Install sqlx-cli once:
cargo install sqlx-cli --no-default-features --features sqlite,rustls

sqlx migrate add -r <description>    # Create reversible migration (.up.sql + .down.sql)
sqlx migrate add <description>       # Create simple migration (.sql, default)
sqlx migrate info                    # Show migration status
sqlx database create                 # Create dev database
# Migrations apply automatically at server startup via sqlx::migrate!() - no manual run needed.
```

### Pre-commit hooks (setup once per clone)

```bash
prek install            # Install git hooks (pre-commit + commit-msg)
prek run --all-files    # Run all hooks manually
```

### Versioning & release

```bash
pnpm version:patch      # Bump patch + sync versions across packages + changelog
pnpm version:minor
pnpm version:major
# Release: push tag `v*` -> release.yml builds all platforms automatically.
```

## Project Structure

```
apps/
  desktop/
    src-tauri/          Rust backend (crate: dropvoice-desktop)
      src/
        commands/       Tauri commands: server.rs, settings.rs, window.rs
        server/         axum HTTP+WS: mod, http, websocket, connection_manager,
                        auth, validation, rate_limit, cors, health, heartbeat, pairing_lookup
        config/         DropVoiceConfig (TOML) + migration.rs
        network/        discovery (UDP broadcast), ip_monitor, pairing_clients, signaling, stun
        text/           injector.rs (enigo keyboard injection, EnigoInjector + MockInjector)
        telemetry/      logging.rs, metrics.rs
        lib.rs          Tauri Builder setup, 8 plugins, system tray, invoke_handler
        main.rs         Entry stub
    src/                React 19 frontend
      App.tsx           QueryClientProvider + JotaiProvider, auto-starts server
      hooks/            useServerState, useAppSettings, useAutoUpdate
      components/       HeaderBar, QRCodeSection, ConnectedDevices, InjectionQueueStatus,
                        LanWarningBanner, SettingsDialog
      lib/invoke.ts     Typed Tauri invoke wrapper
  mobile/
    src/                React 19 PWA
      App.tsx           Multi-device manager, pairing-code flow, send modes
      components/       MobileHeader, DeviceSelectorPanel, TextInputPanel, PairingCodeDialog, ...
      hooks/            useDraft, useMultiWebSocket
      lib/              discovery.ts, websocket.ts, storage.ts
  pairing-server/
    src/                axum 0.8 + sqlx 0.8
      api/              mod (router), auth, devices, pairing_codes, error, rate_limit, state
      store/            mod (open_pool), device_repo, pairing_code_repo
      domain/           device, pairing_code (pure data structs)
      batch/            BatchWriter (coalesced status writes)
      cache/            in-memory token + pairing-code cache
      config.rs         Time constants + runtime Config
      observability.rs  Metrics, tracing init
      main.rs           Startup: tracing -> config -> pool+migrate -> tasks -> serve
    migrations/         sqlx migrations (timestamp-prefixed: YYYYMMDDHHMMSS_name.sql)
    docker/             Multi-stage Dockerfile (cargo-chef), s6-overlay, Caddy
packages/
  core/                 Pure-TS: state machines (reducers, NOT xstate), jotai atoms, types, hooks
  i18n/                 i18next: en, zh, zh-TW, ja x {common,devices,errors,settings}
  ui/                   Design system: primitives, composite, layout; Tailwind v4 tokens
specs/                  Design specs (numbered 01-23) - READ ONLY, do not edit
e2e/                    Playwright e2e tests + config
```

### Tech stack (exact versions)

- **React** 19.0.0, **TypeScript** 5.9.0, **Vite** 6.4.0
- **Tauri** 2 (API ^2.10.0, CLI ^2.8.0, 8 plugins)
- **Rust** edition 2021, MSRV **1.93** (`rust-version` in workspace Cargo.toml)
- **axum** 0.8 (ws feature), **sqlx** 0.8 (sqlite, migrate, macros, chrono; bundled libsqlite3)
- **tower-http** 0.7 (trace, cors, fs), **tokio** 1 (full)
- **Tailwind CSS** 4.2.0, **jotai** 2.18, **@tanstack/react-query** 5.90
- **enigo** 0.6 (keyboard injection), **directories** 6 (path resolution)
- **vitest** 4.1, **@playwright/test** 1.49+, **oxlint** 0.15, **prettier** 3.6
- **Node** >=22, **pnpm** >=11 (packageManager pnpm@11.0.0)

## Code Style

### Rust - typed errors via thiserror

```rust
// Pattern: AppError enum with error_code(), AppResult = Result<T, AppError>
// apps/desktop/src-tauri/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("server already running")]
    ServerAlreadyRunning,
    #[error("max devices reached")]
    MaxDevicesReached,
}

impl AppError {
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::ServerAlreadyRunning => "SERVER_ALREADY_RUNNING",
            Self::MaxDevicesReached => "MAX_DEVICES",
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
```

### TypeScript - pure reducers for state machines

```typescript
// packages/core/src/machines/connection.ts - pure functions, no side effects (spec 02 section 5.3)
export type ConnectionState = 'idle' | 'connecting' | 'connected' | 'error';
export type ConnectionEvent = { type: 'CONNECT' } | { type: 'OPEN' } | { type: 'CLOSE' };

export function connectionReducer(state: ConnectionState, event: ConnectionEvent): ConnectionState {
    switch (event.type) {
        case 'CONNECT': return state === 'idle' ? 'connecting' : state;
        case 'OPEN':    return state === 'connecting' ? 'connected' : state;
        case 'CLOSE':   return 'idle';
    }
}
```

### Naming conventions

- Rust: `snake_case` for functions/variables, `PascalCase` for types. Module-level docs with `//!`.
- TypeScript: `camelCase` for variables/functions, `PascalCase` for types/components. Co-located tests as `*.test.ts(x)`.
- Tauri commands: `snake_case` function names, exposed to frontend via `invoke::<ReturnType>("snake_case_name")`.

## Git Workflow

- **Conventional Commits** enforced by commitlint (types: feat, fix, docs, style, refactor, perf, test, chore, revert).
- Example: `feat(desktop): add connection token persistence`
- Scope convention: `(desktop)`, `(mobile)`, `(pairing-server)`, `(core)`, `(ui)`, `(i18n)`, `(ci)`, `(docs)`.
- Versioning: `pnpm version:patch|minor|major` bumps root + syncs all packages + regenerates CHANGELOG.
- Release: push a `v*` tag -> `.github/workflows/release.yml` builds Windows/macOS/Linux bundles + creates GitHub release.

## Boundaries (do NOT directly edit)

1. **`specs/`** - design specifications are read-only. Propose changes via discussion, never edit in place.
2. **`packages/ui/src/tokens/theme.css`** - generated from `DESIGN.md`. Edit `DESIGN.md`, then run `pnpm design:sync` (lint + export CSS).
3. **`Cargo.lock`** - do not hand-edit, especially the `sqlx` version. It is pinned to 0.8 due to the rustc 1.93 toolchain constraint (sqlx 0.9 needs 1.94+). Regenerate only via `cargo update` for unrelated crates.
4. **`AGENTS.md` vs `CLAUDE.md`** - `AGENTS.md` is the source; `CLAUDE.md` must be an identical copy. Edit `AGENTS.md`, then `cp AGENTS.md CLAUDE.md`. The precommit hook rejects commits where they differ.
5. **Secrets** - never commit. Use environment variables (see `.env.example`).

## Key Conventions

### Workspace dependency inheritance

Both Cargo and pnpm workspaces centralize versions at the root. Member crates/packages inherit via `workspace = true` (Cargo) or `workspace:*` (pnpm). Never pin a version inline in a member if the root declares it - this caused axum 0.7/0.8 divergence before. When adding a shared dependency, add it to the root `[workspace.dependencies]` first.

### Pairing flow (spec 11, two-tier architecture)

1. Desktop `start_server` issues a local pairing code (in-memory) + starts heartbeat task.
2. Heartbeat registers with the public pairing server (`POST /devices`), gets a `pairing_token`, generates a remote code (`POST /devices/{id}/pairing-code`, 6-digit).
3. Mobile connects `ws://<lan-ip>:38425/ws?code=<pairing-code>`. Desktop validates: local -> cache -> remote lookup.
4. On success, desktop issues a UUID v4 connection `token`; mobile reconnects with `?token=` thereafter.

Full details in `specs/full/11-*.md`.

### i18n

4 locales (`en`, `zh`, `zh-TW`, `ja`) x 4 namespaces (`common`, `devices`, `errors`, `settings`) = 16 JSON files in `packages/i18n/locales/`. Detection: querystring `?lang=` -> localStorage `dropvoice-lang` -> navigator.

### Ports

| Service | Port |
|---------|------|
| Desktop frontend (Vite dev) | 5173 |
| Mobile PWA (Vite dev) | 5174 |
| Desktop Tauri backend (HTTP+WS) | 38425 |
| LAN discovery (UDP broadcast) | 38426 |
| Pairing server | 38424 |
```

### B3. 复制 `AGENTS.md` 到 `CLAUDE.md`

【精确操作】
```bash
cp AGENTS.md CLAUDE.md
```

> **说明**：用 `cp`（文件系统级复制），不用 git 操作。这样两个文件在磁盘上的字节完全一致（包括行尾符）。`core.autocrlf=true` 只在 git add/checkout 时转换，不影响已存在的本地文件。

### B4. 验证 B 阶段

【精确操作】
```bash
python scripts/sync-agents.py
echo "exit: $?"
diff AGENTS.md CLAUDE.md
echo "diff exit: $?"
```

【预期结果】
- `python scripts/sync-agents.py` 输出 `exit: 0`
- `diff` 无任何输出，`diff exit: 0`

【失败处理】如果 sync 脚本 exit 1，重新执行 `cp AGENTS.md CLAUDE.md`，再次验证。

---

## 阶段 C：功能点6（prek.toml）

### C1. 删除旧配置

【精确操作】
```bash
rm .pre-commit-config.yaml
ls .pre-commit-config.yaml 2>&1
```

【预期】`ls` 报告 "No such file or directory"（文件已删）。

### C2. 创建 `prek.toml`

【精确操作】创建文件 `prek.toml`（仓库根目录），内容**完整复制**：

```toml
# prek pre-commit configuration.
# Docs: https://prek.j178.dev/
# Setup: run `prek install` once after cloning (installs pre-commit + commit-msg git hooks).

minimum_prek_version = "0.2.0"
default_install_hook_types = ["pre-commit", "commit-msg"]

# --- Built-in fast hooks (Rust-native, no Python needed) ---
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

# --- Local hooks (project-specific tools) ---
[[repos]]
repo = "local"

# Frontend lint
[[repos.hooks]]
id = "oxlint"
name = "oxlint"
entry = "pnpm oxlint"
language = "system"
types_or = ["javascript", "typescript"]
pass_filenames = false

# Frontend format
[[repos.hooks]]
id = "prettier"
name = "Prettier"
entry = "pnpm prettier --write"
language = "system"
types_or = ["javascript", "typescript", "json", "css"]

# Rust format - workspace root covers desktop + pairing-server
[[repos.hooks]]
id = "cargo-fmt"
name = "cargo fmt"
entry = "cargo fmt --"
language = "system"
types = ["rust"]
pass_filenames = true

# Rust lint - workspace covers all crates
[[repos.hooks]]
id = "cargo-clippy"
name = "cargo clippy"
entry = "cargo clippy --workspace -- -D warnings"
language = "system"
types = ["rust"]
pass_filenames = false

# Design system lint
[[repos.hooks]]
id = "design-md-lint"
name = "DESIGN.md lint"
entry = "npx @google/design.md lint DESIGN.md"
language = "system"
files = "^DESIGN\\.md$"
pass_filenames = false

# AGENTS.md / CLAUDE.md sync check
[[repos.hooks]]
id = "agents-claude-sync"
name = "AGENTS.md/CLAUDE.md sync"
entry = "python scripts/sync-agents.py"
language = "system"
files = "^(AGENTS|CLAUDE)\\.md$"
pass_filenames = false

# Commit message lint (commit-msg stage)
[[repos.hooks]]
id = "commitlint"
name = "commitlint"
entry = "pnpm commitlint --edit"
language = "system"
stages = ["commit-msg"]
```

### C3. 验证 prek.toml 语法

【精确操作】
```bash
python -m prek validate-config 2>&1
echo "exit: $?"
```

【预期】exit 0，无错误输出（可能有 warning 关于 hook 环境，忽略）。

> **注意**：用 `python -m prek` 而非 `prek`，因为 pip 安装的可执行文件可能不在 Git Bash 的 PATH 中。如果 `python -m prek` 也报 "No module named prek"，先执行 `pip install prek`。

【失败处理】如果 validate-config 报 TOML 语法错误，检查是否完整复制了 C2 的内容（特别是 `\\.` 转义和 `[[repos.hooks]]` 结构）。

---

## 阶段 D：功能点7（AI agent hooks）

本阶段创建 3 个 Python 脚本，分发到 3 个 AI 目录（`.claude/hooks/`、`.codex/hooks/`、`.zcode/hooks/`），再配置 3 个 agent 的配置文件。

### D1. 准备三个脚本内容

**先在仓库根目录创建 3 个临时文件**，内容如下。D2 步骤会把它们分发到各 AI 目录，然后删除临时文件。

#### D1a. 临时文件 `_protect_generated_files.py`

【精确操作】创建文件 `_protect_generated_files.py`（仓库根目录，注意文件名前缀有下划线，标记为临时），内容**完整复制**：

```python
#!/usr/bin/env python3
"""PreToolUse hook: block direct edits to generated/protected files.

Reads tool_input.file_path from stdin JSON. If it targets a protected file,
exit 2 (block) with guidance on stderr. Otherwise exit 0 (allow).

Protected files:
  - packages/ui/src/tokens/theme.css  (generated from DESIGN.md)
  - Cargo.lock                        (toolchain-pinned, do not hand-edit)
"""
import json
import sys

PROTECTED = {
    "packages/ui/src/tokens/theme.css": (
        "theme.css is generated from DESIGN.md. Edit DESIGN.md then run: pnpm design:sync"
    ),
    "Cargo.lock": (
        "Cargo.lock is toolchain-pinned. Do not hand-edit "
        "(sqlx is pinned to 0.8 for rustc 1.93). Use 'cargo update <crate>' for unrelated deps."
    ),
}


def main() -> int:
    try:
        data = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        return 0  # can't parse input; allow (don't block on malformed hook data)

    file_path = data.get("tool_input", {}).get("file_path", "")
    if not file_path:
        return 0

    # Normalize to forward slashes for matching.
    normalized = file_path.replace("\\", "/")
    for protected_path, message in PROTECTED.items():
        if normalized == protected_path or normalized.endswith("/" + protected_path):
            print(f"BLOCKED: {message}", file=sys.stderr)
            return 2

    return 0


if __name__ == "__main__":
    sys.exit(main())
```

#### D1b. 临时文件 `_format_on_edit.py`

【精确操作】创建文件 `_format_on_edit.py`，内容**完整复制**：

```python
#!/usr/bin/env python3
"""PostToolUse hook: auto-format files after AI edits them.

Reads tool_input.file_path from stdin JSON. Routes by extension:
  - .ts/.tsx/.js/.jsx/.json/.css/.html/.md  -> prettier --write
  - .rs                                      -> cargo fmt (workspace)

Always exits 0 (formatting must not block the agent). Failures go to stderr
as feedback. The repo root is resolved from CLAUDE_PROJECT_DIR /
ZCODE_PROJECT_DIR environment variable, falling back to git toplevel.
"""
import json
import os
import subprocess
import sys
from pathlib import Path

PRETTIER_EXTS = {".ts", ".tsx", ".js", ".jsx", ".json", ".css", ".html", ".md"}
RUST_EXTS = {".rs"}


def repo_root() -> Path:
    env = os.environ.get("CLAUDE_PROJECT_DIR") or os.environ.get("ZCODE_PROJECT_DIR")
    if env:
        return Path(env)
    try:
        out = subprocess.check_output(["git", "rev-parse", "--show-toplevel"], text=True)
        return Path(out.strip())
    except Exception:
        return Path.cwd()


def main() -> int:
    try:
        data = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        return 0

    file_path = data.get("tool_input", {}).get("file_path", "")
    if not file_path:
        return 0

    path = Path(file_path)
    ext = path.suffix.lower()
    root = repo_root()

    try:
        if ext in PRETTIER_EXTS:
            subprocess.run(
                ["pnpm", "prettier", "--write", str(path)],
                cwd=str(root),
                capture_output=True,
                text=True,
                timeout=30,
            )
        elif ext in RUST_EXTS:
            subprocess.run(
                ["cargo", "fmt"],
                cwd=str(root),
                capture_output=True,
                text=True,
                timeout=60,
            )
    except Exception as e:
        print(f"format warning: {e}", file=sys.stderr)

    return 0


if __name__ == "__main__":
    sys.exit(main())
```

#### D1c. 临时文件 `_lint_on_stop.py`

【精确操作】创建文件 `_lint_on_stop.py`，内容**完整复制**：

```python
#!/usr/bin/env python3
"""Stop hook: run full lint before the agent ends the session.

Runs oxlint (frontend) + cargo clippy --workspace (Rust). If either fails,
exit 2 (block) with the error output on stderr, forcing the agent to fix
issues before it can stop. If all pass, exit 0.

Note: ZCode limits Stop blocks to 3 attempts. If the agent can't fix all
lint errors in 3 rounds, it will end anyway - the precommit hook is the
final safety net.
"""
import os
import subprocess
import sys
from pathlib import Path


def repo_root() -> Path:
    env = os.environ.get("CLAUDE_PROJECT_DIR") or os.environ.get("ZCODE_PROJECT_DIR")
    if env:
        return Path(env)
    try:
        out = subprocess.check_output(["git", "rev-parse", "--show-toplevel"], text=True)
        return Path(out.strip())
    except Exception:
        return Path.cwd()


def run(cmd, cwd: Path):
    """Returns (exit_code, combined_output)."""
    try:
        result = subprocess.run(cmd, cwd=str(cwd), capture_output=True, text=True, timeout=240)
        output = result.stdout + result.stderr
        return result.returncode, output
    except subprocess.TimeoutExpired:
        return 1, "TIMEOUT: " + " ".join(cmd)
    except Exception as e:
        return 1, "ERROR running " + " ".join(cmd) + ": " + str(e)


def main() -> int:
    root = repo_root()
    failures = []

    # Frontend lint
    code, out = run(["pnpm", "oxlint"], root)
    if code != 0:
        failures.append(("oxlint", out))

    # Rust lint (all workspace crates)
    code, out = run(["cargo", "clippy", "--workspace", "--", "-D", "warnings"], root)
    if code != 0:
        failures.append(("cargo clippy", out))

    if failures:
        print("Lint check FAILED - fix these before ending:", file=sys.stderr)
        for name, out in failures:
            print("\n--- " + name + " ---", file=sys.stderr)
            # Truncate to last 3000 chars to avoid huge output.
            print(out[-3000:], file=sys.stderr)
        return 2

    return 0


if __name__ == "__main__":
    sys.exit(main())
```

> **关于 `run()` 函数的类型注解**：上面 D1c 的 `run(cmd, cwd: Path)` 刻意**没有**给 `cmd` 加类型注解（不写 `cmd: list[str]`）。这是因为虽然开发环境是 Python 3.14，但脚本要兼容可能存在的 Python 3.8 环境（`list[str]` 语法需要 3.9+）。保持无注解最安全。

### D2. 分发脚本到三个 AI 目录

【精确操作】逐条执行：

```bash
mkdir -p .claude/hooks .codex/hooks .zcode/hooks

cp _protect_generated_files.py .claude/hooks/protect_generated_files.py
cp _format_on_edit.py .claude/hooks/format_on_edit.py
cp _lint_on_stop.py .claude/hooks/lint_on_stop.py

cp _protect_generated_files.py .codex/hooks/protect_generated_files.py
cp _format_on_edit.py .codex/hooks/format_on_edit.py
cp _lint_on_stop.py .codex/hooks/lint_on_stop.py

cp _protect_generated_files.py .zcode/hooks/protect_generated_files.py
cp _format_on_edit.py .zcode/hooks/format_on_edit.py
cp _lint_on_stop.py .zcode/hooks/lint_on_stop.py

rm _protect_generated_files.py _format_on_edit.py _lint_on_stop.py
```

【验证】
```bash
ls .claude/hooks/ .codex/hooks/ .zcode/hooks/
```
【预期】每个目录都列出相同的 3 个文件：`format_on_edit.py`、`lint_on_stop.py`、`protect_generated_files.py`。

【验证副本一致性】
```bash
diff .claude/hooks/protect_generated_files.py .codex/hooks/protect_generated_files.py && echo "codex OK"
diff .claude/hooks/protect_generated_files.py .zcode/hooks/protect_generated_files.py && echo "zcode OK"
diff .claude/hooks/format_on_edit.py .codex/hooks/format_on_edit.py && echo "codex fmt OK"
diff .claude/hooks/lint_on_stop.py .zcode/hooks/lint_on_stop.py && echo "zcode lint OK"
```
【预期】4 行 `OK`，diff 无输出。

### D3. 配置 `.claude/settings.json`

【精确操作】**完全替换** `.claude/settings.json` 的内容（该文件已存在，当前是 `{"hooks":{},"enabledPlugins":{}}`）。新内容：

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Edit|Write|ApplyPatch",
        "hooks": [
          {
            "type": "command",
            "command": "python \"${CLAUDE_PROJECT_DIR}/.claude/hooks/protect_generated_files.py\""
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "Edit|Write|ApplyPatch",
        "hooks": [
          {
            "type": "command",
            "command": "python \"${CLAUDE_PROJECT_DIR}/.claude/hooks/format_on_edit.py\""
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "python \"${CLAUDE_PROJECT_DIR}/.claude/hooks/lint_on_stop.py\"",
            "timeout": 240
          }
        ]
      }
    ]
  },
  "enabledPlugins": {}
}
```

> **字段说明**：
> - `timeout: 240` 单位是秒（Claude Code command hook 的 timeout 字段单位）。cargo clippy 全量可能需要 1-3 分钟。
> - `${CLAUDE_PROJECT_DIR}` 是 Claude Code 注入的环境变量，指向项目根。
> - `matcher: "Edit|Write|ApplyPatch"` 用竖线分隔匹配多个工具名（正则）。
> - `Stop` 事件不设 matcher（对所有停止触发）。

### D4. 配置 `.zcode/config.json`

【精确操作】创建新文件 `.zcode/config.json`，内容：

```json
{
  "hooks": {
    "enabled": true,
    "events": {
      "PreToolUse": [
        {
          "matcher": "Edit|Write|ApplyPatch",
          "hooks": [
            {
              "type": "command",
              "command": "python \"${ZCODE_PROJECT_DIR}/.zcode/hooks/protect_generated_files.py\""
            }
          ]
        }
      ],
      "PostToolUse": [
        {
          "matcher": "Edit|Write|ApplyPatch",
          "hooks": [
            {
              "type": "command",
              "command": "python \"${ZCODE_PROJECT_DIR}/.zcode/hooks/format_on_edit.py\""
            }
          ]
        }
      ],
      "Stop": [
        {
          "hooks": [
            {
              "type": "command",
              "command": "python \"${ZCODE_PROJECT_DIR}/.zcode/hooks/lint_on_stop.py\"",
              "timeout": 240
            }
          ]
        }
      ]
    }
  }
}
```

> **与 Claude 的差异**：
> - ZCode **必须** `"enabled": true`（默认禁用）。
> - 事件放在 `events` 下（Claude 是直接 `hooks.<Event>`）。
> - 用 `${ZCODE_PROJECT_DIR}`（ZCode 注入）而非 `${CLAUDE_PROJECT_DIR}`。

### D5. 配置 `.codex/config.toml`

【精确操作】创建新文件 `.codex/config.toml`，内容：

```toml
# Codex CLI configuration.
# Docs: https://learn.chatgpt.com/docs/config-file/config-reference

[features]
hooks = true

# --- PreToolUse: protect generated files ---
[[hooks.PreToolUse]]
matcher = "Edit|Write|ApplyPatch"

[[hooks.PreToolUse.hooks]]
type = "command"
command = "python3 .codex/hooks/protect_generated_files.py"
command_windows = "python .codex/hooks/protect_generated_files.py"

# --- PostToolUse: auto-format after edits ---
[[hooks.PostToolUse]]
matcher = "Edit|Write|ApplyPatch"

[[hooks.PostToolUse.hooks]]
type = "command"
command = "python3 .codex/hooks/format_on_edit.py"
command_windows = "python .codex/hooks/format_on_edit.py"

# --- Stop: lint before ending ---
[[hooks.Stop]]

[[hooks.Stop.hooks]]
type = "command"
command = "python3 .codex/hooks/lint_on_stop.py"
command_windows = "python .codex/hooks/lint_on_stop.py"
timeout = 240
```

> **Codex 特有字段说明**：
> - `features.hooks = true` 必须开启（Codex 默认不启用 hooks）。
> - `command`（Unix 用 `python3`）+ `command_windows`（Windows 用 `python`）：Codex 原生支持平台分流，这是它的优势。
> - `timeout: 240` 单位是秒。
> - Codex 的 `apply_patch` 是工具名，但 matcher 接受 `Edit`/`Write` 兼容别名，所以写 `Edit|Write|ApplyPatch` 全集即可覆盖。

### D6. 验证 hook 脚本

【精确操作】逐条执行并核对 exit code：

```bash
# 测试 1: protect - 命中 Cargo.lock，应 exit 2
echo '{"tool_input":{"file_path":"Cargo.lock"}}' | python .claude/hooks/protect_generated_files.py
echo "test1 exit: $? (expect 2)"

# 测试 2: protect - 命中 theme.css，应 exit 2
echo '{"tool_input":{"file_path":"packages/ui/src/tokens/theme.css"}}' | python .claude/hooks/protect_generated_files.py
echo "test2 exit: $? (expect 2)"

# 测试 3: protect - 普通文件，应 exit 0
echo '{"tool_input":{"file_path":"apps/desktop/src/main.tsx"}}' | python .claude/hooks/protect_generated_files.py
echo "test3 exit: $? (expect 0)"

# 测试 4: protect - 畸形 JSON，应 exit 0（不阻塞）
echo 'not json' | python .claude/hooks/protect_generated_files.py
echo "test4 exit: $? (expect 0)"

# 测试 5: format - 普通文件，应 exit 0（可能实际格式化文件）
echo '{"tool_input":{"file_path":"AGENTS.md"}}' | python .claude/hooks/format_on_edit.py
echo "test5 exit: $? (expect 0)"

# 测试 6: format - 无 file_path，应 exit 0
echo '{"tool_input":{}}' | python .claude/hooks/format_on_edit.py
echo "test6 exit: $? (expect 0)"
```

【预期】6 个测试的 exit code 分别是：2, 2, 0, 0, 0, 0。

> **注意**：测试 5 会实际调用 `pnpm prettier --write AGENTS.md`，可能修改文件格式。这是正常的。如果担心，测试后用 `git diff AGENTS.md` 检查，prettier 只会做格式规范化。

> **不测试 lint_on_stop.py**：它跑全量 oxlint + clippy，耗时 1-3 分钟，留到阶段 I 统一验证。

---

## 阶段 E：功能点2（CI/CD）

### E1. 替换 `.github/workflows/quality.yml`

【精确操作】**完全替换** `.github/workflows/quality.yml`（当前 125 行）。新内容：

```yaml
name: Quality Checks

on:
  pull_request:
    branches: [main]
    paths-ignore:
      - '**.md'
      - 'docs/**'
      - 'specs/**'
      - 'LICENSE'
      - '.gitignore'
  push:
    branches: [main]
    paths-ignore:
      - '**.md'
      - 'docs/**'
      - 'specs/**'
      - 'LICENSE'
      - '.gitignore'

concurrency:
  group: quality-${{ github.ref }}
  cancel-in-progress: true

jobs:
  changes:
    name: Detect Changes
    runs-on: ubuntu-latest
    outputs:
      frontend: ${{ steps.filter.outputs.frontend }}
      desktop: ${{ steps.filter.outputs.desktop }}
      pairing: ${{ steps.filter.outputs.pairing }}
    steps:
      - uses: actions/checkout@v4
      - uses: dorny/paths-filter@de90cc6fb66fc756703f280f72eb1f71e65ce00f
        id: filter
        with:
          filters: |
            frontend:
              - 'apps/desktop/src/**'
              - 'apps/mobile/**'
              - 'packages/**'
              - 'e2e/**'
              - '*.json'
              - '*.ts'
              - '.oxlintrc.json'
              - '.prettierrc'
              - 'tsconfig.base.json'
            desktop:
              - 'apps/desktop/src-tauri/**'
              - 'Cargo.toml'
              - 'Cargo.lock'
              - 'rustfmt.toml'
            pairing:
              - 'apps/pairing-server/**'
              - 'Cargo.toml'
              - 'Cargo.lock'
              - 'rustfmt.toml'

  frontend:
    name: Frontend
    needs: changes
    if: needs.changes.outputs.frontend == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with:
          version: 11
      - uses: actions/setup-node@v4
        with:
          node-version: 22
          cache: pnpm
      - run: pnpm install --frozen-lockfile
      - name: TypeScript
        run: pnpm typecheck
      - name: Lint
        run: pnpm lint
      - name: Format
        run: pnpm format:check
      - name: Test
        run: pnpm test:coverage
      - name: DESIGN.md
        run: pnpm design:lint

  design-sync:
    name: Design Sync
    needs: changes
    if: needs.changes.outputs.frontend == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with:
          version: 11
      - uses: actions/setup-node@v4
        with:
          node-version: 22
          cache: pnpm
      - run: pnpm install --frozen-lockfile
      - name: Export DESIGN.md tokens
        run: pnpm design:export:css
      - name: Check sync
        run: |
          if [ -n "$(git diff packages/ui/src/tokens/theme.css)" ]; then
            echo "::error::DESIGN.md tokens out of sync. Run: pnpm design:sync"
            exit 1
          fi
          echo "Design tokens in sync"

  backend:
    name: Rust (${{ matrix.os }})
    needs: changes
    if: needs.changes.outputs.desktop == 'true'
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: apps/desktop/src-tauri
      - name: Format
        run: cargo fmt --check --manifest-path apps/desktop/src-tauri/Cargo.toml
      - name: Clippy
        run: cargo clippy --manifest-path apps/desktop/src-tauri/Cargo.toml -- -D warnings
      - name: Test
        run: cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml

  pairing-server:
    name: Pairing Server
    needs: changes
    if: needs.changes.outputs.pairing == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: apps/pairing-server
      - name: Format
        run: cargo fmt --check --manifest-path apps/pairing-server/Cargo.toml
      - name: Clippy
        run: cargo clippy --manifest-path apps/pairing-server/Cargo.toml -- -D warnings
      - name: Unit + Integration tests
        run: cargo test --manifest-path apps/pairing-server/Cargo.toml
```

【关键改动说明】（供理解，不需额外操作）：
1. `on.pull_request` 和 `on.push` 都加了 `paths-ignore`（纯文档不触发）。
2. 新增 `changes` job，用 `dorny/paths-filter@de90cc6fb66fc756703f280f72eb1f71e65ce00f`（pin 到 v3 的具体 commit SHA，防供应链攻击）。
3. 每个业务 job 加了 `needs: changes` 和 `if:` 条件。
4. desktop 保留 3 平台矩阵，pairing-server 单 ubuntu。

### E2. release.yml 不改动

【精确操作】不触碰 `.github/workflows/release.yml`。

【验证】
```bash
git diff .github/workflows/release.yml
```
【预期】无输出（未改动）。

---

## 阶段 F：功能点3（VSCode）

### F1. 创建 `.vscode/tasks.json`

【精确操作】创建新文件 `.vscode/tasks.json`，内容：

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "dev:desktop",
      "detail": "Full Tauri app (frontend + Rust backend)",
      "type": "shell",
      "command": "pnpm dev:tauri",
      "isBackground": true,
      "problemMatcher": [
        {
          "owner": "tauri-dev",
          "pattern": { "regexp": "^$" },
          "background": {
            "activeOnStart": true,
            "beginsPattern": "VITE v",
            "endsPattern": "Local:.*5173|Compiling|VITE.*ready"
          }
        }
      ],
      "group": { "kind": "build", "isDefault": true },
      "presentation": { "reveal": "always", "panel": "dedicated", "clear": true }
    },
    {
      "label": "dev:pairing-server",
      "detail": "Pairing server (Rust, port 38424)",
      "type": "shell",
      "command": "cargo run --manifest-path apps/pairing-server/Cargo.toml",
      "isBackground": true,
      "problemMatcher": [
        {
          "owner": "pairing-server",
          "pattern": { "regexp": "^$" },
          "background": {
            "activeOnStart": true,
            "beginsPattern": "Compiling",
            "endsPattern": "pairing server listening|sqlite pool opened"
          }
        }
      ],
      "presentation": { "reveal": "always", "panel": "dedicated", "clear": true }
    },
    {
      "label": "dev:mobile",
      "detail": "Mobile PWA (Vite, port 5174)",
      "type": "shell",
      "command": "pnpm dev:mobile",
      "isBackground": true,
      "problemMatcher": [
        {
          "owner": "mobile-dev",
          "pattern": { "regexp": "^$" },
          "background": {
            "activeOnStart": true,
            "beginsPattern": "VITE v",
            "endsPattern": "Local:.*5174"
          }
        }
      ],
      "presentation": { "reveal": "always", "panel": "dedicated", "clear": true }
    },
    {
      "label": "dev:full",
      "detail": "Desktop + pairing server (parallel)",
      "dependsOn": ["dev:desktop", "dev:pairing-server"],
      "dependsOrder": "parallel",
      "problemMatcher": []
    }
  ]
}
```

> **使用方式**：
> - `dev:desktop` 是默认 build task，按 `Ctrl+Shift+B` 启动。
> - 其他 task：`Ctrl+Shift+P` → "Tasks: Run Task" → 选择。
> - `dev:full` 并行启动 desktop + pairing-server。

### F2. 创建 `.vscode/extensions.json`

【精确操作】创建新文件 `.vscode/extensions.json`，内容：

```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "oxc.oxc-vscode",
    "esbenp.prettier-vscode",
    "tauri-apps.tauri-vscode",
    "lokalise.i18n-ally",
    "ms-python.python"
  ]
}
```

### F3. 修改 `.vscode/settings.json`

【精确操作】**完全替换** `.vscode/settings.json`，新内容：

```json
{
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "esbenp.prettier-vscode",
  "editor.codeActionsOnSave": {
    "source.fixAll.oxlint": "explicit"
  },
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.check.extraArgs": [],
  "typescript.tsdk": "node_modules/typescript/lib",
  "i18n-ally.localesPaths": ["packages/i18n/locales"],
  "python.defaultInterpreterPath": "${workspaceFolder}/.venv"
}
```

> **唯一改动**：`rust-analyzer.check.extraArgs` 从 `["--manifest-path", "apps/desktop/src-tauri/Cargo.toml"]` 改为 `[]`（让 rust-analyzer 检查整个 workspace 而非只 desktop）。新增 `python.defaultInterpreterPath`。

---

## 阶段 G：功能点4（数据库迁移）

### G1. 创建 `apps/pairing-server/sqlx.toml`

【精确操作】创建新文件 `apps/pairing-server/sqlx.toml`，内容：

```toml
# sqlx-cli configuration for dropvoice-pairing-server.
# Docs: https://docs.rs/sqlx/latest/sqlx/macro.migrate.html

[create]
# Simple migrations (single .sql file) are the default. Use --reversible /
# -r flag on `sqlx migrate add` for migrations that need a down-migration.
default_migration_type = "simple"

# Timestamp versioning (YYYYMMDDHHMMSS) matches existing migration naming.
default_versioning = "timestamp"
```

### G2. 创建 `apps/pairing-server/.env.example`

【精确操作】创建新文件 `apps/pairing-server/.env.example`，内容：

```bash
# Database URL for local development. The server reads DATABASE_URL at startup.
# `mode=rwc` creates the file if it doesn't exist (read/write/create).
DATABASE_URL=sqlite://apps/pairing-server/data/dropvoice.db?mode=rwc

# Optional: override the listen address (default: 127.0.0.1:38424).
# PAIRING_SERVER_LISTEN_ADDR=0.0.0.0:38424
```

### G3. 创建 `docs/migrations.md`

【精确操作】创建新文件 `docs/migrations.md`，内容**完整复制**：

````markdown
# Database Migration Workflow (pairing-server)

The pairing-server uses **sqlx 0.8** with **embedded migrations**. Migrations are
compiled into the binary via `sqlx::migrate!("./migrations")` and applied
automatically at server startup — no manual `migrate run` is ever needed in
any environment.

## One-time setup

```bash
cargo install sqlx-cli --no-default-features --features sqlite,rustls
```

## Creating a migration

```bash
cd apps/pairing-server

# Simple migration (default): single .sql file, cannot be reverted.
sqlx migrate add add_user_preferences_column
# → creates migrations/<timestamp>_add_user_preferences_column.sql

# Reversible migration: .up.sql + .down.sql pair.
sqlx migrate add -r refactor_devices_schema
# → creates migrations/<timestamp>_refactor_devices_schema.up.sql
#             + migrations/<timestamp>_refactor_devices_schema.down.sql
```

**When to use which:**
- **Simple** (`add`): schema additions that have no meaningful rollback (new tables, new columns with defaults). The `init` migration is simple.
- **Reversible** (`add -r`): migrations with a clear, safe down-path (e.g., renaming a column back). Reserve for cases where rollback is genuinely needed. Never use reversible for destructive changes where data would be lost.

## File naming convention

```
YYYYMMDDHHMMSS_<description>.sql           (simple)
YYYYMMDDHHMMSS_<description>.up.sql        (reversible, up)
YYYYMMDDHHMMSS_<description>.down.sql      (reversible, down)
```

Migrations are applied in ascending version-number order. Never edit an
already-applied migration — always add a new one.

## How migrations apply

1. `store::open_pool()` creates the SQLite connection pool (WAL mode, pragmas).
2. `sqlx::migrate!("./migrations").run(&pool)` runs all pending migrations.
3. The `_sqlx_migrations` table tracks which versions are applied.

This happens on every server start (dev, test, production). It is idempotent:
already-applied migrations are skipped.

## Local database management

```bash
cd apps/pairing-server

# Create a fresh database (uses DATABASE_URL from .env or default).
sqlx database create

# Drop and recreate (destroys all data).
sqlx database drop -f && sqlx database create

# Inspect migration state.
sqlx migrate info
```

## Query style

The pairing-server uses **runtime queries** (`query_as` + `FromRow`), NOT
compile-time `query!` macros. This means:

- No `DATABASE_URL` is required at compile time.
- No `cargo sqlx prepare` / `.sqlx` offline directory is needed.
- CI builds work without a database connection.

If a future change introduces `query!` macros, run `cargo sqlx prepare` and
commit the `.sqlx/` directory.
````

> **注意**：上面的内容包含嵌套的 ```` ```` ```` 代码块（4 个反引号包裹 3 个反引号的代码块）。复制时确保最外层是 4 个反引号，内层是 3 个。这样 Markdown 才能正确渲染。

---

## 阶段 H：.gitignore

### H1. 修改 `.gitignore`

【精确操作】找到 `.gitignore` 中的这两行（第 17-18 行附近）：

```
!.vscode/settings.json
!.vscode/extensions.json
```

在 `!.vscode/extensions.json` **下方紧接一行**追加：

```
!.vscode/tasks.json
```

修改后该段应为：
```
.vscode/*
!.vscode/settings.json
!.vscode/extensions.json
!.vscode/tasks.json
```

【验证】
```bash
grep -n "tasks.json" .gitignore
```
【预期】输出一行包含 `!.vscode/tasks.json`。

> **不要改其他任何行**。`.claude/`、`.zcode/`、`.codex/`、`scripts/` 都不加入 .gitignore（它们是团队共享配置）。

---

## 阶段 I：全量验证

按顺序执行。任何一步失败则停止并报告。

### I1. 确认工具已安装

【精确操作】
```bash
python --version          # 应输出 Python 3.x
python -m prek --version  # 应输出 prek 版本号；若无，执行 pip install prek
cargo --version           # 应输出 cargo 1.93
pnpm --version            # 应输出 11.x
```

### I2. Rust 全量

【精确操作】
```bash
cargo fmt --check 2>&1 | tail -3
echo "fmt exit: $?"
cargo clippy --workspace -- -D warnings 2>&1 | tail -3
echo "clippy exit: $?"
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --lib 2>&1 | tail -3
echo "desktop test exit: $?"
```

【预期】四个 exit 全为 0。`cargo fmt --check` 若输出文件名列表，说明有未格式化的文件，执行 `cargo fmt` 后重试。

### I3. 前端全量

【精确操作】
```bash
pnpm install --frozen-lockfile 2>&1 | tail -2
pnpm typecheck 2>&1 | tail -3
pnpm lint 2>&1 | tail -3
pnpm format:check 2>&1 | tail -3
pnpm test 2>&1 | tail -5
```

【预期】全部 exit 0。若 `format:check` 失败，执行 `pnpm format` 后重试。

### I4. AGENTS/CLAUDE 同步

【精确操作】
```bash
python scripts/sync-agents.py
echo "sync exit: $? (expect 0)"
diff AGENTS.md CLAUDE.md
echo "diff exit: $? (expect 0, no output)"
```

### I5. prek 验证

【精确操作】
```bash
python -m prek validate-config 2>&1
echo "validate exit: $?"
python -m prek install 2>&1 | tail -3
echo "install exit: $?"
ls .git/hooks/pre-commit .git/hooks/commit-msg 2>&1
```

【预期】
- validate exit 0
- install exit 0
- `.git/hooks/pre-commit` 和 `.git/hooks/commit-msg` 文件存在（不再是 `.sample`）

### I6. hook 脚本最终验证

【精确操作】（与 D6 相同，确认配置文件改动后仍正常）
```bash
echo '{"tool_input":{"file_path":"Cargo.lock"}}' | python .claude/hooks/protect_generated_files.py
echo "protect blocked exit: $? (expect 2)"

echo '{"tool_input":{"file_path":"README.md"}}' | python .claude/hooks/protect_generated_files.py
echo "protect allow exit: $? (expect 0)"

echo '{"tool_input":{"file_path":"AGENTS.md"}}' | python .claude/hooks/format_on_edit.py
echo "format exit: $? (expect 0)"
```

### I7. lint_on_stop 全量验证

【精确操作】
```bash
python .claude/hooks/lint_on_stop.py 2>&1 | tail -10
echo "lint_on_stop exit: $? (expect 0)"
```

【预期】exit 0（前提是代码无 lint 错误）。若 exit 2，根据 stderr 输出的 oxlint/clippy 错误修复后重试。

> **耗时**：此步骤跑全量 oxlint + cargo clippy，可能需要 1-3 分钟。

### I8. 迁移工作流验证（需 sqlx-cli）

【精确操作】
```bash
# 若未安装 sqlx-cli：
cargo install sqlx-cli --no-default-features --features sqlite,rustls 2>&1 | tail -2

cd apps/pairing-server
sqlx migrate info 2>&1
cd ../..
```

【预期】`sqlx migrate info` 输出一个表格，显示 `20260723000001_init` 迁移（状态可能是 pending 或 applied，取决于是否有数据库文件）。

---

## 附录：变更清单

| 操作 | 文件路径 | 阶段 |
|------|----------|------|
| 新建 | `scripts/sync-agents.py` | B1 |
| 新建 | `AGENTS.md` | B2 |
| 新建（cp 自 AGENTS.md） | `CLAUDE.md` | B3 |
| 删除 | `.pre-commit-config.yaml` | C1 |
| 新建 | `prek.toml` | C2 |
| 新建 | `.claude/hooks/protect_generated_files.py` | D2 |
| 新建 | `.claude/hooks/format_on_edit.py` | D2 |
| 新建 | `.claude/hooks/lint_on_stop.py` | D2 |
| 新建 | `.codex/hooks/protect_generated_files.py` | D2 |
| 新建 | `.codex/hooks/format_on_edit.py` | D2 |
| 新建 | `.codex/hooks/lint_on_stop.py` | D2 |
| 新建 | `.zcode/hooks/protect_generated_files.py` | D2 |
| 新建 | `.zcode/hooks/format_on_edit.py` | D2 |
| 新建 | `.zcode/hooks/lint_on_stop.py` | D2 |
| 替换 | `.claude/settings.json` | D3 |
| 新建 | `.zcode/config.json` | D4 |
| 新建 | `.codex/config.toml` | D5 |
| 替换 | `.github/workflows/quality.yml` | E1 |
| 不改 | `.github/workflows/release.yml` | E2 |
| 新建 | `.vscode/tasks.json` | F1 |
| 新建 | `.vscode/extensions.json` | F2 |
| 替换 | `.vscode/settings.json` | F3 |
| 新建 | `apps/pairing-server/sqlx.toml` | G1 |
| 新建 | `apps/pairing-server/.env.example` | G2 |
| 新建 | `docs/migrations.md` | G3 |
| 修改（+1行） | `.gitignore` | H1 |

**共 26 项变更**（17 新建 + 4 替换 + 1 删除 + 1 修改 + 3 不改/含功能点1 已完成的改动）。

> **注意**：功能点1 的改动（Cargo.toml 系列、Rust 源码、删除 clippy.toml）已在前期完成，不在本清单内。
```

文件已写入。让我做最后的一致性检查——确认计划里的关键事实与项目现状吻合。[..., ]
