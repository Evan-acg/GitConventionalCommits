#!/usr/bin/env bash
set -euo pipefail

BUILD=true
DEPLOY=true

while [[ $# -gt 0 ]]; do
    case "$1" in
        --skip-build) BUILD=false ;;
        --no-deploy)  DEPLOY=false ;;
        *) echo "Usage: $0 [--skip-build] [--no-deploy]"; exit 1 ;;
    esac
    shift
done

if $BUILD; then
    echo "Building release..."
    cargo build --release
    cp -f target/release/agc build/agc
    echo "Build complete: build/agc"
fi

if $DEPLOY; then
    if [ ! -f build/agc ]; then
        echo "Error: build/agc not found" >&2
        exit 1
    fi
    mkdir -p /usr/local/bin
    cp -f build/agc /usr/local/bin/agc
    echo "Deployed to /usr/local/bin/agc"
fi
