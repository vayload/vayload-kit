#!/usr/bin/env bash

set -euo pipefail

# Default values
APP_NAME="vk"
VERSION="v0.1.0-alpha.3"

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

INSTALL_DIR="$HOME/.vk"
BIN_DIR="$INSTALL_DIR/bin"

echo "Building $APP_NAME (release)..."

if [ "$APP_NAME" = "vk-ci" ]; then
  make build-minimal-release
else
  make build-full-release
fi

echo "Installing locally..."

mkdir -p "$BIN_DIR"

# Detect Windows binary
if [ -f "target/release/$APP_NAME.exe" ]; then
  cp "target/release/$APP_NAME.exe" "$BIN_DIR/$APP_NAME.exe"
else
  cp "target/release/$APP_NAME" "$BIN_DIR/$APP_NAME"
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
echo "Done."
echo "Restart your terminal or run:"
echo "export PATH=\"$BIN_DIR:\$PATH\""
echo ""
echo "Then test with:"
echo "$APP_NAME --help"
