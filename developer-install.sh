#!/usr/bin/env bash
set -euo pipefail

# User confirmation for install.
echo "Install steps:"
echo "  1. Check prerequisites"
echo "  2. Clone arti-chat repository to ~/.local/src"
echo "  3. Build project from source"
echo "   3a. Cargo install --path arti-chat-daemon-bin"
echo "  4. Install binaries in ~/.local/bin"
echo
echo "Press Ctrl + C to abort or wait 15 seconds to continue..."
sleep 15

# Check prerequisites.
echo
echo
echo "1. Checking prerequisites..."

MISSING_DEPS=false

require() {
    if (( $# == 1 )); then
        if command -v "$1" >/dev/null 2>&1; then
            printf "  ✅"
        else
            printf "  ❌"
            MISSING_DEPS=true
        fi
        printf " $1\n"
    else
        if "$@" --version >/dev/null 2>&1; then
            printf "  ✅"
        else
            printf "  ❌"
            MISSING_DEPS=true
        fi
        printf " $*\n"
    fi
}

require git
require rustc
require cargo
require cargo tauri
require npm

echo
if [ "$MISSING_DEPS" = true ]; then
    echo "❌ One or more required dependencies are missing..."
    echo "Install them and retry installing..."
    exit 1
fi
echo "✅ All prerequisites are satisfied."

# Clone repo.
echo
echo
echo "2. Cloning arti-chat repository..."
REPO="https://github.com/NielDuysters/arti-chat.git"
SRC_DIR="$HOME/.local/src/arti-chat"
BIN_DIR="$HOME/.local/bin"

if [ ! -d "$SRC_DIR" ]; then
    git clone "$REPO" "$SRC_DIR"
    echo "✅ git clone success"
else
    echo "Repo already exists... Pulling latest changes..."
    git -C "$SRC_DIR" pull --rebase
    echo "✅ git pull success"
fi


# Build project.
echo
echo
echo "3. Building project..."
ARTI_CHAT_DAEMON_BIN="./target/release/arti-chat-daemon-bin"
ARTI_CHAT_DESKTOP_APP_BIN="./target/release/arti-chat-desktop-app"

cd "$SRC_DIR"

echo
echo "Building arti-chat-daemon-bin..."
cargo build --release -p arti-chat-daemon-bin
if [ ! -x "$ARTI_CHAT_DAEMON_BIN" ]; then
    echo "❌ Failed to find arti-chat-daemon-bin binary after build..."
    exit 1
fi
echo "✅ arti-chat-daemon-bin build successfully."
echo "3a. Installing arti-chat-daemon-bin as command..."
cargo install --path arti-chat-daemon-bin
if command -v "arti-chat-daemon-bin" >/dev/null 2>&1; then
    echo "✅ arti-chat-daemon-bin installed as command..."
else
    echo "❌ Failed to install arti-chat-daemon-bin as command..."
    exit 1
fi

OS="$(uname)"
case "$OS" in
    Darwin)
        echo "Copying arti-chat-daemon-bin to external tauri binaries..."
        cp "$ARTI_CHAT_DAEMON_BIN" "arti-chat-desktop-app/src-tauri/binaries/arti-chat-daemon-bin-aarch64-apple-darwin"
        ;;
esac

echo
echo "Building arti-chat-desktop-app..."
cd arti-chat-desktop-app
npm install
cargo tauri build
cd ..
if [ ! -x "$ARTI_CHAT_DESKTOP_APP_BIN" ]; then
    echo "❌ Failed to find arti-chat-desktop-app binary after build..."
    exit 1
fi
echo "✅ arti-chat-desktop-app build successfully."

# Install binaries.
echo
echo
echo "4. Installing binaries..."
echo "Installing arti-chat-daemon-bin..."
rm -f "$BIN_DIR/arti-chat-daemon-bin"
CARGO_ARTI_CHAT_DAEMON_BIN="$(command -v arti-chat-daemon-bin)"
if [ ! -x "$CARGO_ARTI_CHAT_DAEMON_BIN" ]; then
    echo "❌ arti-chat-daemon-bin not found in PATH"
    exit 1
fi
mkdir -p "$BIN_DIR"
install -m 755 "$CARGO_ARTI_CHAT_DAEMON_BIN" "$BIN_DIR/arti-chat-daemon-bin"
echo "✅ arti-chat-daemon-bin installed to $BIN_DIR."

echo "Installing arti-chat-desktop-app..."

case "$OS" in
    Linux)
        echo
        echo "Installing arti-chat-desktop-app (Linux)..."
        install -m 755 "$ARTI_CHAT_DESKTOP_APP_BIN" "$BIN_DIR/arti-chat"
        echo "✅ arti-chat-desktop-app installed to $BIN_DIR as arti-chat..."
        ;;

    Darwin)
        APP_BUNDLE_DIR="$SRC_DIR/target/release/bundle/macos"
        APP_BUNDLE="$(ls -d "$APP_BUNDLE_DIR"/*.app 2>/dev/null | head -1)"

        if [ -z "$APP_BUNDLE" ]; then
          echo "❌ .app bundle not found..."
          exit 1
        fi

        echo
        echo "MacOS build complete..."
        echo
        echo "Follow these instructions to install:"
        echo "The app bundle is located at:"
        echo "  $APP_BUNDLE"
        echo
        echo "To install it, run:"
        echo
        echo "  sudo cp -R \"$APP_BUNDLE\" /Applications/"
        echo
        echo "Or drag it into Applications manually."
        ;;
esac

