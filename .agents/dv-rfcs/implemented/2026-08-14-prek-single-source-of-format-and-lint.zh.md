# DV-RFC: prek is the single source of format, lint, and test gating

Status: implemented

## Problem

DropVoice 由多个 AI coding agent（Claude Code、ZCode、Codex、OpenCode）与 CI 协同开发，横跨一个 pnpm workspace：四个 TypeScript 应用、三个 TS 包、两个 Rust crate。这些界面中的每一个都有自己的原生 hook/检查声明方式。若不为 format/lint/test 接线设一个唯一的声明之家，同一条规则就会按工具、按项目、在 CI 里各定义一遍——然后彼此漂移：不同工具的 hook 格式化结果不一致、子项目内联固定了偏差的依赖版本（整合之前就发生过 axum 0.7/0.8 分裂）、CI 成为违规唯一暴露的地方，而那已距离引入违规的编辑很久了。

## Decision

全部质量门控只在 **prek**（workspace 模式）中声明一次：根 `prek.toml` 加每个项目一个 `prek.toml`（`apps/desktop`、`apps/mobile`、`apps/landing`、`apps/pairing-server`、`packages/{core,i18n,ui}`）。prek 发现子配置并以该项目目录为 cwd 运行其钩子，项目相对的文件过滤因此保持诚实。

Hook **组**与 stage 正交：

- `format`——会改写的修复器（prettier、oxlint --fix、cargo fmt、内建空白/EOF 修复器），
- `lint`——只读门控（oxlint、tsc --noEmit、cargo clippy -D warnings），
- `check`——结构性检查、生成物漂移、lockfile 新鲜度；pre-push stage 跑测试套件。

每个入口调用同一套定义：AI post-edit hook 跑 `prek run --group format --files <edited>`，AI Stop hook 跑 `prek run --group lint --all-files`，commit 跑 pre-commit stage，push 跑 pre-push，CI（`.github/workflows/quality.yml`）跑 `prek run --all-files` 与 `prek run --stage pre-push --all-files` 两者。`pnpm quality` 镜像 CI。

四个工具目录（`.claude/`、`.zcode/`、`.codex/`、`.opencode/`）是不携带任何自身逻辑的一次性适配器；它们的 PostToolUse/Stop 钩子只做载荷格式翻译并调用 prek。生成物与锁文件（`theme.css`、`openapi.{json,yaml}`、`Cargo.lock`、`pnpm-lock.yaml`）豁免于所有修复器，改由漂移门控守护（`design-sync`、`openapi-drift`、`lockfiles-fresh`、`agents-sync`）。

## Alternatives considered

**各工具原生配置。** 每个 agent 工具接自己的 formatter/linter 钩子，CI 保留自己的 job。落选：一个事实每个工具多一个家，必然在工具间漂移——正是整合要消除的失败；只调用 prek 的适配器才是解法。

**lefthook / husky+lint-staged。** 成熟的 git hook 运行器。落选：两者都假设单一配置文件和以 Node 为中心的仓库；都无法像 prek 的多 `prek.toml` workspace 模式那样原生发现混合 cargo+pnpm workspace 里的各项目配置。markpost 刚在 2026-08-12 的记录里写下同样的整合；DropVoice 两天后采纳同一模式，并适配到带 Rust crate 的 workspace。

**只靠 CI 门控。** 一切定义在 GitHub Actions，本地无钩子。落选：违规晚一整个反馈回路才暴露，而主要编辑者是 AI agent——它们在本地编辑迭代，那里将什么检查都没有。

**不要聚合器——以各项目 package.json 脚本为源。** 落选：`pnpm lint`/`format` 脚本已存在，但那是给人的便捷包装；让它们成为定义会再造按项目漂移，且根级文件（workflow、scripts、文档）无人门控。

## Consequences

- 改任何门控行为 = 改某个 `prek.toml`——绝不改 AI 适配器、绝不改 CI。适配器按设计即弃；根 `AGENTS.md` 明文记录此规则。
- 生成物正确性由"重新生成再比对"的漂移门控强制，而非 formatter，生成器输出保持字节稳定。
- 提交绝不绕过（禁止 `--no-verify`）；conventional commits 由 commitlint 在 commit-msg stage 强制，同样经由 prek。
- 任何新检查先落 `prek.toml`（通常是根这份）；新工具集成只加薄适配器。agent-harness 记录引入的文档门控走的就是这条路径。
