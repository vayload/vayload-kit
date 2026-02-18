# Install-VK.ps1
# PowerShell script to install vk CLI with version/type arguments

param(
    [string]$AppName = "vk",      # Default binary
    [string]$Version = "v0.1.0-alpha.3"  # Default version
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$INSTALL_DIR = "$HOME\.${AppName}"
$BIN_DIR = "$INSTALL_DIR\bin"

# Print header
Write-Host "📦 Installing $AppName version $Version..." -ForegroundColor Cyan

# Check for curl
if (-not (Get-Command curl -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Error: curl is required but not installed." -ForegroundColor Red
    exit 1
}

# Detect OS and Architecture
if ($IsWindows) {
    $OS = "windows"
    $ARCH = (Get-CimInstance Win32_Processor).AddressWidth
    $PLATFORM = "pc-windows-msvc"
} elseif ($IsLinux) {
    $OS = "linux"
    $ARCH = (uname -m)
    $PLATFORM = "unknown-linux-gnu"
} elseif ($IsMacOS) {
    $OS = "macos"
    $ARCH = (uname -m)
    $PLATFORM = "apple-darwin"
} else {
    Write-Host "❌ Unsupported OS" -ForegroundColor Red
    exit 1
}

# Determine binary name
$BINARY = "$AppName-$Version-$ARCH-$PLATFORM"
if ($OS -eq "windows") { $BINARY += ".exe" }

$DOWNLOAD_URL = "https://github.com/vayload/vayload-kit/releases/download/$Version/$BINARY"
Write-Host "🌐 Downloading $DOWNLOAD_URL ..." -ForegroundColor Yellow

# Create bin directory
if (-not (Test-Path $BIN_DIR)) { New-Item -ItemType Directory -Path $BIN_DIR | Out-Null }
$DEST_PATH = Join-Path $BIN_DIR $BINARY

# Download file
Invoke-WebRequest -Uri $DOWNLOAD_URL -OutFile $DEST_PATH -UseBasicParsing

Write-Host "✅ Download complete: $DEST_PATH" -ForegroundColor Green

# Add to PATH if not already there
$CurrentPath = [System.Environment]::GetEnvironmentVariable("PATH", "User")
if (-not $CurrentPath.Split(";") -contains $BIN_DIR) {
    [System.Environment]::SetEnvironmentVariable("PATH", "$BIN_DIR;$CurrentPath", "User")
    Write-Host "🔧 Added $BIN_DIR to PATH. Restart your terminal to apply changes." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "🎉 Installation complete!"
Write-Host "Run:"
Write-Host "$AppName" -ForegroundColor Cyan
