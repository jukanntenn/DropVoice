# Database Migration Workflow (pairing-server)

The pairing-server uses **sqlx 0.8** with **embedded migrations**. Migrations are
compiled into the binary via `sqlx::migrate!("./migrations")` and applied
automatically at server startup — no manual `migrate run` is ever needed in
any environment.

## One-time setup

```bash
cargo install sqlx-cli --no-default-features --features sqlite,rustls
```

## Creating a migration

```bash
cd apps/pairing-server

# Simple migration (default): single .sql file, cannot be reverted.
sqlx migrate add add_user_preferences_column
# → creates migrations/<timestamp>_add_user_preferences_column.sql

# Reversible migration: .up.sql + .down.sql pair.
sqlx migrate add -r refactor_devices_schema
# → creates migrations/<timestamp>_refactor_devices_schema.up.sql
#             + migrations/<timestamp>_refactor_devices_schema.down.sql
```

**When to use which:**
- **Simple** (`add`): schema additions that have no meaningful rollback (new tables, new columns with defaults). The `init` migration is simple.
- **Reversible** (`add -r`): migrations with a clear, safe down-path (e.g., renaming a column back). Reserve for cases where rollback is genuinely needed. Never use reversible for destructive changes where data would be lost.

## File naming convention

```
YYYYMMDDHHMMSS_<description>.sql           (simple)
YYYYMMDDHHMMSS_<description>.up.sql        (reversible, up)
YYYYMMDDHHMMSS_<description>.down.sql      (reversible, down)
```

Migrations are applied in ascending version-number order. Never edit an
already-applied migration — always add a new one.

## How migrations apply

1. `store::open_pool()` creates the SQLite connection pool (WAL mode, pragmas).
2. `sqlx::migrate!("./migrations").run(&pool)` runs all pending migrations.
3. The `_sqlx_migrations` table tracks which versions are applied.

This happens on every server start (dev, test, production). It is idempotent:
already-applied migrations are skipped.

## Local database management

```bash
cd apps/pairing-server

# Create a fresh database (uses DATABASE_URL from .env or default).
sqlx database create

# Drop and recreate (destroys all data).
sqlx database drop -f && sqlx database create

# Inspect migration state.
sqlx migrate info
```

## Query style

The pairing-server uses **runtime queries** (`query_as` + `FromRow`), NOT
compile-time `query!` macros. This means:

- No `DATABASE_URL` is required at compile time.
- No `cargo sqlx prepare` / `.sqlx` offline directory is needed.
- CI builds work without a database connection.

If a future change introduces `query!` macros, run `cargo sqlx prepare` and
commit the `.sqlx/` directory.
