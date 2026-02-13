# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2025-02-13

### Added

- **ATmega328P CPU support** — `--cpu 328p` CLI option for Gamebuino Classic / Arduino Uno
- **`CpuType` enum** — `Atmega32u4` (default) / `Atmega328p` selection at construction
- **`Arduboy::new_with_cpu()`** — Constructor with explicit CPU type
- **Timer2 peripheral** — 8-bit asynchronous timer for ATmega328P (reuses Timer8 with dedicated register addresses and interrupt vectors)
- **ATmega328P interrupt vector table** — 26-entry vector table (separate from 32u4's 43-entry table)
- **ATmega328P memory map** — SRAM 2 KB (vs 32u4's 2.5 KB), dynamic `Memory::new_with_size()`
- **Gamebuino Classic button mapping** — 328P pin layout: UP=PB1, DOWN=PD6, LEFT=PB0, RIGHT=PD7, A=PD4, B=PD2
- **328P PCD8544 SPI routing** — CS=PC2, DC=PC3 for Gamebuino Classic display

### Changed

- Timer8 and Timer16 now take interrupt vector addresses via `Addrs` struct (no hardcoded vectors)
- `peripherals::mod.rs` reorganized with separate 32u4 and 328P vector constant blocks
- Timer3, Timer4, USB serial conditionally active based on `cpu_type`
- SPI output tuple expanded to 4-tuple `(byte, portd, portf, portc)` for 328P PORTC routing
- `Memory` doc updated for dual-CPU support

## [0.4.0] - 2025-02-13

### Added

- **`.arduboy` file support**: ZIP archive parser with `info.json` metadata
  extraction, automatic `.hex` and FX `.bin` file detection
  - Minimal ZIP reader (stored + deflate) with RFC 1951 inflate
  - Simple JSON string value extractor for title/author
- **EEPROM persistence**: Automatic save/load to `.eep` file alongside game
  - Auto-save every 10 seconds when dirty, auto-load on startup
  - `--no-save` flag to disable persistence
  - Saved before hot-reload and game switch
- **GIF recording**: G key toggles recording, LZW-compressed animated GIF
  - Custom GIF89a encoder with Netscape infinite loop extension
  - 2-color (monochrome) palette for Arduboy's 1-bit display
  - Saved on stop or window close (`recording_NNNN.gif`)
- **PNG screenshots at any scale**: S key saves at current display scale
  - 1× scale: efficient 8-bit grayscale PNG (monochrome)
  - 2×–6× scale: RGBA PNG with nearest-neighbor upscale
  - Custom PNG encoder (no dependencies, stored deflate blocks)
- **RGB LED state tracking**: Red (PB6), Green (PB7), Blue (PB5) from PORTB
  - TX LED (PD5, active-low) and RX LED (PB0, active-low) detection
  - LED state displayed in window title bar
- **FPS limiter toggle**: F key switches between 60fps and unlimited
- **Game browser**: Scan current directory for `.hex`/`.arduboy` files
  - N key = load next game, P key = load previous game
  - O key = print numbered file list to terminal
  - Alphabetical sorting, circular navigation
  - EEPROM saved/loaded per game automatically
- **Hot reload**: R key reloads current game file from disk

### Changed

- Screenshot format: BMP → PNG (smaller files, widely supported)
- Screenshot naming: `screenshot_NNNN_Sx.png` includes scale factor
- Window title shows LED state, recording indicator, FPS mode

## [0.3.0] - 2025-02-13

### Added

- **Timer4 (10-bit high-speed)**: ATmega32u4 Timer/Counter4 emulation
  - Normal, CTC, and PWM modes with OCR4C as TOP
  - Extended prescaler (/1 through /16384)
  - 10-bit counter with TC4H high-byte buffer register
  - OCR4A/B/C/D compare registers, dead time (DT4)
  - Overflow and compare-match interrupts (TIMSK4/TIFR4)
  - Tone detection in CTC mode for audio output
- **Sample-accurate audio waveform buffer**: `AudioBuffer` module
  - Records pin-level transitions with CPU tick timestamps per frame
  - Converts edge buffers to stereo interleaved PCM at target sample rate
  - Hybrid audio source in frontend: sample-accurate PCM when GPIO edges
    exist, automatic fallback to timer frequency synthesis otherwise
  - Ring buffer between main thread and rodio audio thread

### Changed

- Audio source replaced: `StereoSquareWave` → `HybridAudioSource`
  with `Arc<Mutex<VecDeque<f32>>>` ring buffer for sample-accurate output
- Timer tone priority: Timer3 > Timer4 > GPIO PC6 (left),
  Timer1 > GPIO PB5 (right)

## [0.2.0] - 2025-02-13

### Added

- **Disassembler**: `disasm` module with `disassemble()`, `format_sreg()`,
  `disassemble_range()` for instruction-level debugging
- **Breakpoints**: `--break <addr>` CLI option (repeatable), hex byte-address,
  stops execution when PC matches
- **Step mode**: `--step` interactive debugger with commands:
  Enter=step, N=step N, r=run to break, d=dump, q=quit
- **Register dump**: `dump_regs()` showing R0-R31, PC, SP, SREG flags, X/Y/Z
  pairs; D key in GUI mode, integrated into step and breakpoint output
- **SSD1306 display invert**: 0xA6 (normal) / 0xA7 (inverse) commands,
  XOR applied during framebuffer rendering
- **SSD1306 contrast control**: 0x81 command, brightness scaled by contrast
  byte (0x00=black, 0xFF=full)
- **Window scale toggle**: 1–6 number keys change scale in GUI mode
- **Fullscreen**: F11 toggles borderless fullscreen (12× scale)
- **Screenshot**: S key saves BMP file (screenshot_NNNN.bmp)
- **Scale CLI option**: `--scale N` sets initial scale (1–6)
- **2-channel stereo audio output**: Timer3 → left, Timer1 → right,
  GPIO PC6 → left fallback, GPIO PB5 → right fallback
- **GPIO speaker 2 (PB5)**: Bit-bang edge detection for right channel
- **USB Serial emulation**: UENUM, UEDATX, UEINTX register handling,
  CDC endpoint capture (EP3+), --serial flag outputs to stderr
- **USB register stubs**: USBCON, USBSTA, UDADDR, UESTA0X, UEBCLX for
  programs that check USB state

## [0.1.0] - 2025-02-13

Initial release.

### Added

- **AVR CPU core**: 80+ ATmega32u4 instructions with accurate flag computation
  - Arithmetic: ADD, ADC, SUB, SUBI, SBC, SBCI, AND, ANDI, OR, ORI, EOR, COM,
    NEG, INC, DEC, MUL, MULS, MULSU, FMUL, FMULS, ADIW, SBIW
  - Compare: CP, CPC, CPI, CPSE
  - Branch: RJMP, RCALL, RET, RETI, JMP, CALL, IJMP, ICALL, BRBS, BRBC,
    SBRC, SBRS, SBIC, SBIS
  - Data transfer: MOV, MOVW, LDI, LDS, STS, LD/ST with X/Y/Z (plain,
    post-increment, pre-decrement, displacement)
  - I/O: IN, OUT, SBI, CBI, PUSH, POP
  - Shift/Bit: LSR, ASR, ROR, SWAP, BST, BLD
  - Program memory: LPM (3 modes), ELPM (3 modes, RAMPZ:Z 24-bit addressing)
  - Status register: SEI, CLI, SEC, CLC, SEN, CLN, SEZ, CLZ, SEV, CLV,
    SES, CLS, SEH, CLH, SET, CLT
  - Misc: NOP, SLEEP, WDR
- **SSD1306 OLED display**: 128×64 monochrome, horizontal/vertical addressing,
  column/page windowing, command processing
- **PCD8544 Nokia LCD**: 84×48 monochrome (Gamebuino Classic), auto-detected,
  centered in 128×64 framebuffer
- **Audio output** (3 detection methods):
  - Timer3 CTC mode (standard Arduboy `tone()`)
  - Timer1 CTC mode
  - GPIO bit-bang (PC6 pin toggle via `digitalWrite`)
- **Peripherals**:
  - Timer0 (8-bit): Normal, CTC, Fast PWM modes with prescaler, overflow and
    compare-match interrupts (millis/delay support)
  - Timer1/Timer3 (16-bit): CTC mode, compare-match toggle, tone generation
  - SPI master controller (instant transfer, SPIF flag)
  - ADC with pseudo-random output (xorshift PRNG)
  - PLL frequency synthesizer (instant lock)
  - EEPROM controller (1 KB, read/write via EECR)
- **Arduboy FX**: W25Q128 16 MB SPI flash emulation
  - Commands: Read Data (0x03), Fast Read (0x0B), JEDEC ID (0x9F),
    Release Power Down (0xAB), Read Status (0x05), Power Down (0xB9),
    Write Enable/Disable (0x06/0x04), Page Program (0x02), Sector Erase (0x20)
  - Lazy allocation (16 MB allocated only on first use)
  - Auto-detection of `.bin` / `-fx.bin` companion files
- **Input**: Keyboard (arrows, Z/X) + gamepad (gilrs, cross-platform, hot-plug)
  with OR-combined merging and analog stick deadzone
- **Frontend**: minifb window with 6× nearest-neighbor scaling,
  rodio square wave audio, FPS counter, mute toggle (M key)
- **Headless mode**: `--headless` with `--frames`, `--press`, `--snapshot`,
  `--debug` options, Unicode half-block display rendering
- **Intel HEX parser**: Record types 00 (data), 01 (EOF), 02 (extended segment)
- **Debug output**: `--debug` flag gates all diagnostic `eprintln!` output
