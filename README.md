# FSR-RS Profile Manager

A Rust-based profile manager for FSR dance pads using Teejsub's firmware with real-time threshold control and WebSocket communication.

## Features

- Real-time sensor data streaming via WebSocket
- Profile management with customizable thresholds
- Serial communication with FSR hardware
- Web-based user interface
- Debug mode for development

## Usage

### Basic Usage

```bash
# Run with default settings (COM6, port 3000, localhost)
cargo run

# Run with custom COM port
cargo run -- --com-port COM3

# Run with custom port
cargo run -- --port 8080

# Run with custom host and port
cargo run -- --host 0.0.0.0 --port 8080

# Run with all custom parameters
cargo run -- --com-port COM3 --host 0.0.0.0 --port 8080
```

### Command Line Options

- `-c, --com-port <COM_PORT>`: COM port to use for serial communication (default: COM6)
- `-p, --port <PORT>`: Web server port to listen on (default: 3000)
- `--host <HOST>`: Host address to bind to (default: 127.0.0.1)

### Examples

```bash
# Development mode - accessible from other devices
cargo run -- --host 0.0.0.0 --port 3000

# Use different COM port
cargo run -- --com-port COM4

# Production server on port 80 (requires admin privileges)
cargo run -- --host 0.0.0.0 --port 80
```

## Web Interface

Once running, open your browser to:
- Main interface: `http://localhost:3000/` (or your custom port)
- Debug mode: `http://localhost:3000/debug` (or your custom port)

## WebSocket API

The application provides a WebSocket endpoint at `ws://localhost:3000/ws` (or your custom port) for real-time communication. The web interface automatically connects to the WebSocket on the same server that serves the page.

## Building

### Development Build
```bash
# Build in debug mode (default)
cargo build

# Run the debug binary
cargo run
```

### Release Build
```bash
# Build in release mode (includes HTTP files and creates zip)
cargo build --release

# Run the release binary
./target/release/fsr-rs --com-port COM3 --port 8080
```

### File Structure After Release Build
```
target/release/
├── fsr-rs.exe          # Main executable
├── fsr-rs-0.1.0.zip    # Release package (executable + HTTP files)
├── http/               # Web interface files
│   ├── index.html
│   ├── script.js
│   └── style.css
└── profiles.json       # User profiles (created on first run)
```

### Distribution
The generated zip file (`fsr-rs-{version}.zip`) contains everything needed to run the application:
- `fsr-rs.exe` - The main executable
- `http/` directory - All web interface files

Users can extract the zip file and run `fsr-rs.exe` from any location - the application will automatically find the HTTP files relative to the executable's location. 