# Windows Terminal Color Support

## Problem
On older Windows terminals (Command Prompt, older PowerShell), colored output appears as garbled ANSI escape codes like `←[1;36m[0]←[0m`.

## Solution
ClutchCtl now includes automatic detection and handling of terminal color support on Windows:

### Automatic Detection
The application automatically:
1. Attempts to enable ANSI color support on Windows 10+
2. Detects if running in a compatible terminal (Windows Terminal, VSCode, etc.)
3. Disables colors if the terminal doesn't support them

### Supported Terminals on Windows
Colors work correctly in:
- **Windows Terminal** (recommended)
- **PowerShell 7+**
- **VSCode integrated terminal**
- **Windows 10+ Command Prompt** (with virtual terminal processing)
- **Git Bash / MinGW / Cygwin**

### Manual Control
If you still see garbled output, you can manually disable colors:

```powershell
# Option 1: Use --no-color flag
clutchctl --no-color list

# Option 2: Set NO_COLOR environment variable
$env:NO_COLOR = "1"
clutchctl list
```

### For Older Windows Versions
On Windows versions before Windows 10 (Windows 7, 8, 8.1), colors are automatically disabled in Command Prompt to prevent garbled output.

## Recommendation
For the best experience on Windows, we recommend using **Windows Terminal**, which is available for free from the Microsoft Store and provides full ANSI color support along with many other modern terminal features.