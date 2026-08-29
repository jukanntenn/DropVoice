# 开发

操作步骤：日常命令、质量门控、数据库迁移、部署与发布。系统现状事实在 [architecture.md](architecture.zh.md)；规则在[根 `AGENTS.md`](../AGENTS.md) 与 [docs/AGENTS.md](AGENTS.md)。

除特别说明外，所有命令都在仓库根执行。项目使用 **pnpm**（>=11）与 **Cargo workspaces**。

## 日常开发

每个环境一条命令；所需配置已烘焙进 `.vscode/tasks.json` 任务 env（自包含——无需配置文件、无需手动 env）。`Alt+R`（工作区键绑定）运行 `dev:full`；除此之外无需配置：

| 任务                 | 运行内容                                                                                                                  |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `dev:full`           | desktop + pairing-server + mobile 并行（绑定 `Alt+R`）                                                                    |
| `dev:desktop`        | `pnpm dev:tauri`，内置 `PAIRING_SERVER_URL=http://localhost:7380`                                                         |
| `dev:pairing-server` | `cargo run` → 127.0.0.1:7380，env 注入（`DATABASE_URL`=gitignored `dev.db`、`RATE_LIMIT_PER_SEC=10`、`ENABLE_DOCS=true`） |
| `dev:mobile`         | `pnpm dev:mobile` → :5174（任务 env 把 `/api` 代理到 :7380）                                                              |
| `accept:desktop`     | desktop 对接验收容器（`PAIRING_SERVER_URL=:8080`）                                                                        |
| `accept:mobile`      | mobile，`/api` 代理到验收容器（`:8080`）                                                                                  |

CLI 等价物（不用 VS Code 时）：`pnpm dev:tauri`、`pnpm dev:mobile`、`pnpm dev:landing`（→ :5175）、`cargo run --manifest-path apps/pairing-server/Cargo.toml`——为 desktop 进程导出 `PAIRING_SERVER_URL=http://localhost:7380`，为 mobile 导出 `VITE_API_PROXY_TARGET=http://localhost:7380`（见根 `.env.example`；VS Code 任务是受认可的零配置入口）。新 clone 后跑一次 `pnpm doctor`，一条命令检查工具链、prek 钩子与端口。

## 质量门控（prek——单一事实源）

全部质量门控都在 **prek**（workspace 模式）：根 `prek.toml` 加每个项目一份（`apps/desktop`、`apps/landing`、`apps/mobile`、`apps/pairing-server`、`packages/{core,i18n,ui}`）。CI、git 钩子与 AI 工具钩子都调用 prek——别处不存在平行的 lint/format 定义。

```bash
prek install            # once per clone: pre-commit + pre-push + commit-msg hooks
prek run --all-files    # format + lint + gates (pre-commit stage)
prek run --stage pre-push --all-files   # tests (pre-push stage)
pnpm quality            # alias for both of the above
pnpm docs:check         # documentation gates (dv-rfcs format, doc pairs, budgets, links)
pnpm test:e2e           # Playwright e2e (root e2e/ dir; also a CI job)
```

Hook **组**（与 stage 正交）：

- `format`——自动修复的改写型 fixer（prettier --write、oxlint --fix、cargo fmt、内建空白/EOF 修复器）
- `lint`——只读门控（oxlint、tsc --noEmit、cargo clippy -D warnings）
- `check`——结构性检查、生成物漂移、lockfile 新鲜度（pre-commit stage）；vitest + cargo test（pre-push stage）

调用映射：AI post-edit 钩子跑 `prek run --group format --files <edited>`；AI Stop 钩子跑 `prek run --group lint --all-files`；commit → pre-commit stage；push → pre-push stage；CI → `prek run --all-files` 与 `prek run --stage pre-push --all-files` 两者（`.github/workflows/quality.yml`），另有 Windows/macOS desktop-Rust 平台矩阵与 Playwright e2e job。

生成物/锁文件豁免于所有 fixer，改由漂移门控守护：`packages/ui/src/tokens/theme.css`（`design-sync`，修复：`pnpm design:sync`）、`apps/pairing-server/docs/openapi.{json,yaml}`（`openapi-drift`，修复：`gen-openapi`）、`pnpm-lock.yaml`/`Cargo.lock`（`lockfiles-fresh`，修复：`pnpm install --lockfile-only` + 任意 cargo 命令）、`AGENTS.md`/`CLAUDE.md` 各对与 skills 镜像（`agents-sync`，修复：`python scripts/sync_agent_files.py`）、Markdown 内容（`doc-check` → `python scripts/doc_sync.py`）。

## AI 工具钩子（prek 的薄包装）

`.claude/`、`.zcode/`、`.codex/`、`.opencode/` 不携带任何 lint/format 逻辑——它们的钩子是 prek 之上的一次性适配器：

- PostToolUse（Edit|Write|ApplyPatch）→ `prek run --group format --files <edited>`（从不阻断；退出码 0/1 都可接受）
- Stop → `prek run --group lint --all-files`，失败时带诊断阻断（OpenCode 没有可阻断的 Stop 通道——退回 pre-commit 门控 + CI）

工具间差异仅在载荷解析（snake_case 与 camelCase 键）。要改 formatter/linter 行为，改 `prek.toml`——绝不改编适配器。

## 代码风格

Rust——经 thiserror 的类型化错误（带 `error_code()` 的 `AppError` 枚举，`AppResult = Result<T, AppError>`）；见 `apps/desktop/src-tauri/src/error.rs`：

```rust
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

TypeScript——状态机用纯 reducer、零副作用（spec 02 第 5.3 节）；见 `packages/core/src/machines/connection.ts`：

```typescript
export type ConnectionState = 'idle' | 'connecting' | 'connected' | 'error';
export type ConnectionEvent = { type: 'CONNECT' } | { type: 'OPEN' } | { type: 'CLOSE' };

export function connectionReducer(state: ConnectionState, event: ConnectionEvent): ConnectionState {
  switch (event.type) {
    case 'CONNECT':
      return state === 'idle' ? 'connecting' : state;
    case 'OPEN':
      return state === 'connecting' ? 'connected' : state;
    case 'CLOSE':
      return 'idle';
  }
}
```

命名：Rust `snake_case`/`PascalCase`，模块级 `//!` 文档；TypeScript `camelCase`/`PascalCase`，测试与代码同目录（`*.test.ts(x)`）；Tauri 命令 `snake_case`，前端 `invoke::<ReturnType>("snake_case_name")` 调用。

## 单独的 lint / format 命令（prek 包装的就是它们）

```bash
pnpm lint               # oxlint (all @dropvoice/* packages)
pnpm lint:fix           # oxlint --fix
pnpm format             # prettier --write
pnpm format:check       # prettier --check
cargo fmt               # Rust format (workspace root covers all crates)
cargo clippy --workspace -- -D warnings   # Rust lint, all crates
```

## OpenAPI 文档（pairing-server）

```bash
cargo run -p dropvoice-pairing-server --bin gen-openapi   # Regenerate docs/openapi.{json,yaml}
```

- 文档由 utoipa 5（编译期）从 `#[utoipa::path]` / `#[derive(ToSchema)]` 注解生成。
- API 面变化时重新生成并提交 `docs/openapi.{json,yaml}`。
- 漂移检测：`tests/openapi.rs` 把再生成 spec 与已提交文件逐字节比对；在 `cargo test` 中运行，且 `apps/pairing-server/` 的任何变更触发 `openapi-drift` prek 钩子。
- Swagger UI 仅在 `ENABLE_DOCS=true` 时挂载于 `/docs/swagger-ui`（仅 env 的调试开关，不是 TOML 键；默认关，dev 任务与验收 compose 会设置）。

## 数据库迁移（pairing-server）

```bash
# Install sqlx-cli once:
cargo install sqlx-cli --no-default-features --features sqlite,rustls

sqlx migrate add -r <description>    # Create reversible migration (.up.sql + .down.sql)
sqlx migrate add <description>       # Create simple migration (.sql, default)
sqlx migrate info                    # Show migration status
sqlx database create                 # Create dev database
# Migrations apply automatically at server startup via sqlx::migrate!() - no manual run needed.
```

## 构建

```bash
pnpm build:tauri        # Production desktop installer (MSI/NSIS/dmg/deb)
pnpm ci:build           # Same as build:tauri (CI alias)
```

## 部署 pairing-server（Windows / 容器化 ansible）

Windows 无法原生跑 ansible（POSIX fork/sh 依赖），所以部署在一次性 Linux 容器（`docker run --rm`）内运行 ansible。runner 镜像是仅本地的工具——不推送——首次运行构建、之后缓存。`devops/deploy.py`（Python，跨平台）驱动这一切。`ansible.cfg` 在仓库根（inventory + ssh 调优），playbook 命令因此无需 `-i` 参数：`ansible-playbook apps/pairing-server/devops/ansible/deploy.yml -l <env>`。

环境（单一 playbook，差异全部收在 `group_vars/<env>.yml`）：

- **本地开发验收**——不部署。两种形态：
  - dev 循环：VS Code 任务 `dev:full` / `Alt+R`（= pairing-server `cargo run` → :7380 + desktop + mobile）；手机打开 `http://<dev-ip>:5174`（vite 把 `/api` 代理到 :7380）。全部配置由任务 env 注入，不手工配置。
  - 容器验收（针对工件）：`pnpm accept:up`（HTTP :8080，配置来自已提交的 `docker/config.acceptance.toml`）；desktop/mobile 用 `accept:desktop` / `accept:mobile` 任务。
- **staging** = fn @ 192.168.5.200——dogfooding；`*.dropvoice.bytehome.fun` 上与生产同构的三主机布局（`dropvoice.`=landing、`app.`=PWA+/api、`ps.`=API；泛解析 DNS + 隧道证书运维托管，TLS 由内网隧道终结）。Caddyfile 是 `templates/Caddyfile.j2` 的 HTTP 渲染。部署 LAN registry 的滚动 `main` 标签。翻转 domain/TLS/trusted-proxy 值即可原样搬进 Cloudflare。
- **production**——Cloudflare 之后的公网 VPS（dropvoice.online）：`dropvoice.online`=landing、`app.`=PWA+/api、`ps.`=纯 API。仅 Cloudflare 回源（`:4443`、Origin CA 证书、Origin Rule 端口改写、Full strict）；Caddyfile 是 `templates/Caddyfile.j2` 的 TLS 渲染。部署 Docker Hub 的固定版本标签（CI 把 release 同时发布到 ghcr.io 与 Docker Hub；生产拉公共 Docker Hub 镜像，VPS 上无需 registry 登录）。

镜像标签语义：

| 标签                   | Registry                    | 指向                                   | 由谁移动                             |
| ---------------------- | --------------------------- | -------------------------------------- | ------------------------------------ |
| `main`                 | 192.168.5.50:5000（仅 LAN） | 本地 workspace 最近构建的内容          | `docker/build.py --push`             |
| `0.1.3` / `0.1.3-rc.1` | ghcr.io + Docker Hub        | git tag `v0.1.3` / `v0.1.3-rc.1`       | CI `docker-publish.yml`（`v*` push） |
| `latest`               | ghcr.io + Docker Hub        | 仅最新**稳定** release（绝不是预发布） | CI `docker-publish.yml`              |

稳定门控：精确的 `^v\d+\.\d+\.\d+$` 才移动 `latest`。预发布用连字符 semver 形态（`v0.1.3-rc.1`，绝不写 `v0.1.3rc1`）。CI 多平台构建用原生 runner（amd64 + arm64，无 QEMU）；本地跨平台构建走 buildx + QEMU。`main` 绝不由 CI 构建。

构建 → 部署 → 验收流程（全部在开发机）：

```bash
# 1. Build image (single self-contained artifact: backend + Caddy + baked-in
#    mobile PWA + landing).
#    Defaults: LAN registry, host-machine platform, tags = main (+ --tags extras,
#    deduplicated). No registry cache by design (zero local gain, registry bloat).
pnpm image:push                                                       # -> LAN registry, tag: main
python apps/pairing-server/docker/build.py --push --all-platforms --tags v0.3.0  # cross-arch + immutable tag
pnpm build:tauri                                                      # desktop installer -> local install

# 2. Local container acceptance (against the artifact, before any deploy).
#    Plain HTTP :8080 — no TLS on the local box (this machine's WSL2 breaks
#    Go TLS servers; deployed forms terminate TLS externally anyway).
pnpm accept:up        # compose up (config: committed docker/config.acceptance.toml)
pnpm accept:test      # e2e-caddy (--ignored) + scripts/smoke.py http://localhost:8080
pnpm accept:down      # compose down -v (clean slate)

# 3. Deploy staging. The playbook renders config.toml + compose + Caddyfile from
#    group_vars, then runs scripts/smoke.py against public_url (health + PWA +
#    round trip + version match via /health APP_VERSION), so a green deploy
#    already passed the smoke gate.
pnpm deploy:staging                    # deploys `main` tag
pnpm deploy:staging -- --tag v0.3.0    # pin / rollback to immutable tag
python apps/pairing-server/devops/deploy.py staging --check --diff   # dry-run preview
```

生产晋级前的人工验收门：`apps/pairing-server/devops/runbooks/accept-staging.md`。

挂载：仓库根 → `/workspace`（playbook 修改实时生效，无需重建镜像；仓库根 ansible.cfg 在此工作目录生效）、`~/.ssh` → `/ssh`（entrypoint 拷贝密钥并为 Windows 权限 chmod 600）、`~/.ansible-vault` → `/vault`。见 `apps/pairing-server/devops/runner/` 与 `devops/deploy.py`。

前置（一次性）：`~/.ansible-vault/dropvoice-<env>.pwd` 的 vault 密码文件（如 `dropvoice-staging.pwd`）；CI 镜像发布所需 repo secrets `DOCKERHUB_USERNAME` / `DOCKERHUB_TOKEN`（ghcr.io 用内建 GITHUB_TOKEN）。

## 版本与发布

```bash
pnpm version:patch      # Bump patch + sync versions across packages + changelog
pnpm version:minor
pnpm version:major
# Release: push tag `v*` -> release.yml builds all platforms automatically,
# docker-publish.yml publishes the pairing-server image (ghcr.io + Docker Hub).
```

提交遵循 Conventional Commits（commitlint 强制）；scope：`(desktop)`、`(mobile)`、`(pairing-server)`、`(core)`、`(ui)`、`(i18n)`、`(ci)`、`(docs)`。
