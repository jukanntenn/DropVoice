//! 可观测性。
//!
//! - 日志：`tracing` + `tracing-subscriber`（stdout + JSON file 双 layer）。
//!   运行时由 `tower-http::TraceLayer` 产出 access log。
//! - L3 计数器：AtomicU64，周期 `info!` 打印（不接 OTLP）。

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// L3 业务指标。
#[derive(Debug, Default)]
pub struct Metrics {
    pub devices_registered_total: AtomicU64,
    pub devices_online: AtomicU64,
    /// 活跃信令会话数（periodically refreshed from sessions map）。
    pub webrtc_sessions_active: AtomicU64,
    /// 累计接收的 offer 数。
    pub webrtc_offers_received_total: AtomicU64,
    /// 累计完成的 answer 数（accepted + rejected）。
    pub webrtc_answers_completed_total: AtomicU64,
}

pub type MetricsHandle = Arc<Metrics>;

/// 初始化 tracing（stdout + JSON 文件双 layer）。
pub fn init_tracing() {
    use std::path::PathBuf;

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,dropvoice_pairing_server=debug"));

    let stdout_layer = fmt::layer().with_target(false);

    // JSON file layer：日志目录优先 LOG_DIR 环境变量，否则当前目录。
    let log_dir: PathBuf = std::env::var("LOG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("logs"));
    let file_init = std::fs::create_dir_all(&log_dir)
        .map_err(|e| e.to_string())
        .map(|_| {
            let file_appender = tracing_appender::rolling::daily(&log_dir, "pairing-server.log");
            // non_blocking 包装保证文件 IO 不阻塞请求线程。
            let (writer, guard) = tracing_appender::non_blocking(file_appender);
            // guard 必须存活至程序结束；泄漏它（服务进程永不返回）。
            std::mem::forget(guard);
            fmt::layer().json().with_writer(writer)
        });

    match file_init {
        Ok(fl) => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(stdout_layer)
                .with(fl)
                .init();
        }
        Err(e) => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(stdout_layer)
                .init();
            tracing::warn!(error = %e, "json file log layer unavailable, continuing with stdout only");
        }
    }
}

/// 启动周期指标打印任务（L3 → 日志输出）。
pub fn start_metrics_reporter(metrics: MetricsHandle, period: Duration) {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(period);
        loop {
            tick.tick().await;
            tracing::info!(
                target: "dropvoice::metrics",
                devices_registered_total = metrics.devices_registered_total.load(Ordering::Relaxed),
                devices_online = metrics.devices_online.load(Ordering::Relaxed),
                webrtc_sessions_active = metrics.webrtc_sessions_active.load(Ordering::Relaxed),
                webrtc_offers_received_total = metrics.webrtc_offers_received_total.load(Ordering::Relaxed),
                webrtc_answers_completed_total = metrics.webrtc_answers_completed_total.load(Ordering::Relaxed),
                "periodic metrics snapshot"
            );
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// UT-metrics-01：计数器 fetch_add。
    #[test]
    fn metrics_counter_increment() {
        let m = Metrics::default();
        m.devices_registered_total.fetch_add(1, Ordering::Relaxed);
        assert_eq!(m.devices_registered_total.load(Ordering::Relaxed), 1);

        m.webrtc_offers_received_total
            .fetch_add(5, Ordering::Relaxed);
        assert_eq!(m.webrtc_offers_received_total.load(Ordering::Relaxed), 5);
    }

    /// UT-metrics-02：Relaxed 不丢增量（单线程）。
    #[test]
    fn metrics_counter_accumulates() {
        let m = Metrics::default();
        for _ in 0..100 {
            m.devices_registered_total.fetch_add(1, Ordering::Relaxed);
        }
        assert_eq!(m.devices_registered_total.load(Ordering::Relaxed), 100);
    }

    /// UT-metrics-03：start_metrics_reporter 不 panic，且循环体执行。
    #[tokio::test]
    async fn metrics_reporter_starts_and_ticks() {
        let m = Arc::new(Metrics::default());
        m.devices_registered_total.fetch_add(42, Ordering::Relaxed);
        m.devices_online.fetch_add(7, Ordering::Relaxed);
        m.webrtc_sessions_active.fetch_add(3, Ordering::Relaxed);
        m.webrtc_offers_received_total
            .fetch_add(10, Ordering::Relaxed);
        m.webrtc_answers_completed_total
            .fetch_add(20, Ordering::Relaxed);
        // 用极短周期确保循环体内的 tracing::info! 被执行。
        start_metrics_reporter(m.clone(), Duration::from_millis(50));
        tokio::time::sleep(Duration::from_millis(150)).await;
        // 等待至少 1 个 tick 触发。
    }

    /// UT-metrics-04：所有计数器独立递增。
    #[test]
    fn all_counters_work_independently() {
        let m = Metrics::default();
        m.devices_registered_total.fetch_add(1, Ordering::Relaxed);
        m.devices_online.fetch_add(2, Ordering::Relaxed);
        m.webrtc_sessions_active.fetch_add(3, Ordering::Relaxed);
        m.webrtc_offers_received_total
            .fetch_add(4, Ordering::Relaxed);
        m.webrtc_answers_completed_total
            .fetch_add(5, Ordering::Relaxed);

        assert_eq!(m.devices_registered_total.load(Ordering::Relaxed), 1);
        assert_eq!(m.devices_online.load(Ordering::Relaxed), 2);
        assert_eq!(m.webrtc_sessions_active.load(Ordering::Relaxed), 3);
        assert_eq!(m.webrtc_offers_received_total.load(Ordering::Relaxed), 4);
        assert_eq!(m.webrtc_answers_completed_total.load(Ordering::Relaxed), 5);
    }
}
