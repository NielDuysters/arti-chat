#!/usr/bin/env bash
set -euo pipefail

# User confirmation for install.
echo "Install steps:"
echo "  1. Check prerequisites"
echo "  2. Clone arti-chat repository"
echo "  3. Build project from source"
echo "  4. Install binaries in ~/.local/bin"
echo
echo "Press Ctrl + C to abort or wait 10 seconds to continue..."
sleep 10

# Check prerequisites.
echo
echo
printf "Checking prerequisites...\n"
require() {
    if (( $# == 1 )); then
        if command -v "$1" >/dev/null 2>&1; then
            printf "  ✅"
        else
            printf "  ❌"
        fi
        printf " $1\n"
    else
        if "$@" --version >/dev/null 2>&1; then
            printf "  ✅"
        else
            printf "  ❌"
        fi
        printf " $*\n"
    fi
}

require git
require rustc
require cargo
require cargo tauri
require npm

# Clone repo.
echo
echo
printf "Cloning arti-chat repository...\n"
REPO_URL="https://github.com/NielDuysters/arti-chat.git"
REPO_DIR="arti-chat"

if [ -d "$REPO_DIR" ]; then
  echo "❌ Directory '$REPO_DIR' already exists."
  echo "Remove it or run in clean directory."
  exit 1
fi

git clone "$REPO_URL"
echo "✅ Success"


