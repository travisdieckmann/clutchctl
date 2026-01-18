# Windows Build and Usage Guide

## Native Windows HID Support

ClutchCtl now uses the native Windows HID driver via the `hidapi` library. **No additional driver installation (like Zadig) is required!** The foot pedal devices work directly with the built-in Windows HID class driver.

## Building for Windows (Cross-compilation from Linux)

### Prerequisites

Install the MinGW cross-compiler:

```bash
sudo apt-get update
sudo apt-get install gcc-mingw-w64-x86-64
```

### Building

```bash
# Add Windows target (only needed once)
rustup target add x86_64-pc-windows-gnu

# Build for Windows
cargo build --release --target x86_64-pc-windows-gnu

# Or use the build script
./build-windows.sh
```

The executable will be at: `target/x86_64-pc-windows-gnu/release/clutchctl.exe`

## Building on Windows (Native)

### Prerequisites

1. Install [Rust for Windows](https://www.rust-lang.org/tools/install)
2. Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022) or Visual Studio with C++ support

### Building

```powershell
# Build for Windows
cargo build --release

# The executable will be at:
# target\release\clutchctl.exe
```

## Running on Windows

### Command Prompt

```cmd
clutchctl.exe list
clutchctl.exe show 0
clutchctl.exe set 0 1 keyboard "ctrl+s"
```

### PowerShell

```powershell
.\clutchctl.exe list
.\clutchctl.exe show 0
.\clutchctl.exe set 0 1 keyboard "ctrl+s"
```

## Troubleshooting

### "Access Denied" or "Permission Denied"

- Try running as Administrator
- Check that no other program is using the device (close any vendor configuration software)
- Unplug and replug the device

### "Device Not Found"

- Verify the device is connected
- Check Device Manager - the device should appear under "Human Interface Devices"
- Make sure the device is not being used by another application

### Previously Used Zadig

If you previously installed a WinUSB driver using Zadig for this device, you may need to restore the original HID driver:

1. Open Device Manager
2. Find your foot pedal device (may be under "Universal Serial Bus devices")
3. Right-click → "Update driver"
4. Select "Browse my computer for drivers"
5. Select "Let me pick from a list of available drivers"
6. Choose "USB Input Device" or "HID-compliant device"
7. Click "Next" to install the HID driver

After restoring the HID driver, the device should work with ClutchCtl without any additional configuration.

## Distribution

When distributing the Windows executable:

1. The `.exe` file is standalone (statically linked)
2. **No driver installation is needed** - uses native Windows HID driver
3. Users can simply run the executable after connecting their device

## Notes

- The Windows version uses the same USB protocol as Linux/macOS
- All configuration commands work identically across platforms
- Performance should be comparable to the Linux version
- The executable is about 5-10 MB (includes Rust runtime)
- Uses native Windows HID API via `hidapi` library
