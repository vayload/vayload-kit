#!/usr/bin/env sh

set -e

APP_NAME="vk" # Vayload kit
INSTALL_DIR="$HOME/.vk"
BIN_DIR="$INSTALL_DIR/bin"

echo "Building $APP_NAME (release)..."

cargo build --release

echo "Installing locally..."

mkdir -p "$BIN_DIR"

# Detect Windows binary
if [ -f "target/release/$APP_NAME.exe" ]; then
  cp "target/release/$APP_NAME.exe" "$BIN_DIR/$APP_NAME.exe"
else
  cp "target/release/$APP_NAME" "$BIN_DIR/$APP_NAME"
  chmod +x "$BIN_DIR/$APP_NAME"
fi

# Add to PATH if needed
if ! echo "$PATH" | grep -q "$BIN_DIR"; then
  echo ""
  echo "Adding $BIN_DIR to PATH..."

  SHELL_NAME="$(basename "$SHELL")"

  case "$SHELL_NAME" in
    bash) PROFILE="$HOME/.bashrc" ;;
    zsh) PROFILE="$HOME/.zshrc" ;;
    *) PROFILE="$HOME/.profile" ;;
  esac

  echo "export PATH=\"$BIN_DIR:\$PATH\"" >> "$PROFILE"
  echo "Added to $PROFILE"
fi

echo ""
echo "Done."
echo "Restart your terminal or run:"
echo "export PATH=\"$BIN_DIR:\$PATH\""
echo ""
echo "Then test with:"
echo "$APP_NAME --help"
