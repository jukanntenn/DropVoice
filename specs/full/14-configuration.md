# 配置规范

> 版本: 1.0.0
> 最后更新: 2026-07-16
> 状态: 已批准

## 1. 概述

本规范定义 DropVoice 的配置文件格式、存储位置和管理策略。

## 2. 决策

### 2.1 配置格式

统一使用 **TOML** 格式。

### 2.2 存储分工

| 存储方式 | 用途 | 数据类型 | 说明 |
|---------|------|---------|------|
| TOML 文件 | 静态配置 | 启动时加载，不频繁变更 | 用户可手动编辑 |
| tauri-plugin-store | 动态设置 | 用户操作频繁变更 | 通过 UI 操作 |

### 2.3 配置文件结构

```toml
# dropvoice.toml

[meta]
config_version = 1
created_at = "2026-07-16"
last_modified = "2026-07-16"

[app]
version = "0.1.0"
language = "zh"              # en | zh | zh-TW | ja
theme = "system"             # light | dark | system

[server]
port = 38425
host = "0.0.0.0"
max_connections = 10

[injection]
delay_ms = 10                # 按字符注入的间隔（ms）；0 走 enigo 快路径
max_text_length = 10000      # 最大文本长度
queue_size = 100             # 注入队列大小（超出返回 QUEUE_FULL）

[security]
pairing_code_length = 8
pairing_code_expiry_minutes = 5
rate_limit_per_minute = 30
rate_limit_per_hour = 500

[updater]
enabled = true
check_interval_hours = 24
endpoint = "https://releases.dropvoice.app/update/manifest.json"  # 占位

[telemetry]
enabled = true
output = "file"              # file | otlp
log_dir = "logs"
log_retention_days = 30

[window]
width = 800
height = 600
resizable = true
minimize_to_tray = true

[devices]
max_count = 5
auto_connect = true
```

### 2.4 配置文件位置

| 平台 | 路径 |
|------|------|
| Windows | `%APPDATA%\dropvoice\dropvoice.toml` |
| macOS | `~/Library/Application Support/dropvoice/dropvoice.toml` |
| Linux | `~/.config/dropvoice/dropvoice.toml` |

### 2.5 配置版本迁移

```rust
// src-tauri/src/config/migration.rs

fn migrate_config(config: &mut DropVoiceConfig) {
    match config.meta.config_version {
        0 => {
            // v0 → v1 迁移
            config.meta.config_version = 1;
            // 添加新字段默认值...
        }
        1 => { /* 当前版本，无需迁移 */ }
        _ => {
            warn!("Unknown config version, using defaults");
            *config = DropVoiceConfig::default();
        }
    }
}
```

### 2.6 TOML vs tauri-plugin-store 分工

**TOML 文件存储:**
- 服务器端口、主机地址
- 注入延迟、文本长度限制
- 安全配置（配对码、速率限制）
- 更新配置
- 遥测配置
- 窗口配置
- 设备配置

**tauri-plugin-store 存储:**
- 主题设置（light/dark/system）
- 语言设置
- 窗口位置和大小
- 最后活跃设备 ID
- 用户偏好

## 3. 决策依据

### 3.1 选择 TOML 而非 JSON/YAML

**选择理由:**
- TOML 可读性好
- 支持注释
- Rust 生态标准（Cargo.toml）

**否决方案:**
- ❌ JSON: 不支持注释
- ❌ YAML: 缩进敏感，容易出错

### 3.2 TOML vs tauri-plugin-store 分工

**选择理由:**
- TOML 存静态配置，用户可手动编辑
- tauri-plugin-store 存动态设置，通过 UI 操作
- 分离关注点，便于管理

## 4. 接口规范

### 4.1 配置结构

```rust
// src-tauri/src/config/mod.rs

#[derive(Debug, Deserialize)]
struct DropVoiceConfig {
    meta: MetaConfig,
    app: AppConfig,
    server: ServerConfig,
    injection: InjectionConfig,
    security: SecurityConfig,
    updater: UpdaterConfig,
    telemetry: TelemetryConfig,
    window: WindowConfig,
    devices: DevicesConfig,
}

#[derive(Debug, Deserialize)]
struct MetaConfig {
    config_version: u32,
    created_at: String,
    last_modified: String,
}

#[derive(Debug, Deserialize)]
struct AppConfig {
    version: String,
    language: String,
    theme: String,
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
    port: u16,
    host: String,
    max_connections: usize,
}

#[derive(Debug, Deserialize)]
struct InjectionConfig {
    delay_ms: u64,
    max_text_length: usize,
    queue_size: usize,
}

#[derive(Debug, Deserialize)]
struct SecurityConfig {
    pairing_code_length: usize,
    pairing_code_expiry_minutes: u64,
    rate_limit_per_minute: usize,
    rate_limit_per_hour: usize,
}

#[derive(Debug, Deserialize)]
struct UpdaterConfig {
    enabled: bool,
    check_interval_hours: u64,
    endpoint: String,
}

#[derive(Debug, Deserialize)]
struct TelemetryConfig {
    enabled: bool,
    output: String,
    log_dir: String,
    log_retention_days: u64,
}

#[derive(Debug, Deserialize)]
struct WindowConfig {
    width: u32,
    height: u32,
    resizable: bool,
    minimize_to_tray: bool,
}

#[derive(Debug, Deserialize)]
struct DevicesConfig {
    max_count: usize,
    auto_connect: bool,
}
```

### 4.2 配置加载

```rust
impl DropVoiceConfig {
    fn load() -> Self;      // 加载配置，不存在则创建默认
    fn save(&self);         // 保存配置
    fn config_path() -> PathBuf;  // 获取配置文件路径
    fn default() -> Self;   // 默认配置
}
```

### 4.3 Cargo.toml 依赖

```toml
[dependencies]
toml = "0.8"
dirs = "5"
```

## 5. 配对服务器配置

配对服务器（`apps/pairing-server`）使用环境变量配置，不使用 TOML 文件。

| 环境变量 | 默认值 | 说明 |
|----------|--------|------|
| `LISTEN_ADDR` | `127.0.0.1:38424` | 监听地址（仅回环，经 Caddy 反代） |
| `DATABASE_URL` | `<data_dir>/pairing.db` | SQLite 数据库路径 |
| `RATE_LIMIT_PER_SEC` | `1` | 每 IP 每秒请求数 |
| `LOG_DIR` | `logs` | 日志输出目录 |

时间常量在 `config.rs` 中定义为 Rust `const`（spec 11 §4）：

| 常量 | 值 | 说明 |
|------|-----|------|
| `PAIRING_CODE_TTL` | 5 分钟 | 配对码有效期 |
| `DEVICE_TOKEN_TTL` | 24 小时 | 设备 token 有效期 |
| `DEVICE_STATUS_INTERVAL` | 5 分钟 | 心跳上报周期 |
| `PAIRING_CODE_LEN` | 6 | 配对码位数 |
| `PAIRING_TOKEN_LEN` | 64 | Token 字符长度 |
| `SQLITE_BUSY_TIMEOUT_MS` | 50 | SQLite 忙等待 |
| `CACHE_CLEANUP_INTERVAL_SECS` | 60 | 缓存清理周期 |
| `BATCH_WRITE_INTERVAL_SECS` | 1 | 批量写入周期 |

## 6. 约束

1. **默认配置**: 首次运行自动创建默认配置
2. **版本迁移**: 配置文件变更必须提供迁移逻辑
3. **配置验证**: 加载时验证配置合法性
4. **配置分离**: 静态配置用 TOML，动态设置用 tauri-plugin-store
5. **配置路径**: 使用系统标准配置目录
