# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

FSR-RS is a Rust-based profile manager for FSR dance pads using Teejsub's firmware. It provides real-time threshold control via WebSocket communication and a web UI.

## Build Commands

```bash
cargo build              # Debug build
cargo run               # Run with defaults (COM6, port 3000, localhost)
cargo run -- --com-port COM3 --port 8080 --host 0.0.0.0  # Custom config
cargo run -- --mock-serial  # Use mock device (no hardware required)
cargo test              # Run all tests (~40 total)
cargo build --release   # Creates release zip with executable + http/ + lua/
```

## Architecture

### Communication Pattern
- **WebSocket-based** real-time client-server communication (not REST)
- **Broadcast channel** pattern using Tokio's `broadcast::channel` for one-to-many sensor data streaming (~60Hz)
- All clients receive sensor and player state updates simultaneously

### Core Modules

- **`src/main.rs`** (~1260 lines): Server, WebSocket handler, command processing, background tasks
- **`src/serial.rs`** (~420 lines): Hardware abstraction via `SensorPort` trait
- **`src/profile.rs`** (~475 lines): Data models (`Profile`, `Player`, `Profiles`), persistence, Command/Response types

### Key Design: SensorPort Trait
Minimal trait (`read()`, `write()`) enabling polymorphism for:
- `SerialPortAdapter` - wraps real hardware
- `MockSerialPort` - simulates hardware with sine wave generator
- `DummySerialPort` - no-op for testing

### Background Tasks
- `sensor_stream_task`: Polls hardware at ~60Hz (16.67ms intervals) when active
- `active_player_broadcast_task`: Broadcasts player state every 1 second

### State Synchronization
Dual state management: local profiles + device thresholds. Automatically detects drift and syncs device when profile changes.

## Serial Protocol

Commands to device (line-based, LF terminated, 115200 baud):
- `v\n` → Response: `v V1 V2 V3 V4\n` (sensor values 0-1023)
- `t\n` → Response: `t T1 T2 T3 T4\n` (current thresholds)
- `I V\n` → Set threshold index I to value V → Response: `t T1 T2 T3 T4\n`

## WebSocket API

Endpoint: `ws://localhost:3000/ws`

Command types: `UpdateThreshold`, `AddProfile`, `RemoveProfile`, `ChangeProfile`, `ChangePlayer`, `SetDefaultProfile`, `GetCurrentThresholds`, `StartSensorStream`, `StopSensorStream`

Response types: `sensor_stream`, `active_player_broadcast`, `command_response`

## File Structure

- `http/` - Web UI (index.html, script.js, style.css, debug.html)
- `lua/` - Scripting for external integrations (ProfileSwitcher.lua)
- `profiles.json` - Runtime-generated user data (gitignored)
