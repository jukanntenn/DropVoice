use std::sync::atomic::{AtomicU64, Ordering};

/// Lightweight business metrics backed by atomic counters.
///
/// In v0.2.0 these are surfaced via the `/health` endpoint and structured logs.
/// A future version can export them to OpenTelemetry.
#[derive(Debug)]
pub struct BusinessMetrics {
    connections_active: AtomicU64,
    connections_total: AtomicU64,
    transfers_total: AtomicU64,
    transfers_bytes: AtomicU64,
    injections_total: AtomicU64,
    injection_failed: AtomicU64,
}

impl BusinessMetrics {
    pub fn new() -> Self {
        Self {
            connections_active: AtomicU64::new(0),
            connections_total: AtomicU64::new(0),
            transfers_total: AtomicU64::new(0),
            transfers_bytes: AtomicU64::new(0),
            injections_total: AtomicU64::new(0),
            injection_failed: AtomicU64::new(0),
        }
    }

    /// Records a connection attempt.
    pub fn record_connection(&self, mode: &str, success: bool, _duration_secs: f64) {
        if success {
            self.connections_total.fetch_add(1, Ordering::Relaxed);
            self.connections_active.fetch_add(1, Ordering::Relaxed);
        }
        tracing::info!(
            target: "dropvoice::metrics",
            metric = "connection",
            mode = mode,
            success = success,
            "connection recorded"
        );
    }

    /// Records a text transfer from a client.
    pub fn record_transfer(&self, bytes: u64, chars: u64, _duration_ms: f64, success: bool) {
        if success {
            self.transfers_total.fetch_add(1, Ordering::Relaxed);
            self.transfers_bytes.fetch_add(bytes, Ordering::Relaxed);
        }
        tracing::info!(
            target: "dropvoice::metrics",
            metric = "transfer",
            bytes = bytes,
            chars = chars,
            success = success,
            "transfer recorded"
        );
    }

    /// Records a text injection attempt.
    pub fn record_injection(&self, chars: u64, _duration_ms: f64, success: bool) {
        if success {
            self.injections_total.fetch_add(1, Ordering::Relaxed);
        } else {
            self.injection_failed.fetch_add(1, Ordering::Relaxed);
        }
        tracing::info!(
            target: "dropvoice::metrics",
            metric = "injection",
            chars = chars,
            success = success,
            "injection recorded"
        );
    }

    /// Decrements the active connection gauge (called on disconnect).
    pub fn decrement_active_connections(&self) {
        self.connections_active.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn active_connections(&self) -> u64 {
        self.connections_active.load(Ordering::Relaxed)
    }

    pub fn total_connections(&self) -> u64 {
        self.connections_total.load(Ordering::Relaxed)
    }

    pub fn total_transfers(&self) -> u64 {
        self.transfers_total.load(Ordering::Relaxed)
    }

    pub fn total_injections(&self) -> u64 {
        self.injections_total.load(Ordering::Relaxed)
    }
}

impl Default for BusinessMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_successful_connection() {
        let metrics = BusinessMetrics::new();
        metrics.record_connection("lan", true, 0.1);
        assert_eq!(metrics.active_connections(), 1);
        assert_eq!(metrics.total_connections(), 1);
    }

    #[test]
    fn decrements_active_connections() {
        let metrics = BusinessMetrics::new();
        metrics.record_connection("lan", true, 0.1);
        metrics.decrement_active_connections();
        assert_eq!(metrics.active_connections(), 0);
        assert_eq!(metrics.total_connections(), 1);
    }

    #[test]
    fn records_transfers_and_injections() {
        let metrics = BusinessMetrics::new();
        metrics.record_transfer(42, 10, 5.0, true);
        metrics.record_injection(10, 3.0, true);
        metrics.record_injection(5, 1.0, false);
        assert_eq!(metrics.total_transfers(), 1);
        assert_eq!(metrics.total_injections(), 1);
    }
}
