# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ClutchCtl is a Rust command-line tool for configuring USB HID foot pedal devices. It supports iKKEGOL, PCsensor, Scythe, and other compatible foot switches. It's a complete Rust reimagination inspired by the C++ pedalctl by Schmoller and the C footswitch by Radoslav Gerganov, using a workspace structure with two crates:
- `clutchctl-core`: Reusable library for USB communication and device management
- `clutchctl-cli`: Command-line interface application

## Build & Development Commands

### Building
```bash
cargo build --release              # Release build (binary at target/release/clutchctl)
cargo build                        # Development build
cargo install --path clutchctl-cli  # Install system-wide
```

### Windows Cross-Compilation (from Linux)
```bash
# One-time setup
rustup target add x86_64-pc-windows-gnu
sudo apt-get install gcc-mingw-w64-x86-64

# Build
./build-windows.sh  # Uses Docker if MinGW not available
# Or manually:
cargo build --release --target x86_64-pc-windows-gnu
```

### Testing
```bash
cargo test                    # Run all tests
cargo test -- --nocapture    # With output
cargo test test_packet_size  # Specific test
```

### Code Quality
```bash
cargo fmt        # Format code
cargo clippy     # Lint
cargo doc --open # Generate docs
```

### Debugging
```bash
RUST_LOG=debug clutchctl list    # Enable debug logging
RUST_LOG=trace clutchctl show 0  # Trace level
```

## Architecture

### Layered Design
1. **USB Layer** (`clutchctl-core/src/usb/`): Low-level USB communication with RAII interface management
2. **Protocol Layer** (`clutchctl-core/src/protocol/`): Binary packet structures (40-byte fixed size), encoding/decoding
3. **Device Layer** (`clutchctl-core/src/device/`): `PedalDevice` trait, `IkkegolDevice` implementation, device discovery
4. **Configuration Layer** (`clutchctl-core/src/configuration/`): Keyboard, Mouse, Text, Media, Gamepad configuration types
5. **CLI Layer** (`clutchctl-cli/src/commands/`): list, show, set commands with Clap parsing

### Key USB Constants
- Vendor ID: `0x1a86`, Product ID: `0xe026`
- Config Interface: `1`, Config Endpoint: `0x02`
- Protocol: 40-byte binary packets

### Supported Device Models
- **iKKEGOL Models** (VID: 0x1a86, PID: 0xe026)
  - FS2020U1IR: 3 pedals (indexed 0-2, named left/middle/right)
  - FS2017U1IR: 1 pedal (indexed at 1, named "pedal")
- **PCsensor Models** (multiple VID/PIDs)
  - 0x3553:0xb001, 0x0c45:0x7403, 0x0c45:0x7404, 0x413d:0x2107
  - All have 3 pedals (left/middle/right)
- **Scythe Models**
  - Scythe (0x0426:0x3011): 3 pedals
  - Scythe2 (0x055a:0x0998): 3 pedals
- **Single Pedal Models**
  - FootSwitch1P (0x5131:0x2019): 1 pedal

### PedalDevice Trait
The core abstraction that enables extensibility for new device models. All device interactions go through this trait which handles configuration loading/saving and per-pedal configuration management.

## CLI Usage Examples

```bash
# List devices
clutchctl list

# Show configuration
clutchctl show 0

# Set configurations
clutchctl set 0 1 keyboard "ctrl+c"                    # Keyboard shortcut
clutchctl set 0 left keyboard "f1" --once --invert     # One-shot, on release
clutchctl set 0 1 mouse buttons "left+right"           # Mouse buttons
clutchctl set 0 1 mouse axis 10 -5 0                   # Mouse movement
clutchctl set 0 1 text "Hello, World!"                 # Text string
clutchctl set 0 1 media "play"                         # Media control
clutchctl set 0 1 game "button1"                       # Gamepad button
clutchctl set 0 1 none                                 # Unconfigure
```

Pedals can be referenced by index (1-based) or name (left/middle/right/pedal).

## Platform Requirements

### Linux
- libusb-1.0-dev required
- Udev rules needed for non-root access (see README.md Linux Setup section for configuration)

### Windows
- WinUSB driver installation via Zadig required
- Statically linked libusb in release builds

### macOS
- libusb via Homebrew (`brew install libusb`)

## Important Implementation Details

- Binary protocol maintains 100% compatibility with the C++ pedalctl and C footswitch implementations
- Configuration packets are exactly 40 bytes (validated in tests)
- USB timeout is 100ms (constant in `clutchctl-core/src/lib.rs`)
- Device discovery performs linear USB scan filtering by vendor/product ID
- UsbInterfaceLock uses RAII pattern for automatic interface claim/release
- Error handling converts rusb errors to user-friendly messages via PedalError enum

## Testing Focus Areas
- Protocol encoding/decoding correctness
- 40-byte packet structure validation
- Configuration type serialization
- USB communication error handling
- Cross-platform compatibility