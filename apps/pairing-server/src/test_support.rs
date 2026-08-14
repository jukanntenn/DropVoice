//! 测试夹具（仅 #[cfg(test)]）。
//!
//! spec 21 §4.1 通用夹具：TestDb、fixed_clock、sample_device_id。

use sqlx::SqlitePool;
use tempfile::TempDir;

use crate::clock::FakeClock;
use crate::store;

pub struct TestDb {
    pub pool: SqlitePool,
    pub _tmp: TempDir,
}

impl TestDb {
    pub async fn new() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let url = format!("sqlite://{}?mode=rwc", tmp.path().join("t.db").display());
        let pool = store::open_pool(&url).await.unwrap();
        Self { pool, _tmp: tmp }
    }
}

/// 固定基准时间 2026-07-25T00:00:00Z，保证可重复。
pub fn fixed_clock() -> FakeClock {
    FakeClock::new(
        chrono::DateTime::parse_from_rfc3339("2026-07-25T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc),
    )
}

/// 固定 UUID，保证测试可重复。
pub fn sample_device_id() -> uuid::Uuid {
    uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
}
