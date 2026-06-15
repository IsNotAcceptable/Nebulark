# 🌌 Nebulark

> **AmneziaWG 2.0 mesh networking toolkit** - A high-performance, cross-platform solution for building secure mesh networks with AmneziaWG 2.0.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Language: Rust](https://img.shields.io/badge/Language-Rust-CE422B)](https://www.rust-lang.org/)
[![Status: WIP](https://img.shields.io/badge/Status-WIP-orange)]()

---

## Overview

Nebulark is a modular Rust framework for creating and managing WireGuard-based mesh networks. It provides:

-  **CLI tools** for network configuration and management
-  **Cryptographic primitives** (X25519 key exchange, base64 encoding)
-  **Cross-platform support** (Linux, Windows)
-  **Async-first architecture** (built on Tokio)
-  **AmneziaWG integration** (AWG - fork WireGuard)

---

## Architecture

Nebulark is organized as a **Rust workspace** with modular crates:

### Core Crates

| Crate | Purpose | Status |
|-------|---------|--------|
| **nebulark-cli** | Command-line interface for end users | Main entry point |
| **nebulark-core** | Core networking and mesh logic | Foundation |
| **nebulark-awg** | Async WireGuard wrapper | Protocol layer |
| **nebulark-common** | Shared types and utilities | Shared |
| **nebulark-platform-linux** | Linux-specific implementation | Platform-specific |
| **nebulark-platform-windows** | Windows-specific implementation | Platform-specific |

### Key Technologies

- **Tokio** - Async runtime with full feature set
- **Serde/JSON/TOML** - Configuration serialization
- **Tracing** - Structured logging and diagnostics
- **Clap** - CLI argument parsing
- **egui/eframe** - GUI framework (future UI)
- **X25519-Dalek** - Key exchange cryptography
- **Warp** - HTTP server for API endpoints
- **Reqwest** - HTTP client with TLS support

---

## Quick Start

### Prerequisites

- Rust 1.75+ (2021 edition)
- Cargo workspace support
- Platform requirements:
  - **Linux**: glibc-based systems
  - **Windows**: Windows 10+

### Building

```bash
# Clone the repository
git clone https://github.com/IsNotAcceptable/Nebulark.git
cd Nebulark

# Build all crates
cargo build --release

# Build specific crate
cargo build -p nebulark-cli --release
```

### Running

```bash
# Show CLI help
./target/release/nebulark --help

# Run with debug logging
RUST_LOG=debug ./target/release/nebulark [COMMAND]
```

---

## Crate Details

### nebulark-cli

Command-line interface for Nebulark operations.

**Key dependencies:**

- `nebulark-core` - Main networking logic
- `nebulark-awg` - WireGuard integration
- `clap` - CLI parsing with derive macros
- `dialoguer` - Interactive prompts
- `console` - Terminal formatting
- `indicatif` - Progress bars
- `warp` - REST API server

**Features:**

- nteractive configuration wizard
- JSON/TOML configuration support
- Real-time status monitoring
- HTTP API for remote management

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

Nebulark supports both JSON and TOML configuration formats:

```toml
# Example nebulark.toml
[network]
name = "my-mesh"
listen_port = 51820

[peers]
# Peer configurations here
```

```json
{
  "network": {
    "name": "my-mesh",
    "listen_port": 51820
  },
  "peers": []
}
```

---

## Logging & Diagnostics

Nebulark uses `tracing` for structured logging:

```bash
# Set log level
RUST_LOG=nebulark=debug cargo run

# Filter specific modules
RUST_LOG=nebulark_core=trace,nebulark_awg=debug cargo run

# Use env-filter syntax
RUST_LOG='nebulark[{ip="192.168.1.1"}]=debug' cargo run
```
**Log levels:** trace, debug, info, warn, error

---

## Testing

```bash
# Run all tests
cargo test --workspace

# Run tests for specific crate
cargo test -p nebulark-cli

# Run tests with logging
RUST_LOG=debug cargo test -- --nocapture
```

---

## Development

### Project Structure

```
Nebulark/
├── crates/
│   ├── nebulark-cli/          # Entry point
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
3. Commit changes: `git commit -am 'Add amazing feature'`
4. Push to branch: `git push origin feature/amazing-feature`
5. Submit a pull request

### Code Standards

Format code: `cargo fmt --all`
Lint: `cargo clippy --all-targets --all-features`
Document: `cargo doc --open`

---

## Troubleshooting

### Build Issues

Issue: `error: failed to compile`

Solution:

```bash
# Update dependencies
cargo update

# Clean build
cargo clean
cargo build
```

### Runtime Issues

Issue: `Permission denied on Linux`

Solution: WireGuard requires elevated privileges:
```bash
sudo ./target/release/nebulark [COMMAND]
```

---

## Documentation

- **API Docs:** `cargo doc --open`
- **Module Documentation:** See individual crate READMEs
- **Architecture:** See [Architecture](https://github.com/IsNotAcceptable/Nebulark/new/master?filename=README.md#architecture) section above

---

## License
This project is licensed under the MIT License - see [LICENSE](https://github.com/IsNotAcceptable/Nebulark/new/LICENSE) file for details.

---

## Roadmap

- [ ] Complete core mesh networking implementation
- [ ] GUI application 
- [ ] Peer auto-discovery
- [ ] Docker integration
- [ ] Comprehensive test suite
- [ ] Performance benchmarks
