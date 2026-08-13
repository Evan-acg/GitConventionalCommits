param(
    [switch]$SkipBuild,
    [switch]$NoDeploy
)

if (-not $SkipBuild) {
    Write-Host "Building release..." -ForegroundColor Cyan
    cargo build --release
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    Copy-Item -Force target/release/agc.exe build/agc.exe
    $sizeMb = [math]::Round((Get-Item build/agc.exe).Length / 1MB, 2)
    Write-Host "Build complete: build/agc.exe (${sizeMb} MB)" -ForegroundColor Green
}

if (-not $NoDeploy) {
    if (-not (Test-Path "build/agc.exe")) {
        Write-Host "Error: build/agc.exe not found. Run without -SkipBuild or build first." -ForegroundColor Red
        exit 1
    }
    $null = New-Item -ItemType Directory -Force -Path "C:\Tools"
    Copy-Item -Force build/agc.exe C:\Tools/agc.exe
    Write-Host "Deployed to C:\Tools\agc.exe" -ForegroundColor Green
}
