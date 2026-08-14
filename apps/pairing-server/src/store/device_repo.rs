//! Device 仓储（spec 11 §5.1.1 幂等 upsert）。
//!
//! 使用 sqlx 运行时查询（`query_as` + `FromRow`），不依赖编译期 `query!` 宏，
//! 因此无需 `DATABASE_URL` 即可编译。

use chrono::{DateTime, Utc};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

use crate::clock::Clock;
use crate::domain::{Device, DeviceAddress, DeviceRegisterRequest};
use crate::error::{AppError, AppResult};

/// upsert 结果：区分新建（201）与复用/续期（200），供 handler 选择状态码。
#[derive(Debug, Clone)]
pub enum RegisterOutcome {
    /// 新建设备（HTTP 201）。
    Created(Device),
    /// 复用/续期设备（HTTP 200）。续期时携带旧 token 供缓存失效。
    Reused {
        device: Device,
        old_token: Option<String>,
    },
}

/// 行结构（运行时 FromRow 映射）。时间戳以 unix 秒存。
/// 部分 field 仅用于反序列化占位（sqlx 需要完整列映射），允许 dead_code。
#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
struct DeviceRow {
    id: String,
    platform: String,
    device_name: Option<String>,
    ip: String,
    port: i64,
    pairing_token: String,
    is_online: i64,
    last_seen: i64,
    created_at: i64,
}

/// 幂等 upsert：device_id 不存在→新建；存在且 token 有效→复用；过期→刷新。
///
/// Pre-req-4：新建分支使用 `INSERT ... ON CONFLICT(id) DO NOTHING` 防并发竞态。
pub async fn upsert(
    pool: &SqlitePool,
    req: &DeviceRegisterRequest,
    new_token: &str,
    clock: &dyn Clock,
) -> AppResult<RegisterOutcome> {
    let id = req.device_id.to_string();
    let now = clock.now();

    // 先尝试 SELECT（大多数请求走复用路径）。
    let existing: Option<DeviceRow> = sqlx::query_as::<_, DeviceRow>(
        r#"SELECT id, platform, device_name, ip, port, pairing_token,
                  is_online, last_seen, created_at
           FROM devices WHERE id = ?"#,
    )
    .bind(&id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    if let Some(row) = existing {
        let created = DateTime::from_timestamp(row.created_at, 0).unwrap_or(now);
        let token_expired = now - created > crate::config::DEVICE_TOKEN_TTL;

        if token_expired {
            // 续期：刷新 token + 地址 + 在线状态。返回旧 token 供缓存失效。
            let old_token = row.pairing_token.clone();
            refresh(pool, &id, &req.address, new_token, now).await?;
            let device = Device {
                id: req.device_id,
                platform: req.platform,
                device_name: req.device_name.clone().or(row.device_name),
                address: req.address.clone(),
                pairing_token: new_token.to_string(),
                created_at: created,
            };
            return Ok(RegisterOutcome::Reused {
                device,
                old_token: Some(old_token),
            });
        }

        // 复用原 token。
        mark_online(pool, &id, &req.address, now).await?;
        let device = Device {
            id: req.device_id,
            platform: req.platform,
            device_name: req.device_name.clone().or(row.device_name),
            address: req.address.clone(),
            pairing_token: row.pairing_token,
            created_at: created,
        };
        return Ok(RegisterOutcome::Reused {
            device,
            old_token: None,
        });
    }

    // 新建：Pre-req-4 使用 ON CONFLICT 防并发竞态。
    let created_at = now.timestamp();
    let last_seen = now.timestamp();
    let result = sqlx::query(
        r#"INSERT INTO devices (id, platform, device_name, ip, port, pairing_token,
                                 is_online, last_seen, created_at)
           VALUES (?, ?, ?, ?, ?, ?, 1, ?, ?)
           ON CONFLICT(id) DO NOTHING"#,
    )
    .bind(&id)
    .bind("desktop")
    .bind(&req.device_name)
    .bind(&req.address.ip)
    .bind(req.address.port as i64)
    .bind(new_token)
    .bind(last_seen)
    .bind(created_at)
    .execute(pool)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    if result.rows_affected() == 0 {
        // 并发插入已有记录，回退到复用分支。
        let row: DeviceRow = sqlx::query_as::<_, DeviceRow>(
            r#"SELECT id, platform, device_name, ip, port, pairing_token,
                      is_online, last_seen, created_at
               FROM devices WHERE id = ?"#,
        )
        .bind(&id)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        let created = DateTime::from_timestamp(row.created_at, 0).unwrap_or(now);
        mark_online(pool, &id, &req.address, now).await?;
        let device = Device {
            id: req.device_id,
            platform: req.platform,
            device_name: req.device_name.clone().or(row.device_name),
            address: req.address.clone(),
            pairing_token: row.pairing_token,
            created_at: created,
        };
        return Ok(RegisterOutcome::Reused {
            device,
            old_token: None,
        });
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

/// 刷新 token + 地址 + 在线状态（续期时调用）。
async fn refresh(
    pool: &SqlitePool,
    id: &str,
    addr: &DeviceAddress,
    new_token: &str,
    now: DateTime<Utc>,
) -> AppResult<()> {
    sqlx::query(
        r#"UPDATE devices SET ip = ?, port = ?, pairing_token = ?, is_online = 1,
                               last_seen = ? WHERE id = ?"#,
    )
    .bind(&addr.ip)
    .bind(addr.port as i64)
    .bind(new_token)
    .bind(now.timestamp())
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;
    Ok(())
}

/// 更新地址（可选 device_name）+ 在线状态。`PUT /status` 心跳调用。
/// 返回设备当前 token（用于认证校验）。设备不存在→`DeviceNotFound`。
///
/// 注意：生产路径统一走 batch writer，此函数仅用于仓储层直接测试。
#[cfg(test)]
pub async fn update_status(
    pool: &SqlitePool,
    device_id: Uuid,
    addr: &DeviceAddress,
    device_name: Option<&str>,
    clock: &dyn Clock,
) -> AppResult<String> {
    let id = device_id.to_string();
    let now = clock.now();

    #[derive(FromRow)]
    struct TokenRow {
        pairing_token: String,
    }
    let row = sqlx::query_as::<_, TokenRow>(
        r#"UPDATE devices
           SET ip = ?, port = ?, device_name = COALESCE(?, device_name),
               is_online = 1, last_seen = ?
           WHERE id = ?
           RETURNING pairing_token"#,
    )
    .bind(&addr.ip)
    .bind(addr.port as i64)
    .bind(device_name)
    .bind(now.timestamp())
    .bind(&id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Internal(e.into()))?
    .ok_or(AppError::DeviceNotFound)?;

    Ok(row.pairing_token)
}

/// 按 token 查询设备 id（认证用）。返回 None 表示 token 无效。
pub async fn find_id_by_token(pool: &SqlitePool, token: &str) -> AppResult<Option<Uuid>> {
    #[derive(FromRow)]
    struct IdRow {
        id: String,
    }
    let row = sqlx::query_as::<_, IdRow>(r#"SELECT id FROM devices WHERE pairing_token = ?"#)
        .bind(token)
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

    /// UT-device_repo-01：新建 → Created。
    #[tokio::test]
    async fn upsert_new_device_returns_created() {
        let db = TestDb::new().await;
        let clock = fixed_clock();
        let req = sample_request();
        let outcome = upsert(&db.pool, &req, "test-token", &clock).await.unwrap();
        match outcome {
            RegisterOutcome::Created(d) => {
                assert_eq!(d.id, sample_device_id());
                assert_eq!(d.pairing_token, "test-token");
            }
            _ => panic!("expected Created"),
        }
    }

    /// UT-device_repo-02：复用（token 有效）→ Reused 原 token。
    #[tokio::test]
    async fn upsert_existing_reuses_token() {
        let db = TestDb::new().await;
        let clock = fixed_clock();
        let req = sample_request();

        upsert(&db.pool, &req, "first-token", &clock).await.unwrap();
        let outcome = upsert(&db.pool, &req, "second-token", &clock)
            .await
            .unwrap();
        match outcome {
            RegisterOutcome::Reused { device, old_token } => {
                assert_eq!(device.pairing_token, "first-token");
                assert!(old_token.is_none());
            }
            _ => panic!("expected Reused"),
        }
    }

    /// UT-device_repo-03：续期（token 过期）→ Reused 新 token。
    #[tokio::test]
    async fn upsert_expired_token_refreshes() {
        let db = TestDb::new().await;
        let clock = fixed_clock();
        let req = sample_request();

        upsert(&db.pool, &req, "old-token", &clock).await.unwrap();

        // 推进 25 小时（超过 DEVICE_TOKEN_TTL 24h）。
        clock.advance(chrono::Duration::hours(25));

        let outcome = upsert(&db.pool, &req, "new-token", &clock).await.unwrap();
        match outcome {
            RegisterOutcome::Reused { device, old_token } => {
                assert_eq!(device.pairing_token, "new-token");
                assert_eq!(old_token, Some("old-token".to_string()));
            }
            _ => panic!("expected Reused with new token"),
        }
    }

    /// UT-device_repo-04：device_name 缺省时复用 DB 值。
    #[tokio::test]
    async fn upsert_reuses_db_device_name() {
        let db = TestDb::new().await;
        let clock = fixed_clock();
        let req = sample_request();

        upsert(&db.pool, &req, "token", &clock).await.unwrap();

        // 第二次不带 device_name。
        let mut req2 = req.clone();
        req2.device_name = None;
        let outcome = upsert(&db.pool, &req2, "token2", &clock).await.unwrap();
        match outcome {
            RegisterOutcome::Reused { device, .. } => {
                assert_eq!(device.device_name, Some("Test PC".into()));
            }
            _ => panic!("expected Reused"),
        }
    }

    /// UT-device_repo-05：mark_online 更新地址+在线。
    #[tokio::test]
    async fn mark_online_updates_address() {
        let db = TestDb::new().await;
        let clock = fixed_clock();
        let req = sample_request();

        upsert(&db.pool, &req, "token", &clock).await.unwrap();

        let new_addr = DeviceAddress {
            ip: "10.0.0.1".into(),
            port: 9999,
        };
        mark_online(
            &db.pool,
            &sample_device_id().to_string(),
            &new_addr,
            clock.now(),
        )
        .await
        .unwrap();

        let addr = current_address(&db.pool, sample_device_id()).await.unwrap();
        assert_eq!(addr.ip, "10.0.0.1");
        assert_eq!(addr.port, 9999);
    }

    /// UT-device_repo-06：find_id_by_token 命中。
    #[tokio::test]
    async fn find_id_by_token_hit() {
        let db = TestDb::new().await;
        let clock = fixed_clock();
        let req = sample_request();

        upsert(&db.pool, &req, "my-token", &clock).await.unwrap();
        let id = find_id_by_token(&db.pool, "my-token").await.unwrap();
        assert_eq!(id, Some(sample_device_id()));
    }

    /// UT-device_repo-07：find_id_by_token 未命中。
    #[tokio::test]
    async fn find_id_by_token_miss() {
        let db = TestDb::new().await;
        let id = find_id_by_token(&db.pool, "nonexistent").await.unwrap();
        assert!(id.is_none());
    }

    /// UT-device_repo-09：mark_stale_offline 阈值边界。
    #[tokio::test]
    async fn mark_stale_offline_threshold() {
        let db = TestDb::new().await;
        let clock = fixed_clock();
        let req = sample_request();

        upsert(&db.pool, &req, "token", &clock).await.unwrap();

        // 阈值 = now → 设备不离线（last_seen == now，严格 < 不成立）。
        let threshold = clock.now();
        let n = mark_stale_offline(&db.pool, threshold).await.unwrap();
        assert_eq!(n, 0);

        // 阈值 = now + 1s → 设备离线（last_seen < threshold）。
        let threshold = clock.now() + chrono::Duration::seconds(1);
        let n = mark_stale_offline(&db.pool, threshold).await.unwrap();
        assert_eq!(n, 1);
    }

    /// UT-device_repo-10：count_online。
    #[tokio::test]
    async fn count_online_correct() {
        let db = TestDb::new().await;
        let clock = fixed_clock();
        let req = sample_request();

        upsert(&db.pool, &req, "token", &clock).await.unwrap();
        assert_eq!(count_online(&db.pool).await.unwrap(), 1);

        // 标记离线。
        let threshold = clock.now() + chrono::Duration::minutes(11);
        mark_stale_offline(&db.pool, threshold).await.unwrap();
        assert_eq!(count_online(&db.pool).await.unwrap(), 0);
    }

    /// UT-device_repo-11：current_address 命中。
    #[tokio::test]
    async fn current_address_hit() {
        let db = TestDb::new().await;
        let clock = fixed_clock();
        let req = sample_request();

        upsert(&db.pool, &req, "token", &clock).await.unwrap();
        let addr = current_address(&db.pool, sample_device_id()).await.unwrap();
        assert_eq!(addr.ip, "192.168.1.100");
        assert_eq!(addr.port, 38425);
    }

    /// UT-device_repo-12：current_address 不存在。
    #[tokio::test]
    async fn current_address_not_found() {
        let db = TestDb::new().await;
        let result = current_address(&db.pool, uuid::Uuid::new_v4()).await;
        assert!(matches!(result, Err(AppError::DeviceNotFound)));
    }

    /// UT-device_repo-14：update_status 更新地址和 device_name。
    #[tokio::test]
    async fn update_status_updates_address_and_name() {
        let db = TestDb::new().await;
        let clock = fixed_clock();
        let req = sample_request();
        upsert(&db.pool, &req, "token", &clock).await.unwrap();

        let new_addr = DeviceAddress {
            ip: "10.0.0.99".into(),
            port: 7777,
        };
        let returned_token = update_status(
            &db.pool,
            sample_device_id(),
            &new_addr,
            Some("New Name"),
            &clock,
        )
        .await
        .unwrap();
        assert_eq!(returned_token, "token");

        // 验证地址已更新。
        let addr = current_address(&db.pool, sample_device_id()).await.unwrap();
        assert_eq!(addr.ip, "10.0.0.99");
        assert_eq!(addr.port, 7777);
    }

    /// UT-device_repo-15：update_status 设备不存在。
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

    /// UT-device_repo-16：update_status device_name=None 不覆盖原值。
    #[tokio::test]
    async fn update_status_none_name_preserves_original() {
        let db = TestDb::new().await;
        let clock = fixed_clock();
        let req = sample_request();
        upsert(&db.pool, &req, "token", &clock).await.unwrap();

        let addr = DeviceAddress {
            ip: "192.168.1.100".into(),
            port: 38425,
        };
        update_status(&db.pool, sample_device_id(), &addr, None, &clock)
            .await
            .unwrap();

        // 验证 device_name 未被覆盖。
        let mut req2 = sample_request();
        req2.device_name = None;
        let outcome = upsert(&db.pool, &req2, "token2", &clock).await.unwrap();
        match outcome {
            RegisterOutcome::Reused { device, .. } => {
                assert_eq!(device.device_name, Some("Test PC".into()));
            }
            _ => panic!("expected Reused"),
        }
    }

    /// UT-device_repo-17：并发同 id 注册（Pre-req-4 ON CONFLICT 路径）。
    /// 100 个并发请求同 device_id，全部应返回 Created 或 Reused，无 500。
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
                upsert(&pool, &r, &token, c.as_ref() as &dyn Clock).await
            });
        }

        while let Some(result) = join_set.join_next().await {
            let outcome = result.unwrap();
            assert!(
                outcome.is_ok(),
                "concurrent upsert failed: {:?}",
                outcome.err()
            );
        }
    }
}
