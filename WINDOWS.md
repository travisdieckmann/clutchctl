# Windows Build and Usage Guide

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
3. Install [Zadig](https://zadig.akeo.ie/) for USB driver management

### Building

```powershell
# Build for Windows
cargo build --release

# The executable will be at:
# target\release\clutchctl.exe
```

## USB Driver Setup on Windows

Windows requires proper USB drivers for libusb to work. The distribution includes helper files for driver installation.

### Option 1: Automated Installation (Recommended)

Run the included batch file:
```cmd
install-driver.bat
```
This will download Zadig if needed and guide you through the installation.

### Option 2: INF File Installation

For systems with Windows Driver Kit:
1. Right-click `clutchctl-winusb.inf`
2. Select "Install"

Or from Administrator Command Prompt:
```cmd
pnputil /add-driver clutchctl-winusb.inf /install
```

### Option 3: Manual Zadig Installation

1. Download and run [Zadig](https://zadig.akeo.ie/)
2. Connect your USB pedal device
3. In Zadig:
   - Click "Options" → "List All Devices"
   - Select your device from the dropdown:
     - iKKEGOL: VID 1a86, PID e026
     - PCsensor: VID 3553, PID b001
   - Select "WinUSB" as the driver
   - Click "Install Driver"

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

- Make sure you've installed the WinUSB driver using Zadig
- Try running as Administrator
- Check that no other program is using the device

### "Device Not Found"

- Verify the device is connected
- Check Device Manager for the device
- Make sure the driver is installed for your specific device:
  - iKKEGOL devices: VID 1a86, PID e026
  - PCsensor FootSwitch: VID 3553, PID b001
- Reinstall the WinUSB driver using Zadig or the provided install-driver.bat

### "Cannot find -lusb-1.0"

- This means libusb is not properly installed
- The cross-compiled version from Linux includes libusb statically
- For native Windows builds, libusb should be bundled automatically by cargo

## Distribution

When distributing the Windows executable:

1. The `.exe` file is standalone (statically linked)
2. Users will need to install WinUSB driver using Zadig
3. Consider creating an installer that:
   - Includes the executable
   - Provides Zadig or driver installation instructions
   - Creates Start Menu shortcuts

## Creating a Windows Installer (Optional)

You can use [Inno Setup](https://jrsoftware.org/isinfo.php) to create an installer:

```iss
[Setup]
AppName=ClutchCtl
AppVersion=0.3.0
DefaultDirName={pf}\ClutchCtl
DefaultGroupName=ClutchCtl
OutputBaseFilename=clutchctl-setup

[Files]
Source: "clutchctl.exe"; DestDir: "{app}"
Source: "README.md"; DestDir: "{app}"; Flags: isreadme

[Icons]
Name: "{group}\ClutchCtl"; Filename: "{app}\clutchctl.exe"

[Run]
Filename: "https://zadig.akeo.ie/"; Description: "Download Zadig for USB driver setup"; Flags: shellexec postinstall skipifsilent
```

## Notes

- The Windows version uses the same USB protocol as Linux/macOS
- All configuration commands work identically across platforms
- Performance should be comparable to the Linux version
- The executable is about 5-10 MB (includes Rust runtime and libusb)