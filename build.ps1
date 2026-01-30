param(
    [string]$Target = "all"
)

function Build-Client {
    Write-Host "Building Rust client..." -ForegroundColor Cyan
    Push-Location client
    cargo build --release
    $exitCode = $LASTEXITCODE
    Pop-Location
    if ($exitCode -eq 0) {
        Write-Host "Client built: client/target/release/rslogger.exe" -ForegroundColor Green
    } else {
        Write-Host "Client build failed" -ForegroundColor Red
        exit 1
    }
}

function Build-Server {
    Write-Host "Building Go server..." -ForegroundColor Cyan
    Push-Location server
    go build -tags server -ldflags="-s -w" -o server.exe server.go
    $exitCode = $LASTEXITCODE
    Pop-Location
    if ($exitCode -eq 0) {
        Write-Host "Server built: server/server.exe" -ForegroundColor Green
    } else {
        Write-Host "Server build failed" -ForegroundColor Red
        exit 1
    }
}

function Clean-Build {
    Write-Host "Cleaning build artifacts..." -ForegroundColor Cyan
    Push-Location client
    cargo clean
    Pop-Location
    Remove-Item -Force -ErrorAction SilentlyContinue client/src/sender.exe, server/server.exe, server/go.sum, keylog.txt, mouselog.txt
    Remove-Item -Recurse -Force -ErrorAction SilentlyContinue received/
    Write-Host "Clean complete" -ForegroundColor Green
}

function Build-All {
    Build-Client
    Build-Server
    Write-Host "`nAll builds complete!" -ForegroundColor Green
    Write-Host "  Client: target\release\rslogger.exe" -ForegroundColor Gray
    Write-Host "  Server: server.exe" -ForegroundColor Gray
}

switch ($Target.ToLower()) {
    "client" { Build-Client }
    "server" { Build-Server }
    "sender" { Build-Sender }
    "clean"  { Clean-Build }
    "rebuild" {
        Clean-Build
        Build-All
    }
    "all"    { Build-All }
    default {
        Write-Host "Unknown target: $Target" -ForegroundColor Red
        Write-Host "Valid targets: all, client, server, sender, clean, rebuild"
        exit 1
    }
}
