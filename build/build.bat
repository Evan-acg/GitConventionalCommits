@echo off
set CGO_ENABLED=0
go build -ldflags="-s -w" -o build\agc.exe .\cmd\agc\
