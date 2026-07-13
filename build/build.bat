@echo off
cargo build --release
copy /Y target\release\agc.exe build\agc.exe
echo Build complete: build\agc.exe
