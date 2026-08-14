# 配对服务器测试补全规范

> 版本: 1.0.0
> 最后更新: 2026-07-25
> 状态: 待批准

> **背景**：对 `apps/pairing-server`（spec 11 落地）的测试审计发现，单元/e2e/压测
> 三层覆盖均未达到 spec 11 §13 的要求，且现有测试不足以暴露若干已存在的实现缺陷。
> 本规范定义**完整的补全计划**，使配对服务器的测试覆盖满足 spec 11 §13 全部场景、
> edge cases 与异常状态，并满足 §13.3 "合并门槛"。

---

## 1. 概述

### 1.1 目标

| # | 目标 | 验收口径 |
|---|------|---------|
| G1 | 单元测试覆盖全部纯逻辑模块与仓储层 | §4 用例表逐项落地，行覆盖率 ≥ 85% |
| G2 | e2e 测试覆盖 spec 11 §13.2 ⑥ 场景全集 + 走**同构 Caddy 通道** | §5 用例表逐项落地 |
| G3 | 压测覆盖 spec 11 §13.3 ④ 场景全集，附容量验证报告 | §6 四个脚本 + 一份报告 |
| G4 | edge cases 与异常状态全覆盖 | §7 用例全集逐项落地 |
| G5 | 暴露并修复审计发现的实现缺陷 | §3 先决条件逐项修复 |

### 1.2 范围

- **本规范只约束 `apps/pairing-server`**（A 线配对服务器）的测试。
- **不涉及**：桌面端本地服务器（B 线）、移动端 PWA、UDP 广播、WebSocket 协议——这些
  的测试归属各自规范（桌面端见 spec 11 §6.1/§5.2 的实现说明，移动端见 spec 06）。
- **测试通用策略**（金字塔分层、Mock 策略、命名、覆盖率门控）引用 spec 06，本规范
  **不重复定义**，仅补充配对服务器特有的部分。

### 1.3 与其他规范的关系

| 规范 | 关系 | 本规范如何引用 |
|------|------|--------------|
| spec 06 测试策略 | **上位规范**，本规范是其下属 | 分层/Mock/覆盖率门控遵循 06，配对服务器特有用例在本文 |
| spec 11 连接系统 | **被测对象**的权威定义 | 所有用例的"期望行为"以 spec 11 为唯一事实源 |
| spec 13 安全 | 安全用例的威胁模型来源 | §7 安全相关 edge case 引用 spec 13 §device_id 公开 caveat |
| spec 16 可观测性 | L3 指标/日志契约 | §4 指标测试引用 spec 16 §2.4 命名规范 |
| spec 19 CI/CD | CI 闸门接入点 | §8 定义 pairing-server job，挂入 spec 19 流水线 |
| `openapi/pairing-server.yaml` | API 契约权威源 | 所有 e2e 断言的响应形状以 YAML 为准 |

### 1.4 测试金字塔（配对服务器特化）

配对服务器是纯 HTTP 服务（无 UI、无 Tauri IPC），金字塔与 spec 06 §2.1 通用金字塔
略有差异——**无组件测试层**，三层即可：

```mermaid
pyramid
    title 配对服务器测试金字塔
    "e2e（同构 Caddy + reqwest）" : 10
    "集成测试（axum 内存 + 临时 SQLite）" : 35
    "单元测试（纯逻辑 + 仓储，不启动 HTTP）" : 55
```

| 层级 | 工具 | 边界 | 不 Mock 什么 | 覆盖目标 |
|------|------|------|-------------|---------|
| L1 单元 | `cargo test`（`#[cfg(test)]`） | 时钟、RNG | 被测函数内部逻辑 | 85%+ |
| L2 集成 | `cargo test`（`tests/`） | — | 真实 axum + 真实 SQLite（临时文件） | 关键路径 |
| L3 e2e | reqwest + docker-compose + Caddy | — | 真实 Caddy 反代 + 真实 TLS + 真实 gzip | spec 11 §13.2 ⑥ |

> **与 spec 06 §2.2 的映射**：本规范的 L1=spec06 L1（Rust 单元），L2≈spec06 L3
> （集成，但不经 Caddy），L3=spec06 L4（E2E，不 Mock）。spec 06 的 L2 组件测试层
> 在配对服务器不存在（无 React 组件）。

### 1.5 术语

| 术语 | 定义 |
|------|------|
| 同构 Caddy 通道 | 通过 `docker/docker-compose.local.yml` 启动 Caddy:4443 + axum:38424，e2e 客户端打 `https://localhost:4443`（自签，忽略校验），与生产拓扑同构 |
| 时钟注入 | 将 `Utc::now()` 替换为可注入的 `Clock` 抽象，使"过期/续期"逻辑可确定性测试 |
| 用例 ID | `UT-<模块>-<序号>`（单元）/ `IT-<序号>`（集成）/ `E2E-<序号>`（e2e）/ `LT-<场景>-<序号>`（压测）/ `EC-<分类>-<序号>`（edge case） |

---

## 2. 测试矩阵总表

> 本表是 spec 11 §13.2 ⑥ 场景 + §13.3 ④ 场景的**全集到用例的反向追踪矩阵**。
> 每一行必须可追溯到 spec 11 原文条目。实现者据此核对无遗漏。

### 2.1 spec 11 §13.2 e2e ⑥ 场景 → 用例

| spec 11 §13.2 场景 | spec 11 出处 | 用例 ID | 当前状态 | 章节 |
|-------------------|-------------|---------|---------|------|
| ① 设备注册 upsert（201/200/续期） | §5.1.1 | E2E-01, E2E-02, **E2E-03（续期）** | 01/02 有, 03 缺 | §5.2 |
| ② 配对码生成+查询+过期 | §5.1, §6.2 | E2E-04, E2E-05, **E2E-06（过期 410）**, **E2E-07（used 410）** | 04/05 有, 06/07 缺 | §5.2 |
| ③ 状态上报+IP 变化 | §7.4 | E2E-08, **E2E-09（IP 变化刷新断言）**, **E2E-10（device_name 改名）** | 08 有, 09/10 缺 | §5.2 |
| ④ token 过期续期 | §7.5 | **E2E-11（401→重 POST→新 token→重放心跳）** | 缺 | §5.2 |
| ⑤ 错误响应格式 | §5.1.3 | **E2E-12 ~ E2E-17（6 个错误码体形状）** | 仅 PAIRING_CODE_NOT_FOUND 部分覆盖 | §5.2 |
| ⑥ 速率限制 | §14.1, §4 `RATE_LIMIT` | **E2E-18, E2E-19** | 缺（且限流未接入） | §5.2 |

### 2.2 spec 11 §13.3 压测 ④ 场景 → 脚本

| spec 11 §13.3 场景 | spec 11 出处 | 脚本 | 当前状态 | 章节 |
|-------------------|-------------|------|---------|------|
| ① 注册洪流（10 万设备并发） | §13.3 场景① | `register.js`（**重写**） | 存在但设备数不足 | §6.2 |
| ② 心跳稳态（1014 QPS 持续） | §13.3 场景② | **`heartbeat.js`（新增）** | 缺 | §6.3 |
| ③ 配对码读写混合 | §13.3 场景③ | **`pairing_code.js`（新增）** | 缺 | §6.4 |
| ④ SQLite 锁竞争（并发写入） | §13.3 场景④ | **`sqlite_contention.js`（新增）** | 缺 | §6.5 |

### 2.3 spec 11 §13.1 同构要求 → 通道

| spec 11 §13.1 要素 | 实现方式 | 章节 |
|--------------------|---------|------|
| 真实 axum + 真实 SQLite | docker-compose 起 pairing-server 容器 | §8.3 |
| 真实 Caddy 反代 | docker-compose 起 caddy:2.11 + `Caddyfile.test` | §8.3 |
| 真实 gzip | `Caddyfile.test` 的 `encode gzip` | §8.3 |
| 自签 TLS（唯一允许的差异） | `tls internal` | §8.3 |
| 客户端忽略证书校验 | reqwest `danger_accept_invalid_certs(true)` / k6 `insecureSkipTLSVerify` | §8.3 |

### 2.4 覆盖率目标（引用 spec 06 §4.2，配对服务器细化）

| 模块 | 行覆盖目标 | 行覆盖硬门控（CI） |
|------|----------|------------------|
| `domain/*` | 95%+ | 90% |
| `store/*` | 90%+ | 80% |
| `cache/*` | 90%+ | 80% |
| `batch/*` | 85%+ | 75% |
| `api/auth.rs`（含 RateLimiter） | 90%+ | 80% |
| `api/error.rs` | 90%+ | 80% |
| `api/devices.rs`, `api/pairing_codes.rs` | 80%+ | 70% |
| `config.rs`, `observability.rs` | 80%+ | 70% |
| **整体（`src/`）** | **85%+** | **80%** |

> 覆盖率工具：`cargo-tarpaulin`（Linux）或 `cargo-llvm-cov`（跨平台）。CI 用
> `cargo-llvm-cov --workspace --fail-under-lines 80`（§8.5）。

---

## 3. 可测性先决条件（Pre-req）

> **审计发现的实现缺陷**——这些必须**先修复**，否则对应用例根本无法编写（或写了也是
> 测死代码）。每个 Pre-req 标注其阻塞的用例 ID。实现者按 §3.1 → §3.8 顺序修复。

### 3.1 Pre-req-1：引入时钟注入（阻塞 E2E-03/06/07/11, UT-device_repo-03/04, UT-pairing_code_repo-02/03）

**问题**：`device_repo::upsert`（token 过期 refresh 判定）、`pairing_code_repo::lookup`
（过期 410）、`device_repo::mark_stale_offline`（离线阈值）均直接调用 `Utc::now()`，
无法确定性测试"过期"语义。

**修复**：

1. 新增 `src/clock.rs`：

```rust
//! 时钟抽象（可测性）。生产用 SystemClock，测试用 FakeClock。

use chrono::{DateTime, Utc};

pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

/// 生产时钟，包装 chrono::Utc::now()。
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// 测试时钟，时间可控。
#[derive(Debug, Clone)]
pub struct FakeClock {
    inner: std::sync::Arc<std::sync::Mutex<DateTime<Utc>>>,
}

impl FakeClock {
    pub fn new(initial: DateTime<Utc>) -> Self {
        Self { inner: std::sync::Arc::new(std::sync::Mutex::new(initial)) }
    }
    pub fn advance(&self, dur: chrono::Duration) {
        let mut g = self.inner.lock().unwrap();
        *g = *g + dur;
    }
    pub fn set(&self, t: DateTime<Utc>) {
        *self.inner.lock().unwrap() = t;
    }
}

impl Clock for FakeClock {
    fn now(&self) -> DateTime<Utc> {
        *self.inner.lock().unwrap()
    }
}
```

2. `AppState` 增加 `pub clock: Arc<dyn Clock>` 字段。
3. 所有 handler / repo 函数的 `Utc::now()` 替换为 `clock.now()`。repo 函数签名增加
   `clock: &dyn Clock` 参数（或从 `AppState` 取）。
4. **`main.rs` 用 `SystemClock`，测试用 `FakeClock`**。`main.rs` 不变行为。

**验收**：`grep -rn "Utc::now()" apps/pairing-server/src/` 仅剩 `clock.rs::SystemClock`
内一处，其余全部走 `clock.now()`。

### 3.2 Pre-req-2：接入速率限制（阻塞 E2E-18/19, UT-RateLimiter-*）

**问题**：`RateLimiter` 定义于 `api/auth.rs`，但**从未接入路由或 AppState**。
`RATE_LIMIT_PER_SEC=1`（spec 11 §4）形同虚设，违反 spec 11 §14.1。

**修复**：

1. `AppState` 增加 `pub rate_limiter: RateLimiter` 字段。
2. 实现一个 axum 中间件层（`tower::Layer`）或在每个 handler 入口检查；推荐中间件：

```rust
// src/api/rate_limit.rs（新增）
use axum::extract::State;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use crate::api::auth::{extract_client_ip, RateLimiter};
use crate::api::state::AppState;
use crate::error::AppError;

pub async fn rate_limit_layer<B>(
    State(state): State<AppState>,
    req: Request<B>,
    next: Next,
) -> Result<Response, AppError> {
    let ip = extract_client_ip(req.parts());
    if !state.rate_limiter.check(&ip, state.rate_limit_per_sec).await {
        return Err(AppError::RateLimited);
    }
    Ok(next.run(req).await)
}
```

   注：从 `ConnectInfo<SocketAddr>` 提取需要 `axum::serve` 时 `.into_make_service_with_connect_info::<SocketAddr>()`，`serve` 函数需相应更新。`extract_client_ip` 已存在（`auth.rs:88`），复用。

3. 在 `build_router` 中 `.layer(from_fn_with_state(state.clone(), rate_limit_layer))`。
4. `RATE_LIMIT_PER_SEC` 默认 1（spec 11 §4）。**测试环境调高**（避免 e2e 串行请求误触限流）：通过 `Config.rate_limit_per_sec` 注入，e2e 夹具设为 `1000`。

**验收**：连续 2 次 POST /devices（同 IP）第二次返回 429 `RATE_LIMITED`；e2e 夹具用
高限流值避免污染。

### 3.3 Pre-req-3：修复 token 缓存陈旧（阻塞 E2E-11, UT-cache-04, 安全相关）

**问题**：`devices.rs:39-42` 在复用/续期时只 `put_token(new)`，**不 invalidate 旧
token**。续期后旧 token 仍在缓存，可继续认证——违反 spec 11 §14.1 token 过期语义。

**修复**：

1. `upsert` 的 refresh 分支返回**旧 token**（修改 `RegisterOutcome::Reused` 携带
   `(Device, Option<String: old_token>)`，或在 handler 内查一次旧 token）。
2. handler 在 refresh 分支调用 `state.cache.invalidate_token(&old_token).await`。
3. **同时为 token 缓存增加 TTL**：`CachedToken { device_id, inserted: Instant }`，
   `get_token` 时若 `inserted` 超过 `DEVICE_TOKEN_TTL`（24h）视为过期返回 None。

**验收**：续期后用旧 token 调 `PUT /status` 立即返回 401。

### 3.4 Pre-req-4：修复并发注册竞态（阻塞 EC-concurrency-01, E2E-20）

**问题**：`device_repo::upsert` 先 SELECT 再 INSERT，无 `ON CONFLICT`。两个并发请求
同 `device_id` 会都走"不存在"分支，第二个 INSERT 撞 `PRIMARY KEY(id)` → 500。
违反 spec 11 §5.1.1 幂等 upsert。

**修复**：将"新建"分支改为单条 `INSERT ... ON CONFLICT(id) DO NOTHING` + 检查
`rows_affected`：为 0 说明并发已有，回退到复用分支。

```sql
INSERT INTO devices (...) VALUES (...)
ON CONFLICT(id) DO NOTHING;
-- 若 rows_affected == 0，重新 SELECT 走复用/续期分支。
```

**验收**：100 个并发同 `device_id` 的 POST，全部返回 201/200，无 500。

### 3.5 Pre-req-5：修复配对码生成碰撞 + 服务端重试（阻塞 EC-concurrency-02）

**问题**：`generate_pairing_code` 是 6 位随机（10^6 空间），`pairing_code_repo::create`
的 INSERT 撞 `PRIMARY KEY(code)` → 500。无服务端 `PAIRING_CODE_RETRY`（spec 11 §4）。

**修复**：`pairing_codes::create` handler 内循环调用 `generate_pairing_code` +
`pairing_code_repo::create`，捕获 UNIQUE 冲突重试，最多 `PAIRING_CODE_RETRY`（3）次，
耗尽返回 500（内部错误，打日志）。

**验收**：构造已占用 code（直接插表），handler 能重试到未占用 code 返回 201。

### 3.6 Pre-req-6：修复配对码 used 标记（阻塞 E2E-07, UT-pairing_code_repo-03）

**问题**：`pairing_code_repo::lookup` 查询后**从不置 `used=1`**。"已使用→410"分支
（spec 11 §5.1.3 `PAIRING_CODE_EXPIRED`）恒不可达；`used` 字段成摆设。

**决策**：spec 11 §2.2 `PairingCode.used` 字段 + §5.1.3 "已使用"语义要求**一次性**。
`lookup` 成功返回后应 `UPDATE pairing_codes SET used = 1 WHERE code = ?`。

**修复**：在 `pairing_code_repo::lookup` 成功路径末尾，事务内 `UPDATE ... SET used=1`。
或新增 `mark_used(code)` 显式调用，handler 在 lookup 成功后调用。

> **设计选择**：一次性语义意味着**第二次查询同一码 → 410**。本规范采用此语义（与
> spec 11 §5.1.3 "已使用"一致）。若产品改为"可多次查询直到自然过期"，需先改 spec 11，
> 再删 used 分支。**本规范默认一次性**。

**验收**：同一码查询两次，第二次返回 410 `PAIRING_CODE_EXPIRED`。

### 3.7 Pre-req-7：修复状态上报 device_name 丢失（阻塞 E2E-10, UT-batch-04）

**问题**：`devices.rs:72` `report_status` 走 `batch.enqueue`，而 `BatchWriter::enqueue`
（`batch/mod.rs:74`）**只传 ip/port/ts，丢弃 device_name**。`device_repo::update_status`
（带 device_name 的 COALESCE 更新）成死代码。违反 spec 11 §7.4 "device_name 变化触发上报"。

**修复**：

1. `PendingStatusUpdate` 增加 `device_name: Option<String>` 字段。
2. `BatchWriter::enqueue` 签名增加 `device_name: Option<&str>`。
3. `flush` 的 UPDATE 增加 `device_name = COALESCE(?, device_name)`。
4. **删除 `device_repo::update_status` 死代码**（或保留并加 `#[cfg(test)]` 仓储测试，
   但生产路径统一走 batch）。

**验收**：上报带 `device_name`，flush 后 `current_address` / 新查询能读到新 name。

### 3.8 Pre-req-8：修复配对码缓存 TTL 错误（阻塞 E2E-06, UT-cache-02）

**问题**：`pairing_codes.rs:62` 缓存过期时间用 `Utc::now() + PAIRING_CODE_TTL`，
而非 DB 行的真实 `expires_at`。若码已临近过期，缓存会用一个错误的未来时间，
导致码实际过期后缓存仍命中返回（陈旧数据）。违反 spec 11 §10.1 缓存策略。

**修复**：`pairing_code_repo::lookup` 返回值携带真实 `expires_at`（响应体已隐含，
但需单独暴露给 cache 层）。`put_pairing_code` 用真实 `expires_at`。

**验收**：插入一个 `expires_at = now + 1s` 的码，查询命中缓存，1s 后再查返回 410
（缓存过期，回查 DB 也过期）。

### 3.9 Pre-req-9：修复 OpenAPI 响应形状不一致（阻塞 E2E-04 断言修正）

**问题**：`domain/pairing_code.rs:33-41` 返回**扁平** `device.ip`/`device.port`，
但 OpenAPI `PairingCodeLookupResponse.device.address.{ip,port}` 是**嵌套**。
现有 e2e `api.rs:166-167` 反而固化了错误形状。

**决策**：以 **OpenAPI 为准**（spec 11 §5 "OpenAPI YAML 权威源"）。修改领域模型：

```rust
pub struct LookupDevice {
    pub id: Uuid,
    pub platform: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_name: Option<String>,
    pub address: DeviceAddress,  // 嵌套，与 OpenAPI 一致
}
```

`pairing_code_repo::lookup` 与缓存夹具相应调整。现有 e2e 断言改为 `body["device"]["address"]["ip"]`。

**验收**：`GET /pairing-codes/{code}` 响应 JSON 形状与 OpenAPI example 逐字段一致。

### 3.10 Pre-req 总览与阻塞关系

> **Note (2026-07-25):** All prerequisites (1-9) have been fulfilled in the current codebase.

| Pre-req | 阻塞用例 | 影响章节 | 状态 |
|---------|---------|---------|------|
| 1 时钟注入 | E2E-03/06/07/11, UT 多个 | §4/§5 | implemented |
| 2 限流接入 | E2E-18/19, UT-RateLimiter | §4/§5 | implemented |
| 3 token 缓存陈旧 | E2E-11, UT-cache-04 | §4/§5 | implemented |
| 4 并发注册竞态 | EC-concurrency-01, E2E-20 | §5/§7 | implemented |
| 5 配对码碰撞 | EC-concurrency-02 | §7 | implemented |
| 6 used 标记 | E2E-07, UT-pairing_code_repo-03 | §4/§5 | implemented |
| 7 device_name 丢失 | E2E-10, UT-batch-04 | §4/§5 | implemented |
| 8 缓存 TTL | E2E-06, UT-cache-02 | §4/§5 | implemented |
| 9 OpenAPI 形状 | E2E-04 断言 | §5 | implemented |

---

## 4. 单元测试规范（L1）

> 共置 `#[cfg(test)] mod tests`，引用 spec 06 §2.5。每个用例**独立**，不依赖执行顺序。
> 时间相关用例用 `FakeClock`（§3.1）；DB 相关用临时 SQLite（`tempfile`）。

### 4.1 通用夹具（`src/test_support.rs`，`#[cfg(test)]`）

> 新增测试支持模块，集中夹具，避免各模块重复。

```rust
//! 测试夹具（仅 #[cfg(test)]）。
use sqlx::SqlitePool;
use tempfile::TempDir;
use crate::clock::FakeClock;
use crate::store;

pub struct TestDb {
    pub pool: SqlitePool,
    pub _tmp: TempDir,  // 保活
}

impl TestDb {
    pub async fn new() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let url = format!("sqlite://{}?mode=rwc", tmp.path().join("t.db").display());
        let pool = store::open_pool(&url).await.unwrap();
        Self { pool, _tmp: tmp }
    }
}

pub fn fixed_clock() -> FakeClock {
    // 固定基准时间 2026-07-25T00:00:00Z，保证可重复。
    FakeClock::new(chrono::DateTime::parse_from_rfc3339("2026-07-25T00:00:00Z").unwrap().with_timezone(&chrono::Utc))
}

pub fn sample_device_id() -> uuid::Uuid {
    // 固定 UUID，保证测试可重复。
    uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
}
```

### 4.2 `config.rs` 单元测试

| 用例 ID | 名称 | 输入 | 期望 | 现有 |
|--------|------|------|------|------|
| UT-config-01 | 时间常量匹配 spec | — | `PAIRING_CODE_TTL==300s`, `DEVICE_TOKEN_TTL==24h`, `PAIRING_CODE_LEN==6`, `PAIRING_TOKEN_LEN==64`, `RATE_LIMIT_PER_SEC==1`, `SQLITE_BUSY_TIMEOUT_MS==50`, `CACHE_CLEANUP_INTERVAL_SECS==60`, `BATCH_WRITE_INTERVAL_SECS==1` | 部分（补全） |
| UT-config-02 | 默认监听地址回环 38424 | 默认 Config | `listen_addr=="127.0.0.1:38424"` | 有 |
| UT-config-03 | 环境变量注入 | `LISTEN_ADDR`/`DATABASE_URL` env | Config 字段被覆盖 | 缺 |
| UT-config-04 | `Config.rate_limit_per_sec` 默认值 | 默认 | ==`RATE_LIMIT_PER_SEC`(1) | 缺 |

### 4.3 `domain/*` 单元测试

| 用例 ID | 名称 | 输入 | 期望 |
|--------|------|------|------|
| UT-domain-01 | `DeviceAddress` serde 往返 | `{ip,port}` | 反序列化+序列化对称 |
| UT-domain-02 | `Platform::Desktop` 序列化为 `"desktop"` | Platform::Desktop | `"desktop"` |
| UT-domain-03 | `Platform` 拒绝枚举外值 | `"mobile"` JSON | 反序列化 Err |
| UT-domain-04 | `device_name=None` 不出现在 JSON | Device 无 name | 序列化结果无 `device_name` 键 |
| UT-domain-05 | `Device→DeviceRegisterResponse` From | Device | 字段一一对应 |
| UT-domain-06 | `DeviceRegisterRequest` 缺 `address` 拒绝 | 无 address | Err |
| UT-domain-07 | `StatusReportRequest.device_name` 可选 | 无 device_name | Ok（None） |
| UT-domain-08 | `LookupDevice` 嵌套 address（Pre-req-9） | — | JSON 形状 `address:{ip,port}` |

### 4.4 `store/device_repo.rs` 单元测试（仓储层，临时 SQLite）

| 用例 ID | 名称 | 步骤 | 期望 |
|--------|------|------|------|
| UT-device_repo-01 | 新建 → Created | upsert 不存在 id | `RegisterOutcome::Created`, DB 有行, token==传入 |
| UT-device_repo-02 | 复用（token 有效）→ Reused 原token | 先 insert，再 upsert（时钟未过 TTL） | `Reused`, token==原 token, address 更新 |
| UT-device_repo-03 | 续期（token 过期）→ Reused 新token | insert 后 `clock.advance(25h)`，再 upsert | `Reused`, token==新 token ≠ 原 |
| UT-device_repo-04 | `device_name` 缺省时复用 DB 值 | insert 带 name，upsert 不带 name | 响应 device_name==原值 |
| UT-device_repo-05 | `mark_online` 更新地址+在线 | — | DB `is_online==1`, ip/port 更新 |
| UT-device_repo-06 | `find_id_by_token` 命中 | 有效 token | `Some(id)` |
| UT-device_repo-07 | `find_id_by_token` 未命中 | 伪造 token | `None` |
| UT-device_repo-08 | `find_id_by_token` 损坏 UUID | DB 插入非 UUID id（绕过约束直接 SQL） | `Err(Internal)` |
| UT-device_repo-09 | `mark_stale_offline` 阈值边界 | 设备 last_seen = 阈值 - 1s / 阈值 / 阈值 + 1s | 严格 `<` 时标记离线 |
| UT-device_repo-10 | `count_online` | 插入若干在线/离线 | 计数正确 |
| UT-device_repo-11 | `current_address` 命中 | — | 返回 ip/port |
| UT-device_repo-12 | `current_address` 不存在 | 伪造 id | `Err(DeviceNotFound)` |
| UT-device_repo-13 | 并发同 id（Pre-req-4） | 100 并发 upsert 同 id | 无 Err，全部 Created/Reused |

### 4.5 `store/pairing_code_repo.rs` 单元测试

| 用例 ID | 名称 | 步骤 | 期望 |
|--------|------|------|------|
| UT-pairing_code_repo-01 | `create` 插入 | — | DB 有行, expires_at==now+5min |
| UT-pairing_code_repo-02 | `lookup` 命中 | create 后立即 lookup | Ok, 形状嵌套 address（Pre-req-9） |
| UT-pairing_code_repo-03 | `lookup` 过期 → PairingCodeExpired | create 后 `clock.advance(5min+1s)` | `Err(PairingCodeExpired)` |
| UT-pairing_code_repo-04 | `lookup` used=1 → PairingCodeExpired（Pre-req-6） | 手动 UPDATE used=1 | `Err(PairingCodeExpired)` |
| UT-pairing_code_repo-05 | `lookup` 不存在 | 伪造码 | `Err(PairingCodeNotFound)` |
| UT-pairing_code_repo-06 | `lookup` 设备已删（FK 孤儿） | 删 device 行（绕过 FK 或禁用 FK） | `Err(PairingCodeNotFound)` 或 Internal（明确期望） |
| UT-pairing_code_repo-07 | `delete_expired` | 插入过期+未过期+used | 仅删过期与 used |
| UT-pairing_code_repo-08 | `count_active` | 插入混合 | 仅计 expires_at>now && used=0 |

### 4.6 `cache/mod.rs` 单元测试

| 用例 ID | 名称 | 步骤 | 期望 |
|--------|------|------|------|
| UT-cache-01 | 配对码缓存命中 | put 后 get | Some |
| UT-cache-02 | 配对码缓存过期（真实 expires_at，Pre-req-8） | put 时 expires_at=now+1s，advance 2s | None |
| UT-cache-03 | 配对码缓存未命中 | get 未 put 的码 | None |
| UT-cache-04 | token 缓存续期失效（Pre-req-3） | put old, put new, get old | None |
| UT-cache-05 | token 缓存 TTL 过期 | put 后 advance 25h, get | None |
| UT-cache-06 | `invalidate_pairing_code` | put 后 invalidate, get | None |
| UT-cache-07 | `invalidate_token` | put 后 invalidate, get | None |
| UT-cache-08 | `cleanup_expired` 清理过期配对码 | put 2 个（1 过期） | 返回 1，剩余 1 |
| UT-cache-09 | LRU 驱逐（`MAX_PAIRING_CODES`） | put 到上限+1 | 最旧被驱逐，总数==上限 |
| UT-cache-10 | token 上限驱逐（`MAX_TOKENS`） | put 到上限+1 | 总数==上限 |

> UT-cache-09/10 因 `MAX_PAIRING_CODES=100_000`/`MAX_TOKENS=1_000_000` 较大，
> 测试可通过 `cfg(test)` 暴露一个 `set_limits_for_test(small)` 钩子调小阈值，避免跑
> 10 万次。**实现约定**：`Cache` 增加 `#[cfg(test)] pub fn with_limits(pc, t)` 构造器。

### 4.7 `batch/mod.rs` 单元测试

| 用例 ID | 名称 | 步骤 | 期望 |
|--------|------|------|------|
| UT-batch-01 | enqueue + flush 落库 | enqueue 1 条，触发 flush | DB ip/port/last_seen 更新 |
| UT-batch-02 | 阈值 1000 即时 flush | enqueue 1000 条 | 触发即时 flush（不等周期） |
| UT-batch-03 | 队列满丢弃 | 队列容量内塞满再 enqueue | enqueue 不阻塞，日志 warn |
| UT-batch-04 | device_name 落库（Pre-req-7） | enqueue 带 name | DB device_name 更新（COALESCE） |
| UT-batch-05 | 空 flush no-op | 直接 flush 空缓冲 | Ok，无 DB 写 |
| UT-batch-06 | 同设备多次 enqueue 去重/覆盖 | 同 id enqueue 2 次不同 ip | flush 后 DB==最后一次 ip |

> flush 触发：测试用短 `flush_interval`（如 50ms）+ `tokio::time::sleep` 等待，或暴露
> `flush_now()` 测试钩子。**实现约定**：`BatchWriter` 增加 `#[cfg(test)] pub async fn flush_now(&self)`。

### 4.8 `api/auth.rs` 单元测试

| 用例 ID | 名称 | 步骤 | 期望 |
|--------|------|------|------|
| UT-RateLimiter-01 | 窗口内允许 N 次 | limit=1，check 1 次 | true |
| UT-RateLimiter-02 | 超限拒绝 | limit=1，check 2 次 | 第 2 次 false |
| UT-RateLimiter-03 | 滑动窗口过期 | limit=1，check，advance 1s+，check | 第 2 次 true |
| UT-RateLimiter-04 | 不同 IP 独立计数 | ip A 用尽，ip B check | B true |
| UT-extract_ip-01 | XFF 单段 | `X-Forwarded-For: 1.2.3.4` | `"1.2.3.4"` |
| UT-extract_ip-02 | XFF 多段 | `X-Forwarded-For: 1.2.3.4, 5.6.7.8` | `"1.2.3.4"` |
| UT-extract_ip-03 | XFF 含空白 | `X-Forwarded-For: 1.2.3.4 , 5.6.7.8` | `"1.2.3.4"`（trim） |
| UT-extract_ip-04 | 无 XFF 回退 ConnectInfo | 无头，扩展有 ConnectInfo | 连接 IP |
| UT-extract_ip-05 | 无 XFF 无 ConnectInfo | 无头无扩展 | `""` |
| UT-AuthenticatedDevice-01 | 无 Authorization 头 | — | `Err(TokenInvalid)` |
| UT-AuthenticatedDevice-02 | 空 Bearer | `Bearer ` | `Err(TokenInvalid)` |
| UT-AuthenticatedDevice-03 | 错 scheme | `Basic xxx` | `Err(TokenInvalid)` |
| UT-AuthenticatedDevice-04 | 缓存命中 | cache 有 token | Ok(id)，不查 DB |
| UT-AuthenticatedDevice-05 | DB 命中 | cache 无，DB 有 | Ok(id)，写缓存 |
| UT-AuthenticatedDevice-06 | DB 未命中 | cache 无，DB 无 | `Err(TokenInvalid)` |

> UT-AuthenticatedDevice-* 用 `axum::extract::from_request_parts` 或直接构造 `Parts`
> 测试，依赖一个最小 `AppState`（临时 DB + 空 cache）。

### 4.9 `api/error.rs` 与 `api/error.rs`（生成器）单元测试

| 用例 ID | 名称 | 期望 | 现有 |
|--------|------|------|------|
| UT-error-01 | 6 错误码 HTTP 状态正确 | 见 §5.1.3 表 | 部分（补全 6 个） |
| UT-error-02 | 6 错误码字符串匹配 OpenAPI | INVALID_REQUEST 等 | 部分 |
| UT-error-03 | `IntoResponse` 体形状 | `{error:{code,message}}`，无 details 时无 details 键 | 缺 |
| UT-error-04 | Internal 隐藏 message | AppError::Internal → body message=="internal server error" | 缺 |
| UT-error-05 | `From<sqlx::Error>` | sqlx err → AppError::Internal | 缺 |
| UT-gen-01 | token 长度 64 | — | 有 |
| UT-gen-02 | 配对码 6 位全数字 | — | 有 |
| UT-gen-03 | token 字符集限定 | 全在 TOKEN_ALPHABET | 缺 |
| UT-gen-04 | 配对码前导零 | 强制 rng 返回小数（mock）| 缺 |

### 4.10 `observability.rs` 单元测试

| 用例 ID | 名称 | 期望 |
|--------|------|------|
| UT-metrics-01 | 5 计数器 fetch_add | 注册后 `devices_registered_total==1` |
| UT-metrics-02 | Relaxed 不丢增量（单线程） | 多次 add 后值正确 |
| UT-metrics-03 | `start_metrics_reporter` 不 panic | 启动后立即 drop handle |

> 指标测试不验证日志输出内容（脆弱），只验证计数器值。

---

## 5. e2e 测试规范（L3，同构 Caddy 通道）

> 引用 spec 11 §13.2。**两条通道并存**：
> - **L2 集成通道**（现有 `tests/e2e/api.rs` 的 axum 内存 + 临时 SQLite）：保留，覆盖
>   业务逻辑，快速反馈。但**必须改名/拆分**为 `tests/integration/`，不再叫 e2e（避免
>   误导，spec 11 §13.2 e2e 要求同构 Caddy）。
> - **L3 e2e 通道**（新增）：通过 docker-compose 起 Caddy+axum，reqwest 打
>   `https://localhost:4443`。这是 spec 11 §13.2 真正的 e2e。

### 5.1 通道结构重构

```
tests/
├── integration/              # L2（原 e2e/api.rs 拆分而来）
│   ├── common/mod.rs         # spawn_server 夹具（axum 内存）
│   ├── register.rs           # E2E-01/02/03 + EC 校验类
│   ├── pairing_codes.rs      # E2E-04/05/06/07
│   ├── status.rs             # E2E-08/09/10
│   └── errors.rs             # E2E-12~17（4xx 体形状）
├── e2e/                      # L3（新增，同构 Caddy）
│   ├── README.md             # 运行说明（docker-compose up 后跑）
│   ├── caddy_channel.rs      # E2E-CADDY-*（TLS/gzip/反代路径）
│   └── full_flow.rs          # E2E-11（端到端续期链路）+ 限流 E2E-18/19
└── load/                     # k6 脚本（§6）
```

**Cargo.toml `[[test]]` 条目**：

```toml
[[test]]
name = "integration"
path = "tests/integration/main.rs"  # 用 main.rs 聚合多模块

[[test]]
name = "e2e-caddy"
path = "tests/e2e/caddy_channel.rs"
required-features = []  # 默认启用；CI 用 feature gate 控制是否运行
```

> **e2e 默认不参与 `cargo test`**（需先起 docker-compose）。通过 `#[ignore]` 标记，
> `cargo test -- --ignored` 显式运行；或在 CI 单独 job。详见 §8.5。

### 5.2 e2e 用例全集（spec 11 §13.2 ⑥ 场景）

> 所有断言的响应 JSON 形状以 `openapi/pairing-server.yaml` 为准。

#### 5.2.1 ① 设备注册 upsert

| ID | 名称 | 步骤 | 期望 |
|----|------|------|------|
| E2E-01 | 新建 201 | POST /devices 新 id | 201, body 有 id/platform/pairing_token(64)/address/created_at |
| E2E-02 | 复用 200 原token | POST 后立即再 POST 同 id | 200, pairing_token==首次 |
| E2E-03 | 续期 200 新token | 插入 25h 前 created_at 行（SQL 直插或 FakeClock），POST 同 id | 200, pairing_token≠首次（Pre-req-1） |

#### 5.2.2 ② 配对码

| ID | 名称 | 步骤 | 期望 |
|----|------|------|------|
| E2E-04 | 生成 201 + 查询 200 | 注册→生成→查询 | 201 code(6位), 200 body device.address.{ip,port}（Pre-req-9） |
| E2E-05 | 查询不存在 404 | GET /pairing-codes/999999 | 404, error.code==PAIRING_CODE_NOT_FOUND |
| E2E-06 | 过期 410 | 生成后 clock.advance(5min+1s) 或插过期行，查询 | 410, error.code==PAIRING_CODE_EXPIRED（Pre-req-1/8） |
| E2E-07 | 已使用 410 | 查询一次成功后再查同码 | 410, error.code==PAIRING_CODE_EXPIRED（Pre-req-6） |

#### 5.2.3 ③ 状态上报

| ID | 名称 | 步骤 | 期望 |
|----|------|------|------|
| E2E-08 | 上报 happy 200 | 注册→PUT /status | 200, body.ok==true |
| E2E-09 | IP 变化刷新断言 | 上报新 ip，查 current_address 或重新生成配对码查地址 | 地址==新 ip |
| E2E-10 | device_name 改名落库 | 上报带新 device_name，生成配对码查 | device_name==新值（Pre-req-7） |

#### 5.2.4 ④ token 过期续期

| ID | 名称 | 步骤 | 期望 |
|----|------|------|------|
| E2E-11 | 401→重 POST→新 token→重放心跳 | PUT /status 用过期 token → 401；POST /devices 同 id → 200 新 token；PUT /status 新 token → 200 | 全链路成功，旧 token 失效（Pre-req-3） |

#### 5.2.5 ⑤ 错误响应格式（6 错误码体形状）

> 每个用例断言：`status` + `body.error.code` + `body.error.message` 非空 +
> `details`（若有）形状。

| ID | 名称 | 触发 | 期望 status / code |
|----|------|------|-------------------|
| E2E-12 | INVALID_REQUEST 400 | 缺 platform / 畸形 JSON / device_id 非 UUID | 400 / INVALID_REQUEST |
| E2E-13 | TOKEN_INVALID 401 | 无 Authorization / 错 scheme / 伪造 token | 401 / TOKEN_INVALID |
| E2E-14 | DEVICE_NOT_FOUND 404 | PUT /status 合法 token 但 path 用不存在 id（注：先 path≠auth→401，需构造合法 token 对应不存在的 path id 的边界） | 404 / DEVICE_NOT_FOUND |
| E2E-15 | PAIRING_CODE_NOT_FOUND 404 | 查不存在码 | 404 / PAIRING_CODE_NOT_FOUND |
| E2E-16 | PAIRING_CODE_EXPIRED 410 | 过期/已使用码 | 410 / PAIRING_CODE_EXPIRED |
| E2E-17 | RATE_LIMITED 429 | 连续超限请求 | 429 / RATE_LIMITED（Pre-req-2） |

> E2E-14 边界说明：当前 `report_status` 先做 `auth.0 != device_id → 401`，再入队 batch。
> `DEVICE_NOT_FOUND` 实际上**不会从 PUT /status 触发**（batch writer 不校验存在性）。
> 这暴露另一缺陷：**batch writer 对不存在的 device_id 静默成功**。Pre-req：batch flush
> 后对未命中行打 warn，或在 enqueue 前校验。**本规范要求**：PUT /status 对不存在的
> device（合法 token 但 token 对应设备与 path id 不一致已被 401 覆盖；真正 DEVICE_NOT_FOUND
> 场景需 token 对应设备已被删）应返回 404。实现者需在 enqueue 前加存在性校验
> （或接受 batch 静默并从 spec 删除 DEVICE_NOT_FOUND for PUT /status——**本规范选择加校验**）。

#### 5.2.6 ⑥ 速率限制

| ID | 名称 | 步骤 | 期望 |
|----|------|------|------|
| E2E-18 | 单 IP 超限 429 | 配置 limit=1，同 IP 连续 2 次 POST | 第 2 次 429 |
| E2E-19 | 不同 IP 不互相影响 | IP A 用尽，IP B 请求 | B 200/201 |

### 5.3 L3 同构 Caddy 通道用例（新增 `tests/e2e/caddy_channel.rs`）

> 这些用例**只在 docker-compose 起来后运行**（`#[ignore]`）。验证 spec 11 §13.1 同构要素。

| ID | 名称 | 验证点 | 期望 |
|----|------|--------|------|
| E2E-CADDY-01 | TLS 握手成功 | reqwest `danger_accept_invalid_certs(true)` 打 https://localhost:4443/health | 200 "ok" |
| E2E-CADDY-02 | gzip 压缩生效 | 请求带 `Accept-Encoding: gzip`，检查响应 `Content-Encoding: gzip` | 头存在 |
| E2E-CADDY-03 | 反代路径 /devices/* | POST /devices 经 Caddy | 201（与直连 axum 一致） |
| E2E-CADDY-04 | 反代路径 /pairing-codes/* | GET /pairing-codes/{code} 经 Caddy | 与直连一致 |
| E2E-CADDY-05 | X-Forwarded-For 透传 | Caddy 设置 XFF，限流按 XFF 计数 | 限流生效（Pre-req-2 + extract_client_ip） |
| E2E-CADDY-06 | 完整配对流程经 Caddy | 注册→生成→查询 全走 4443 | 全 200/201 |

> **运行前置**：`docker compose -f docker/docker-compose.local.yml up -d`，等待 health。
> CI 中由 §8.5 的 e2e job 负责。reqwest 客户端：

```rust
fn caddy_client() -> reqwest::Client {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(true)  // 自签证书（spec 11 §13.1）
        .build().unwrap()
}
```

### 5.4 e2e 用例与 Pre-req 依赖汇总

| 用例 | 依赖 Pre-req |
|------|-------------|
| E2E-03 | 1 |
| E2E-06 | 1, 8 |
| E2E-07 | 6 |
| E2E-10 | 7 |
| E2E-11 | 1, 3 |
| E2E-14 | （新增 batch 存在性校验） |
| E2E-17/18/19 | 2 |
| E2E-CADDY-05 | 2 |
| E2E-04 | 9（断言形状） |

---

## 6. 压测规范（L3，k6）

> 引用 spec 11 §13.3。4 个脚本 + 1 份容量验证报告。**合并门槛**（spec 11 §13.3）：
> PR 必须附带脚本 + 报告。

### 6.1 通用约定（所有脚本）

| 项 | 约定 |
|----|------|
| BASE_URL | `__ENV.BASE_URL`，默认 `https://localhost:4443` |
| TLS | `insecureSkipTLSVerify: true`（spec 11 §13.1 自签） |
| 阈值（硬） | `http_req_duration: ['p(95)<100']`, `http_req_failed: ['rate<0.01']`（spec 11 §13.3） |
| 报告输出 | `tests/load/reports/<scene>-<date>.json` + stdout 文本摘要 |
| 设备 ID 派生 | **必须保证唯一性**：`uuid()` 或 `${__VU}-${__ITER}` 组合，**禁止**仅用 `__VU` |
| 限流规避 | 压测时 `Config.rate_limit_per_sec` 调高（如 10000），否则 1 req/IP/s 会污染结果 |

> **限流规避决策**：压测目的是验证容量（§10.4），不是验证限流。限流正确性由 E2E-18/19
> 验证。压测环境通过 `RATE_LIMIT_PER_SEC=10000` env 注入调高。**压测脚本不测限流**。

### 6.2 场景①：注册洪流 `register.js`（重写）

**目标**（spec 11 §13.3 场景①）：10 万设备并发注册。

| 维度 | 值 |
|------|-----|
| 唯一设备数 | **100 000**（关键：原脚本仅 ~200） |
| 并发 | ramping 0→500→500→0 |
| 持续 | ramp 30s + 稳态 2m + down 30s |
| 断言 | 201（首次）/ 200（重复，同 VU 内复用） |

**设备 ID 派生**（保证 10 万唯一）：

```javascript
// 每个 VU 用计数器 + VU 号生成唯一 UUID v4 格式
import exec from 'k6/execution';
const DEVICE_ID = `00000000-0000-4000-8000-${(exec.scenario.iterationInTest % 100000).toString(16).padStart(12, '0')}`;
```

或用 k6 内置 UUID：`import { uuid } from 'k6/data'`（每次 iter 新建）。

**自定义指标**：`devices_created_201`, `devices_reused_200`，最终断言 created≈100000。

### 6.3 场景②：心跳稳态 `heartbeat.js`（新增）

**目标**（spec 11 §13.3 场景②）：1014 QPS 持续（spec 11 §10.4 峰值）。

| 维度 | 值 |
|------|-----|
| 预置 | 先注册 N 个设备（setup 阶段）拿 token |
| 主循环 | PUT /devices/{id}/status（携带 token） |
| 目标 QPS | **1014**（constant-arrival-rate executor） |
| 持续 | 3m |
| 断言 | P95<100ms, 错误率<1%, 实测 RPS≈1014 |

```javascript
export const options = {
  scenarios: {
    heartbeat: {
      executor: 'constant-arrival-rate',
      rate: 1014, timeUnit: '1s',
      duration: '3m',
      preAllocatedVUs: 1500, maxVUs: 3000,
    },
  },
  thresholds: { http_req_duration: ['p(95)<100'], http_req_failed: ['rate<0.01'] },
};
```

**setup**：注册 1500 个设备（`maxVUs` 上限），每个 VU 绑定一个 token。

### 6.4 场景③：配对码读写混合 `pairing_code.js`（新增）

**目标**（spec 11 §13.3 场景③）：配对码生成 + 查询混合负载。

| 维度 | 值 |
|------|-----|
| 预置 | 注册设备池 |
| 混合比 | 20% POST pairing-code（写）+ 80% GET pairing-codes/{code}（读） |
| 并发 | 200 VUs，2m |
| 断言 | 写 201，读 200/410（过期可接受），P95<100ms |

**实现**：每个 VU 维护"自己生成的 code 池"，写后入池供读；读随机从池取或造不存在码。

### 6.5 场景④：SQLite 锁竞争 `sqlite_contention.js`（新增）

**目标**（spec 11 §13.3 场景④）：并发写入验证 WAL + busy_timeout(50ms) 表现。

| 维度 | 值 |
|------|-----|
| 负载 | 100% PUT /status（纯写，触发 batch flush 事务） |
| 并发 | ramping 0→1000→1000→0 |
| 关注 | **错误率**（SQLITE_BUSY 不应泄漏为 500）、P99 尾延迟 |
| 断言 | 错误率<1%（允许少量 busy 重试后成功），无 5xx 洪流 |

**关键观察点**：若错误率飙升，说明 `SQLITE_BUSY_TIMEOUT_MS=50ms`（spec 11 §4/§9.3）
不足，需调大或增加连接池。报告须记录 `sqlite_busy_count`（从日志统计）。

### 6.6 容量验证报告（合并门槛）

每次压测后生成 `tests/load/reports/capacity-report-<date>.md`，**模板**：

```markdown
# 配对服务器容量验证报告

> 日期: YYYY-MM-DD
> 环境: docker-compose.test.yml（Caddy 2.11 + axum + SQLite, 本地 / staging VPS）
> spec 11 §13.3 合并门槛

## 1. 容量目标（spec 11 §1.3 / §10.4）

| 指标 | 目标 | 实测 | 达标 |
|------|------|------|------|
| 设备总量 | 100 万 | <填写> | ✅/❌ |
| 峰值 QPS | 1014 | <填写> | ✅/❌ |

## 2. 场景结果

| 场景 | 脚本 | RPS | P95 | P99 | 错误率 | 达标 |
|------|------|-----|-----|-----|--------|------|
| ① 注册洪流 | register.js | | | | | |
| ② 心跳稳态 | heartbeat.js | | | | | |
| ③ 配对码混合 | pairing_code.js | | | | | |
| ④ SQLite 锁竞争 | sqlite_contention.js | | | | | |

阈值（spec 11 §13.3）：P95<100ms, 错误率<1%。

## 3. 资源占用

| 资源 | 平均 | 峰值 | 限制 | 达标 |
|------|------|------|------|------|
| CPU | | | 2 核 | |
| 内存 | | | 2G | |
| 带宽 | | | 3 Mbps | |

（docker stats 采集）

## 4. 结论

<是否满足 spec 11 §1.3 容量目标>
```

---

## 7. Edge Cases 与异常状态用例全集

> 引用 spec 11 §5.1.3 错误码全集 + §6 异常处理表。按分类组织，每条标注 spec 出处。

### 7.1 请求校验类（→400 INVALID_REQUEST）

| ID | 输入 | 期望 | spec 出处 |
|----|------|------|----------|
| EC-req-01 | 畸形 JSON（`"not json"`） | 400 INVALID_REQUEST | §5.1.3 |
| EC-req-02 | 空请求体 | 400 | §5.1.3 |
| EC-req-03 | `device_id` 非 UUID（`"not-a-uuid"`） | 400 | OpenAPI format:uuid |
| EC-req-04 | `port` 超 u16（99999） | 400 | OpenAPI format:uint16 |
| EC-req-05 | `port` 负数 | 400 | OpenAPI |
| EC-req-06 | `address` 缺失 | 400 | OpenAPI required |
| EC-req-07 | `address.ip` 缺失 | 400 | OpenAPI required |
| EC-req-08 | `platform: "mobile"`（枚举外） | 400 | OpenAPI enum |
| EC-req-09 | 未知额外字段（`{"foo":"bar",...}`） | **当前放行**；决策见下 | OpenAPI（未禁） |
| EC-req-10 | `device_name` 超 64 字符（OpenAPI maxLength） | 400（**需服务端加校验**） | OpenAPI maxLength:64 |
| EC-req-11 | `GET /pairing-codes/abc123`（非 6 位数字） | 400 或 404（决策见下） | OpenAPI pattern |
| EC-req-12 | `GET /pairing-codes/12345`（5 位） | 400/404 | OpenAPI pattern |

> **EC-req-09 决策**：OpenAPI 未用 `additionalProperties: false`，serde 默认
> `#[serde(deny_unknown_fields)]` 未启用，故额外字段被忽略。**本规范要求保持放行**
> （前向兼容），但需加测试固化行为（防止有人误加 deny）。
>
> **EC-req-10 决策**：服务端需在 `DeviceRegisterRequest`/`StatusReportRequest` 加
> `device_name` 长度校验（`#[validate(length(max=64))]` 或手写），返回 400。**列为
> Pre-req-10**（轻量，归入 §3）。
>
> **EC-req-11/12 决策**：路径参数 `code: String` 不校验 pattern。两种选择：
> (a) 加正则校验返回 400；(b) 视为不存在返回 404。OpenAPI pattern 是约束客户端。
> **本规范选择 (b) 404**（与现有 PAIRING_CODE_NOT_FOUND 一致，最小改动），但需测试
> 固化：非数字码 → 404。

### 7.2 认证类（→401 TOKEN_INVALID）

| ID | 输入 | 期望 | spec 出处 |
|----|------|------|----------|
| EC-auth-01 | 无 Authorization 头 | 401 | §14.1 |
| EC-auth-02 | `Authorization: Bearer`（无 token） | 401 | §14.1 |
| EC-auth-03 | `Authorization: Bearer `（空白） | 401 | §14.1 |
| EC-auth-04 | `Authorization: Basic xxx` | 401 | §14.1 |
| EC-auth-05 | 伪造 token（64 字符随机） | 401 | §14.1 |
| EC-auth-06 | token 含前后空白（`Bearer  xxx `） | trim 后校验，无效则 401 | §14.1 |
| EC-auth-07 | path id ≠ token 对应 id（PUT /status） | 401 | `devices.rs:68` |
| EC-auth-08 | path id ≠ token 对应 id（POST pairing-code） | 401 | `pairing_codes.rs:28` |
| EC-auth-09 | 续期后旧 token（Pre-req-3） | 401 | §14.1, §7.5 |

### 7.3 配对码生命周期类

| ID | 输入 | 期望 | spec 出处 |
|----|------|------|----------|
| EC-code-01 | 过期码查询 | 410 PAIRING_CODE_EXPIRED | §5.1.3 |
| EC-code-02 | used=1 码查询 | 410 PAIRING_CODE_EXPIRED | §5.1.3（Pre-req-6） |
| EC-code-03 | 码存在但设备已删（FK 孤儿） | 404 或 410（明确期望） | §9.1 FK |
| EC-code-04 | 同一码并发查询（Pre-req-6 一次性） | 恰有一个 200，其余 410 | §5.1.3 |
| EC-code-05 | 生成码碰撞重试（Pre-req-5） | handler 重试成功 201 | §4 PAIRING_CODE_RETRY |

### 7.4 并发与竞态类（高价值）

| ID | 场景 | 期望 | spec 出处 |
|----|------|------|----------|
| EC-concurrency-01 | 100 并发 POST /devices 同 id | 全 201/200，无 500 | §5.1.1（Pre-req-4） |
| EC-concurrency-02 | 配对码生成碰撞 | handler 重试成功，无 500 | §4（Pre-req-5） |
| EC-concurrency-03 | 并发 PUT /status 同设备 | 最后写入获胜（或顺序一致），无 500 | §10.2 |
| EC-concurrency-04 | 注册同时心跳（同设备） | 无死锁/无 500 | §7 |

> EC-concurrency-* 用 `tokio::spawn` + `futures::join_all` 或 `tokio::time::timeout`
> 构造并发，断言无 5xx。这是 e2e 层（或集成层）用例，归入 `tests/integration/concurrency.rs`。

### 7.5 安全相关（引用 spec 13）

| ID | 场景 | 期望 | spec 出处 |
|----|------|------|----------|
| EC-sec-01 | Internal 错误不泄露细节 | DB 故障时 body.message=="internal server error"，无堆栈/sql | §5.1.3, spec 13 |
| EC-sec-02 | token 不出现在日志（access log） | TraceLayer 不记录 Authorization | §11.3, spec 13 |
| EC-sec-03 | device_id 可从配对码查询获得（设计内） | GET /pairing-codes 返回 device.id（这是公开的，§14.4） | §14.4 |

> EC-sec-01 通过制造 DB 错误（如关闭池）触发；EC-sec-02 检查 TraceLayer 配置不含
> req headers；EC-sec-03 是固化设计决策的测试（防止误删 id 字段）。

### 7.6 可观测性与生命周期

| ID | 场景 | 期望 | spec 出处 |
|----|------|------|----------|
| EC-obs-01 | `/health` 返回 200 "ok" | — | §11.2, spec 16 §2.5 |
| EC-obs-02 | 注册后 `devices_registered_total` +1 | — | §11.2 |
| EC-obs-03 | 配对码生成后 `pairing_codes_issued_total` +1 | — | §11.2 |
| EC-obs-04 | 查询后 `pairing_code_lookups_total` +1 | — | §11.2 |
| EC-obs-05 | 后台清理删除过期码 | 插过期码，等 `CACHE_CLEANUP_INTERVAL`（测试调小），码被删 | §7.2, main.rs |
| EC-obs-06 | 后台标记 stale 设备离线 | 设备 last_seen 旧，等清理周期，`is_online==0`, `devices_online` 计数刷新 | §7.2 |

> EC-obs-05/06 用短周期：测试夹具启动后台任务时 `CACHE_CLEANUP_INTERVAL_SECS` 设为
> 1（秒），避免等 60s。**实现约定**：后台任务周期参数化（从 Config 取，测试注入小值）。

---

## 8. 测试基础设施

### 8.1 目录结构（最终形态）

```
apps/pairing-server/
├── src/
│   ├── clock.rs                    # 新增（Pre-req-1）
│   ├── api/
│   │   ├── rate_limit.rs           # 新增（Pre-req-2）
│   │   └── ...
│   ├── test_support.rs             # 新增（#[cfg(test)] 夹具）
│   └── ...
├── tests/
│   ├── integration/                # L2（原 e2e/api.rs 拆分）
│   │   ├── main.rs                 # 聚合 mod 声明
│   │   ├── common/mod.rs           # spawn_server 夹具
│   │   ├── register.rs
│   │   ├── pairing_codes.rs
│   │   ├── status.rs
│   │   ├── errors.rs
│   │   └── concurrency.rs          # EC-concurrency-*
│   ├── e2e/                        # L3（同构 Caddy）
│   │   ├── README.md
│   │   ├── caddy_channel.rs        # #[ignore]
│   │   └── full_flow.rs            # #[ignore]
│   └── load/                       # k6
│       ├── register.js             # 重写
│       ├── heartbeat.js            # 新增
│       ├── pairing_code.js         # 新增
│       ├── sqlite_contention.js    # 新增
│       ├── README.md               # 更新
│       └── reports/                # 报告产物（gitignore 或归档）
└── docker/                        # 容器构建 + 自托管 + 本地 e2e
    ├── Dockerfile                 # all-in-one（Rust + Caddy + s6）
    ├── Caddyfile                  # 自托管 :8080
    ├── Caddyfile.local            # 本地 e2e :4443（tls internal）
    └── docker-compose.local.yml   # 本地 e2e 编排（含 healthcheck）
```

### 8.2 `Cargo.toml` 变更

```toml
[dev-dependencies]
reqwest = { workspace = true }
tempfile = { workspace = true }
# 新增：覆盖率与并发测试辅助（如已在 workspace 则引用）
tokio = { workspace = true, features = ["test-util", "macros", "rt-multi-thread"] }

[[test]]
name = "integration"
path = "tests/integration/main.rs"

[[test]]
name = "e2e-caddy"
path = "tests/e2e/caddy_channel.rs"
# e2e 默认不跑，用 #[ignore] + cargo test -- --ignored
```

### 8.3 docker-compose.test.yml healthcheck（补全）

> 现有 compose 无 healthcheck，e2e 启动时无法判断 axum 就绪。补全：

```yaml
services:
  pairing-server:
    # ... 现有
    healthcheck:
      test: ["CMD", "wget", "-qO-", "http://localhost:38424/health"]
      interval: 2s
      timeout: 2s
      retries: 30
  caddy:
    # ... 现有
    depends_on:
      pairing-server:
        condition: service_healthy
```

### 8.4 限流与周期任务的测试参数化

| 参数 | 生产值 | 测试值 | 注入方式 |
|------|--------|--------|---------|
| `rate_limit_per_sec` | 1 | 1000（集成）/ 10000（压测） | `Config` 字段，env 注入 |
| `CACHE_CLEANUP_INTERVAL_SECS` | 60 | 1 | `Config` 字段（新增），env 注入 |
| `BATCH_WRITE_INTERVAL_SECS` | 1 | 0.05 | `Config` 字段，env 注入 |
| 时钟 | SystemClock | FakeClock | `AppState.clock` 注入 |

> **实现约定**：`Config` 增加上述周期字段（带生产默认值），`main.rs` 读 env，测试夹具
> 注入小值。后台任务从 `AppState`/`Config` 取周期，不硬编码。

### 8.5 CI 闸门（接入 spec 19）

> spec 19 §2.2 现有 `backend` job 只测 desktop src-tauri。新增 pairing-server job。

**`.github/workflows/quality.yml` 新增 job**（伪代码，实现者按 spec 19 体例完善）：

```yaml
  pairing-server:
    name: Pairing Server
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: rustfmt, clippy }
      - uses: Swatinem/rust-cache@v2
        with: { workspaces: apps/pairing-server }
      - run: cargo fmt --check --manifest-path apps/pairing-server/Cargo.toml
      - run: cargo clippy --manifest-path apps/pairing-server/Cargo.toml -- -D warnings
      - name: Unit + Integration tests
        run: cargo test --manifest-path apps/pairing-server/Cargo.toml
      - name: Coverage gate (≥80% lines)
        run: |
          cargo install cargo-llvm-cov
          cargo llvm-cov --manifest-path apps/pairing-server/Cargo.toml \
            --workspace --fail-under-lines 80

  pairing-server-e2e:
    name: Pairing Server E2E (Caddy)
    runs-on: ubuntu-latest
    needs: pairing-server
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
        with: { workspaces: apps/pairing-server }
      - run: docker compose -f apps/pairing-server/docker/docker-compose.local.yml up -d --build
      - run: |
          cargo test --manifest-path apps/pairing-server/Cargo.toml \
            --test e2e-caddy -- --ignored
      - if: always()
        run: docker compose -f apps/pairing-server/docker/docker-compose.local.yml down -v

  pairing-server-load:
    name: Pairing Server Load (k6, manual/lower branch)
    runs-on: ubuntu-latest
    if: github.event_name == 'workflow_dispatch'   # 压测手动触发，不阻塞 PR
    steps:
      - uses: actions/checkout@v4
      - run: docker compose -f apps/pairing-server/docker/docker-compose.local.yml up -d --build
      - uses: grafana/k6-action@v0.3.1
        with:
          filename: apps/pairing-server/tests/load/register.js
        env: { BASE_URL: https://localhost:4443, RATE_LIMIT_PER_SEC: '10000' }
      # ... 其余 3 脚本
```

> **压测 job 决策**：压测耗时长、资源大，**不阻塞每个 PR**（`workflow_dispatch` 手动
> 或 nightly）。但 spec 11 §13.3 合并门槛要求"PR 附带报告"——由 PR 作者本地跑后提交
> 报告（§6.6），CI 仅做 smoke 验证脚本可执行。

### 8.6 package.json scripts 更新

```json
{
  "scripts": {
    "build": "cargo build --release --manifest-path Cargo.toml",
    "test": "cargo test --manifest-path Cargo.toml",
    "test:e2e": "cargo test --manifest-path Cargo.toml --test e2e-caddy -- --ignored",
    "test:load": "k6 run tests/load/register.js && k6 run tests/load/heartbeat.js && k6 run tests/load/pairing_code.js && k6 run tests/load/sqlite_contention.js",
    "coverage": "cargo llvm-cov --manifest-path Cargo.toml --workspace",
    "lint": "cargo clippy --manifest-path Cargo.toml -- -D warnings"
  }
}
```

---

## 9. 决策依据

### 9.1 为什么引入时钟注入而非直接 SQL 操纵数据

**选择理由**：
- 直接 UPDATE 行的 `created_at`/`expires_at` 能测 DB 层，但测不到 handler 内
  `now - created > TTL` 的**判定逻辑**本身（边界 off-by-one）。
- `FakeClock` 让"刚刚过期""刚好未过期"边界可确定性测试。
- 时钟注入是可测性最佳实践，生产代码零行为变化（SystemClock 包装同一调用）。

**否决方案**：
- ❌ 全局 `mockall` mock `chrono::Utc::now`：全局状态，测试并行不安全。
- ❌ 真实 `tokio::time::pause`：只控 tokio 定时器，不控 `chrono::Utc::now()`。

### 9.2 为什么 e2e 必须走 Caddy（不能只测 axum 内存）

**选择理由**（spec 11 §13.1）：
- 生产链路是 Cloudflare → Caddy:4443 → axum:38424。axum 内存测试**绕过了 Caddy 的
  TLS 终止、gzip、路径分发、XFF 透传**——这些是生产事故高发点。
- spec 11 §13.1 明确"不 Mock：真实 Caddy 反代 + 真实 gzip"。
- docker-compose.test.yml 已就绪，成本低。

**否决方案**：
- ❌ 仅 axum 内存：满足不了 spec 11 §13.2 "同构"承诺，TLS/gzip/XFF 永远测不到。

### 9.3 为什么保留 L2 集成通道（不全部用 e2e）

**选择理由**：
- e2e（起 docker）慢（秒级启动），不适合开发期快速反馈。
- L2 集成（axum 内存 + 临时 SQLite）毫秒级，覆盖业务逻辑足够。
- 两者互补：L2 测逻辑正确性，L3 测集成正确性。spec 06 §2.1 金字塔也鼓励多层。

### 9.4 为什么压测不测限流

**选择理由**：
- 压测目标（spec 11 §10.4）是验证容量（100 万设备、1014 QPS），与限流（1 req/IP/s）
  语义冲突——限流会压制压测本身的吞吐。
- 限流正确性由 E2E-18/19（功能测试）验证。
- 压测环境调高 `RATE_LIMIT_PER_SEC` 是行业惯例。

### 9.5 为什么配对码采用"一次性"语义（Pre-req-6）

**选择理由**：
- spec 11 §2.2 `PairingCode.used` 字段存在，§5.1.3 `PAIRING_CODE_EXPIRED` 说明含
  "已使用"——字段存在即意味着一次性语义。
- 一次性缩小重放攻击窗口（扫码后被他人重放查询的风险）。

**否决方案**：
- ❌ "可多次查询直到自然过期"：则 `used` 字段无存在必要，与 spec 11 §2.2 冲突。
  若产品要改，先改 spec 11，删 `used` 字段与 "已使用" 语义。

### 9.6 为什么覆盖率硬门控 80% 而非 spec 06 的 60%

**选择理由**：
- spec 06 §4.2 "整体 60%" 是全项目（含 UI）目标。配对服务器是**纯后端无 UI**，
  覆盖门槛应更高（无 UI 测试难覆盖的盲区）。
- 配对服务器是安全敏感（认证、配对码）+ 高并发（心跳）组件，80% 是合理底线。

---

## 10. 实施顺序（建议）

> 实现者按此顺序，每步可独立提交、独立验证。

| 阶段 | 内容 | 依赖 | 验证 |
|------|------|------|------|
| P0-1 | Pre-req-1 时钟注入 | — | 现有测试仍通过 |
| P0-2 | Pre-req-2 限流接入 + Pre-req-10 长度校验 | — | E2E-17/18/19 可写 |
| P0-3 | Pre-req-3/4/5/6/7/8/9 | P0-1 | 各对应用例可写 |
| P1 | §4 单元测试全集 | P0 | `cargo test` 全绿，覆盖率 ≥85% |
| P2 | §5.2 L2 集成用例全集（拆分原 api.rs） | P0/P1 | `cargo test --test integration` 全绿 |
| P3 | §5.3 L3 e2e Caddy 通道 + §7 edge cases | P2 | `cargo test -- --ignored` 全绿 |
| P4 | §6 四个压测脚本 + 容量报告 | P3 | 手动跑通，附报告 |
| P5 | §8.5 CI job 接入 | P3 | PR 流水线全绿 |

---

## 11. 约束

1. **Pre-req 必须先于对应用例**：§3 列出的 10 项修复是实现对应用例的前置条件，禁止
   "跳过修复只补测试"（会测到死代码或假阳性）。
2. **测试独立性**（继承 spec 06 §5.1）：每个用例独立夹具（临时 DB、独立时钟），不依赖
   执行顺序，可并行。
3. **断言以 OpenAPI 为准**：所有 e2e 响应形状断言以 `openapi/pairing-server.yaml` 为
   唯一权威源，发现实现与 OpenAPI 不一致时**修实现**（如 Pre-req-9），而非改测试迁就。
4. **同构承诺**（spec 11 §13.1）：e2e 必须经 docker-compose + Caddy，唯一允许差异是
   自签 TLS。禁止 e2e 直连 axum:38424 绕过 Caddy。
5. **覆盖率硬门控**：CI `cargo-llvm-cov --fail-under-lines 80` 必须通过才能合并
   （spec 06 §5.4 + 本规范 §2.4 加严）。
6. **合并门槛**（spec 11 §13.3）：PR 必须附 `tests/load/reports/capacity-report-<date>.md`，
   无报告不得合并。
7. **不 Mock 业务逻辑**（spec 06 §2.3）：DB/HTTP/Caddy 均真实，仅 Mock 时钟（FakeClock）
   与边界 RNG（必要时）。
8. **用例 ID 稳定**：UT-*/IT-*/E2E-*/LT-*/EC-* 编号一旦定义不可重排，新增追加，便于
   追踪与回归。
9. **压测不阻塞 PR**：压测 job 手动/nightly 触发，PR 以附带报告满足门槛；CI 仅 smoke
   验证脚本可执行。
10. **限流参数化**：`RATE_LIMIT_PER_SEC` 必须可经 env 注入，否则 e2e（串行请求）与
    压测（高并发）均会被限流污染。

---

## 12. 验收清单

> 实现者完成后逐项打勾，作为 PR 自检表。

### 12.1 Pre-req（§3）

> **Note (2026-07-25):** All prerequisites have been fulfilled in the current codebase.

- [x] Pre-req-1 时钟注入（`src/clock.rs`，`AppState.clock`，`grep Utc::now` 仅剩 SystemClock）
- [x] Pre-req-2 限流接入路由（`src/api/rate_limit.rs`，AppState 字段，429 可触发）
- [x] Pre-req-3 token 缓存续期失效 + TTL
- [x] Pre-req-4 并发注册 `ON CONFLICT`
- [x] Pre-req-5 配对码碰撞服务端重试
- [x] Pre-req-6 配对码 used 标记（一次性语义）
- [x] Pre-req-7 device_name 经 batch 落库 + 删死代码
- [x] Pre-req-8 缓存 TTL 用真实 expires_at
- [x] Pre-req-9 OpenAPI 嵌套 address 形状
- [x] Pre-req-10 device_name maxLength 64 校验

### 12.2 单元测试（§4）

- [ ] UT-config-01~04
- [ ] UT-domain-01~08
- [ ] UT-device_repo-01~13
- [ ] UT-pairing_code_repo-01~08
- [ ] UT-cache-01~10
- [ ] UT-batch-01~06
- [ ] UT-RateLimiter-01~04, UT-extract_ip-01~05, UT-AuthenticatedDevice-01~06
- [ ] UT-error-01~05, UT-gen-01~04
- [ ] UT-metrics-01~03
- [ ] `cargo llvm-cov` 整体行覆盖 ≥ 85%

### 12.3 集成测试（§5.2，L2）

- [ ] E2E-01~03（注册 upsert 三态）
- [ ] E2E-04~07（配对码四态）
- [ ] E2E-08~10（状态上报）
- [ ] E2E-11（token 续期链路）
- [ ] E2E-12~17（6 错误码体形状）
- [ ] E2E-18~19（限流）
- [ ] E2E-20（并发注册，EC-concurrency-01）
- [ ] EC-concurrency-02~04

### 12.4 e2e Caddy 通道（§5.3，L3）

- [ ] E2E-CADDY-01~06
- [ ] docker-compose healthcheck 就绪

### 12.5 edge cases（§7）

- [ ] EC-req-01~12
- [ ] EC-auth-01~09
- [ ] EC-code-01~05
- [ ] EC-sec-01~03
- [ ] EC-obs-01~06

### 12.6 压测（§6）

- [ ] `register.js` 重写（10 万设备）
- [ ] `heartbeat.js`（1014 QPS）
- [ ] `pairing_code.js`（读写混合）
- [ ] `sqlite_contention.js`（并发写）
- [ ] `capacity-report-<date>.md` 附 PR

### 12.7 CI（§8.5）

- [ ] `pairing-server` job（lint + test + coverage gate）
- [ ] `pairing-server-e2e` job（docker-compose + ignored tests）
- [ ] `pairing-server-load` job（workflow_dispatch）
- [ ] package.json scripts 更新

---

## 附录 A：用例 ID 与 spec 11 条目反向索引

> 便于回归时定位"某条 spec 要求被哪个用例覆盖"。

| spec 11 条目 | 覆盖用例 |
|-------------|---------|
| §4 时间常量 | UT-config-01 |
| §4 RATE_LIMIT | E2E-18/19, UT-RateLimiter-* |
| §4 PAIRING_CODE_RETRY | EC-code-05 |
| §5.1.1 upsert 201 | E2E-01 |
| §5.1.1 upsert 200 复用 | E2E-02 |
| §5.1.1 upsert 200 续期 | E2E-03 |
| §5.1.1 并发安全 | EC-concurrency-01 |
| §5.1.2 配对码查询 LAN 地址 | E2E-04 |
| §5.1.3 INVALID_REQUEST | E2E-12, EC-req-* |
| §5.1.3 TOKEN_INVALID | E2E-13, EC-auth-* |
| §5.1.3 DEVICE_NOT_FOUND | E2E-14 |
| §5.1.3 PAIRING_CODE_NOT_FOUND | E2E-15, EC-code-05 |
| §5.1.3 PAIRING_CODE_EXPIRED | E2E-16, EC-code-01/02 |
| §5.1.3 RATE_LIMITED | E2E-17 |
| §6.2 配对码生成+查询+过期 | E2E-04/05/06/07 |
| §7.4 状态上报 + IP 变化 | E2E-08/09/10 |
| §7.5 token 过期续期 | E2E-11, EC-auth-09 |
| §9.1 FK 孤儿 | EC-code-03 |
| §10.1 缓存命中/过期/驱逐 | UT-cache-*, E2E-06 |
| §10.2 批量写入 | UT-batch-* |
| §11.2 L3 指标 | UT-metrics-*, EC-obs-02/03/04 |
| §11.2 /health | EC-obs-01 |
| §11.3 access log 不泄 token | EC-sec-02 |
| §12.4/§13.1 Caddy 同构 | E2E-CADDY-* |
| §13.3 压测 ①②③④ | register/heartbeat/pairing_code/sqlite_contention |
| §14.1 token 过期 | E2E-11, EC-auth-09, UT-cache-04/05 |
| §14.4 device_id 公开 | EC-sec-03 |

---

## 附录 B：现有 e2e/api.rs 迁移映射

> 现有 `tests/e2e/api.rs` 的 6 个用例如何迁移到新结构。

| 现有用例 | 迁移目标 | 备注 |
|---------|---------|------|
| `register_new_device_returns_201` | `integration/register.rs` E2E-01 | 保留 |
| `register_existing_device_returns_200_reuses_token` | `integration/register.rs` E2E-02 | 保留 |
| `pairing_code_issue_and_lookup` | `integration/pairing_codes.rs` E2E-04 | **断言改嵌套 address**（Pre-req-9） |
| `pairing_code_lookup_missing_returns_404` | `integration/pairing_codes.rs` E2E-05 | 保留 |
| `status_report_requires_valid_token` | `integration/status.rs` E2E-08（+ auth 部分） | 拆分：无 token→EC-auth-01，有 token→E2E-08 |
| `invalid_request_body_returns_400` | `integration/errors.rs` E2E-12 | 强化：断言 error.code==INVALID_REQUEST |

---

> **本规范结束。** 实现者按 §10 实施顺序、对照 §12 验收清单逐项落地。任何与 spec 11
> 或 OpenAPI 的冲突，以 spec 11 + OpenAPI 为准并反向更新本规范。
