$ErrorActionPreference = "Stop"

$AppName = "vk"
$InstallDir = "$HOME\.vk"
$BinDir = "$InstallDir\bin"

Write-Host "Installing $AppName..."

$Arch = if ([Environment]::Is64BitOperatingSystem) {
    if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") {
        "arm64"
    } else {
        "x86_64"
    }
} else {
    throw "Unsupported architecture"
}

$Binary = "$AppName-windows-$Arch.exe"
$DownloadUrl = "https://github.com/alex-zweiter/vayload-kit/releases/latest/download/$Binary"

Write-Host "Downloading $DownloadUrl"

New-Item -ItemType Directory -Force -Path $BinDir | Out-Null

Invoke-WebRequest $DownloadUrl -OutFile "$BinDir\$AppName.exe"

# Add to user PATH if not already there
$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")

if (-not ($UserPath -split ";" | Where-Object { $_ -eq $BinDir })) {
    [Environment]::SetEnvironmentVariable(
        "PATH",
        "$BinDir;$UserPath",
        "User"
    )
    Write-Host "Added $BinDir to PATH"
    Write-Host "Restart your terminal to apply changes."
}

Write-Host ""
Write-Host "Installation complete!"
Write-Host "Run:"
Write-Host "$AppName"
