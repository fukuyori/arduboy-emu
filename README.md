# arduboy-emu

**v0.4.0** — A cycle-accurate Arduboy emulator written in Rust.

Emulates the ATmega32u4 microcontroller at 16 MHz with display, audio, gamepad, and Arduboy FX flash support.

## Features

- **AVR CPU core** — 80+ instructions with accurate flag computation (ADD, SUB, SBC/SBCI carry chains, MUL, etc.)
- **SSD1306 OLED display** — 128×64 monochrome with horizontal/vertical addressing, contrast control, and invert
- **PCD8544 LCD** — 84×48 Nokia display for Gamebuino Classic compatibility (auto-detected)
- **Stereo audio** — Two independent channels with sample-accurate waveform rendering:
  - Left: Timer3 CTC / Timer4 CTC / GPIO bit-bang on PC6 (Speaker 1)
  - Right: Timer1 CTC / GPIO bit-bang on PB5 (Speaker 2)
  - Hybrid audio: sample-accurate PCM for GPIO bit-bang, square wave synthesis fallback for timers
- **Gamepad support** — Cross-platform via gilrs (Windows/Linux/macOS), with hot-plug
- **Arduboy FX** — W25Q128 16 MB SPI flash emulation (Read, Fast Read, JEDEC ID, erase, program)
- **Peripherals** — Timer0/1/3/4, SPI, ADC, PLL, EEPROM, USB Serial output
- **Debugger** — Disassembler, breakpoints, step-by-step execution, register dump
- **Dynamic display** — Scale 1×–6× toggle, fullscreen, PNG screenshots
- **USB Serial** — Captures `Serial.print()` output via UEDATX register interception
- **Headless mode** — Automated testing with frame snapshots and diagnostics
- **.arduboy file support** — Load ZIP archives with info.json, hex, and FX bin
- **EEPROM persistence** — Auto-save/load to .eep file alongside game
- **GIF recording** — Capture gameplay as animated GIF (G key toggle, LZW compressed)
- **LED status** — RGB LED, TX LED, RX LED state displayed in title bar
- **FPS control** — Toggle between 60fps locked and unlimited (F key)
- **Hot reload** — Reload current game file without restart (R key)
- **Game browser** — N/P keys to cycle through games in directory, O to list

## Building

```bash
# Linux: install dependencies
sudo apt install libudev-dev libasound2-dev

# Build and run
cargo build --release
cargo run --release -- game.hex
```

## Usage

```
arduboy-emu <file.hex|file.arduboy> [options]

Options:
  --fx <file.bin>    Load FX flash data
  --mute             Disable audio
  --debug            Show per-frame diagnostics
  --headless         Run without GUI
  --frames N         Run N frames (headless, default 60)
  --press N          Press A button on frame N (headless)
  --snapshot F       Print display at frame F (repeatable)
  --break <addr>     Set breakpoint at hex byte-address (repeatable)
  --step             Interactive step-by-step debugger
  --scale N          Initial display scale 1-6 (default 6)
  --serial           Show USB serial output on stderr
  --no-save          Disable EEPROM auto-save
```

### File Formats

| Format | Description |
|--------|------------|
| `.hex` | Intel HEX binary (auto-detects companion `.bin` / `-fx.bin` for FX data) |
| `.arduboy` | ZIP archive containing `info.json`, `.hex`, and optional FX `.bin` |

### FX Flash Auto-Detection

FX data is loaded automatically if a matching `.bin` file exists alongside the `.hex`:

```
game.hex + game.bin       → auto-loaded
game.hex + game-fx.bin    → auto-loaded
game.hex --fx custom.bin  → explicit path
game.arduboy              → hex + fx extracted from ZIP
```

### EEPROM Persistence

EEPROM is automatically saved to a `.eep` file alongside the game:

```
game.hex → game.eep (auto-saved every 10s + on exit)
```

Use `--no-save` to disable. EEPROM data survives hot reload (R key).

### Game Browser

Press **O** to list all `.hex` and `.arduboy` files in the game's directory, then use **N** (next) and **P** (previous) to switch between them. EEPROM state is saved and loaded per game automatically.

```
--- Games in ./roms (5 found) ---
    1. arcodia.hex
    2. breakout.hex <<
    3. circuit-dude.arduboy
    4. nineteen44.hex
    5. starduino.hex
---
```

## Controls

| Arduboy     | Keyboard   | Xbox Controller             | PlayStation                   |
|-------------|------------|-----------------------------|-------------------------------|
| D-pad       | Arrow keys | D-pad / Left stick          | D-pad / Left stick            |
| A           | Z          | X, Y, LB, RB, LT, RT, Select | □, △, L1, R1, L2, R2, Select |
| B           | X          | A, B, Start                 | ×, ○, Start                   |
| Scale 1×–6× | 1–6 keys   | —                           | —                             |
| Fullscreen  | F11        | —                           | —                             |
| Screenshot  | S          | —                           | — (PNG at current scale)      |
| GIF record  | G          | —                           | —                             |
| Next game   | N          | —                           | —                             |
| Prev game   | P          | —                           | —                             |
| List games  | O          | —                           | —                             |
| Reload      | R          | —                           | —                             |
| FPS toggle  | F          | —                           | — (60fps ↔ unlimited)         |
| Reg dump    | D          | —                           | —                             |
| Mute       | M          | —                           | —                             |
| Quit       | Escape     | —                           | —                             |

Keyboard and gamepad inputs are OR-combined, so both can be used simultaneously.

## Architecture

```
arduboy-emu/
├── crates/
│   ├── core/                    # Platform-independent emulation core
│   │   └── src/
│   │       ├── lib.rs           # Arduboy struct: top-level emulator
│   │       ├── cpu.rs           # AVR CPU state and instruction execution
│   │       ├── opcodes.rs       # Instruction decoder (16/32-bit → enum)
│   │       ├── memory.rs        # Data space, flash, EEPROM
│   │       ├── display.rs       # SSD1306 OLED controller (contrast/invert)
│   │       ├── pcd8544.rs       # PCD8544 Nokia LCD controller
│   │       ├── hex.rs           # Intel HEX parser
│   │       ├── disasm.rs        # Instruction disassembler (debugger)
│   │       ├── audio_buffer.rs  # Sample-accurate waveform buffer
│   │       ├── arduboy_file.rs  # .arduboy ZIP file parser
│   │       ├── png.rs           # PNG encoder (no dependencies)
│   │       ├── gif.rs           # Animated GIF encoder (LZW compressed)
│   │       └── peripherals/
│   │           ├── timer8.rs    # Timer/Counter0 (millis/delay)
│   │           ├── timer16.rs   # Timer/Counter1 & 3 (audio tone)
│   │           ├── timer4.rs    # Timer/Counter4 (10-bit high-speed PWM)
│   │           ├── spi.rs       # SPI master controller
│   │           ├── adc.rs       # ADC (random seed)
│   │           ├── pll.rs       # PLL frequency synthesizer
│   │           ├── eeprom.rs    # EEPROM controller
│   │           └── fx_flash.rs  # W25Q128 external flash (16 MB)
│   └── frontend-minifb/         # Desktop frontend
│       └── src/main.rs          # Window, stereo audio, gamepad, debugger
└── roms/                        # Test ROM directory
```

### Emulation Loop

Each frame (~13.5 ms at 60 FPS):

1. Poll keyboard and gamepad → set GPIO pin states
2. Execute CPU instructions until 216,000 cycles elapsed (with breakpoint checks)
3. Flush SPI buffer → route bytes to display or FX flash
4. Update timers and fire pending interrupts
5. Read tone frequency (Timer3 / Timer1 / GPIO) → update stereo audio
6. Capture USB serial output bytes
7. Blit RGBA framebuffer to window at configurable scale

### Audio (Stereo, Sample-Accurate)

GPIO bit-bang audio is rendered sample-accurately using a per-frame edge buffer.
Timer-driven audio falls back to frequency-based square wave synthesis.

| Channel | Priority | Method | Mechanism | Example |
|---------|----------|--------|-----------|---------|
| Left  | 1 | Timer3 CTC | OC3A (PC6) toggle on compare match | Arduboy2 `tone()` |
| Left  | 2 | Timer4 CTC | OC4A toggle on compare match | PWM audio games |
| Left  | 3 | GPIO bit-bang | Direct PORTC bit 6 toggling | Arcodia |
| Right | 1 | Timer1 CTC | OC1A (PB5) toggle on compare match | Dual-tone games |
| Right | 2 | GPIO bit-bang | Direct PORTB bit 5 toggling | Custom engines |

## Tested Games

- **Nineteen44** — Scrolling shooter (Timer3 audio, complex SPI)
- **Arcodia** — Space Invaders clone (GPIO bit-bang audio)
- **101 Starships** — Fleet management game
- Various Arduboy2 library games

## Roadmap

See [ROADMAP.md](ROADMAP.md) for a detailed feature comparison with ProjectABE
and the planned development phases toward v1.0.0.

See [CHANGELOG.md](CHANGELOG.md) for the release history.

## Notice

This software was generated by AI (Claude by Anthropic) through interactive
development sessions with a human operator. No code from existing emulator
projects (such as ProjectABE) was used. The implementation is based solely
on publicly available hardware datasheets (ATmega32u4, SSD1306, PCD8544,
W25Q128) and the Intel HEX format specification.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
