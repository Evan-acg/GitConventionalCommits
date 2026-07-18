#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BINARY="$SCRIPT_DIR/build/agc"

if [ ! -f "$BINARY" ]; then
    echo "Error: build/agc not found. Run build/build.sh first."
    exit 1
fi

# Determine install target
if [ -d "/usr/local/bin" ] && [ -w "/usr/local/bin" ]; then
    TARGET="/usr/local/bin/agc"
elif [ -d "$HOME/.local/bin" ]; then
    TARGET="$HOME/.local/bin/agc"
else
    mkdir -p "$HOME/.local/bin"
    TARGET="$HOME/.local/bin/agc"
fi

cp "$BINARY" "$TARGET"
chmod +x "$TARGET"

echo "✅ Deployed agc → $TARGET"

# Check if target dir is in PATH
TARGET_DIR="$(dirname "$TARGET")"
case ":$PATH:" in
    *:"$TARGET_DIR":*) ;;
    *)
        echo "⚠️  $TARGET_DIR is not in your PATH."
        echo "   Add the following line to your ~/.zshrc or ~/.bashrc:"
        echo "   export PATH=\"\$PATH:$TARGET_DIR\""
        ;;
esac

echo "Now you can run 'agc' from any directory."
