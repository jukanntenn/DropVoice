# Logging Guidelines

> How logging is done in this project.

---

## Overview

DropVoice uses the `tracing` crate for structured logging with `tracing-subscriber` for output formatting.

---

## Setup

Logging is initialized in `lib.rs`:

```rust
// src-tauri/src/lib.rs:51-53
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .init();
```

**Note**: Only INFO and above are logged in production. For debugging, change to `tracing::Level::DEBUG`.

---

## Log Levels

| Level | When to Use | Examples |
|-------|-------------|----------|
| `error!` | Unrecoverable errors, operation failures | Server crashes, injection failures |
| `warn!` | Recoverable issues, invalid input | Invalid config, text too long |
| `info!` | Important lifecycle events | Server start/stop, connection open/close |
| `debug!` | Detailed operation info | (Not used in production) |
| `trace!` | Very detailed tracing | (Not used) |

---

## Usage Examples

### Server Lifecycle

```rust
// src-tauri/src/commands.rs:89
info!("Server started on {}", url);

// src-tauri/src/commands.rs:113
info!("Server stopped");

// src-tauri/src/commands.rs:95
error!("Server error: {}", e);
```

### Connection Events

```rust
// src-tauri/src/server/websocket.rs:85
info!("WebSocket connection established");

// src-tauri/src/server/websocket.rs:87
info!("Active connections: {}", conn_id);

// src-tauri/src/server/websocket.rs:189
info!("WebSocket connection closed. Active connections: {}", conn_count);
```

### Settings Changes

```rust
// src-tauri/src/commands.rs:300
info!("Language preference saved: {}", validated);

// src-tauri/src/commands.rs:339
info!("Theme preference saved: {}", theme);
```

### Validation Warnings

```rust
// src-tauri/src/commands.rs:229
warn!("Invalid language in storage: {}, using system language", trimmed);

// src-tauri/src/server/websocket.rs:143
warn!("Text too long: {} characters", content.len());
```

---

## What to Log

### Always Log

- Server start/stop
- WebSocket connections (open/close)
- Settings changes
- Error conditions
- File operation failures

### Log Format

```rust
// Good - descriptive with context
info!("Server started on {}", url);
info!("Active connections: {}", count);
error!("Failed to inject text: {}", e);

// Avoid - vague messages
info!("Started");
info!("Count: {}", count);
error!("Error: {}", e);
```

---

## What NOT to Log

### Never Log

- User text content (privacy)
- IP addresses (privacy)
- WebSocket message content (privacy)
- File paths with user data

### Example

```rust
// Bad - logs user's text
info!("Received message: {}", text);

// Good - logs without content
info!("Received message of {} bytes", text.len());
```

---

## Common Patterns

### Resource Path Discovery

```rust
// src-tauri/src/commands.rs:195-202
for candidate in candidates {
    if matches_candidate(&candidate) {
        info!("Using resource dir: {:?}", candidate);
        return candidate;
    }
}

warn!("Could not find mobile entry, using default: {:?}", default_path);
```

### Operation Result

```rust
// src-tauri/src/server/websocket.rs:157-165
match injector::inject_text(content) {
    Ok(_) => {
        info!("Text injected successfully");
        // ...
    }
    Err(e) => {
        error!("Failed to inject text: {}", e);
        // ...
    }
}
```
