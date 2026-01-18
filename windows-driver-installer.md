# Windows Driver Installation Options for ClutchCtl

## Option 1: Bundle Zadig with Distribution (Simplest)

Create a Windows distribution package that includes:
```
clutchctl-windows/
├── clutchctl.exe
├── zadig-2.8.exe  (or latest version)
├── install-driver.bat
└── README.txt
```

### install-driver.bat
```batch
@echo off
echo ClutchCtl USB Driver Installer
echo ==============================
echo.
echo This will install the WinUSB driver for your USB pedal device.
echo Please make sure your pedal is connected.
echo.
pause
echo.
echo Starting Zadig driver installer...
echo.
echo In Zadig:
echo 1. Select your pedal device from the dropdown
echo    - iKKEGOL: VID 1a86, PID e026
echo    - PCsensor: VID 3553, PID b001
echo 2. Select "WinUSB" as the driver
echo 3. Click "Install Driver"
echo.
start zadig-2.8.exe
```

## Option 2: Automated Driver Installation with libwdi

Use libwdi (Windows Driver Install) library to programmatically install drivers.

### Rust Integration Approach

Create a separate `clutchctl-driver-installer` binary:

```rust
// Cargo.toml addition
[dependencies.libwdi-sys]
version = "0.1"
optional = true

[features]
driver-installer = ["libwdi-sys"]

[[bin]]
name = "clutchctl-driver-installer"
required-features = ["driver-installer"]
```

### Implementation (pseudo-code)
```rust
// src/bin/driver_installer.rs
use std::process;
use libwdi_sys::*;

fn main() {
    println!("ClutchCtl Driver Installer");

    // Check for admin rights
    if !is_admin() {
        eprintln!("This installer must be run as Administrator");
        process::exit(1);
    }

    // Install WinUSB for our devices
    install_driver(0x1a86, 0xe026, "iKKEGOL Pedal");
    install_driver(0x3553, 0xb001, "PCsensor FootSwitch");

    println!("Driver installation complete!");
}
```

## Option 3: PowerShell Script with Embedded Driver

Create a PowerShell script that downloads and installs the driver:

### install-clutchctl-driver.ps1
```powershell
#Requires -RunAsAdministrator

$ErrorActionPreference = "Stop"

Write-Host "ClutchCtl USB Driver Installer" -ForegroundColor Cyan
Write-Host "=============================" -ForegroundColor Cyan

# Check if Zadig is installed or download it
$zadigUrl = "https://github.com/pbatard/libwdi/releases/download/v1.5.0/zadig-2.8.exe"
$zadigPath = "$env:TEMP\zadig.exe"

if (-not (Test-Path $zadigPath)) {
    Write-Host "Downloading Zadig..." -ForegroundColor Yellow
    Invoke-WebRequest -Uri $zadigUrl -OutFile $zadigPath
}

# Create INF file for WinUSB
$infContent = @'
[Version]
Signature = "$Windows NT$"
Class = USBDevice
ClassGUID = {88BAE032-5A81-49f0-BC3D-A4FF138216D6}
Provider = %ManufacturerName%
CatalogFile = WinUSBInstallation.cat
DriverVer = 09/04/2012,13.54.20.543
PnpLockdown = 1

[Manufacturer]
%ManufacturerName% = Standard,NT$ARCH$

[Standard.NT$ARCH$]
%DeviceName1% = USB_Install, USB\VID_1a86&PID_e026
%DeviceName2% = USB_Install, USB\VID_3553&PID_b001

[USB_Install]
Include = winusb.inf
Needs = WINUSB.NT

[USB_Install.Services]
Include = winusb.inf
AddService = WinUSB,0x00000002,WinUSB_ServiceInstall

[USB_Install.Wdf]
KmdfService = WINUSB, WinUSB_Install

[USB_Install.HW]
AddReg = Dev_AddReg

[Dev_AddReg]
HKR,,DeviceInterfaceGUIDs,0x10000,"{9f543223-cede-4fa3-b376-a25ce9a30e74}"

[USB_Install.CoInstallers]
AddReg = CoInstallers_AddReg
CopyFiles = CoInstallers_CopyFiles

[CoInstallers_AddReg]
HKR,,CoInstallers32,0x00010000,"WdfCoInstaller$KMDFCOINSTALLERVERSION$.dll,WdfCoInstaller"

[CoInstallers_CopyFiles]
WdfCoInstaller$KMDFCOINSTALLERVERSION$.dll

[DestinationDirs]
CoInstallers_CopyFiles=11

[SourceDisksNames]
1 = %DiskName%,,,""

[SourceDisksFiles]
WdfCoInstaller$KMDFCOINSTALLERVERSION$.dll=1

[Strings]
ManufacturerName = "ClutchCtl"
DiskName = "ClutchCtl Installation Disk"
DeviceName1 = "iKKEGOL USB Pedal"
DeviceName2 = "PCsensor FootSwitch"
'@

$infPath = "$env:TEMP\clutchctl.inf"
$infContent | Out-File -FilePath $infPath -Encoding ASCII

# Install the driver
Write-Host "Installing WinUSB driver..." -ForegroundColor Green
pnputil /add-driver $infPath /install

Write-Host "Driver installation complete!" -ForegroundColor Green
```

## Option 4: Inno Setup Installer with Driver

Enhance the Inno Setup script to include driver installation:

### clutchctl-installer.iss
```iss
[Setup]
AppName=ClutchCtl
AppVersion=0.3.0
DefaultDirName={autopf}\ClutchCtl
DefaultGroupName=ClutchCtl
OutputBaseFilename=clutchctl-setup
PrivilegesRequired=admin
ArchitecturesInstallIn64BitMode=x64

[Files]
Source: "clutchctl.exe"; DestDir: "{app}"
Source: "zadig-2.8.exe"; DestDir: "{tmp}"; Flags: deleteafterinstall
Source: "README.md"; DestDir: "{app}"; Flags: isreadme

[Icons]
Name: "{group}\ClutchCtl"; Filename: "{app}\clutchctl.exe"
Name: "{group}\Install USB Driver"; Filename: "{tmp}\zadig-2.8.exe"

[Run]
; Optionally run Zadig automatically
Filename: "{tmp}\zadig-2.8.exe"; Parameters: "--vid 0x1a86 --pid 0xe026 --winusb"; \
  Description: "Install USB driver for iKKEGOL devices"; \
  Flags: postinstall skipifsilent

Filename: "{tmp}\zadig-2.8.exe"; Parameters: "--vid 0x3553 --pid 0xb001 --winusb"; \
  Description: "Install USB driver for PCsensor devices"; \
  Flags: postinstall skipifsilent

[Code]
function InitializeSetup(): Boolean;
begin
  Result := True;
  if not IsAdmin then
  begin
    MsgBox('This installer requires Administrator privileges to install USB drivers.', mbError, MB_OK);
    Result := False;
  end;
end;
```

## Option 5: Self-Installing Driver in Application

Add driver installation capability directly to clutchctl.exe:

### Implementation in Rust
```rust
// src/driver_installer.rs (Windows only)
#[cfg(windows)]
pub fn check_and_install_driver() -> Result<()> {
    use winreg::enums::*;
    use winreg::RegKey;

    // Check if WinUSB driver is installed
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    // Check for our device IDs in the registry
    let driver_needed = !is_driver_installed(0x1a86, 0xe026) ||
                        !is_driver_installed(0x3553, 0xb001);

    if driver_needed {
        println!("USB driver not found. Would you like to install it? (requires admin)");
        // Download and run Zadig or use embedded driver installer
        install_winusb_driver()?;
    }

    Ok(())
}
```

## Recommended Approach

**For immediate implementation:** Option 1 (Bundle Zadig)
- Simplest to implement
- No additional dependencies
- Clear user control

**For best user experience:** Option 4 (Inno Setup with driver)
- Professional installer
- Can automate driver installation
- Handles admin privileges properly

**For advanced integration:** Option 5 (Self-installing)
- Most seamless experience
- Requires more development effort
- Best for commercial distribution

## Driver-Free Alternative: WinUSB Co-installer

Include a `.inf` file that Windows can use to automatically install WinUSB:

### clutchctl.inf
```inf
; Installation inf for ClutchCtl USB devices
[Version]
Signature = "$Windows NT$"
Class = USBDevice
ClassGUID = {88BAE032-5A81-49f0-BC3D-A4FF138216D6}
Provider = %ManufacturerName%
DriverVer = 01/16/2025,0.3.0.0

[Manufacturer]
%ManufacturerName% = Standard,NTamd64

[Standard.NTamd64]
%DeviceName1% = USB_Install, USB\VID_1a86&PID_e026
%DeviceName2% = USB_Install, USB\VID_3553&PID_b001

[USB_Install]
Include = winusb.inf
Needs = WINUSB.NT

[USB_Install.Services]
Include = winusb.inf
AddService = WinUSB,0x00000002,WinUSB_ServiceInstall

[USB_Install.HW]
AddReg = Dev_AddReg

[Dev_AddReg]
HKR,,DeviceInterfaceGUIDs,0x10000,"{9f543223-cede-4fa3-b376-a25ce9a30e74}"

[Strings]
ManufacturerName = "ClutchCtl"
DeviceName1 = "iKKEGOL USB Pedal"
DeviceName2 = "PCsensor FootSwitch"
```

Users can right-click this file and select "Install" to set up the driver.