# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Description

DropVoice enables sending voice-to-text input from your mobile phone to a PC.

## Common Commands

```bash
# Development
pnpm tauri dev              # Start full Tauri app (frontend + backend)
pnpm dev                    # Frontend only (for UI development on localhost:5173)

# Building
pnpm build                  # Build frontend (TypeScript compilation + Vite)
pnpm tauri build            # Build production installer
# Output: src-tauri/target/release/bundle/msi/DropVoice_0.0.1_x64_en-US.msi

# Icon Generation
pnpm tauri icon                # Generate all platform icons from app-icon.png
pnpm tauri icon <path>         # Use custom source icon file

# Rust Backend (cd src-tauri)
cargo build                    # Build Rust backend
cargo test                     # Run Rust tests
cargo clippy                   # Rust linter
```

## Architecture Overview

DropVoice is a Tauri application that enables text transmission from mobile browsers to a PC via LAN WebSocket connection.

### High-Level Architecture

```
┌─────────────────┐         ┌──────────────────┐         ┌─────────────────┐
│  Mobile Browser │ ──WS──> │  Tauri Backend   │ ──API──> │  React Frontend │
│  (public/*.html) │         │  (Rust/Tokio)    │         │  (src/*.tsx)    │
└─────────────────┘         └──────────────────┘         └─────────────────┘
                                     │
                                     └──> Keyboard Injection (enigo)
```

### Core Components

**Backend (Rust) - `src-tauri/src/`**

- **`commands.rs`**: Tauri command handlers exposed to frontend
  - `start_server`: Binds TCP port (default 38425), starts HTTP+WebSocket server
  - `stop_server`: Stops server (state management only - server runs for app lifetime)
  - `get_connection_info`: Returns server state, URL, and active connection count
  - Resource path resolution with multiple fallbacks for production bundle detection

- **`server/`**: Dual HTTP/WebSocket server on single port
  - **`http.rs`**: Serves static mobile interface (`public/mobile.html`) and handles WebSocket upgrade
  - **`websocket.rs`**: WebSocket message handling, connection tracking via `ConnectionManager`
  - Messages are JSON: `{"text": "content"}` → forwarded to text injector

- **`text/injector.rs`**: Windows keyboard input simulation using `enigo`
  - Simulates keystrokes at current cursor position
  - Handles special characters and newlines

- **`network/discovery.rs`**: LAN IP address detection
  - Finds local IPv4 address for QR code generation
  - Generates connection URL: `http://<local-ip>:<port>`

- **`lib.rs`**: Tauri app setup, system tray, window management
  - System tray with Show/Hide/Quit menu
  - Close button minimizes to tray (doesn't exit)
  - Server lifecycle managed via `ServerState` (AtomicBool + Mutex)

**Frontend (React/TypeScript) - `src/`**

- **`App.tsx`**: Main desktop UI
  - Auto-starts server on launch
  - Polls `get_connection_info` every 1 second for state updates
  - Displays QR code, connection URL, active client count
  - Prominent LAN-only security warning

- **`components/QRCodeDisplay.tsx`**: QR code generation from connection URL
- **`components/ConnectionInfo.tsx`**: Displays connection URL with copy button

- **`i18n/`**: Internationalization support
  - English and Chinese translations
  - Language detection and switching
  - Namespaced translation keys

- **Theme System**: Theme management
  - Light, dark, and system-following modes
  - Persistent theme preference
  - CSS variable-based theming

**Mobile Interface - `public/`**

- **`mobile.html`**: Standalone HTML page served by backend
- **`mobile.js`**: WebSocket client with auto-reconnect (max 5 attempts)
  - Draft text persistence (localStorage, namespaced by host)
  - Character counter, Enter/Ctrl+Enter shortcuts
- **`mobile.css`**: Mobile-optimized styling

### Data Flow

1. User opens DropVoice → Tauri app starts → `App.tsx` calls `start_server`
2. Server binds port → `discovery::get_connection_url()` detects LAN IP
3. QR code generated → User scans with mobile
4. Mobile browser loads `mobile.html` → WebSocket connection established
5. User types text → Clicks "Send to PC" → WebSocket message sent
6. Backend receives → `injector::inject_text()` simulates keyboard input
7. Text appears at cursor position on PC

### Key Design Decisions

- **Single-port server**: HTTP and WebSocket share port 38425 (HTTP serves mobile page, upgrade path for WebSocket)
- **Resource path fallbacks**: `resolve_resource_dir()` tries 15+ paths to find `public/` folder in dev/production bundles
- **Server lifetime**: Once started, server runs for app lifetime (stop button only updates state flag)
- **Connection tracking**: `ConnectionManager` uses `AtomicUsize` for thread-safe connection counting
- **Namespaced storage**: Mobile draft storage keyed by host to support multiple DropVoice instances on same LAN

## Configuration Files

- **`tauri.conf.json`**: Tauri app configuration
  - Window: 800x650, resizable, centered
  - Bundle target: MSI installer
  - Resources: `public/` folder included in bundle
  - System tray enabled

- **`package.json`**: Frontend dependencies
  - React 18, TypeScript, Vite
  - `@tauri-apps/api` and `@tauri-apps/plugin-clipboard-manager`

- **`src-tauri/Cargo.toml`**: Rust dependencies
  - `tokio` (async runtime), `hyper` (HTTP server), `tokio-tungstenite` (WebSocket)
  - `enigo` (keyboard injection), `local-ip-address` (network discovery)
  - `tracing` (structured logging)

## Security Considerations

**Current State (v0.0.1)**:

- No authentication beyond basic WebSocket handshake
- No encryption for WebSocket traffic
- Designed for trusted LAN networks only
- Prominent UI warnings for LAN-only usage

**When modifying security features**:

- Firewall rules should restrict to local network
- Future versions should add: E2E encryption, proper authentication, rate limiting, input sanitization

## Common Patterns

**Adding new Tauri commands**:

1. Define async function in `src-tauri/src/commands.rs` with `#[tauri::command]`
2. Add to `invoke_handler!` macro in `src-tauri/src/lib.rs`
3. Call from frontend using `invoke::<ReturnType>("command_name", { args })`

**Modifying mobile interface**:

- Edit `public/mobile.{html,css,js}` directly
- Changes are hot-reloaded in development
- No build step required (static files served directly by Rust backend)

**Resource path issues**:

- If mobile.html not found in production, check `resolve_resource_dir()` in `commands.rs`
- The function tries multiple fallback paths for different bundle scenarios
- Add logging to verify which path is being used

## Development Notes

- **Port conflicts**: If port 38425 is in use, server falls back to OS-assigned port
- **Window close behavior**: Closing window minimizes to tray (use tray menu or system tray to exit)
- **Auto-start**: Server automatically starts when app launches (see `App.tsx` useEffect)
- **Polling interval**: Frontend polls connection info every 1 second (could be optimized to event-driven)
- **Mobile testing**: Use `npm run tauri dev`, scan QR code with mobile on same LAN
