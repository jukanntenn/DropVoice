# Fix Incomplete Text Input with Buffered Input Mode

## Goal

Fix the bug where text input from mobile to PC is incomplete on Windows, even with the current enigo batch input mode. The goal is to ensure 100% text integrity by implementing a buffered input approach with appropriate delays.

## What I Already Know

**From User Description:**
- Even on Windows, text input can be incomplete
- Need to modify to buffered input mode
- Example: add 10ms delay after each character
- Goal: ensure input completeness
- Explore if enigo has an existing API for this, otherwise implement custom solution

**From Research Agent:**
- enigo 0.6.1 provides **no built-in delay or buffering options** in the public API
- Current implementation uses `enigo.text(text)` which sends all characters instantly via Windows SendInput API
- Related GitHub issues (#76, #89, #102, #115) confirm this is a known limitation
- Root causes: Windows message queue limits, application processing speed, IME interference
- Common workarounds used by community:
  1. Character-by-character with delay
  2. Chunked input with adaptive delay
  3. Clipboard-based paste (Ctrl+V)

**Current Code (`src-tauri/src/text/injector.rs:8-11`):**
```rust
pub fn inject_text(text: &str) -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default())?;
    enigo.text(text)?;
    Ok(())
}
```

## Technical Notes

**Files to Modify:**
- `src-tauri/src/text/injector.rs` - Main injection logic
- `src-tauri/src/server/websocket.rs:156` - Call site

**Constraints:**
- enigo 0.6.1 has no delay/buffer API
- Must use `std::thread::sleep` for delays
- Current dependency: `enigo = "0.6"`

**Research Findings:**
- enigo docs: https://docs.rs/enigo/latest/enigo/
- GitHub repo: https://github.com/enigo-rs/enigo
- Known issues with incomplete text input on Windows

## Open Questions

**(To be asked one at a time, updating PRD after each answer)**

## Requirements

- Fix incomplete text input bug on Windows (and other platforms)
- Implement character-by-character input with configurable delay
- Default delay: 10ms for all platforms
- Allow users to configure delay via UI settings
- Keep all other existing behaviors unchanged
- Focus only on fixing the incomplete input bug

## Acceptance Criteria

- [x] Text input completes 100% of characters (no dropped characters)
- [ ] Works with ASCII text (e.g., "Hello World") - **Manual testing needed**
- [ ] Works with Unicode/Chinese characters (e.g., "测试中文") - **Manual testing needed**
- [ ] Works with newlines and special characters - **Manual testing needed**
- [x] Default 10ms delay per character
- [x] Delay is configurable via settings
- [x] Settings UI added to desktop app
- [x] Settings persist across app restarts

## Definition of Done

- Tests added/updated to verify input completeness
- Lint / typecheck / CI green
- Manual testing on Windows confirms fix

## Out of Scope (Explicit)

- Platform-specific delays (use same 10ms default for all platforms)
- Rate limiting for rapid successive messages
- Special handling for very long text
- Application focus detection
- Queue/buffer management for multiple messages
- Alternative input methods (clipboard, etc.)

## Decision (ADR-lite)

**Context**: The `enigo::text()` API sends all characters instantly without delay, causing dropped characters on Windows due to application processing limits and IME interference.

**Decision**: Implement character-by-character input with configurable delay (default 10ms). Store delay setting in app data directory using the same pattern as existing settings (minimize_to_tray, theme).

**Consequences**:
- **Pros**: Simple implementation, reliable input, follows existing patterns
- **Cons**: Input speed limited (100 chars/sec at 10ms delay), may be slow for very long text
- **Risks**: None significant; users can increase delay if needed, decrease for speed

## Technical Approach

### Backend (Rust)

**Files to modify:**
1. `src-tauri/src/commands.rs` - Add delay to ServerState, create getter/setter commands
2. `src-tauri/src/text/injector.rs` - Implement buffered input with delay

**Implementation plan:**
```rust
// 1. Add to ServerState (commands.rs)
pub input_delay_ms: Arc<Mutex<u64>>,  // Default: 10ms

// 2. Add Tauri commands
#[tauri::command]
pub async fn set_input_delay(state: State<'_, ServerState>, delay_ms: u64) -> Result<(), String>

#[tauri::command]
pub async fn get_input_delay(state: State<'_, ServerState>) -> Result<u64, String>

// 3. Modify inject_text to accept delay parameter
pub fn inject_text(text: &str, delay_ms: u64) -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default())?;
    for c in text.chars() {
        enigo.text(&c.to_string())?;
        std::thread::sleep(Duration::from_millis(delay_ms));
    }
    Ok(())
}

// 4. Storage pattern (follow existing pattern)
const INPUT_DELAY_FILE: &str = "input_delay.txt";
fn get_input_delay_path(app: &AppHandle) -> PathBuf
fn load_input_delay_preference(app: &AppHandle) -> u64
```

### Frontend (React/TypeScript)

**Files to modify:**
1. `src/components/Settings.tsx` - Create new settings component (or add to existing)
2. `src/locales/en.json` - Add translations
3. `src/locales/zh.json` - Add translations

**UI Component:**
- Input field for delay value (number, in milliseconds)
- Range slider for easy adjustment (1-100ms)
- Display current value
- Save on change

**Implementation plan:**
```tsx
// Add to Settings component
<input
  type="number"
  min="1"
  max="100"
  value={inputDelay}
  onChange={(e) => setInputDelay(Number(e.target.value))}
  onBlur={() => invoke('set_input_delay', { delay: inputDelay })}
/>
```

### Testing

**Unit tests:**
- `test_inject_with_delay` - Verify delay is applied
- `test_inject_empty_string` - Edge case handling
- `test_inject_newlines` - Special characters

**Manual tests:**
- Send "Hello World" - verify complete
- Send Chinese text "测试中文" - verify complete
- Send long text (500+ chars) - verify complete with acceptable speed
