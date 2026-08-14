//! 批量写入队列（spec 11 §10.2）。
//!
//! 状态上报请求入队，每 `BATCH_WRITE_INTERVAL`（1 秒）执行一次批量 UPDATE，
//! 使用事务保证原子性。减少 SQLite 锁竞争、提高写入吞吐量。
//!
//! Pre-req-7：PendingStatusUpdate 携带 device_name，flush 时 COALESCE 更新。

use std::sync::Arc;
use std::time::Duration;

use sqlx::SqlitePool;
use tokio::sync::mpsc;
use tokio::time::interval;
use uuid::Uuid;

use crate::clock::Clock;
use crate::domain::DeviceAddress;
use crate::error::AppResult;

#[derive(Debug, Clone)]
struct PendingStatusUpdate {
    device_id: Uuid,
    ip: String,
    port: u16,
    ts: i64,
    device_name: Option<String>,
}

/// 批量写入管理器。`start()` 返回后开始周期 flush。
pub struct BatchWriter {
    tx: mpsc::Sender<PendingStatusUpdate>,
    _handle: Arc<tokio::task::JoinHandle<()>>,
}

impl BatchWriter {
    /// 创建并启动后台 flush 任务。
    pub fn start(pool: SqlitePool, flush_interval: Duration, _clock: Arc<dyn Clock>) -> Self {
        let (tx, mut rx) = mpsc::channel::<PendingStatusUpdate>(10_000);
        let handle = tokio::spawn(async move {
            let mut tick = interval(flush_interval);
            let mut buf: Vec<PendingStatusUpdate> = Vec::new();
            loop {
                tokio::select! {
                    biased;
                    _ = tick.tick() => {
                        while let Ok(item) = rx.try_recv() {
                            buf.push(item);
                        }
                        if !buf.is_empty() {
                            if let Err(e) = flush(&pool, &mut buf).await {
                                tracing::warn!(error = %e, count = buf.len(), "batch flush failed");
                            }
                        }
                    }
                    Some(item) = rx.recv() => {
                        buf.push(item);
                        // 即时 flush 触发：缓冲超阈值立即写（防突发心跳积压）。
                        if buf.len() >= 1000 {
                            if let Err(e) = flush(&pool, &mut buf).await {
                                tracing::warn!(error = %e, count = buf.len(), "batch flush (threshold) failed");
                            }
                        }
                    }
                }
            }
        });

        Self {
            tx,
            _handle: Arc::new(handle),
        }
    }

    /// 入队一次状态上报。队列满时丢弃并告警（心跳 best-effort）。
    pub async fn enqueue(
        &self,
        device_id: Uuid,
        addr: &DeviceAddress,
        device_name: Option<String>,
        clock: &dyn Clock,
    ) {
        let item = PendingStatusUpdate {
            device_id,
            ip: addr.ip.clone(),
            port: addr.port,
            ts: clock.now().timestamp(),
            device_name,
        };
        if self.tx.try_send(item).is_err() {
            tracing::warn!("batch writer queue full, dropping status update");
        }
    }
}

/// 单事务批量 UPDATE（Pre-req-7：含 device_name COALESCE）。
async fn flush(pool: &SqlitePool, buf: &mut Vec<PendingStatusUpdate>) -> AppResult<()> {
    if buf.is_empty() {
        return Ok(());
    }
    let mut tx = pool.begin().await?;
    for item in buf.iter() {
        sqlx::query(
            r#"UPDATE devices SET ip = ?, port = ?, device_name = COALESCE(?, device_name),
                   is_online = 1, last_seen = ? WHERE id = ?"#,
        )
        .bind(&item.ip)
        .bind(item.port as i64)
        .bind(item.device_name.as_deref())
        .bind(item.ts)
        .bind(item.device_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|e| crate::error::AppError::Internal(e.into()))?;
    }
    tx.commit().await?;
    let flushed = buf.len();
    buf.clear();
    tracing::debug!(flushed, "batch status flush complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DeviceAddress, DeviceRegisterRequest, Platform};
    use crate::store::device_repo;
    use crate::test_support::{fixed_clock, sample_device_id, TestDb};

    async fn setup_device(pool: &sqlx::SqlitePool, clock: &dyn crate::clock::Clock) {
        let req = DeviceRegisterRequest {
            device_id: sample_device_id(),
            platform: Platform::Desktop,
            device_name: Some("Original".into()),
            address: DeviceAddress {
                ip: "192.168.1.100".into(),
                port: 38425,
            },
        };
        device_repo::upsert(pool, &req, "token", clock)
            .await
            .unwrap();
    }

    /// UT-batch-01：enqueue + flush 落库。
    #[tokio::test]
    async fn enqueue_and_flush_updates_db() {
        let db = TestDb::new().await;
        let clock = fixed_clock();
        setup_device(&db.pool, &clock).await;

        let mut buf = vec![PendingStatusUpdate {
            device_id: sample_device_id(),
            ip: "10.0.0.1".into(),
            port: 9999,
            ts: clock.now().timestamp(),
            device_name: None,
        }];
        flush(&db.pool, &mut buf).await.unwrap();

        let addr = device_repo::current_address(&db.pool, sample_device_id())
            .await
            .unwrap();
        assert_eq!(addr.ip, "10.0.0.1");
        assert_eq!(addr.port, 9999);
    }

    /// UT-batch-04：device_name 落库（Pre-req-7）。
    #[tokio::test]
    async fn flush_updates_device_name() {
        let db = TestDb::new().await;
        let clock = fixed_clock();
        setup_device(&db.pool, &clock).await;

        let mut buf = vec![PendingStatusUpdate {
            device_id: sample_device_id(),
            ip: "192.168.1.100".into(),
            port: 38425,
            ts: clock.now().timestamp(),
            device_name: Some("New Name".into()),
        }];
        flush(&db.pool, &mut buf).await.unwrap();

        // 验证 device_name 已更新。
        let req = DeviceRegisterRequest {
            device_id: sample_device_id(),
            platform: Platform::Desktop,
            device_name: None,
            address: DeviceAddress {
                ip: "192.168.1.100".into(),
                port: 38425,
            },
        };
        let outcome = device_repo::upsert(&db.pool, &req, "token", &clock)
            .await
            .unwrap();
        match outcome {
            device_repo::RegisterOutcome::Reused { device, .. } => {
                assert_eq!(device.device_name, Some("New Name".into()));
            }
            _ => panic!("expected Reused"),
        }
    }

    /// UT-batch-05：空 flush no-op。
    #[tokio::test]
    async fn flush_empty_is_noop() {
        let db = TestDb::new().await;
        let mut buf = vec![];
        flush(&db.pool, &mut buf).await.unwrap();
    }

    /// UT-batch-06：同设备多次 enqueue 去重/覆盖。
    #[tokio::test]
    async fn flush_last_wins_for_same_device() {
        let db = TestDb::new().await;
        let clock = fixed_clock();
        setup_device(&db.pool, &clock).await;

        let mut buf = vec![
            PendingStatusUpdate {
                device_id: sample_device_id(),
                ip: "10.0.0.1".into(),
                port: 1111,
                ts: clock.now().timestamp(),
                device_name: None,
            },
            PendingStatusUpdate {
                device_id: sample_device_id(),
                ip: "10.0.0.2".into(),
                port: 2222,
                ts: clock.now().timestamp(),
                device_name: None,
            },
        ];
        flush(&db.pool, &mut buf).await.unwrap();

        let addr = device_repo::current_address(&db.pool, sample_device_id())
            .await
            .unwrap();
        assert_eq!(addr.ip, "10.0.0.2");
        assert_eq!(addr.port, 2222);
    }

    /// UT-batch-07：阈值 1000 即时 flush（覆盖 lines 58-60）。
    #[tokio::test]
    async fn threshold_flush_at_1000_items() {
        let db = TestDb::new().await;
        let clock = fixed_clock();
        setup_device(&db.pool, &clock).await;

        // 用很长的 flush interval，确保只有阈值触发 flush。
        let clk: Arc<dyn crate::clock::Clock> = Arc::new(clock.clone());
        let writer = BatchWriter::start(db.pool.clone(), Duration::from_secs(3600), clk);

        let addr = DeviceAddress {
            ip: "10.0.0.99".into(),
            port: 7777,
        };

        // enqueue 1000 条触发阈值 flush。
        for _ in 0..1000 {
            writer
                .enqueue(sample_device_id(), &addr, None, &clock)
                .await;
        }

        // 等待 flush 完成。
        tokio::time::sleep(Duration::from_millis(500)).await;

        // 验证地址已更新。
        let updated = device_repo::current_address(&db.pool, sample_device_id())
            .await
            .unwrap();
        assert_eq!(updated.ip, "10.0.0.99");
        assert_eq!(updated.port, 7777);
    }
}
