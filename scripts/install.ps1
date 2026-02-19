#!/usr/bin/env pwsh
# Requires PowerShell Core 7+
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

param(
    [string]$Version = "v0.1.0-alpha.5",
    [switch]$VK,
    [switch]$VKCI
)

# Determine APP_NAME
if ($VK) { $AppName = "vk" }
elseif ($VKCI) { $AppName = "vk-ci" }
else { $AppName = "vk" }

$InstallDir = Join-Path $HOME ".$AppName"
$BinDir = Join-Path $InstallDir "bin"
if (-not (Test-Path $BinDir)) { New-Item -ItemType Directory -Force -Path $BinDir | Out-Null }

# Detect OS
$OS = $PSVersionTable.OS
if ($IsWindows) {
    $OSName = "windows"; $Ext = "zip"
} elseif ($IsLinux) {
    $OSName = "linux"; $Ext = "tar.gz"
} elseif ($IsMacOS) {
    $OSName = "macos"; $Ext = "tar.gz"
} else {
    throw "Unsupported OS: $OS"
}

# Detect architecture
$Arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { throw "Unsupported architecture" }

# Platform string for releases
$Platform = switch ($OSName) {
    "windows" { "pc-windows-msvc" }
    "linux"   { "unknown-linux-gnu" }
    "macos"   { "apple-darwin" }
}

$BinaryName = "$AppName-$Arch-$Platform.$Ext"
$DownloadUrl = "https://github.com/vayload/vayload-kit/releases/download/$Version/$BinaryName"

Write-Host "🌐 Downloading $DownloadUrl..."
$TmpFile = [System.IO.Path]::GetTempFileName()
Invoke-WebRequest -Uri $DownloadUrl -OutFile $TmpFile

Write-Host "📦 Extracting $BinaryName..."
if ($Ext -eq "zip") {
    Expand-Archive -LiteralPath $TmpFile -DestinationPath $BinDir -Force
} else {
    # Use tar for Linux/macOS
    tar -xzf $TmpFile -C $BinDir
}
Remove-Item $TmpFile

# Make executables in Unix
if (-not $IsWindows) {
    Get-ChildItem -Path $BinDir -File | ForEach-Object { chmod +x $_.FullName }
}

# Add to PATH if not already
$CurrentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if (-not $CurrentPath.Split([System.IO.Path]::PathSeparator) -contains $BinDir) {
    [Environment]::SetEnvironmentVariable("PATH", "$BinDir$([System.IO.Path]::PathSeparator)$CurrentPath", "User")
    Write-Host "✅ Added $BinDir to PATH for current user"
    Write-Host "⚠ Restart your terminal to apply changes."
}

Write-Host ""
Write-Host "🎉 Installation complete!"
Write-Host "Run:"
Write-Host "$AppName"
