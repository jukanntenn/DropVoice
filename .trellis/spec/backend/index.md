# Backend Development Guidelines

> Best practices for backend development in this project.

---

## Overview

This directory contains guidelines for backend development in DropVoice. The backend is a Rust-based Tauri application that handles HTTP/WebSocket server, keyboard injection, and system integration.

---

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Directory Structure](./directory-structure.md) | Module organization and file layout | Filled |
| [Data Persistence](./database-guidelines.md) | File-based settings storage | Filled |
| [Error Handling](./error-handling.md) | Error types, handling strategies | Filled |
| [Quality Guidelines](./quality-guidelines.md) | Code standards, forbidden patterns | Filled |
| [Logging Guidelines](./logging-guidelines.md) | Structured logging, log levels | Filled |

---

## Quick Reference

### Project Type
- **Language**: Rust (Edition 2021)
- **Framework**: Tauri 2.x
- **Async Runtime**: Tokio
- **HTTP/WebSocket**: hyper + tokio-tungstenite
- **Logging**: tracing + tracing-subscriber

### Key Dependencies

| Crate | Purpose |
|-------|---------|
| `tauri` | Desktop app framework |
| `tokio` | Async runtime |
| `hyper` | HTTP server |
| `tokio-tungstenite` | WebSocket support |
| `enigo` | Keyboard injection |
| `anyhow` | Error handling |
| `tracing` | Structured logging |

### Architecture

```
┌─────────────────┐         ┌──────────────────┐         ┌─────────────────┐
│  Mobile Browser │ ──WS──> │  Tauri Backend   │ ──API──> │  React Frontend │
│  (public/*.html) │         │  (Rust/Tokio)    │         │  (src/*.tsx)    │
└─────────────────┘         └──────────────────┘         └─────────────────┘
                                     │
                                     └──> Keyboard Injection (enigo)
```

### Core Modules

| Module | Responsibility |
|--------|---------------|
| `lib.rs` | App lifecycle, tray, window events |
| `commands.rs` | Tauri IPC command handlers |
| `server/` | HTTP + WebSocket server |
| `network/` | IP discovery, URL generation |
| `text/` | Keyboard input simulation |

---

### Common Commands

```bash
# Build
cargo build                    # Debug build
cargo build --release          # Release build

# Test
cargo test                     # Run tests
cargo clippy                   # Lint

# Run with Tauri
pnpm tauri dev                 # Development
pnpm tauri build               # Production build
```

---

**Language**: All documentation is written in **English**.
