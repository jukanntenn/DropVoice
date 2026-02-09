# Quality Guidelines

> Code quality standards for backend development.

---

## Overview

DropVoice backend follows Rust best practices with emphasis on:
- Thread-safe state management
- Proper error handling and logging
- Clean async/await patterns
- Platform-specific conditional compilation

---

## Forbidden Patterns

### 1. Blocking the Tokio Runtime

```rust
// Bad - blocks async runtime
std::thread::sleep(Duration::from_secs(1));

// Good - async sleep
tokio::time::sleep(Duration::from_secs(1)).await;
```

### 2. Unwrap in Command Handlers

```rust
// Bad - panic propagates to frontend
let state = state.lock().unwrap();

// Acceptable - Mutex poisoning is unrecoverable
let state = state.lock().unwrap();

// Better - handle gracefully
let state = state.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
```

### 3. Ignoring Errors Silently

```rust
// Bad - error is swallowed
let _ = fs::write(&path, content);

// Good - log the error
if let Err(e) = fs::write(&path, content) {
    error!("Failed to write: {}", e);
}
```

---

## Required Patterns

### 1. Thread-Safe State

Use `Arc` + `Atomic*` or `Arc<Mutex<T>>` for shared state:

```rust
// src-tauri/src/commands.rs:16-22
pub struct ServerState {
    is_running: Arc<AtomicBool>,
    port: Arc<Mutex<Option<u16>>>,
    connection_manager: ConnectionManager,
    pub language: Arc<Mutex<String>>,
    pub minimize_to_tray_enabled: Arc<AtomicBool>,
}
```

### 2. Async Tauri Commands

All Tauri commands that do I/O should be async:

```rust
#[tauri::command]
pub async fn start_server(
    state: State<'_, ServerState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    // ...
}
```

### 3. Proper Error Messages

Return descriptive, user-friendly errors:

```rust
Err(format!("Failed to bind to port {}: {}", port, e))
```

### 4. Logging Important Events

```rust
info!("Server started on {}", url);
info!("WebSocket connection established");
error!("Failed to inject text: {}", e);
```

---

## Testing Requirements

### Unit Tests

Located in the same file under `#[cfg(test)]` module:

```rust
// src-tauri/src/text/injector.rs:14-41
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inject_simple_text() {
        let result = inject_text("Hello, World!");
        assert!(result.is_ok());
    }

    #[test]
    fn test_inject_unicode() {
        let result = inject_text("Hello 世界 🦀");
        assert!(result.is_ok());
    }
}
```

### Running Tests

```bash
cd src-tauri
cargo test
```

### Clippy Linting

```bash
cargo clippy
```

---

## Code Review Checklist

### Before Committing

- [ ] `cargo build` succeeds
- [ ] `cargo test` passes
- [ ] `cargo clippy` has no warnings
- [ ] Important events are logged
- [ ] Errors are properly formatted

### Thread Safety

- [ ] Shared state uses `Arc`
- [ ] Atomic types used where appropriate
- [ ] Mutex locks are held briefly

### Async Code

- [ ] No blocking in async context
- [ ] Proper use of `tokio::spawn` for background tasks
- [ ] Cancellation is handled

### Platform-Specific Code

- [ ] `#[cfg(target_os = "windows")]` for Windows-only code
- [ ] Non-Windows code has fallback

---

## Common Mistakes

### 1. Missing Platform Guards

```rust
// Bad - enigo fails on non-Windows
pub fn inject_text(text: &str) -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default())?;
    // ...
}

// Currently acceptable - project is Windows-only
// Future: Add platform guards
```

### 2. Not Handling TcpListener Errors

```rust
// Bad - assumes bind succeeds
let listener = TcpListener::bind("0.0.0.0:38425").await?;

// Good - fallback to OS-assigned port
let listener = match TcpListener::bind(format!("0.0.0.0:{}", default_port)).await {
    Ok(l) => l,
    Err(e) if e.kind() == ErrorKind::AddrInUse => {
        TcpListener::bind("0.0.0.0:0").await?
    }
    Err(e) => return Err(e.into()),
};
```

**Reference**: `src-tauri/src/commands.rs:60-68`

### 3. Forgetting to Spawn Async Tasks

```rust
// Bad - blocks the handler
http::serve_with_listener(listener, ...).await;

// Good - spawn in background
tokio::spawn(async move {
    if let Err(e) = http::serve_with_listener(listener, ...).await {
        error!("Server error: {}", e);
    }
});
```

**Reference**: `src-tauri/src/commands.rs:93-101`
