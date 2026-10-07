# DropVoice

English | [中文](README.zh.md)

Send voice-to-text input from your mobile phone to your PC via LAN.

## Quick Start

### Prerequisites

- [Node.js](https://nodejs.org/) ≥ 22
- [pnpm](https://pnpm.io/) ≥ 11
- [Rust](https://www.rust-lang.org/) ≥ 1.75

### Install

```bash
git clone https://github.com/jukanntenn/DropVoice.git
cd DropVoice
pnpm install
```

### Development

```bash
# Start the full Tauri app (frontend + backend)
pnpm tauri dev

# Frontend only (for UI development)
pnpm dev

# Mobile PWA (accessible on LAN)
pnpm dev:mobile
```

### Build

```bash
pnpm build          # Build frontend
pnpm tauri build    # Build production installer
```

### Test

```bash
pnpm test                    # Frontend tests
pnpm test:rust               # Rust backend tests
pnpm test:all                # All tests
cargo test -p dropvoice-pairing-server  # Pairing server tests
```

## Architecture

DropVoice is a Tauri application with:

- **Desktop app** (`apps/desktop/`) — Tauri v2 + React + Rust backend
- **Mobile PWA** (`apps/mobile/`) — React + WebSocket client
- **Pairing server** (`apps/pairing-server/`) — Axum HTTP service for device discovery
- **Shared packages** (`packages/`) — Core logic, UI components, i18n

```
┌─────────────────┐         ┌──────────────────┐         ┌─────────────────┐
│  Mobile Browser │ ──WS──> │  Tauri Backend   │ ──API──> │  React Frontend │
│  (PWA)          │         │  (Rust/Tokio)    │         │  (src/*.tsx)    │
└─────────────────┘         └──────────────────┘         └─────────────────┘
                                     │
                                     └──> Keyboard Injection (enigo)
```

## Configuration

Configuration is stored in `config.toml` (platform-specific config directory).

See [specs/full/01-configuration.md](specs/full/01-configuration.md) for the full configuration reference.

## Documentation

- [Configuration spec](specs/full/01-configuration.md) — the configuration mechanism
- [Architecture](docs/architecture.md) / [开发](docs/development.md) — how the system fits together, and how to work on it
- [Design System](DESIGN.md) — Visual language and design tokens
- [Changelog](CHANGELOG.md) — Version history

## License

MIT
