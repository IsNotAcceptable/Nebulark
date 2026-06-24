# 🌌 Nebulark

> **AmneziaWG 2.0 client** - A high-performance, cross-platform GUI client for AmneziaWG 2.0 secure tunneling.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Language: Rust](https://img.shields.io/badge/Language-Rust-CE422B)](https://www.rust-lang.org/)
[![Status: WIP](https://img.shields.io/badge/Status-WIP-orange)]()

---

## Overview

Nebulark is a modular Rust framework for creating and managing WireGuard-based mesh networks. It provides:

-  **Native GUI** built with egui/eframe — minimal dark theme
-  **Cryptographic primitives** (X25519 key exchange, base64 encoding)
-  **Cross-platform support** (Linux, Windows)
-  **Async-first architecture** (built on Tokio)
-  **AmneziaWG integration** (AWG - fork WireGuard with obfuscation)

---

## Architecture

Nebulark is organized as a **Rust workspace** with modular crates:

### Core Crates

| Crate | Purpose | Status |
|-------|---------|--------|
| **nebulark-ui** | Native GUI application (egui/eframe) | Main entry point |
| **nebulark-core** | Core networking and mesh logic | Foundation |
| **nebulark-awg** | Async WireGuard wrapper | Protocol layer |
| **nebulark-common** | Shared types and utilities | Shared |
| **nebulark-platform-linux** | Linux-specific implementation | Platform-specific |
| **nebulark-platform-windows** | Windows-specific implementation | Platform-specific |

### Key Technologies

- **Tokio** — Async runtime with full feature set
- **egui/eframe** — Native GUI framework, dark theme
- **Serde/TOML** — Configuration serialization
- **Tracing** — Structured logging and diagnostics
- **X25519-Dalek** — Key exchange cryptography
- **Reqwest** — HTTP client for Cloudflare WARP API
- **rfd** — Native file picker dialog

---

## Quick Start

### Prerequisites

- Rust 1.75+ (2021 edition)
- Platform requirements:
  - **Linux**: `amneziawg-dkms` kernel module + `amneziawg-tools`
  - **Windows**: Windows 10+ (Wintun driver, coming soon)

### Building

```bash
# Clone the repository
git clone https://github.com/IsNotAcceptable/Nebulark.git
cd Nebulark

# Build all crates
cargo build --release

# Build specific crate
cargo build -p nebulark-ui --release
```

### Running

```bash
# Launch GUI (will prompt for sudo automatically)
./target/release/nebulark

# Run daemon manually with debug logging
RUST_LOG=debug sudo ./target/release/nebulark daemon ~/.config/nebulark/config.toml myprofile
```

---

## Crate Details

### nebulark-ui

Native GUI application - main entry point for end users.

**Key dependencies:**

- `nebulark-core` - tunnel and profile management
- `nebulark-awg` - AmneziaWG integration
- `egui/eframe` - GUI framework
- `rfd` - native file picker dialog
- `reqwest` - Cloudflare WARP API client

**Features:**

- Profile list with select/connect/delete
- Native file picker for `.conf` import
- Cloudflare WARP config generator (built-in CF API registration)
- Daemon IPC (connect/disconnect without blocking UI)
- Auto sudo elevation on launch

### nebulark-awg

AWG wrapper - High-level abstraction over WireGuard operations.

**Key dependencies:**

- `nebulark-common` - Shared types
- `tokio` - Async runtime
- `base64` - Encoding/decoding

**Purpose:**

- Unified interface for WireGuard operations
- Async-first design for concurrent tunnel management
- Error handling and resilience

### nebulark-common

Shared utilities and types across all crates.

**Exports:**

- Common error types
- Shared data structures
- Configuration schemas
- Utility functions

### nebulark-core

Core mesh networking logic - The heart of Nebulark.

**Expected features:**

- Peer discovery and management
- Routing protocols
- Network topology management
- State synchronization

### nebulark-platform-linux & nebulark-platform-windows

Platform-specific implementations for Linux and Windows.

**Responsibilities:**

- OS-specific WireGuard configuration
- System integration (network namespaces, TUN/TAP)
- Platform-dependent utilities

---

## Configuration

Profiles are stored in `~/.config/nebulark/config.toml`:

```toml
[[profiles]]
name = "my-profile"

[profiles.tunnel]
private_key = "..."
addresses = ["10.0.0.1/32"]
dns = ["1.1.1.1"]
mtu = 1280

[[profiles.tunnel.peers]]
public_key = "..."
endpoint = "1.2.3.4:51820"
allowed_ips = ["0.0.0.0/0", "::/0"]
keepalive = 25

[profiles.tunnel.obfs]
jc = 4
jmin = 40
jmax = 70
s1 = 0
s2 = 0
h1 = 1
h2 = 2
h3 = 3
h4 = 4
```

---

## Logging & Diagnostics

```bash
# Daemon log
cat /tmp/nebulark-daemon.log

# Run with verbose logging
RUST_LOG=nebulark=debug ./target/release/nebulark

# Filter specific modules
RUST_LOG=nebulark_core=info,nebulark_platform_linux=debug ./target/release/nebulark
```

**Log levels:** trace, debug, info, warn, error

---

## Testing

```bash
# Run all tests
cargo test --workspace

# Run tests for specific crate
cargo test -p nebulark-core

# Run tests with logging
RUST_LOG=debug cargo test -- --nocapture
```

---

## Development

### Project Structure

```
Nebulark/
├── crates/
│   ├── nebulark-ui/           # GUI entry point
│   ├── nebulark-core/         # Core logic
│   ├── nebulark-awg/          # WireGuard wrapper
│   ├── nebulark-common/       # Shared code
│   ├── nebulark-platform-linux/
│   └── nebulark-platform-windows/
├── Cargo.toml                 # Workspace manifest
└── README.md                  # This file
```

### Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Commit changes: `git commit -am 'feat: add amazing feature'`
4. Push to branch: `git push origin feature/amazing-feature`
5. Submit a pull request

### Code Standards

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo doc --open
```

---

## Troubleshooting

### No internet after connecting

Check that the `amneziawg` kernel module is loaded:
```bash
lsmod | grep amneziawg
sudo modprobe amneziawg
```

### Stale interface after crash

```bash
sudo ip link del nebulark0
sudo ip -4 rule del table 51820 2>/dev/null
sudo ip -6 rule del table 51820 2>/dev/null
rm -f /tmp/nebulark.sock /tmp/nebulark.pid
```

### Permission denied

The GUI automatically requests sudo on launch. If it fails:
```bash
sudo ./target/release/nebulark
```

---

## Documentation

- **API Docs:** `cargo doc --open`
- **Module Documentation:** See individual crate source

---

## License
This project is licensed under the MIT License - see [LICENSE](LICENSE) file for details.

---

## Roadmap

- [x] Linux backend (amneziawg kernel module)
- [x] Native GUI (egui, dark theme)
- [x] Profile import via file picker
- [x] AWG 2.0 obfuscation params (Jc/S1-S4/H1-H4/I1)
- [x] Cloudflare WARP config generator
- [x] Daemon + IPC (non-blocking connect/disconnect)
- [ ] Windows backend (Wintun)
- [ ] System tray icon
- [x] Traffic statistics (rx/tx graph)
- [ ] Autoconnect on startup
- [ ] Packages (.deb, .rpm, AUR)
