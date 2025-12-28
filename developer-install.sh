#!/usr/bin/env bash
set -euo pipefail

# User confirmation for install.
echo "Install steps:"
echo "  1. Check prerequisites"
echo "  2. Clone arti-chat repository to ~/.local/src"
echo "  3. Build project from source"
echo "   3a. Cargo install --path arti-chat-daemon-bin"
echo "  4. Install launch agent/systemd service for daemon"
echo "  5. Install binaries in ~/.local/bin"
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
    git -C "$SRC_DIR" reset --hard
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

# Instaling launch agent for daemon.
OS="$(uname)"
echo
echo
echo "4. Installing launch agent for daemon..."
case "$OS" in
    Darwin)
        rm -f "$HOME/Library/LaunchAgents/com.arti-chat.daemon.plist"
        cp "arti-chat-daemon-bin/resources/com.arti-chat.daemon.plist" "$HOME/Library/LaunchAgents/"
        sed -i '' "s|%BIN_DIR%|$BIN_DIR|g" "$HOME/Library/LaunchAgents/com.arti-chat.daemon.plist"
        ;;
    Linux)
        mkdir -p "$HOME/.config/systemd/user/"
        rm -f "$HOME/.config/systemd/user/com.arti-chat.daemon.service"
        cp "arti-chat-daemon-bin/resources/com.arti-chat.daemon.service" "$HOME/.config/systemd/user/"
        sed -i "s|%BIN_DIR%|$BIN_DIR|g" "$HOME/.config/systemd/user/com.arti-chat.daemon.service"

        systemctl --user daemon-reexec
        systemctl --user daemon-reload
        systemctl --user enable --now "com.arti-chat.daemon.service"
        ;;
esac
echo "✅ Launch agent installed."

# Install binaries.
echo
echo
echo "5. Installing binaries..."
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
        echo "✅ Linux build complete for arti-chat-desktop-app."
        echo

        DEB_DIR="$SRC_DIR/target/release/bundle/deb"
        RPM_DIR="$SRC_DIR/target/release/bundle/rpm"
        APPIMAGE_DIR="$SRC_DIR/target/release/bundle/appimage"

        DEB_PKG="$(ls -1 "$DEB_DIR"/*.deb 2>/dev/null | head -1)"
        RPM_PKG="$(ls -1 "$RPM_DIR"/*.rpm 2>/dev/null | head -1)"
        APPIMAGE_PKG="$(ls -1 "$APPIMAGE_DIR"/*.AppImage 2>/dev/null | head -1)"

        if [ -n "$DEB_PKG" ]; then
            echo "📦 Debian/Ubuntu package found:"
            echo "  $DEB_PKG"
            echo
            echo "To install on Debian/Ubuntu:"
            echo
            echo "  sudo apt install \"$DEB_PKG\""
            echo
        fi

        if [ -n "$RPM_PKG" ]; then
            echo "📦 RPM package found:"
            echo "  $RPM_PKG"
            echo
            echo "To install on Fedora/RHEL/openSUSE:"
            echo
            echo "  sudo dnf install \"$RPM_PKG\""
            echo
        fi

        if [ -n "$APPIMAGE_PKG" ]; then
            echo "📦 AppImage found:"
            echo "  $APPIMAGE_PKG"
            echo
            echo "To run:"
            echo
            echo "  chmod +x \"$APPIMAGE_PKG\""
            echo "  \"$APPIMAGE_PKG\""
            echo
            echo "Note: AppImages do NOT auto-install launcher entries."
            echo "      Use AppImageLauncher or create a .desktop file if desired."
            echo
        fi

        if [ -z "$DEB_PKG" ] && [ -z "$RPM_PKG" ] && [ -z "$APPIMAGE_PKG" ]; then
            echo "❌ No Linux bundles found."
            exit 1
        fi
        ;;

    Darwin)
        APP_BUNDLE_DIR="$SRC_DIR/target/release/bundle/macos"
        APP_BUNDLE="$(ls -d "$APP_BUNDLE_DIR"/*.app 2>/dev/null | head -1)"

        if [ -z "$APP_BUNDLE" ]; then
          echo "❌ .app bundle not found..."
          exit 1
        fi

        echo
        echo "✅ MacOS build complete..."
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

