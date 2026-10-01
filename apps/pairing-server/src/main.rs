//! 配对/信令服务器入口。
//!
//! 启动序列：
//! 1. init_tracing
//! 2. 加载 Config
//! 3. 打开 SQLite 连接池 + 迁移
//! 4. 启动指标 reporter（L3）
//! 5. 启动后台任务：信令会话 GC + 离线设备标记
//! 6. serve HTTP（axum）

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use dropvoice_pairing_server::{api, clock, config, observability, store, AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. tracing
    observability::init_tracing();
    tracing::info!("DropVoice pairing/signaling server starting");

    // 2. config（内置默认 → TOML 文件 → env 覆盖）。配置文件损坏时启动即失败，
    //    不静默回落默认值（限流/CORS/DB 路径漂移的排查成本远高于一次明确失败）。
    let cfg = config::Config::load().unwrap_or_else(|e| {
        eprintln!("configuration error: {e}");
        std::process::exit(1);
    });
    tracing::info!(listen_addr = %cfg.listen_addr, rate_limit = cfg.rate_limit_per_sec);

    // 3. DB pool + migrations
    let pool = store::open_pool(&cfg.database_url)
        .await
        .context("failed to open sqlite pool")?;

    // 4. metrics handle + reporter（每 60 秒打印 L3 快照）
    let metrics = Arc::new(observability::Metrics::default());
    observability::start_metrics_reporter(metrics.clone(), Duration::from_secs(60));

    // 5. 时钟（生产用 SystemClock）
    let clk: Arc<dyn clock::Clock> = Arc::new(clock::SystemClock);

    // 6. 共享状态（含信令会话存储 + SSE 推送桥）
    let state = AppState::new(
        pool.clone(),
        metrics.clone(),
        clk.clone(),
        cfg.rate_limit_per_sec,
    );

    // 7. 后台任务（需 AppState 引用以做会话 GC）
    spawn_background_tasks(state.clone());

    // 8. serve HTTP
    api::serve(cfg, state).await
}

/// 启动所有后台周期任务：信令会话 GC + 离线设备标记（每 CLEANUP_INTERVAL 1 分钟）。
fn spawn_background_tasks(state: AppState) {
    // 信令会话 GC + 离线设备标记 + L3 指标刷新。
    {
        let state = state.clone();
        tokio::spawn(async move {
            let mut tick =
                tokio::time::interval(Duration::from_secs(config::CLEANUP_INTERVAL_SECS));
            loop {
                tick.tick().await;
                // 信令会话 GC（§4.6 cleanup_expired 周期回收）。
                let removed = state.signaling.cleanup_expired_sessions();
                if removed > 0 {
                    tracing::debug!(removed, "expired signaling sessions cleaned");
                }
                // 刷新 L3 活跃会话计数
                state.metrics.webrtc_sessions_active.store(
                    state.signaling.active_session_count() as u64,
                    Ordering::Relaxed,
                );

                // 离线设备标记：last_seen 超过 2×DEVICE_STATUS_INTERVAL（10 分钟）视为离线。
                let threshold = state.clock.now()
                    - chrono::Duration::seconds(2 * (config::DEVICE_STATUS_INTERVAL.num_seconds()));
                match store::device_repo::mark_stale_offline(&state.pool, threshold).await {
                    Ok(n) if n > 0 => {
                        tracing::debug!(marked_offline = n, "stale devices marked offline")
                    }
                    Ok(_) => {}
                    Err(e) => tracing::warn!(error = %e, "offline marking failed"),
                }
                // 设备表 GC：超过保留期（30 天）未上线的设备删除——POST /api/devices
                // 对新设备无认证，无 GC 则行数可被无界刷大。被删设备重新注册即恢复
                //（新建分支；手机需重扫码）。
                let cutoff = state.clock.now() - config::DEVICE_RETENTION;
                match store::device_repo::delete_stale(&state.pool, cutoff).await {
                    Ok(n) if n > 0 => {
                        tracing::info!(deleted = n, "stale devices garbage collected")
                    }
                    Ok(_) => {}
                    Err(e) => tracing::warn!(error = %e, "device GC failed"),
                }
                // 刷新 L3 在线计数
                if let Ok(c) = store::device_repo::count_online(&state.pool).await {
                    state
                        .metrics
                        .devices_online
                        .store(c as u64, Ordering::Relaxed);
                }
            }
        });
    }
}
