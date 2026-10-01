//! 应用共享状态。

use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use sqlx::SqlitePool;

use crate::api::auth::RateLimiter;
use crate::api::signaling::SignalStore;
use crate::batch::BatchWriter;
use crate::cache::Cache;
use crate::clock::Clock;
use crate::observability::MetricsHandle;

/// `AppState: Clone`，因此 axum 的 blanket impl `FromRef<T> for T where T: Clone`
/// 自动满足——`AuthenticatedDevice` extractor 可直接从 router state 派生。
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub cache: Cache,
    pub batch: Arc<BatchWriter>,
    pub metrics: MetricsHandle,
    pub clock: Arc<dyn Clock>,
    pub rate_limiter: RateLimiter,
    pub rate_limit_per_sec: u32,
    /// 信令会话存储 + SSE offer 推送桥（§4.6）。
    pub signaling: SignalStore,
    /// 当前并发 SSE 连接数（上限 `config::MAX_CONCURRENT_SSE`）。
    pub sse_active: Arc<AtomicUsize>,
    /// 当前并发 answer 长轮询等待数（上限 `config::MAX_CONCURRENT_POLLS`）。
    pub poll_active: Arc<AtomicUsize>,
}

impl AppState {
    /// 构造 AppState。供 main.rs / 测试共用，确保 SignalStore 等新字段一致初始化。
    pub fn new(
        pool: SqlitePool,
        metrics: MetricsHandle,
        clock: Arc<dyn Clock>,
        rate_limit_per_sec: u32,
    ) -> Self {
        // BatchWriter 周期写入，clock 透传。
        let batch = Arc::new(BatchWriter::start(
            pool.clone(),
            std::time::Duration::from_secs(crate::config::BATCH_WRITE_INTERVAL_SECS),
            clock.clone(),
        ));
        Self {
            pool,
            cache: Cache::new(),
            batch,
            metrics,
            clock,
            rate_limiter: RateLimiter::new(),
            rate_limit_per_sec,
            signaling: SignalStore::new(),
            sse_active: Arc::new(AtomicUsize::new(0)),
            poll_active: Arc::new(AtomicUsize::new(0)),
        }
    }
}
