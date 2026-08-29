# DV-RFCs — DropVoice 决策记录

这里只放一种设计文档。**DV-RFC** 记录影响本代码库的决策或提案——_why_、_放弃了什么_，以及代码、specs 和 `AGENTS.md` 装不下的部分。它们是 agent 写的 RFC：持久的提案与决策记录，保存理由、备选方案与后果。

## 目录树

```
.agents/dv-rfcs/
├── AGENTS.md        standing orders for this tree
├── README.md        this contract (normative)
├── README.zh.md     Chinese twin of this contract
├── implemented/     decisions that shipped, kept current with what shipped
├── proposed/        proposals reviewed before implementation
└── rejected/        declines, kept only while their rationale blocks a tempting mistake
```

生命周期树即清单——浏览它或 grep 仓库。刻意**不设索引文件**：生成的索引是合并热点，不带来任何发现价值。

## 命名

`{lifecycle}/yyyy-mm-dd-topic-title.md`——日期是主题**首次提出**之日，以 git 历史为准（回填记录取主题在仓库中的首次出现日，而非记录撰写日）。文件名语法由门控强制：`^\d{4}-\d{2}-\d{2}-[a-z0-9][a-z0-9-]*(\.zh)?\.md$`。

## 头部

每份记录的前三行恰好是：

```markdown
# DV-RFC: <title>

Status: <status>
```

`Status:` 与所在目录一致：`proposed`、`implemented` 或 `rejected — <一行理由>`。状态行不带日期、不带括号补注——文件名承载首次提出日期，其余一切归 git。拒绝理由是唯一带内容的状态，因为被拒记录的裁决正是读者要来看的事实。

## 骨架

每个生命周期有固定章节骨架（`…` 表示自定义章节）：

```markdown
#### proposed/

## Problem

## Proposal

…

## Alternatives considered

## Acceptance criteria

## Risks

#### implemented/

## Problem

## Decision

…

## Alternatives considered

## Consequences

#### rejected/

(keeps whatever proposal-time sections it had, frozen; the verdict lives on the Status: line)
```

`## Problem` 开篇，且必须脱离方案自足。proposal 时期标题（`## Proposal`、`## Plan`、`## Migration plan`、`## Acceptance criteria`）在 `implemented/` 中被禁——格式门控直接拒绝。

## Alternatives considered 是强制的

每份记录都有 `## Alternatives considered` 章节：每个真实备选方案一个粗体引导的段落，写明它为何落选（争议大的可用 `### Why not <X>?` 小节）。备选方案按当时论证的原样记录，绝不通事后编造。没有记录手下败将的决策，必然被反复重审——这正是 DV-RFC 要防止的失败。

## 生命周期迁移

- `proposed → implemented`：把 `## Proposal` 改写为现在时的 `## Decision`，把 `## Acceptance criteria` 与 `## Risks` 并入 `## Consequences`，并逐项核对路径、名称与机制和实际交付一致。
- `proposed → rejected`：在 `Status:` 行加理由并冻结文件。
- 绝不把一份记录改写成另一个决策——取代它并互相交叉链接。

implemented 记录保持最新：后续代码移动文件、重命名包、更改 key/默认值时，同一改动中更新记录（只改事实，绝不改决策本身）。

## 双语对

每份记录 = `foo.md` + `foo.zh.md`：骨架相同、两种语言中标题与机器标记都用英文、同一次改动中一起更新。zh 孪生需要片段链接时用 ASCII 锚（`<a id="…">`），让链接在两种语言中都能解析。成对对等——标题序列一致、围栏代码块字节一致——由门控强制。

## 何时写一份

每个非平凡改动在同一改动中新增或更新至少一份记录。只有纯机械或局部、对行为/契约/结构/流程/理由毫无影响的编辑豁免。理由归这里；操作步骤归 `docs/`；现状事实归 `docs/architecture.md` 或 `AGENTS.md`；已批准的冻结设计在 `specs/`。

## 门控

`python scripts/verify_dv_rfc_format.py [files…]` 强制本契约（文件名语法、头部、Status-目录一致、骨架、强制 Alternatives、禁 proposal 语气、成对对等、流浪文件检测）。`python scripts/doc_sync.py` 把它与其他文档门控聚合；prek 在 Markdown 变更时运行两者。修文档而不是修门控——若门控本身错了，在同一改动中改掉，并在拥有该规则的记录里说明理由。
