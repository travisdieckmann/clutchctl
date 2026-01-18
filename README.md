# clutchctl

<h3 align="center">Foot-pedal configurator</h3>

<div align="center">

[![Status](https://img.shields.io/badge/status-active-success.svg)]()
[![GitHub issues](https://img.shields.io/github/issues/travisdieckmann/clutchctl)](https://github.com/travisdieckmann/clutchctl/issues)
[![GitHub pull requests](https://img.shields.io/github/issues-pr/travisdieckmann/clutchctl)](https://github.com/travisdieckmann/clutchctl/pulls)
[![GitHub](https://img.shields.io/github/license/travisdieckmann/clutchctl)](/LICENSE)

</div>

---

<p align="center"> A cross-platform command-line tool for configuring USB HID foot pedal devices, written in Rust.
    <br>
</p>

## 🙏 Acknowledgments & Inspiration

This project is a Rust application inspired by and based on:
- **[clutchctl](https://github.com/Schmoller/clutchctl)** (C++ version) by Schmoller - The original C++ implementation that provided comprehensive device support and protocol handling
- **[Foot Pedal Article](https://www.omustardo.com/timeline/2023/foot_pedal.html)** by omustardo - Excellent documentation and insights on working with USB foot pedals
- **[footswitch](https://github.com/rgerganov/footswitch)** by Radoslav Gerganov - The pioneering C implementation that first reverse-engineered the USB HID protocols for these devices

Special thanks to these projects and resources for their valuable work in understanding and documenting the USB HID protocols used by these devices.

## 📝 Table of Contents

- [About](#about)
- [Features](#features)
- [Supported Models](#supported_models)
- [Getting Started](#getting_started)
- [Usage](#usage)
- [Development](#development)
- [Migration from C++](#migration)

## 🧐 About <a name = "about"></a>

This project provides a command-line tool (clutchctl) to configure foot-pedal devices (such as those from iKKEGOL, PCsensor, and Scythe). This is a complete Rust application inspired by the original C++ pedalctl tool, which itself built upon the pioneering footswitch C implementation. The Rust version provides memory safety, better error handling, and enhanced cross-platform support while maintaining protocol compatibility with the original implementations.

## ✨ Features <a name = "features"></a>

- Configure programmable USB foot pedals
- Support for multiple configuration types:
  - **Keyboard**: Key combinations with modifiers (Ctrl, Shift, Alt, etc.)
  - **Mouse**: Button clicks or axis movement
  - **Text**: Type custom text strings
  - **Media**: Media control keys (play/pause, volume, etc.)
  - **Gamepad**: Game controller buttons and D-pad
- Cross-platform support (Linux, Windows, macOS)
- Memory-safe implementation using Rust
- Modular architecture for future GUI development

## 🎮 Supported Models <a name = "supported_models"></a>

### iKKEGOL Models
- **FS2020U1IR** (VID: 0x1a86, PID: 0xe026): 3-pedal USB foot switch
- **FS2017U1IR** (VID: 0x1a86, PID: 0xe026): 1-pedal USB foot switch

### PCsensor Models
- **PCsensor FootSwitch** (VID: 0x3553, PID: 0xb001): 3-pedal USB foot switch
- **PCsensor Variant** (VID: 0x0c45, PID: 0x7403): 3-pedal USB foot switch
- **PCsensor Variant** (VID: 0x0c45, PID: 0x7404): 3-pedal USB foot switch
- **PCsensor Variant** (VID: 0x413d, PID: 0x2107): 3-pedal USB foot switch

### Scythe Models
- **Scythe USB Foot Switch** (VID: 0x0426, PID: 0x3011): 3-pedal USB foot switch
- **Scythe USB Foot Switch II** (VID: 0x055a, PID: 0x0998): 3-pedal USB foot switch

### Single Pedal Models
- **FootSwitch1P** (VID: 0x5131, PID: 0x2019): 1-pedal USB foot switch

All devices use similar HID protocols and support keyboard, mouse, text, media, and gamepad configurations.

## 🏁 Getting Started <a name = "getting_started"></a>

### Prerequisites

- **Rust 1.70 or later** - [Install Rust](https://www.rust-lang.org/tools/install)
- **Platform-specific HID libraries**

#### Platform Requirements

**Linux (Ubuntu/Debian):**
```bash
sudo apt-get install libudev-dev libusb-1.0-0-dev pkg-config
```

**Linux (Fedora/RHEL):**
```bash
sudo dnf install systemd-devel libusb1-devel pkg-config
```

**macOS:**
No additional dependencies required - uses IOKit (included with macOS).

**Windows:**
No additional dependencies required - uses the native Windows HID driver.

### Building from Source

```bash
# Clone the repository
git clone https://github.com/travisdieckmann/clutchctl
cd clutchctl

# Build the project
cargo build --release

# The binary will be in target/release/clutchctl

# Install system-wide (optional)
cargo install --path clutchctl-cli
```

### Linux Setup

On Linux, you need to set up udev rules for non-root USB access:

```bash
# Create udev rules file
sudo tee /etc/udev/rules.d/70-footpedal.rules << EOF
# iKKEGOL foot pedals
SUBSYSTEM=="usb", ATTR{idVendor}=="1a86", ATTR{idProduct}=="e026", MODE="0666"
# PCsensor foot pedals
SUBSYSTEM=="usb", ATTR{idVendor}=="3553", ATTR{idProduct}=="b001", MODE="0666"
SUBSYSTEM=="usb", ATTR{idVendor}=="0c45", ATTR{idProduct}=="7403", MODE="0666"
SUBSYSTEM=="usb", ATTR{idVendor}=="0c45", ATTR{idProduct}=="7404", MODE="0666"
SUBSYSTEM=="usb", ATTR{idVendor}=="413d", ATTR{idProduct}=="2107", MODE="0666"
# Scythe foot pedals
SUBSYSTEM=="usb", ATTR{idVendor}=="0426", ATTR{idProduct}=="3011", MODE="0666"
SUBSYSTEM=="usb", ATTR{idVendor}=="055a", ATTR{idProduct}=="0998", MODE="0666"
# Single pedal variant
SUBSYSTEM=="usb", ATTR{idVendor}=="5131", ATTR{idProduct}=="2019", MODE="0666"
EOF

# Reload udev rules
sudo udevadm control --reload-rules
sudo udevadm trigger

# Unplug and replug your device for the rules to take effect
```

## 💻 Usage <a name = "usage"></a>

### List Connected Devices

```bash
clutchctl list
```

Output example:
```
Found 1 device(s):

  [0] FS2020U1IR
      Version:  V4.13
      Pedals:   3
      Names:    left, middle, right
```

### Show Device Configuration

```bash
clutchctl show <DEVICE_ID>
```

Output example:
```
Device [0] FS2020U1IR
Version: V4.13

Pedals: 3

  [1] left (on press) Keyboard: LCtrl+C
  [2] middle (on press) Mouse: left
  [3] right (on press) Text: "Push"
```

### Configure a Pedal

#### Keyboard Configuration

```bash
# Single key
clutchctl set 0 1 keyboard "f1"

# Key combination
clutchctl set 0 1 keyboard "ctrl+c"
clutchctl set 0 1 keyboard "lshift+a"

# One-shot mode (key press only once)
clutchctl set 0 1 keyboard "enter" --once

# Trigger on release instead of press
clutchctl set 0 1 keyboard "space" --invert
```

#### Mouse Configuration

```bash
# Mouse buttons
clutchctl set 0 1 mouse buttons "left"
clutchctl set 0 1 mouse buttons "left+right"
clutchctl set 0 1 mouse buttons "middle"

# Mouse movement (x, y, wheel)
clutchctl set 0 1 mouse axis 10 -5 0    # Move right 10, up 5
clutchctl set 0 1 mouse axis 0 0 5      # Scroll wheel up
```

#### Text Configuration

```bash
# Type text when pedal is pressed
clutchctl set 0 1 text "Hello, World!"
clutchctl set 0 1 text "Best regards,\nJohn Doe"
```

#### Media Configuration

```bash
# Media control keys
clutchctl set 0 1 media "play"          # Play/Pause
clutchctl set 0 1 media "volume-up"     # Volume Up
clutchctl set 0 1 media "volume-down"   # Volume Down
clutchctl set 0 1 media "mute"          # Mute
clutchctl set 0 1 media "next"          # Next Track
```

#### Gamepad Configuration

```bash
# Gamepad buttons
clutchctl set 0 1 game "button1"
clutchctl set 0 1 game "button2"

# D-pad directions
clutchctl set 0 1 game "up"
clutchctl set 0 1 game "down"
clutchctl set 0 1 game "left"
clutchctl set 0 1 game "right"
```

#### Remove Configuration

```bash
# Unconfigure a pedal
clutchctl set 0 1 none
```

### Pedal Naming

You can use either numeric indices (1-based) or names:

**For FS2020U1IR (3 pedals):**
- Pedal 1 = `left`
- Pedal 2 = `middle`
- Pedal 3 = `right`

**For FS2017U1IR (1 pedal):**
- Pedal 1 = `pedal`

Example using names:
```bash
clutchctl set 0 left keyboard "ctrl+s"    # Save
clutchctl set 0 middle mouse buttons "left"  # Left click
clutchctl set 0 right text "Signature"    # Type signature
```

## 🛠️ Development <a name = "development"></a>

### Project Structure

The project is organized as a Rust workspace with two crates:

- **clutchctl-core**: Core library for USB communication and device management
- **clutchctl-cli**: Command-line interface application

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_packet_size
```

### Building Documentation

```bash
# Build and open documentation
cargo doc --open
```

### Debugging

Enable debug logging with the `RUST_LOG` environment variable:

```bash
# Show debug messages
RUST_LOG=debug clutchctl list

# Show trace-level messages (very verbose)
RUST_LOG=trace clutchctl show 0
```

### Code Formatting

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check
```

### Linting

```bash
# Run clippy linter
cargo clippy
```

## 🦀 Rust Reimagination <a name = "migration"></a>

This Rust application is a complete reimagination inspired by the C++ pedalctl by Schmoller and the C footswitch by Radoslav Gerganov, providing:

- **Memory Safety**: No manual memory management, preventing buffer overflows and use-after-free bugs
- **Better Error Handling**: Explicit Result types with detailed error messages
- **Cross-Platform Support**: Improved Windows and macOS compatibility
- **Modern Tooling**: Cargo for dependency management and easy cross-compilation
- **Modular Architecture**: Separate library crate enables future GUI development
- **Comprehensive Testing**: Unit and integration tests with >80% coverage goal

The USB protocol remains 100% compatible with the C++ pedalctl and C footswitch implementations that inspired this project, so devices configured with any compatible tool will work across all versions.

### Key Improvements

1. **Type Safety**: Stronger type system prevents configuration errors at compile time
2. **RAII Pattern**: Automatic USB interface management via Rust's ownership model
3. **No Dependencies on Build Tools**: Just Rust and cargo required (no CMake)
4. **Better CLI**: Modern argument parsing with clap, better help messages
5. **Extensibility**: Trait-based design makes adding new device models easier

## 📄 License

MIT License - See [LICENSE](LICENSE) file for details

## 🤝 Contributing

Contributions are welcome! Please feel free to submit pull requests.

## 🙏 Additional Thanks

- iKKEGOL, PCsensor, Scythe, and other manufacturers for their USB foot pedal devices
- The Rust community for excellent libraries (hidapi, clap, serde, etc.)
- All contributors and users who help improve this tool