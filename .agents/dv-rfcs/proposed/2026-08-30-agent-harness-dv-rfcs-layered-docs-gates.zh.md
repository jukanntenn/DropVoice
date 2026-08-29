# DV-RFC: Agent harness — dv-rfcs, layered agent instructions, docs tiers, local gates

Status: proposed

## Problem

DropVoice 主要由 AI coding agent 开发。底层管线已经存在且保持不动：prek 是 format/lint 的单一来源，四个工具目录（`.claude/`、`.zcode/`、`.codex/`、`.opencode/`）只是它上面的薄适配器。缺的是管线之上的全部，且缺口今天就看得见：

1. **决策理由没有持久的家。** 信令为什么经公网 pairing-server 中继而不是 LAN 发现、设备 token 为什么只存哈希、prek 为什么胜出 per-tool hook 逻辑——这些散落在 commit message、会话记录和根 `AGENTS.md` 的散文段落里。未来的改动从不被迫直面"这个决策当初赢过了什么"，于是每次重新讨论都从零开始。两个参考项目都点名了这个失败模式：没有记录手下败将的决策，必然被反复重审。
2. **根 `AGENTS.md` 把站立令和参考手册混在一起。** 3,438 词的它把 dev 任务、部署 runbook、配置表、配对流程走读和约定全部塞进每个会话——是两个参考项目验证过的可持续上限的 3–4 倍。子树专属指令（desktop 的 WebRTC 应答面、pairing-server 的 migration 规则）没有比根更近的家。
3. **Markdown 没有机械契约。** 相对链接悄悄烂掉、文档增长无人设限、skill 无声漂移：`.agents/skills/commit-old` 至今还在教 agent 用本仓库根本不存在的 Go 工具链（`go test`、`golangci-lint`）做验证——一份从 markpost 抄来的陈旧副本，因为没有东西检查它而活到今天。

两个参考仓库构成本提案的依据。deepseek-harness 是机制源头（agent notes、skills、分层 agent instructions、文档分层、TypeScript 门控套件）。markpost 成功适配了它，其决策历史记录了统治理由：**只在触发信号出现时移植局部，绝不整体照搬**——参考项目的规模是这条路的天花板，不是起点。

## Proposal

以四个组件引入该机制。所有新门控都是 `scripts/` 下的 stdlib-Python 脚本，经 prek（既有单一门控源）接线；CI 零工作流改动即继承。权威的文档分层表与 slop 清单将落在 `docs/AGENTS.md`；本记录固定其背后的决策。

### 1. dv-rfcs — decision records under `.agents/dv-rfcs/`

DV-RFC 记录影响本代码库的决策或提案：why、放弃了什么、后果。放置位置沿用参考项目：`.agents/` 是 agent 工具当作加载路径的目录。

- **树**：`implemented/`、`proposed/`、`rejected/`——扁平生命周期，无 class 子目录、无 archive。生命周期树即清单；刻意不设索引文件。
- **命名**：`{lifecycle}/yyyy-mm-dd-topic-title.md`。日期为主题按 git 历史首次提出之日；文件名正则 `^\d{4}-\d{2}-\d{2}-[a-z0-9][a-z0-9-]*(\.zh)?\.md$` 由门控强制。
- **格式**（门控强制）：首行 `# DV-RFC: <title>`，第三行 `Status: proposed | implemented | rejected — <一行理由>` 且与所在目录一致。正文以 `## Problem` 开头，问题陈述须脱离方案自足。`## Alternatives considered` 在每份记录中强制——每个真实替代方案一个粗体引导的段落，按当时论证的原样记录，绝不通事后编造。
- **骨架**：`proposed/` 含 `## Proposal`、`## Alternatives considered`、`## Acceptance criteria`、`## Risks`。`implemented/` 含 `## Decision`（现在时、实际交付了什么）、`## Alternatives considered`、`## Consequences`；proposal 语气标题在此被禁。`rejected/` 冻结 proposal 时期的章节；裁决写在 `Status:` 行上。
- **生命周期迁移是机械的**：`proposed → implemented` 把 Proposal 改写为现在时 Decision，并把 Acceptance criteria 与 Risks 并入 Consequences；`proposed → rejected` 只在 `Status:` 行加理由。绝不把一份记录改写成另一个决策——取代它并互相交叉链接。
- **双语对**：每份记录 = `foo.md` + `foo.zh.md`，骨架相同、标题与机器标记用英文、同一次改动中一起更新。结构对等（标题序列、围栏块字节一致）由门控强制。
- **回填**（2 份，`implemented/`，日期 = git 首次出现）：`2026-08-14-prek-single-source-of-format-and-lint` 与 `2026-08-14-webrtc-datachannel-public-pairing-server`。在途的 desktop Rust 信令迁移不回填——它落地时写自己的记录。
- **契约文件**：`.agents/dv-rfcs/AGENTS.md`（站立令，约 100 词）+ `README.md`/`README.zh.md`（规范契约：命名、生命周期、骨架、成对）。
- **延迟项（带复活触发器）**：class 分类层（记录 > 50 份）、冻结 archive + hash manifest（被取代记录积压时）、i18n.yaml hash 边车、postmortem 层（首次代价高昂的事故）。

本记录自身就是第一份 DV-RFC。它将在 rollout 结束时迁入 `implemented/` 并改写为 Decision + Consequences——规则约束自己的引入。

### 2. Skills — `.agents/skills/` is the source, `.claude/skills/` the mirror

- 事实源是工具中立路径 `.agents/skills/`（本仓库的主力工具不是 Claude Code）；`.claude/skills/` 是字节一致的镜像，与 AGENTS/CLAUDE 对一样由门控强制。
- 删除 `commit-old`，新写 `commit/` skill 对齐本仓库技术栈（pnpm + cargo + prek 验证、dropvoice scope、绝不 `--no-verify`）；现行那份是引用 Go 工具链的 markpost 陈旧副本。
- 新增 `writing-dv-rfcs/`（RFC 工作流：查重、选生命周期、骨架、双语成对、`python scripts/doc_sync.py <files>` 验证）与精简版 `doc-standards/`（文档放置分层 + 门控失败应对）。
- 保留 `grill-me`、`iterating`、`tauri-icon-generation` 不变。

### 3. Layered AGENTS.md with word budgets

根文件从 3,438 词压缩到约 1,000 词站立令——每条规则 1–3 行并链接其家。子树文件承载子树专属指令，绝不重复根。每个 `AGENTS.md` 配字节一致的 `CLAUDE.md` 孪生，方向固定（AGENTS.md 是源，fix = `cp`），把既有 `agents-sync` 门控扩展到全部对。预算由门控强制（`verify_doc_budgets.py` + `doc_budgets.manifest.json`；manifest 条目对应的文件消失即失败，重命名无法遗孤预算）：

| 文件                            | 上限（词） |
| ------------------------------- | ---------- |
| 根 `AGENTS.md`                  | 1000       |
| `apps/desktop/AGENTS.md`        | 500        |
| `apps/mobile/AGENTS.md`         | 300        |
| `apps/pairing-server/AGENTS.md` | 450        |
| `apps/landing/AGENTS.md`        | 150        |
| `packages/AGENTS.md`            | 250        |
| `docs/AGENTS.md`                | 600        |
| `.agents/dv-rfcs/AGENTS.md`     | 150        |

预算报红时：迁到归属层 → 压缩 → 最后才在同一改动中以带理由的 manifest diff 上调上限。

### 4. docs/ — tiered prose, bilingual pairs

新建根 `docs/` 承接离开根 `AGENTS.md` 的参考材料，事实零丢失——每条迁出的事实都从旧家留有链接：

- `docs/development.md` + `.zh.md` — 完整命令/环境细节（dev 任务、doctor、quality 门控、migrations）。
- `docs/architecture.md` + `.zh.md` — 系统组成：配对流程、信令监督、连接态事件总线、端口、配置单一源大表。
- `docs/AGENTS.md` — 文档标准本体：分层表（一个事实一个家）、每条规则标明其门控、slop 清单。
- 部署内容留在既有之家（`apps/pairing-server/devops/` + runbooks）——分层表只归类不搬迁。`specs/`（只读）与 `PRINCIPLES.md` 仅归类，不动。

双语范围（已定）：DV-RFC 与 `docs/` 正文成对。豁免：所有 `AGENTS.md`/`CLAUDE.md`、skills、生成物（openapi）、`specs/`、`README.md`。

### 5. Gates — stdlib Python behind prek

`scripts/doc_sync.py` 聚合，接受文件参数（本地经 prek 取 staged 文件）或全量运行（CI 经 `prek run --all-files`）：

1. `verify_dv_rfc_format.py` — 文件名正则、标题、Status-目录一致、`## Problem` 开头、强制 Alternatives、各生命周期骨架、`implemented/` 禁 proposal 语气、`.zh.md` 须有原件且结构对等、流浪文件检测。
2. `verify_doc_pairs.py` — 双向成对完整性、标题序列对等、围栏块字节一致，按成对范围与豁免 manifest 运行。
3. `verify_doc_budgets.py` — 上表。
4. `verify_md_links.py` — 相对 markdown 链接可解析。

prek 接线（根 `prek.toml`）：新增 `doc-check` 钩子（check 组，`\.md$`，豁免生成物与四个工具适配器目录——绝不豁免 `.agents/dv-rfcs/`）；`agents-sync` 扩展到全部 AGENTS↔CLAUDE 对加 skills 镜像；fixer 脚本重建孪生与镜像。prettier/whitespace 排除从 `^(\.agents|\.claude|\.codex|\.opencode|\.zcode)/` 收窄到仅四个工具目录——整体排除 `.agents/` 会让门控对真实内容静默变绿（markpost 记录在案的 enforcement-parity 教训）。加 `pnpm docs:check` / `docs:fix` 别名。

### 6. Rollout — three batches, harness-only staging

1. **dv-rfcs 机制**：树 + 契约文件 + 格式门控 + `doc_sync` + prek 接线 + 排除收窄 + `writing-dv-rfcs` skill + 两份回填。
2. **分层与 docs**：子树 AGENTS.md + docs 成对 + 根压缩 + `sync-agents` 扩展 + 预算/链接/成对门控。
3. **收尾**：`commit` skill 重写、`doc-standards` skill、删 `commit-old`、重建镜像、`prek run --all-files` + `doc_sync` 全绿、本记录迁入 `implemented/`。

每批次独立 conventional commit（`docs`/`process` scope），只 stage harness 文件；工作区约 50 个在途产品文件绝不进入 harness 提交。

## Alternatives considered

**整体照搬 deepseek-harness。** 它是被验证过的源头，但其规模（600+ 记录、40+ 子系统页、约 5,300 行门控 TypeScript）是这条路的天花板而非起点。落选原因：DropVoice 从空记录树和寥寥数页文档起步；class 层、冻结 archive、i18n 三元组、文档站投影会给每次编辑加税而在此规模下毫无收益。局部按触发信号移植，每一项都记录了复活条件。

**参考仓库的 TypeScript 门控套件与调度器。** 依赖图调度器跑约 25 个门控服务于 DropVoice 没有的规模，且仓库约定是跨平台 stdlib Python（`scripts/doctor.py`、`smoke.py`、既有 drift 门控）。落选原因：四五个串行门控跑一个小型 Markdown 语料库秒级完成，零新增工具链依赖。

**CLAUDE.md 用 symlink 镜像。** 构造上零漂移。落选原因：Windows 下没有 `core.symlinks=true` 的 checkout 会把 `CLAUDE.md` 物化成只含目标路径九个字节的普通文件——第一次 Windows clone 就把漂移风险换成坏镜像。字节一致副本加门控是本仓库已在运转的机制。

**Direction-free 镜像（markpost 的 git-HEAD 机制）。** 用 git HEAD 判定镜像方向，更优雅地处理"两边都被编辑"。落选原因：DropVoice 已有运转中的固定方向门控（`agents-sync`，文档写明 AGENTS.md 是源），还没遇到它解决的问题；调和机制在被真实双改冲突咬到之前不移植。

**Skills 源放 `.claude/skills/`（markpost 的方向）。** markpost 以 Claude 路径为主因为主力工具是 Claude Code。在本仓库落选：DropVoice 四种 agent 并用、ZCode 为主；工具中立的 `.agents/` 路径才是诚实的事实源，`.claude/skills/` 作门控镜像。

**开局即上 class 子目录与冻结 archive。** 六个 class 在 600 份记录时便于浏览。落选原因：空树下它们只是空目录；扁平生命周期树就是清单。复活触发器已在上方记录。

**AGENTS.md 渐进迁移（先立契约、后迁内容）。** 风险更低，但它留下一个超出自身预算 3 倍的根文件作为例外的活标本。落选原因：内容已存在且是最新的——迁移是带链接的搬运而非新写——markpost 的先例表明一次性规范化是有界的、可 review 的改动。决定：一步到位。

**Agent instructions 与 skills 也双语。** 一致性上"全部成对"最有说服力。落选原因：agent instructions 是机器消费的站立令；翻倍会给未来每次规则编辑加税而没有读者。已定范围：DV-RFC 与 docs 正文成对；instructions、skills、specs、README 保持单语。

**集中式 RFC 索引。** 落选原因：生命周期树就是清单，生成的索引是可预期的合并热点，浏览或搜索之外几乎不带来发现价值——源项目的记录理由，被 markpost 证实。

**什么都不做。** 现状在可证明地漂移：一份教 agent 用 Go 验证的 commit skill 在 Rust/pnpm 仓库里活了几个月，因为没有东西检查它。散文约定熬不过 agent 换代；强制门控可以。

## Acceptance criteria

1. `python scripts/doc_sync.py` 全量语料退出码 0；`prek run --all-files` 与 `prek run --stage pre-push --all-files` 退出码 0。
2. `verify_dv_rfc_format.py` 对包含本记录（已迁入 `implemented/`、改写为 Decision + Consequences）与两份回填的树通过——每份都是双语对。
3. 根 `AGENTS.md` ≤ 1,000 词；每个分层文件不超其预算；无 manifest 孤儿条目。
4. 从根 `AGENTS.md` 迁出的每条事实都能从根或子树文件经链接到达；`verify_md_links.py` 证明目标可解析。
5. `.claude/skills/` 与 `.agents/skills/` 字节一致；`commit-old` 消失；重写后的 `commit` skill 只引用本仓库存在的命令。
6. GitHub workflow 文件零改动；无在途工作区文件进入 harness 提交。

## Risks

**压缩根 AGENTS.md 时丢失事实。** 缓解：只做搬运，rollout 期间逐节 review；链接门控证明每个目标存在；reviewer 抽查每个被移除的章节都有带链接的新家。

**门控对存量 Markdown 误报。** 现有语料从未被这些规则约束过。缓解：每个门控接入 prek 前先全量跑一遍；修文档而不是修门控——若门控本身错了，在同一改动中改掉并在此说明理由。

**双语维护税。** 此后每份 RFC 和 docs 页面都以两种语言编辑。缓解：对等性是结构性的（标题序列、围栏块），范围排除了最高频变动的文件（instructions、skills），全库成对保持为延迟触发项。

**预算上限定错。** 拍脑袋的上限可能不合文件的真实需要。缓解：带理由上调的规则让调整便宜且可见；上限是护栏不是缩减指标。

**与在途工作冲突。** 工作区有约 50 个在途产品文件（desktop Rust 信令迁移）。缓解：harness 提交只 stage harness 文件；批次独立于产品改动。

**对小团队过度工程。** 每个延迟层都带显式复活触发器，交付的一切都被既有流程（commit、push、CI）持续使用，机制不会像 `commit-old` 那样无声腐烂。
