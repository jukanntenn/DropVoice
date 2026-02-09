# Data Persistence

> Data storage patterns for this project.

---

## Overview

DropVoice does **not use a database**. All persistence is handled via simple file storage in the application data directory.

This is intentional - DropVoice has minimal state:
- User preferences (theme, language)
- Dismissal flags (LAN warning)

---

## File-Based Persistence

### Storage Location

Files are stored in the Tauri app data directory:

```rust
// src-tauri/src/commands.rs:214-218
fn get_language_path(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(LANGUAGE_FILE)
}
```

### Storage Files

| File | Content | Format |
|------|---------|--------|
| `language-preference.txt` | User's language code | Plain text: `en` or `zh` |
| `theme-preference.txt` | User's theme | Plain text: `light`, `dark`, or `system` |
| `lan-warning-dismissed.txt` | Dismissal flag | Plain text: `true` |
| `minimize-to-tray-enabled.txt` | Tray setting | Plain text: `true` or `false` |

**Reference**: `src-tauri/src/commands.rs:209-213`

---

## Read/Write Patterns

### Reading

```rust
pub(crate) fn load_language_preference(app: &AppHandle) -> String {
    let path = get_language_path(app);

    match fs::read_to_string(&path) {
        Ok(content) => {
            let trimmed = content.trim();
            match validate_language(trimmed) {
                Ok(_) => trimmed.to_string(),
                Err(_) => get_system_language_internal(),
            }
        }
        Err(_) => get_system_language_internal(),
    }
}
```

**Reference**: `src-tauri/src/commands.rs:220-236`

### Writing

```rust
pub async fn set_language(app: AppHandle, lang: String) -> Result<(), String> {
    let validated = validate_language(&lang)?;

    let path = get_language_path(&app);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!("Failed to create language directory: {}", e)
        })?;
    }
    fs::write(&path, validated).map_err(|e| {
        format!("Failed to write language preference: {}", e)
    })?;

    Ok(())
}
```

**Reference**: `src-tauri/src/commands.rs:274-302`

---

## Adding New Persistent Settings

1. Define the file constant:

```rust
const NEW_SETTING_FILE: &str = "new-setting.txt";
```

2. Create path helper:

```rust
fn get_new_setting_path(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(NEW_SETTING_FILE)
}
```

3. Create getter command:

```rust
#[tauri::command]
pub async fn get_new_setting(app: AppHandle) -> Result<String, String> {
    // Read and validate
}
```

4. Create setter command:

```rust
#[tauri::command]
pub async fn set_new_setting(app: AppHandle, value: String) -> Result<(), String> {
    // Validate and write
}
```

5. Register in `lib.rs` invoke_handler

---

## Common Mistakes

### 1. Not Creating Parent Directory

```rust
// Bad - will fail if app_data_dir doesn't exist
fs::write(&path, content)?;

// Good - ensure parent exists
if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)?;
}
fs::write(&path, content)?;
```

### 2. Not Validating Input

```rust
// Bad - writes any value
fs::write(&path, value)?;

// Good - validate before writing
let validated = validate_language(&lang)?;
fs::write(&path, validated)?;
```

### 3. Not Handling Missing File

```rust
// Bad - returns error if file doesn't exist
let content = fs::read_to_string(&path)?;

// Good - return default if file doesn't exist
match fs::read_to_string(&path) {
    Ok(content) => parse_content(content),
    Err(_) => default_value(),
}
```
