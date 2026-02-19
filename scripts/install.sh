#!/usr/bin/env bash
set -euo pipefail

APP_NAME="vk"
VERSION="v0.1.0-alpha.5"

while [[ $# -gt 0 ]]; do
  case "$1" in
    -v|--version) VERSION="$2"; shift 2 ;;
    --vk) APP_NAME="vk"; shift ;;
    --vk-ci) APP_NAME="vk-ci"; shift ;;
    -h|--help)
      echo "Usage: $0 [--vk | --vk-ci] [-v version]"
      exit 0
      ;;
    *) echo "Unknown argument: $1"; exit 1 ;;
  esac
done

INSTALL_DIR="$HOME/.${APP_NAME}"
BIN_DIR="$INSTALL_DIR/bin"
mkdir -p "$BIN_DIR"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux*) OS="linux" ;;
  Darwin*) OS="macos" ;;
  MINGW*|MSYS*|CYGWIN*) OS="windows" ;;
  *) echo "❌ Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
  x86_64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="arm64" ;;
  *) echo "❌ Unsupported architecture: $ARCH"; exit 1 ;;
esac

case "$OS" in
  linux) PLATFORM="unknown-linux-gnu" ;;
  macos) PLATFORM="apple-darwin" ;;
  windows) PLATFORM="pc-windows-msvc" ;;
esac

# Determine file extension
if [ "$OS" = "windows" ]; then
  EXT="zip"
else
  EXT="tar.gz"
fi

BINARY_NAME="$APP_NAME-$ARCH-$PLATFORM.$EXT"
DOWNLOAD_URL="https://github.com/vayload/vayload-kit/releases/download/$VERSION/$BINARY_NAME"

echo "🌐 Downloading $DOWNLOAD_URL..."
TMP_FILE="$(mktemp)"

curl -L "$DOWNLOAD_URL" -o "$TMP_FILE"

echo "📦 Extracting $BINARY_NAME..."
if [ "$EXT" = "tar.gz" ]; then
  tar -xzf "$TMP_FILE" -C "$BIN_DIR"
elif [ "$EXT" = "zip" ]; then
  unzip -o "$TMP_FILE" -d "$BIN_DIR"
fi

rm "$TMP_FILE"

# Make executables in Unix
if [ "$OS" != "windows" ]; then
  chmod +x "$BIN_DIR/$APP_NAME"
fi

# Add to PATH if not already
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
