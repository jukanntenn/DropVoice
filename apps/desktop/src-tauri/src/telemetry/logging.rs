use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use chrono::Utc;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Returns the platform-specific log directory.
///
/// | Platform | Path                          |
/// |----------|-------------------------------|
/// | Windows  | `%LOCALAPPDATA%\dropvoice\logs` |
/// | macOS    | `~/Library/Logs/dropvoice`    |
/// | Linux    | `~/.local/share/dropvoice/logs` |
pub fn get_log_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        if let Some(base) = directories::BaseDirs::new() {
            return base.home_dir().join("Library/Logs/dropvoice");
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Some(base) = directories::BaseDirs::new() {
            return base.data_local_dir().join("dropvoice").join("logs");
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(base) = directories::BaseDirs::new() {
            return base.data_dir().join("dropvoice").join("logs");
        }
    }
    PathBuf::from("logs")
}

/// Holds the non-blocking file writer guard so logs are flushed on shutdown.
pub struct LogGuard {
    _file_guard: WorkerGuard,
}

/// Initialises tracing with two layers: stdout (human-readable) and a daily
/// rotating JSON file. Returns a `LogGuard` that must be held for the lifetime
/// of the application.
pub fn init_logging() -> LogGuard {
    let log_dir = get_log_dir();
    if let Err(e) = fs::create_dir_all(&log_dir) {
        eprintln!("failed to create log dir {}: {e}", log_dir.display());
    }

    let file_appender = tracing_appender::rolling::daily(&log_dir, "dropvoice.log");
    let (non_blocking_file, file_guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let stdout_layer = fmt::layer().with_writer(std::io::stdout);

    let file_layer = fmt::layer()
        .with_writer(non_blocking_file)
        .json()
        .with_current_span(false);

    let result = tracing_subscriber::registry()
        .with(env_filter)
        .with(stdout_layer)
        .with(file_layer)
        .try_init();

    if let Err(e) = result {
        eprintln!("tracing subscriber already initialised: {e}");
    }

    LogGuard {
        _file_guard: file_guard,
    }
}

/// Deletes log files older than `retention_days`. Best-effort — errors are
/// logged but not propagated.
pub fn cleanup_old_logs(retention_days: u64) {
    let log_dir = get_log_dir();
    let retention = Duration::from_secs(retention_days * 24 * 60 * 60);

    let entries = match fs::read_dir(&log_dir) {
        Ok(e) => e,
        Err(_) => return, // directory may not exist yet
    };

    let now = Utc::now();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if let Ok(metadata) = entry.metadata() {
            if let Ok(modified) = metadata.modified() {
                let modified_utc: chrono::DateTime<Utc> = modified.into();
                if now.signed_duration_since(modified_utc).num_seconds()
                    > retention.as_secs() as i64
                {
                    if let Err(e) = fs::remove_file(&path) {
                        tracing::warn!("failed to remove old log {}: {e}", path.display());
                    }
                }
            }
        }
    }
}
