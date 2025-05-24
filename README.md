# Custom Wayland Compositor

A high-performance Wayland compositor built with Rust and Vulkan, optimized for 4K UI/UX development on Linux. This project features modern aesthetics with glassmorphism and neomorphism effects, anchored by a flagship side-docked app bar.

## Project Vision

This compositor is designed to power next-generation desktop environments with:
- **Performance-first**: Vulkan-accelerated rendering optimized for 4K displays
- **Modern aesthetics**: Glassmorphism and neomorphism UI effects
- **Extensibility**: Plugin-based architecture for easy feature expansion
- **Developer-friendly**: Built with Rust for safety and maintainability

## Architecture

The project uses a modular workspace structure:

```
crates/
├── compositor-core/     # Main Wayland compositor logic
├── vulkan-renderer/     # GPU-accelerated rendering engine
├── ui-framework/        # Custom UI system with modern effects
├── app-bar/            # Side-docked application bar
├── plugin-system/      # Plugin architecture and loading
├── config/             # Configuration management
├── ipc/                # Inter-process communication
└── utils/              # Shared utilities and types
```

## Current Status

**Version**: 0.1.1 - Working Wayland Server Foundation  
**Build Status**: ✅ All 8 crates compile cleanly  
**Testing Ready**: 🔄 Ready for live client connection testing

### Completed (Session 1)
- [x] **Multi-crate workspace** - 8 specialized crates with clear separation of concerns
- [x] **Complete Wayland server** - Full smithay integration with XDG shell support
- [x] **Socket management** - Auto WAYLAND_DISPLAY setup and client connection handling
- [x] **Plugin system foundation** - Registry, manifest, API, and loader modules
- [x] **Error handling** - Unified CompositorError system across all crates
- [x] **Build system** - Zero compilation errors, optimized for development and release
- [x] **Documentation** - Development diary tracking and comprehensive project docs

### Next Steps
- [ ] **Live testing** - Validate Wayland client connections with test script
- [ ] **Vulkan integration** - Connect surface buffers to GPU rendering pipeline
- [ ] **Basic rendering** - Display client windows with Vulkan-accelerated compositing
- [ ] **Input handling** - Route keyboard/mouse events to appropriate clients

## Development Phases

- **✅ Phase 1**: Wayland compositor foundation (COMPLETE)
- **🔄 Phase 2**: Vulkan integration and basic rendering (IN PROGRESS)
- **📋 Phase 3**: Custom UI framework with glassmorphism/neomorphism effects
- **📋 Phase 4**: Side-docked app bar implementation  
- **📋 Phase 5**: Plugin system and configuration management
- **📋 Phase 6**: Performance optimization and polish

## Quick Start

```bash
# Clone and build (all crates compile successfully)
git clone <repository-url>
cd custom_compositor
cargo build

# Test Wayland server (ready for client connections)
cargo run --bin custom-compositor

# In another terminal, test client connection
./test_client.sh
```

## Technology Stack

- **Core Language**: Rust (edition 2021)
- **Graphics API**: Vulkan (via `ash` crate)
- **Wayland Framework**: Smithay
- **Async Runtime**: Tokio
- **Build System**: Cargo workspace
- **Target Platform**: Debian 12 Linux, 4K displays

## Prerequisites

### System Dependencies
```bash
# Debian 12 / Ubuntu
sudo apt update
sudo apt install -y \
    build-essential \
    pkg-config \
    libwayland-dev \
    libxkbcommon-dev \
    libegl1-mesa-dev \
    libvulkan-dev \
    vulkan-tools \
    vulkan-validationlayers-dev \
    libdrm-dev \
    libgbm-dev \
    libudev-dev \
    libinput-dev \
    libseat-dev
```

### Rust Toolchain
```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Ensure you have the latest stable toolchain
rustup update stable
rustup default stable

# Install useful development tools
cargo install cargo-watch cargo-audit cargo-deny cargo-nextest
```

## Development Setup

```bash
# Clone the repository
git clone <repository-url>
cd custom_compositor

# Build the entire workspace
cargo build

# Run tests
cargo test

# Build with optimizations for testing
cargo build --release

# Run the compositor (when ready)
cargo run --bin compositor-core
```

## Development Commands

```bash
# Watch for changes and rebuild
cargo watch -x check -x test

# Run with detailed logging
RUST_LOG=debug cargo run --bin compositor-core

# Performance profiling build
cargo build --profile profiling

# Security audit
cargo audit

# Lint and format
cargo clippy --all-targets --all-features
cargo fmt --all
```

## Project Structure

Each crate serves a specific purpose:

- **`compositor-core`**: The heart of the Wayland compositor, handling window management, input, and compositor protocols
- **`vulkan-renderer`**: Abstraction layer over Vulkan for high-performance GPU rendering
- **`ui-framework`**: Custom UI system designed for modern effects and 4K displays
- **`app-bar`**: The signature side-docked application bar with always-on-top behavior
- **`plugin-system`**: Dynamic plugin loading and management system
- **`config`**: Configuration file parsing and runtime configuration management
- **`ipc`**: Inter-process communication for plugin and client interaction
- **`utils`**: Shared utilities, error types, and common functionality

## Design Philosophy

- **Performance over convenience**: Every design decision prioritizes performance for 4K rendering
- **Modularity**: Each component is independently testable and replaceable
- **Safety**: Leverage Rust's type system to prevent common systems programming errors
- **Extensibility**: Plugin system allows for easy feature additions without core changes
- **Modern UX**: Embrace contemporary design trends while maintaining usability

## Testing Strategy

- **Unit tests**: Each crate has comprehensive unit test coverage
- **Integration tests**: Cross-crate functionality testing
- **Performance tests**: Benchmarks for critical rendering paths
- **Visual regression tests**: Automated UI/UX validation
- **Hardware compatibility**: Testing across different GPU vendors and capabilities

## Performance Targets

- **Latency**: Sub-frame latency for input response (< 16.67ms @ 60Hz)
- **Throughput**: Smooth 60+ FPS at 4K resolution
- **Memory**: Efficient GPU memory usage with proper resource cleanup
- **Startup**: Fast compositor initialization (< 1 second)

## Contributing

This is a learning and development project. Contributions, suggestions, and feedback are welcome!

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
