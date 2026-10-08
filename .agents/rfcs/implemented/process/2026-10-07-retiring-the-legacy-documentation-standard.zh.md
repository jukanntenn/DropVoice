# RFC: 为 hdsh harness 退役旧文档标准

Status: implemented

[English](2026-10-07-retiring-the-legacy-documentation-standard.md) | 中文

## 问题

DropVoice 此前运行自己的文档治理：一个聚合了四个校验脚本（dv-rfcs 格式、文档配对、字数预算、Markdown 链接）的 `doc-check` prek 门、两个 manifest（`scripts/doc_budgets.manifest.json`、`scripts/doc_languages.manifest.json`）、一个带自有头部格式的 `.agents/dv-rfcs/` 决策记录树，以及两个仓库自有 skill（`doc-standards`、`writing-dv-rfcs`）来文档化这套机制。接入 [hdsh harness](2026-10-07-adopting-the-hdsh-harness.zh.md) 会安装第二套有主见的标准——配对记录、RFC 格式与归档门、docs wrap/links/budgets——而 hdsh adopt 不会与仓库自有门并行运行双标准。两套并行的门会让每次文档变更的成本翻倍，并恰恰以旧门要防止的方式互相漂移。

## 决策

按 hdsh 接入手册的退役政策，在同一变更中退役旧标准：

- 全部七篇 dv-rfcs 记录以完整双语三元组（`md` + `zh.md` + `.i18n.yaml`）迁入 `.agents/rfcs/`，归入 harness 的 class 目录（`process`、`architecture`、`bug-fix`）；`DV-RFC:` 标题前缀改为 `RFC:`，每篇记录补充配对契约要求的语言切换行。`.agents/dv-rfcs/` 树、其 README 对及其 AGENTS/CLAUDE 镜像删除。
- `scripts/doc_budgets.manifest.json` 中的字数上限折入 `.hdsh/docs.manifest.json` 的 `docBudgets`；各子树 `AGENTS.md` 预算原值平移，已退役的 `.agents/dv-rfcs/AGENTS.md` 预算删除。
- 双语语料转入配对契约之下：`docs/architecture.md`、`docs/development.md` 与全部迁移记录补上切换行并记录一致性 sidecar；中文优先的压测 README 完成翻转（`README.zh.md` 保留原字节）；根 README 与 icons README 补齐中文侧；作为再生成 agent 镜像的 `docs/CLAUDE.md` 在 `.hdsh/pairing.manifest.json` 中排除。
- `doc-check` 钩子、它调用的五个门脚本与两个 manifest 删除；`hdsh` prek 组接管文档门控。`package.json` 移除 `pnpm docs:check`。
- 删除 `doc-standards` 与 `writing-dv-rfcs` 两个 skill；[documenting](../../../skills/documenting/SKILL.md)、[editing-prose](../../../skills/editing-prose/SKILL.md) 与 [archiving-rfcs](../../../skills/archiving-rfcs/SKILL.md) 在新标准下接管相同工作流，[docs/AGENTS.md](../../../../docs/AGENTS.md) 依新标准重述层级分类。

## 备选方案

**双标准并行。** 旧语料继续用 `doc-check`，hdsh 门只管安装的语料。它输了：每篇在册文档要同时满足两套在细节上互相冲突的配对/预算/链接规则（切换行、sidecar 格式、wrap），且 hdsh 接入手册明确拒绝双标准。

**把旧门改造为输出 hdsh 兼容产物。** 重写四个校验器，让它们产出 `.i18n.yaml` sidecar 与 RFC 头格式。它输了：这等于 fork 了 hdsh 已发布的每一条规则——正是 harness 要消除的重复——并把仓库锁在上游修复之外。

**推迟退役，等接入稳定后再做。** 先接 harness，旧门以后再退。它输了：退役之前两套门会按对方的规范（wrap 规则、切换行要求、记录格式）互相报红，接入 PR 无法绿着落地。

## 后果

- 文档治理只有一个来源：`hdsh` prek 组加上 [docs/AGENTS.md](../../../../docs/AGENTS.md) 与[配对契约](../../../../docs/i18n/README.zh.md)。门的行为变更只能通过在新 ref 下重跑 `hdsh adopt apply` 或把变更引导至上游——绝不 fork 安装文件。
- 决策记录的 URL 发生变化：指向 `.agents/dv-rfcs/...` 的入链已在同一变更中改写；除两份 specs 的引用外，迁移按 `hdsh docs links` 是链接干净的。
- AGENTS.md 字数预算门现在是对 `.hdsh/docs.manifest.json` 运行的 `hdsh docs budgets`；上限值平移，因此没有任何文档需要缩减。
- 仓库本地 skill 不再文档化文档工作流；agent 遵循由 `scripts/sync_agent_files.py` 镜像进各工具目录的 harness skills。
