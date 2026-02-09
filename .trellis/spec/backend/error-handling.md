# Error Handling

> How errors are handled in this project.

---

## Overview

DropVoice uses a pragmatic error handling approach:
- `anyhow::Result` for internal operations
- `Result<T, String>` for Tauri commands (frontend-compatible)
- Early return with error logging

---

## Error Types

### Internal Operations

Use `anyhow::Result`:

```rust
// src-tauri/src/text/injector.rs:8
pub fn inject_text(text: &str) -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default())?;
    enigo.text(text)?;
    Ok(())
}
```

### Tauri Commands

Return `Result<T, String>` for JSON serialization:

```rust
// src-tauri/src/commands.rs:38-41
#[tauri::command]
pub async fn start_server(
    state: State<'_, ServerState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    // ...
}
```

---

## Error Handling Patterns

### Pattern 1: Map Error to String

```rust
// src-tauri/src/commands.rs:60-68
let listener = match tokio::net::TcpListener::bind(format!("0.0.0.0:{}", default_port)).await {
    Ok(listener) => listener,
    Err(e) if e.kind() == ErrorKind::AddrInUse => {
        tokio::net::TcpListener::bind("0.0.0.0:0")
            .await
            .map_err(|err| format!("Failed to bind to port {} or fallback: {}", default_port, err))?
    }
    Err(e) => return Err(format!("Failed to bind to port {}: {}", default_port, e)),
};
```

### Pattern 2: Log and Return

```rust
// src-tauri/src/commands.rs:284-294
fs::create_dir_all(parent).map_err(|e| {
    let msg = format!("Failed to create language directory: {}", e);
    error!("{}", msg);
    msg
})?;
```

### Pattern 3: Graceful Fallback

```rust
// src-tauri/src/commands.rs:223-235
match fs::read_to_string(&path) {
    Ok(content) => {
        let trimmed = content.trim();
        match validate_language(trimmed) {
            Ok(_) => trimmed.to_string(),
            Err(_) => {
                warn!("Invalid language in storage: {}, using system language", trimmed);
                get_system_language_internal()
            }
        }
    }
    Err(_) => get_system_language_internal(),
}
```

---

## API Error Responses

### WebSocket Errors

Send JSON error message to client:

```rust
// src-tauri/src/server/websocket.rs:143-152
if content.len() > 10000 {
    warn!("Text too long: {} characters", content.len());
    let _ = ws_sender
        .send(Message::Text(
            serde_json::json!({
                "type": "error",
                "message": "Text too long (max 10000 characters)"
            }).to_string()
        ))
        .await;
    continue;
}
```

### Tauri Command Errors

Return descriptive string errors:

```rust
Err("Server is already running".to_string())
Err(format!("Failed to bind to port {}: {}", port, e))
```

---

## Common Mistakes

### 1. Using `unwrap()` in Production Code

```rust
// Bad - will panic
let port = state.port.lock().unwrap();

// Acceptable in this project - Mutex poisoning is unrecoverable
let port = state.port.lock().unwrap();

// Better - handle gracefully
let port = state.port.lock().ok().and_then(|lock| *lock);
```

### 2. Silent Failures Without Logging

```rust
// Bad - silent failure
let _ = ws_sender.send(msg).await;

// Good - log the failure
if let Err(e) = ws_sender.send(msg).await {
    error!("Failed to send message: {}", e);
}
```

### 3. Not Providing Context in Errors

```rust
// Bad - generic error
Err(e.to_string())

// Good - contextual error
Err(format!("Failed to write language preference: {}", e))
```
