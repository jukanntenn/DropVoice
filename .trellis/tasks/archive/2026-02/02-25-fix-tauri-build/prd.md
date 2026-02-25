# Fix Tauri Build Error

## Goal
Fix the compilation error that occurs when running `pnpm tauri build` - the `ManagerExt` trait is not imported, preventing the use of the `autolaunch()` method.

## What I already know
- Build error: `no method named 'autolaunch' found for mutable reference '&mut tauri::App'`
- The error occurs in `src-tauri/src/lib.rs` at lines 67-68
- The `ManagerExt` trait from `tauri_plugin_autostart` is not imported
- Dev mode works fine, only production build fails
- The Rust compiler suggests: `use tauri_plugin_autostart::ManagerExt;`

## Root Cause Analysis

### How the bug was introduced
1. **v0.0.1** had the correct import: `use tauri_plugin_autostart::ManagerExt;`
2. **Commit f856df3** ("feat(input): add configurable input delay...") accidentally removed this import when modifying `lib.rs`
3. The removal was likely unintentional - the commit focused on adding input delay feature

### Why dev mode didn't catch this
- Lines 67-71 in `lib.rs` use `#[cfg(not(debug_assertions))]` conditional compilation
- This means `autolaunch()` code is **only compiled in release mode**
- Dev mode (`pnpm tauri dev`) skips this code entirely, so missing import wasn't detected
- Build mode (`pnpm tauri build`) uses release profile, triggers compilation of autolaunch code, fails on missing trait

### The missing import
The `autolaunch()` method is defined in the `ManagerExt` trait from `tauri_plugin_autostart`, but this trait was removed from imports in commit f856df3.

## Requirements
- Import the `ManagerExt` trait to enable the `autolaunch()` method
- Ensure production build completes successfully

## Acceptance Criteria
- [ ] `pnpm tauri build` completes without errors
- [ ] The autostart functionality works as intended in release builds

## Technical Approach
Add the missing import statement at the top of `src-tauri/src/lib.rs`:
```rust
use tauri_plugin_autostart::ManagerExt;
```

## Out of Scope
- Changes to autostart logic or behavior
- Changes to other parts of the codebase
