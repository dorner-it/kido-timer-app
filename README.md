# SerialConverter — TRV Kocab Sports Timer TUI Dashboard

A terminal-based real-time dashboard for the **TRV Kocab Sports Timer**, built in Rust. Connects to the timer via serial port (or replays a captured hex dump) and displays live timing data across 4 channels in a color-coded TUI.

## Features

- **Live serial connection** to TRV Kocab timer hardware at 9600 baud (8N1)
- **Hex dump replay** mode for testing and analysis without physical hardware
- **4-channel timing display** in a 2x2 grid with color-coded statuses
- **Real-time protocol decoding** of the proprietary 41-byte binary frame format
- **Raw hex inspector** showing the latest frame bytes
- **FPS counter** and frame/error statistics
- Configurable replay speed (real-time, fast-forward, or max speed)

## TUI Layout

```
┌─ TRV Kocab Sports Timer ──────────────────────────────────────────┐
│  Device: ACTIVE   Lane: 1   State: MEASURING   FPS: 24  CONNECTED│
├─ Channel 1 ──────────────────┬─ Channel 2 ────────────────────────┤
│                               │                                    │
│   03.620                      │   35.020                           │
│   CONFIRMED                   │   CONFIRMED                       │
│                               │                                    │
├─ Channel 3 ──────────────────┼─ Channel 4 ────────────────────────┤
│                               │                                    │
│   ---.---                     │   ---.---                          │
│   INACTIVE                    │   INACTIVE                        │
│                               │                                    │
├─ Raw ─────────────────────────────────────────────────────────────┤
│  52 57 3A 01 02 01 00 24 0E 00 84 CC 88 00 84 00 00 00 00 ...   │
├───────────────────────────────────────────────────────────────────┤
│  Frames: 142  Errors: 0  |  q=quit                               │
└───────────────────────────────────────────────────────────────────┘
```

### Channel Status Colors

| Status | Color | Meaning |
|--------|-------|---------|
| INACTIVE | Gray | No sensor connected or not triggered |
| RUNNING | Yellow | Timer actively counting |
| CAPTURED | Cyan | Time frozen, pending confirmation |
| CONFIRMED | Green | Final time locked in |

## Installation

### Prerequisites

- [Rust](https://rustup.rs/) (edition 2021+)

### Build

```bash
cargo build --release
```

The binary will be at `target/release/serialconverter`.

## Usage

### Live serial connection

```bash
# Linux
serialconverter --port /dev/ttyUSB0

# macOS
serialconverter --port /dev/cu.usbserial-XXX

# Windows
serialconverter --port COM7

# Custom baud rate (default: 9600)
serialconverter --port /dev/ttyUSB0 --baud 19200
```

### Hex dump replay (no hardware needed)

```bash
# Real-time replay (~24 fps, matching the actual device)
serialconverter --file COM7_9600_20260210_183421.txt

# 5x speed
serialconverter --file COM7_9600_20260210_183421.txt --speed 5

# Max speed (no delay between frames)
serialconverter --file COM7_9600_20260210_183421.txt --speed 0
```

### CLI Reference

```
Usage: serialconverter [OPTIONS]

Options:
  -p, --port <PORT>    Serial port (e.g., /dev/ttyUSB0 or COM7)
  -b, --baud <BAUD>    Baud rate [default: 9600]
  -f, --file <FILE>    Replay a hex dump file instead of reading from a serial port
  -s, --speed <SPEED>  Replay speed multiplier (1.0 = real-time, 0 = max speed) [default: 1.0]
  -h, --help           Print help
```

Either `--port` or `--file` must be provided (mutually exclusive).

### Keyboard

| Key | Action |
|-----|--------|
| `q` / `Q` / `Esc` | Quit |

## Protocol

The TRV Kocab Sports Timer transmits fixed **41-byte frames** at 9600 baud (8N1):

```
Offset  Size  Description
------  ----  -----------
 0..2     3   Header: "RW:" (0x52 0x57 0x3A)
 3        1   Protocol version (0x01)
 4        1   Device mode: 0x00=Standby, 0x02=Active
 5        1   Lane number (1-based)
 6        1   State flag: 0x00=Measuring, 0x01=Armed
 7..22   16   Channels 1-4 (4 bytes each: time_lo, time_hi, reserved, status)
23..38   16   Reserved
39        1   Terminator: 0x0D (CR)
40        1   Trailing null: 0x00
```

Time values are little-endian unsigned 16-bit milliseconds (max 65,535 ms).

### Hex Dump File Format

The replay feature accepts xxd-style hex dumps:

```
--- Connected to COM7 at 9600 baud (8N1, hex) ---
00000000  52 57 3A 01 00 01 00 00  00 00 00 00 00 00 00 00  |RW:.............|
00000010  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|
```

Lines starting with `---` are treated as comments and skipped.

## Project Structure

```
src/
├── main.rs           CLI entry point, event loop, terminal setup
├── protocol.rs       Frame parser, enums (DeviceMode, ChannelStatus, etc.)
├── serial_reader.rs  Serial port thread with frame synchronization
├── file_reader.rs    Hex dump parser and replay thread
├── app.rs            Application state and event handling
└── ui.rs             Ratatui TUI rendering (header, channels, hex, status bar)
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| [ratatui](https://crates.io/crates/ratatui) | Terminal UI framework |
| [crossterm](https://crates.io/crates/crossterm) | Cross-platform terminal manipulation |
| [clap](https://crates.io/crates/clap) | Command-line argument parsing |
| [serialport](https://crates.io/crates/serialport) | Serial port communication |

## License

Private project.
