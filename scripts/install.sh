#!/usr/bin/env bash

set -euo pipefail

# Default values
APP_NAME="vk"
VERSION="v0.1.0-alpha.5"

# Parse arguments
while [[ $# -gt 0 ]]; do
  case "$1" in
    -v|--version)
      VERSION="$2"
      shift 2
      ;;
    --vk)
      APP_NAME="vk"
      shift
      ;;
    --vk-ci)
      APP_NAME="vk-ci"
      shift
      ;;
    -h|--help)
      echo "Usage: $0 [--vk | --vk-ci] [-v version]"
      exit 0
      ;;
    *)
      echo "Unknown argument: $1"
      exit 1
      ;;
  esac
done

INSTALL_DIR="$HOME/.${APP_NAME}"
BIN_DIR="$INSTALL_DIR/bin"

echo "📦 Installing $APP_NAME version $VERSION..."

# Check for curl
if ! command -v curl >/dev/null 2>&1; then
  echo "❌ Error: curl is required but not installed."
  exit 1
fi

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux*) OS="linux" ;;
  Darwin*) OS="macos" ;;
  MINGW*|MSYS*|CYGWIN*) OS="windows" ;;
  *) echo "❌ Unsupported OS: $OS" && exit 1 ;;
esac

case "$ARCH" in
  x86_64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="arm64" ;;
  *) echo "❌ Unsupported architecture: $ARCH" && exit 1 ;;
esac

case "$OS" in
  linux) PLATFORM="unknown-linux-gnu" ;;
  macos) PLATFORM="apple-darwin" ;;
  windows) PLATFORM="pc-windows-msvc" ;;
esac

BINARY="$APP_NAME-$ARCH-$PLATFORM"

# Add extension for Windows
if [ "$OS" = "windows" ]; then
  BINARY="$BINARY.exe"
fi

DOWNLOAD_URL="https://github.com/vayload/vayload-kit/releases/download/$VERSION/$BINARY"

echo "🌐 Downloading $DOWNLOAD_URL..."

mkdir -p "$BIN_DIR"

# Download with progress bar
if command -v pv >/dev/null 2>&1; then
  curl -L "$DOWNLOAD_URL" | pv -n > "$BIN_DIR/$APP_NAME"
else
  curl -# -L "$DOWNLOAD_URL" -o "$BIN_DIR/$APP_NAME"
fi

# Make executable or rename for Windows
if [ "$OS" = "windows" ]; then
  mv "$BIN_DIR/$APP_NAME" "$BIN_DIR/$APP_NAME.exe"
else
  chmod +x "$BIN_DIR/$APP_NAME"
fi

# Add to PATH if not already there
if ! echo "$PATH" | grep -q "$BIN_DIR"; then
  SHELL_NAME="$(basename "${SHELL:-bash}")"
  case "$SHELL_NAME" in
    bash) PROFILE="$HOME/.bashrc" ;;
    zsh) PROFILE="$HOME/.zshrc" ;;
    *) PROFILE="$HOME/.profile" ;;
  esac

  if ! grep -qF "$BIN_DIR" "$PROFILE"; then
    echo "export PATH=\"$BIN_DIR:\$PATH\"" >> "$PROFILE"
    echo "✅ Added $BIN_DIR to PATH in $PROFILE"
  fi

  echo "⚠ Restart your terminal to apply changes."
fi

echo ""
echo "🎉 Installation complete!"
echo "Run:"
echo "$APP_NAME"
