//! 数据访问层。sqlx + SQLite，迁移由 `sqlx::migrate!` 嵌入。

pub mod device_repo;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;

use crate::config::SQLITE_BUSY_TIMEOUT_MS;

/// 嵌入式迁移（编译期校验 migrations/ 目录）。版本号经 `MIGRATOR.version()`
/// 暴露给 `/health` 的 `schema_version`，供部署冒烟 / 回滚决策使用。
pub const MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

/// 打开 SQLite 连接池，应用 spec 11 §9.3 的优化配置 + 运行迁移。
pub async fn open_pool(database_url: &str) -> anyhow::Result<SqlitePool> {
    let options: SqliteConnectOptions = database_url
        .parse::<SqliteConnectOptions>()?
        // §9.3 优化配置
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        .busy_timeout(std::time::Duration::from_millis(SQLITE_BUSY_TIMEOUT_MS))
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    // PRAGMA: cache_size 100MB, mmap_size 256MB（§9.3）。
    sqlx::query("PRAGMA cache_size = -102400")
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA mmap_size = 268435456")
        .execute(&pool)
        .await?;

    // 嵌入式迁移（编译期校验 migrations/ 目录）。
    MIGRATOR.run(&pool).await?;

    tracing::info!("sqlite pool opened + migrations applied");
    Ok(pool)
}
