@echo off
title ClutchCtl USB Driver Installer
color 0A

echo ============================================
echo     ClutchCtl USB Driver Installer
echo ============================================
echo.
echo This will help you install the WinUSB driver
echo for your USB foot pedal device.
echo.
echo Supported devices:
echo   - iKKEGOL (VID: 1a86, PID: e026)
echo   - PCsensor FootSwitch:
echo     * VID: 3553, PID: b001
echo     * VID: 0c45, PID: 7403
echo     * VID: 0c45, PID: 7404
echo     * VID: 413d, PID: 2107
echo   - Scythe (VID: 0426, PID: 3011)
echo   - Scythe2 (VID: 055a, PID: 0998)
echo   - Single Pedal (VID: 5131, PID: 2019)
echo.
echo Please make sure your pedal is connected.
echo.
pause

echo.
echo Checking for Zadig...

if exist "%~dp0zadig.exe" (
    echo Found Zadig in current directory
    set ZADIG_PATH=%~dp0zadig.exe
) else if exist "%~dp0zadig-2.8.exe" (
    echo Found Zadig 2.8 in current directory
    set ZADIG_PATH=%~dp0zadig-2.8.exe
) else (
    echo.
    echo Zadig not found. Attempting to download...
    echo.
    powershell -Command "& {[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; Invoke-WebRequest -Uri 'https://github.com/pbatard/libwdi/releases/download/v1.5.0/zadig-2.8.exe' -OutFile '%TEMP%\zadig.exe'}"

    if exist "%TEMP%\zadig.exe" (
        set ZADIG_PATH=%TEMP%\zadig.exe
        echo Download successful!
    ) else (
        echo.
        echo ERROR: Could not download Zadig.
        echo Please download it manually from: https://zadig.akeo.ie/
        echo.
        pause
        exit /b 1
    )
)

echo.
echo ============================================
echo     INSTRUCTIONS FOR ZADIG
echo ============================================
echo.
echo 1. In the Zadig window that opens:
echo.
echo 2. Click "Options" menu, then "List All Devices"
echo.
echo 3. Select your pedal device from the dropdown:
echo    - Look for "FS2020U1IR" or "FootSwitch"
echo    - Or identify by VID/PID shown above
echo.
echo 4. Make sure "WinUSB" is selected as the driver
echo.
echo 5. Click "Install Driver" or "Replace Driver"
echo.
echo 6. Wait for installation to complete
echo.
echo ============================================
echo.
echo Starting Zadig now...
echo.

start "" "%ZADIG_PATH%"

echo.
echo After installing the driver in Zadig, you can
echo close this window and start using clutchctl.exe
echo.
echo Example commands:
echo   clutchctl.exe list
echo   clutchctl.exe show 0
echo   clutchctl.exe set 0 left keyboard "ctrl+s"
echo.
pause