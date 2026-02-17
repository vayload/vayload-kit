#!/usr/bin/env bash

set -euo pipefail

APP_NAME="vk"
INSTALL_DIR="$HOME/.vk"
BIN_DIR="$INSTALL_DIR/bin"

echo "Installing $APP_NAME..."

if ! command -v curl >/dev/null 2>&1; then
  echo "Error: curl is required but not installed."
  exit 1
fi

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux*) OS="linux" ;;
  Darwin*) OS="macos" ;;
  MINGW*|MSYS*|CYGWIN*) OS="windows" ;;
  *) echo "Unsupported OS: $OS" && exit 1 ;;
esac

case "$ARCH" in
  x86_64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="arm64" ;;
  *) echo "Unsupported architecture: $ARCH" && exit 1 ;;
esac

BINARY="$APP_NAME-$OS-$ARCH"

if [ "$OS" = "windows" ]; then
  BINARY="$BINARY.exe"
fi

DOWNLOAD_URL="https://github.com/alex-zweiter/vayload-kit/releases/latest/download/$BINARY"

echo "Downloading $DOWNLOAD_URL"

mkdir -p "$BIN_DIR"

curl -fsSL "$DOWNLOAD_URL" -o "$BIN_DIR/$APP_NAME"

if [ "$OS" = "windows" ]; then
  mv "$BIN_DIR/$APP_NAME" "$BIN_DIR/$APP_NAME.exe"
else
  chmod +x "$BIN_DIR/$APP_NAME"
fi

if ! echo "$PATH" | grep -q "$BIN_DIR"; then
  SHELL_NAME="$(basename "${SHELL:-bash}")"

  case "$SHELL_NAME" in
    bash) PROFILE="$HOME/.bashrc" ;;
    zsh) PROFILE="$HOME/.zshrc" ;;
    *) PROFILE="$HOME/.profile" ;;
  esac

  echo "export PATH=\"$BIN_DIR:\$PATH\"" >> "$PROFILE"
  echo "Added $BIN_DIR to PATH in $PROFILE"
  echo "Restart your terminal to apply changes."
fi

echo ""
echo "Installation complete!"
echo "Run:"
echo "$APP_NAME"
