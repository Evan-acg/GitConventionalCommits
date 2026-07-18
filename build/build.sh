#!/bin/bash
set -e

CGO_ENABLED=0 go build -ldflags="-s -w" -o build/agc ./cmd/agc/
echo "Build completed: build/agc"
