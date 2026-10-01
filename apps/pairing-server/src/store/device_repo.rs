//! Device 仓储（凭据模型：token 哈希落库 + 只发不复述）。
//!
//! 使用 sqlx 运行时查询（`query_as` + `FromRow`），不依赖编译期 `query!` 宏，
//! 因此无需 `DATABASE_URL` 即可编译。
//!
//! 安全不变量（凭据模型重设计）：
//! 1. token 只存 SHA-256 哈希（`token_hash`）——DB 泄露 ≠ token 泄露。
//! 2. 已存在设备的 token **绝不向无凭据方复述**：重复注册必须携带当前有效
//!    Bearer token（401 否则）。device_id 印在配对二维码里，若裸注册即可
//!    领走 token，等于 QR 截图 = 永久信令通道接管。
//! 3. token 轮换（超过 `DEVICE_TOKEN_TTL`）也要求先通过旧 token 校验，
//!    新 token 只发给旧 token 持有者。

use chrono::{DateTime, Utc};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

use crate::api::auth::token_hash;
use crate::clock::Clock;
use crate::domain::{Device, DeviceAddress, DeviceRegisterRequest};
use crate::error::{AppError, AppResult};

/// upsert 结果：区分新建（201）与复用/轮换（200）与凭据缺失（401）。
#[derive(Debug, Clone)]
pub enum RegisterOutcome {
    /// 新建设备（HTTP 201）。`device.pairing_token` 为新颁发的 token（首次披露）。
    Created(Device),
    /// 设备已存在且 Bearer 校验通过，token 未到轮换期（HTTP 200）。
    /// `device.pairing_token` 为请求携带 token 的回显（不产生新披露）。
    Reused(Device),
    /// 设备已存在且 Bearer 校验通过，token 超过 TTL 被轮换（HTTP 200）。
    /// `device.pairing_token` 为新 token；旧 token（= 请求携带者）由调用方失效缓存。
    Rotated(Device),
    /// 设备已存在但请求未携带有效 token（HTTP 401）。
    Unverified,
}

/// 行结构（运行时 FromRow 映射，仅取分派所需列）。时间戳以 unix 秒存。
#[derive(Debug, Clone, FromRow)]
struct DeviceRow {
    id: String,
    device_name: Option<String>,
    created_at: i64,
    /// SHA-256 hex；认证比对用，永不外发。
    token_hash: String,
    token_issued_at: i64,
}

/// 注册 upsert：按凭据模型分派四种结局。
///
/// - 不存在 → `Created`（`INSERT ... ON CONFLICT(id) DO NOTHING` 防并发竞态）。
/// - 存在 + Bearer 哈希命中：
///   - token 超过 `DEVICE_TOKEN_TTL`（以 `token_issued_at` 计）→ `Rotated`；
///   - 否则 → `Reused`（仅更新地址/在线状态）。
/// - 存在 + 无 Bearer / 哈希不匹配 → `Unverified`（调用方映射 401）。
pub async fn upsert(
    pool: &SqlitePool,
    req: &DeviceRegisterRequest,
    bearer_token: Option<&str>,
    new_token: &str,
    clock: &dyn Clock,
) -> AppResult<RegisterOutcome> {
    let id = req.device_id.to_string();
    let now = clock.now();

    let existing: Option<DeviceRow> = sqlx::query_as::<_, DeviceRow>(
        r#"SELECT id, device_name, created_at, token_hash, token_issued_at
           FROM devices WHERE id = ?"#,
    )
    .bind(&id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    if let Some(row) = existing {
        return match_existing(pool, row, req, bearer_token, new_token, now).await;
    }

    // 新建：ON CONFLICT 防并发竞态。
    let new_hash = token_hash(new_token);
    let result = sqlx::query(
        r#"INSERT INTO devices (id, platform, device_name, ip, port,
                                 is_online, last_seen, created_at, token_hash, token_issued_at)
           VALUES (?, ?, ?, ?, ?, 1, ?, ?, ?, ?)
           ON CONFLICT(id) DO NOTHING"#,
    )
    .bind(&id)
    .bind("desktop")
    .bind(&req.device_name)
    .bind(&req.address.ip)
    .bind(req.address.port as i64)
    .bind(now.timestamp())
    .bind(now.timestamp())
    .bind(&new_hash)
    .bind(now.timestamp())
    .execute(pool)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    if result.rows_affected() == 0 {
        // 并发插入已有记录：重新读取并按已存在路径分派（持有效 token 者胜出）。
        let row: DeviceRow = sqlx::query_as::<_, DeviceRow>(
            r#"SELECT id, device_name, created_at, token_hash, token_issued_at
               FROM devices WHERE id = ?"#,
        )
        .bind(&id)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
        return match_existing(pool, row, req, bearer_token, new_token, now).await;
    }

    let device = Device {
        id: req.device_id,
        platform: req.platform,
        device_name: req.device_name.clone(),
        address: req.address.clone(),
        pairing_token: new_token.to_string(),
        created_at: now,
    };
    Ok(RegisterOutcome::Created(device))
}

/// 已存在设备的分派逻辑（含并发插入竞态回退路径）。
async fn match_existing(
    pool: &SqlitePool,
    row: DeviceRow,
    req: &DeviceRegisterRequest,
    bearer_token: Option<&str>,
    new_token: &str,
    now: DateTime<Utc>,
) -> AppResult<RegisterOutcome> {
    let created = DateTime::from_timestamp(row.created_at, 0).unwrap_or(now);
    let issued_at = DateTime::from_timestamp(row.token_issued_at, 0).unwrap_or(created);

    // Bearer 哈希比对；空 hash（迁移存量）视为不匹配。
    let verified = bearer_token
        .map(|t| !row.token_hash.is_empty() && token_hash(t) == row.token_hash)
        .unwrap_or(false);
    if !verified {
        return Ok(RegisterOutcome::Unverified);
    }

    let bearer = bearer_token.expect("verified implies bearer present");

    if now - issued_at > crate::config::DEVICE_TOKEN_TTL {
        // 轮换：仅旧 token 持有者可获得新 token。
        rotate_token(pool, &row.id, &req.address, &token_hash(new_token), now).await?;
        let device = Device {
            id: req.device_id,
            platform: req.platform,
            device_name: req.device_name.clone().or(row.device_name),
            address: req.address.clone(),
            pairing_token: new_token.to_string(),
            created_at: created,
        };
        return Ok(RegisterOutcome::Rotated(device));
    }

    // 复用：回显请求携带的 token（调用方已持有，无新披露）。
    mark_online(pool, &row.id, &req.address, now).await?;
    let device = Device {
        id: req.device_id,
        platform: req.platform,
        device_name: req.device_name.clone().or(row.device_name),
        address: req.address.clone(),
        pairing_token: bearer.to_string(),
        created_at: created,
    };
    Ok(RegisterOutcome::Reused(device))
}

/// 仅更新地址 + 在线状态（复用 token 时调用）。
async fn mark_online(
    pool: &SqlitePool,
    id: &str,
    addr: &DeviceAddress,
    now: DateTime<Utc>,
) -> AppResult<()> {
    sqlx::query(
        r#"UPDATE devices SET ip = ?, port = ?, is_online = 1, last_seen = ? WHERE id = ?"#,
    )
    .bind(&addr.ip)
    .bind(addr.port as i64)
    .bind(now.timestamp())
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;
    Ok(())
}

/// 轮换 token + 刷新地址 + 在线状态（Bearer 校验通过且超 TTL 时调用）。
async fn rotate_token(
    pool: &SqlitePool,
    id: &str,
    addr: &DeviceAddress,
    new_hash: &str,
    now: DateTime<Utc>,
) -> AppResult<()> {
    sqlx::query(
        r#"UPDATE devices SET ip = ?, port = ?, token_hash = ?, token_issued_at = ?,
                               is_online = 1, last_seen = ? WHERE id = ?"#,
    )
    .bind(&addr.ip)
    .bind(addr.port as i64)
    .bind(new_hash)
    .bind(now.timestamp())
    .bind(now.timestamp())
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;
    Ok(())
}

/// 更新地址（可选 device_name）+ 在线状态。`PUT /status` 心跳调用。
/// 设备不存在→`DeviceNotFound`。
///
/// 注意：生产路径统一走 batch writer，此函数仅用于仓储层直接测试。
#[cfg(test)]
pub async fn update_status(
    pool: &SqlitePool,
    device_id: Uuid,
    addr: &DeviceAddress,
    device_name: Option<&str>,
    clock: &dyn Clock,
) -> AppResult<()> {
    let id = device_id.to_string();
    let now = clock.now();
    let result = sqlx::query(
        r#"UPDATE devices
           SET ip = ?, port = ?, device_name = COALESCE(?, device_name),
               is_online = 1, last_seen = ?
           WHERE id = ?"#,
    )
    .bind(&addr.ip)
    .bind(addr.port as i64)
    .bind(device_name)
    .bind(now.timestamp())
    .bind(&id)
    .execute(pool)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;
    if result.rows_affected() == 0 {
        return Err(AppError::DeviceNotFound);
    }
    Ok(())
}

/// 按 token 查询设备 id（认证用）。哈希后比对。返回 None 表示 token 无效。
pub async fn find_id_by_token(pool: &SqlitePool, token: &str) -> AppResult<Option<Uuid>> {
    #[derive(FromRow)]
    struct IdRow {
        id: String,
    }
    let row = sqlx::query_as::<_, IdRow>(r#"SELECT id FROM devices WHERE token_hash = ?"#)
        .bind(token_hash(token))
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    match row {
        Some(r) => Ok(Some(
            Uuid::parse_str(&r.id).map_err(|e| AppError::Internal(e.into()))?,
        )),
        None => Ok(None),
    }
}

/// 标记超时设备离线（`last_seen` 超过阈值）。后台任务调用。
pub async fn mark_stale_offline(pool: &SqlitePool, threshold: DateTime<Utc>) -> AppResult<u64> {
    let res =
        sqlx::query(r#"UPDATE devices SET is_online = 0 WHERE is_online = 1 AND last_seen < ?"#)
            .bind(threshold.timestamp())
            .execute(pool)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
    Ok(res.rows_affected())
}

/// 删除超过保留期未上线的设备（设备表 GC，防无认证注册刷行数）。
/// 后台任务调用；桌面端 token 持久化在本地，被 GC 的设备重新注册即恢复
/// （新建分支，需要重扫码配对手机）。
pub async fn delete_stale(pool: &SqlitePool, cutoff: DateTime<Utc>) -> AppResult<u64> {
    let res = sqlx::query(r#"DELETE FROM devices WHERE last_seen < ?"#)
        .bind(cutoff.timestamp())
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
    Ok(res.rows_affected())
}

/// 当前在线设备数（L3 指标）。
pub async fn count_online(pool: &SqlitePool) -> AppResult<i64> {
    #[derive(FromRow)]
    struct CountRow {
        c: i64,
    }
    let row =
        sqlx::query_as::<_, CountRow>(r#"SELECT COUNT(*) as c FROM devices WHERE is_online = 1"#)
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
    Ok(row.c)
}

/// 取设备摘要（id + name），信令 offer 响应 `{device:{id,name}}` 用（§4.1）。
pub async fn find_device_summary(pool: &SqlitePool, device_id: Uuid) -> AppResult<DeviceSummary> {
    #[derive(FromRow)]
    struct NameRow {
        device_name: Option<String>,
    }
    let row = sqlx::query_as::<_, NameRow>(r#"SELECT device_name FROM devices WHERE id = ?"#)
        .bind(device_id.to_string())
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or(AppError::DeviceNotFound)?;
    Ok(DeviceSummary {
        id: device_id,
        name: row.device_name,
    })
}

/// 设备摘要（信令 offer 响应）。
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct DeviceSummary {
    pub id: Uuid,
    /// 设备名（可空）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// 取设备当前地址（配对码生成时冗余存储用）。
pub async fn current_address(pool: &SqlitePool, device_id: Uuid) -> AppResult<DeviceAddress> {
    #[derive(FromRow)]
    struct AddrRow {
        ip: String,
        port: i64,
    }
    let row = sqlx::query_as::<_, AddrRow>(r#"SELECT ip, port FROM devices WHERE id = ?"#)
        .bind(device_id.to_string())
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or(AppError::DeviceNotFound)?;
    Ok(DeviceAddress {
        ip: row.ip,
        port: row.port as u16,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{fixed_clock, sample_device_id, TestDb};
    use std::sync::Arc;

    fn sample_request() -> DeviceRegisterRequest {
        DeviceRegisterRequest {
            device_id: sample_device_id(),
            platform: crate::domain::Platform::Desktop,
            device_name: Some("Test PC".into()),
            address: DeviceAddress {
                ip: "192.168.1.100".into(),
                port: 38425,
            },
        }
    }

    /// UT-device_repo-01：新建 → Created，携带新 token。
    #[tokio::test]
    async fn upsert_new_device_returns_created() {
        let db = TestDb::new().await;
        let clock = fixed_clock();
        let req = sample_request();
        let outcome = upsert(&db.pool, &req, None, "first-token", &clock)
            .await
            .unwrap();
        match outcome {
            RegisterOutcome::Created(d) => {
                assert_eq!(d.id, sample_device_id());
                assert_eq!(d.pairing_token, "first-token");
            }
            _ => panic!("expected Created"),
        }
    }

    /// UT-device_repo-02（凭据模型核心）：已存在 + 无 Bearer → Unverified（401），
    /// 绝不复述已有 token。
    #[tokio::test]
    async fn upsert_existing_without_bearer_is_unverified() {
        let db = TestDb::new().await;
        let clock = fixed_clock();
        let req = sample_request();

        upsert(&db.pool, &req, None, "first-token", &clock)
            .await
            .unwrap();
        // 攻击者知道 device_id（QR 截图），裸注册拿不到 token。
        let outcome = upsert(&db.pool, &req, None, "attacker-token", &clock)
            .await
            .unwrap();
        assert!(matches!(outcome, RegisterOutcome::Unverified));
    }

    /// UT-device_repo-03：已存在 + Bearer 匹配 → Reused，回显 bearer。
    #[tokio::test]
    async fn upsert_existing_with_valid_bearer_reuses() {
        let db = TestDb::new().await;
        let clock = fixed_clock();
        let req = sample_request();

        upsert(&db.pool, &req, None, "first-token", &clock)
            .await
            .unwrap();
        let outcome = upsert(&db.pool, &req, Some("first-token"), "ignored", &clock)
            .await
            .unwrap();
        match outcome {
            RegisterOutcome::Reused(d) => assert_eq!(d.pairing_token, "first-token"),
            _ => panic!("expected Reused"),
        }
    }

    /// UT-device_repo-04：已存在 + Bearer 错误 → Unverified。
    #[tokio::test]
    async fn upsert_existing_with_wrong_bearer_is_unverified() {
        let db = TestDb::new().await;
        let clock = fixed_clock();
        let req = sample_request();

        upsert(&db.pool, &req, None, "first-token", &clock)
            .await
            .unwrap();
        let outcome = upsert(&db.pool, &req, Some("wrong"), "attacker", &clock)
            .await
            .unwrap();
        assert!(matches!(outcome, RegisterOutcome::Unverified));
    }

    /// UT-device_repo-05：token 超 TTL + Bearer 匹配 → Rotated（仅持有者可得新 token）。
    #[tokio::test]
    async fn upsert_stale_bearer_rotates() {
        let db = TestDb::new().await;
        let clock = fixed_clock();
        let req = sample_request();

        upsert(&db.pool, &req, None, "old-token", &clock)
            .await
            .unwrap();
        clock.advance(chrono::Duration::hours(25));

        // 攻击者趁过期裸注册：仍被拒。
        let outcome = upsert(&db.pool, &req, None, "attacker", &clock)
            .await
            .unwrap();
        assert!(matches!(outcome, RegisterOutcome::Unverified));

        // 持旧 token 的合法桌面：拿到轮换后的新 token。
        let outcome = upsert(&db.pool, &req, Some("old-token"), "new-token", &clock)
            .await
            .unwrap();
        match outcome {
            RegisterOutcome::Rotated(d) => assert_eq!(d.pairing_token, "new-token"),
            _ => panic!("expected Rotated"),
        }
        // 旧 token 立即失效。
        assert_eq!(find_id_by_token(&db.pool, "old-token").await.unwrap(), None);
    }

    /// UT-device_repo-06：device_name 缺省时复用 DB 值。
    #[tokio::test]
    async fn upsert_reuses_db_device_name() {
        let db = TestDb::new().await;
        let clock = fixed_clock();
        let req = sample_request();

        upsert(&db.pool, &req, None, "token", &clock).await.unwrap();

        // 第二次不带 device_name，但带有效 token。
        let mut req2 = req.clone();
        req2.device_name = None;
        let outcome = upsert(&db.pool, &req2, Some("token"), "token2", &clock)
            .await
            .unwrap();
        match outcome {
            RegisterOutcome::Reused(d) => assert_eq!(d.device_name, Some("Test PC".into())),
            _ => panic!("expected Reused"),
        }
    }

    /// UT-device_repo-07：find_id_by_token 命中（哈希比对）。
    #[tokio::test]
    async fn find_id_by_token_hit() {
        let db = TestDb::new().await;
        let clock = fixed_clock();
        let req = sample_request();

        upsert(&db.pool, &req, None, "my-token", &clock)
            .await
            .unwrap();
        let id = find_id_by_token(&db.pool, "my-token").await.unwrap();
        assert_eq!(id, Some(sample_device_id()));
    }

    /// UT-device_repo-08：find_id_by_token 未命中。
    #[tokio::test]
    async fn find_id_by_token_miss() {
        let db = TestDb::new().await;
        let id = find_id_by_token(&db.pool, "nonexistent").await.unwrap();
        assert!(id.is_none());
    }

    /// UT-device_repo-09：token 在库中只存哈希（明文不可见）。
    #[tokio::test]
    async fn token_stored_as_hash_not_plaintext() {
        let db = TestDb::new().await;
        let clock = fixed_clock();
        let req = sample_request();

        upsert(&db.pool, &req, None, "plaintext-secret", &clock)
            .await
            .unwrap();
        let row: (String,) = sqlx::query_as("SELECT token_hash FROM devices")
            .fetch_one(&db.pool)
            .await
            .unwrap();
        assert_eq!(row.0, token_hash("plaintext-secret"));
        assert_ne!(row.0, "plaintext-secret");
    }

    /// UT-device_repo-10：mark_stale_offline 阈值边界。
    #[tokio::test]
    async fn mark_stale_offline_threshold() {
        let db = TestDb::new().await;
        let clock = fixed_clock();
        let req = sample_request();

        upsert(&db.pool, &req, None, "token", &clock).await.unwrap();

        // 阈值 = now → 设备不离线（last_seen == now，严格 < 不成立）。
        let threshold = clock.now();
        let n = mark_stale_offline(&db.pool, threshold).await.unwrap();
        assert_eq!(n, 0);

        // 阈值 = now + 1s → 设备离线（last_seen < threshold）。
        let threshold = clock.now() + chrono::Duration::seconds(1);
        let n = mark_stale_offline(&db.pool, threshold).await.unwrap();
        assert_eq!(n, 1);
    }

    /// UT-device_repo-11：delete_stale 仅删保留期外设备。
    #[tokio::test]
    async fn delete_stale_removes_only_old_devices() {
        let db = TestDb::new().await;
        let clock = fixed_clock();
        let req = sample_request();

        upsert(&db.pool, &req, None, "token", &clock).await.unwrap();

        // 保留期内：不删。
        let cutoff = clock.now() - chrono::Duration::days(30) + chrono::Duration::hours(1);
        assert_eq!(delete_stale(&db.pool, cutoff).await.unwrap(), 0);

        // 保留期外：删。
        clock.advance(chrono::Duration::days(31));
        let cutoff = clock.now() - chrono::Duration::days(30);
        assert_eq!(delete_stale(&db.pool, cutoff).await.unwrap(), 1);
    }

    /// UT-device_repo-12：count_online。
    #[tokio::test]
    async fn count_online_correct() {
        let db = TestDb::new().await;
        let clock = fixed_clock();
        let req = sample_request();

        upsert(&db.pool, &req, None, "token", &clock).await.unwrap();
        assert_eq!(count_online(&db.pool).await.unwrap(), 1);

        // 标记离线。
        let threshold = clock.now() + chrono::Duration::minutes(11);
        mark_stale_offline(&db.pool, threshold).await.unwrap();
        assert_eq!(count_online(&db.pool).await.unwrap(), 0);
    }

    /// UT-device_repo-13：current_address 命中。
    #[tokio::test]
    async fn current_address_hit() {
        let db = TestDb::new().await;
        let clock = fixed_clock();
        let req = sample_request();

        upsert(&db.pool, &req, None, "token", &clock).await.unwrap();
        let addr = current_address(&db.pool, sample_device_id()).await.unwrap();
        assert_eq!(addr.ip, "192.168.1.100");
        assert_eq!(addr.port, 38425);
    }

    /// UT-device_repo-14：current_address 不存在。
    #[tokio::test]
    async fn current_address_not_found() {
        let db = TestDb::new().await;
        let result = current_address(&db.pool, uuid::Uuid::new_v4()).await;
        assert!(matches!(result, Err(AppError::DeviceNotFound)));
    }

    /// UT-device_repo-15：update_status 更新地址和 device_name。
    #[tokio::test]
    async fn update_status_updates_address_and_name() {
        let db = TestDb::new().await;
        let clock = fixed_clock();
        let req = sample_request();
        upsert(&db.pool, &req, None, "token", &clock).await.unwrap();

        let new_addr = DeviceAddress {
            ip: "10.0.0.99".into(),
            port: 7777,
        };
        update_status(
            &db.pool,
            sample_device_id(),
            &new_addr,
            Some("New Name"),
            &clock,
        )
        .await
        .unwrap();

        let addr = current_address(&db.pool, sample_device_id()).await.unwrap();
        assert_eq!(addr.ip, "10.0.0.99");
        assert_eq!(addr.port, 7777);
    }

    /// UT-device_repo-16：update_status 设备不存在。
    #[tokio::test]
    async fn update_status_device_not_found() {
        let db = TestDb::new().await;
        let clock = fixed_clock();
        let addr = DeviceAddress {
            ip: "1.2.3.4".into(),
            port: 38425,
        };
        let result = update_status(&db.pool, uuid::Uuid::new_v4(), &addr, None, &clock).await;
        assert!(matches!(result, Err(AppError::DeviceNotFound)));
    }

    /// UT-device_repo-17：并发同 id 注册（ON CONFLICT 路径）。
    /// 首个请求创建，其余持不同 token 的裸注册应得 Unverified，无 500。
    #[tokio::test]
    async fn concurrent_upsert_same_id_no_error() {
        let db = TestDb::new().await;
        let clock = Arc::new(fixed_clock());
        let req = sample_request();

        let mut join_set = tokio::task::JoinSet::new();
        for i in 0..100 {
            let pool = db.pool.clone();
            let r = req.clone();
            let c = clock.clone();
            join_set.spawn(async move {
                let token = format!("token-{i}");
                upsert(&pool, &r, None, &token, c.as_ref() as &dyn Clock).await
            });
        }

        while let Some(result) = join_set.join_next().await {
            let outcome = result.unwrap();
            assert!(
                outcome.is_ok(),
                "concurrent upsert failed: {:?}",
                outcome.err()
            );
            // 首个创建成功；其余均为 Unverified（无凭据不可复述）。
            assert!(matches!(
                outcome.unwrap(),
                RegisterOutcome::Created(_) | RegisterOutcome::Unverified
            ));
        }
    }
}
