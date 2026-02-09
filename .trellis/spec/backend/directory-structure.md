# Directory Structure

> How backend code is organized in this project.

---

## Overview

DropVoice backend is a Rust-based Tauri application. The backend handles:
- HTTP server for serving mobile interface
- WebSocket server for real-time text transmission
- Keyboard injection for text input
- System tray management
- Settings persistence

---

## Directory Layout

```
src-tauri/
├── Cargo.toml              # Rust dependencies and build config
├── tauri.conf.json         # Tauri app configuration
├── src/
│   ├── main.rs             # Binary entry point (just calls lib.rs)
│   ├── lib.rs              # Tauri app setup, tray, window management
│   ├── commands.rs         # Tauri command handlers (IPC endpoints)
│   ├── i18n.rs             # Internationalization utilities
│   ├── api.rs              # Additional API utilities
│   ├── server/
│   │   ├── mod.rs          # Module exports
│   │   ├── http.rs         # HTTP server (serves mobile.html)
│   │   └── websocket.rs    # WebSocket handler and connection manager
│   ├── network/
│   │   ├── mod.rs          # Module exports
│   │   └── discovery.rs    # LAN IP detection, URL generation
│   └── text/
│       ├── mod.rs          # Module exports
│       └── injector.rs     # Keyboard input simulation (enigo)
└── icons/                  # App icons for all platforms
```

---

## Module Organization

### Core Modules

| Module | Responsibility |
|--------|---------------|
| `lib.rs` | App lifecycle, tray menu, window events |
| `commands.rs` | Tauri IPC command handlers |
| `server/` | HTTP + WebSocket server |
| `network/` | IP discovery, connection URL generation |
| `text/` | Text injection via keyboard simulation |
| `i18n.rs` | Language detection, tray text translations |

### Module Visibility

```rust
// lib.rs - declares all modules
mod text;
mod network;
mod server;
mod commands;
mod i18n;

// Public exports for use across modules
use commands::ServerState;
```

---

## Naming Conventions

| Type | Convention | Example |
|------|------------|---------|
| Modules | snake_case | `commands.rs`, `discovery.rs` |
| Functions | snake_case | `start_server`, `get_connection_info` |
| Structs | PascalCase | `ServerState`, `ConnectionManager` |
| Constants | SCREAMING_SNAKE | `LANGUAGE_FILE`, `MAX_RECONNECT_ATTEMPTS` |
| Tauri commands | snake_case | `set_language`, `get_theme` |

---

## Adding New Features

### Adding a New Tauri Command

1. Define the function in `commands.rs`:

```rust
#[tauri::command]
pub async fn my_new_command(arg: String) -> Result<String, String> {
    // Implementation
    Ok("result".to_string())
}
```

2. Register in `lib.rs`:

```rust
.invoke_handler(tauri::generate_handler![
    commands::start_server,
    commands::stop_server,
    // Add new command here
    commands::my_new_command,
])
```

### Adding a New Module

1. Create the file: `src-tauri/src/new_module.rs`
2. Declare in `lib.rs`: `mod new_module;`
3. Create `mod.rs` if it's a directory with multiple files
