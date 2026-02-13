//! # arduboy-core
//!
//! Cycle-accurate emulation core for the Arduboy handheld game console (v0.5.0).
//!
//! Emulates the ATmega32u4 microcontroller (Arduboy) and ATmega328P (Gamebuino
//! Classic / Arduino Uno) with 16 MHz clock, 32 KB flash, 2–2.5 KB SRAM,
//! 1 KB EEPROM. Peripheral hardware: SSD1306 OLED display, PCD8544 Nokia LCD
//! (Gamebuino), SPI bus, Timer0/1/2/3/4, ADC, PLL, EEPROM controller,
//! W25Q128 FX external flash, and USB serial output.
//!
//! ## Architecture
//!
//! - [`Arduboy`] — Top-level emulator that wires together CPU, memory, and peripherals
//! - [`CpuType`] — Target CPU selection (ATmega32u4 or ATmega328P)
//! - [`Cpu`] — AVR CPU state (PC, SP, SREG, tick counter, sleep mode)
//! - [`Memory`] — Unified data space (registers + I/O + SRAM), flash, and EEPROM
//! - [`Ssd1306`] — SSD1306 128×64 monochrome OLED display controller
//! - [`pcd8544::Pcd8544`] — PCD8544 84×48 monochrome LCD (Gamebuino compatibility)
//! - [`peripherals`] — Timer8, Timer16, Timer4, SPI, ADC, PLL, EEPROM, FX flash
//! - [`disasm`] — Instruction disassembler for debug views
//!
//! ## Audio
//!
//! Three audio generation methods are detected and reported via [`Arduboy::get_audio_tone`]:
//!
//! 1. **Timer3 CTC** — Standard Arduboy `tone()` using OC3A output compare toggle
//! 2. **Timer1 CTC** — Alternative timer-based tone generation
//! 3. **GPIO bit-bang** — Direct `digitalWrite` toggling of speaker pins PC6/PB5
//!
//! Stereo output: Speaker 1 (PC6) → left channel, Speaker 2 (PB5) → right channel.

pub mod cpu;
pub mod memory;
pub mod opcodes;
pub mod display;
pub mod pcd8544;
pub mod hex;
pub mod peripherals;
pub mod disasm;
pub mod audio_buffer;
pub mod arduboy_file;
pub mod png;
pub mod gif;

pub use cpu::Cpu;
pub use display::Ssd1306;
pub use memory::Memory;
pub use audio_buffer::AudioBuffer;

// ATmega32u4 constants
/// Flash memory size: 32 KB
pub const FLASH_SIZE: usize = 32 * 1024;
/// SRAM size: 2.5 KB (2048 + 512) for ATmega32u4
pub const SRAM_SIZE: usize = 2 * 1024 + 512;
/// SRAM size: 2 KB for ATmega328P
pub const SRAM_SIZE_328P: usize = 2 * 1024;
/// EEPROM size: 1 KB
pub const EEPROM_SIZE: usize = 1024;
/// CPU clock frequency: 16 MHz
pub const CLOCK_HZ: u32 = 16_000_000;

/// SSD1306 display width in pixels
pub const SCREEN_WIDTH: usize = 128;
/// SSD1306 display height in pixels
pub const SCREEN_HEIGHT: usize = 64;

/// Number of general-purpose registers (R0–R31)
pub const REG_COUNT: usize = 32;
/// I/O + extended I/O register space size (0x20..0xFF)
pub const IO_SIZE: usize = 224;
/// Total data space (ATmega32u4): registers + I/O + SRAM
pub const DATA_SIZE: usize = REG_COUNT + IO_SIZE + SRAM_SIZE;
/// Total data space (ATmega328P)
pub const DATA_SIZE_328P: usize = REG_COUNT + IO_SIZE + SRAM_SIZE_328P;

/// Target CPU type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuType {
    /// ATmega32u4 (Arduboy, Leonardo)
    Atmega32u4,
    /// ATmega328P (Gamebuino Classic, Arduino Uno)
    Atmega328p,
}

// SREG bit positions
pub const SREG_C: u8 = 0;
pub const SREG_Z: u8 = 1;
pub const SREG_N: u8 = 2;
pub const SREG_V: u8 = 3;
pub const SREG_S: u8 = 4;
pub const SREG_H: u8 = 5;
pub const SREG_T: u8 = 6;
pub const SREG_I: u8 = 7;

// I/O register addresses (data space addresses, not I/O addresses)
pub const SREG_ADDR: u16 = 0x5F;
pub const SPH_ADDR: u16 = 0x5E;
pub const SPL_ADDR: u16 = 0x5D;

/// Arduboy button identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
}

/// Main Arduboy emulator combining all subsystems
pub struct Arduboy {
    pub cpu: Cpu,
    pub mem: Memory,
    pub display: Ssd1306,
    pub timer0: peripherals::Timer8,
    pub timer1: peripherals::Timer16,
    pub timer3: peripherals::Timer16,
    pub timer4: peripherals::Timer4,
    /// Timer2 (ATmega328P only, 8-bit async)
    pub timer2: peripherals::Timer8,
    pub spi: peripherals::Spi,
    pub pll: peripherals::Pll,
    pub adc: peripherals::Adc,
    pub eeprom_ctrl: peripherals::EepromCtrl,
    /// Arduboy FX external SPI flash
    pub fx_flash: peripherals::FxFlash,
    /// SPI data received from flash (MISO byte)
    spdr_in: u8,
    /// Pin states for GPIO (active-low buttons etc)
    pub pin_b: u8,
    pub pin_c: u8,
    pub pin_d: u8,
    pub pin_e: u8,
    pub pin_f: u8,
    /// SPI output buffer with raw port state per byte
    spi_out: Vec<(u8, u8, u8, u8)>, // (byte, portd_val, portf_val, portc_val)
    /// Random state for ADC
    rng_state: u32,
    /// Debug counter: total SPDR writes since reset
    pub dbg_spdr_writes: u64,
    /// Display type detection
    pub display_type: DisplayType,
    /// PCD8544 display (Gamebuino)
    pub pcd8544: pcd8544::Pcd8544,
    /// Frame counter for debug
    frame_count: u32,
    /// Track previous PD2 state for FX CS edge detection
    fx_cs_prev: bool,
    /// Enable debug output (eprintln)
    pub debug: bool,
    /// GPIO speaker 1 (PC6): previous state for edge detection
    speaker_prev_pc6: bool,
    /// GPIO speaker 1: tick of last PC6 edge
    speaker_last_edge: u64,
    /// GPIO speaker 1: measured half-period in ticks
    speaker_half_period: u64,
    /// GPIO speaker 1: tick when last tone was detected
    speaker_last_active: u64,
    /// GPIO speaker 2 (PB5): previous state for edge detection
    speaker2_prev_pb5: bool,
    /// GPIO speaker 2: tick of last PB5 edge
    speaker2_last_edge: u64,
    /// GPIO speaker 2: measured half-period in ticks
    speaker2_half_period: u64,
    /// GPIO speaker 2: tick when last tone was detected
    speaker2_last_active: u64,
    /// Breakpoint addresses (word addresses)
    pub breakpoints: Vec<u16>,
    /// True if execution stopped at a breakpoint
    pub breakpoint_hit: bool,
    /// USB Serial output buffer (UEDATX writes)
    pub serial_buf: Vec<u8>,
    /// USB endpoint number (UENUM register)
    usb_uenum: u8,
    /// USB device configured flag
    usb_configured: bool,
    /// Sample-accurate audio waveform buffer
    pub audio_buf: AudioBuffer,
    /// RGB LED state: (red, green, blue) brightness 0–255
    pub led_rgb: (u8, u8, u8),
    /// TX LED state (PD5, active-low)
    pub led_tx: bool,
    /// RX LED state (PB0, active-low)
    pub led_rx: bool,
    /// EEPROM dirty flag (true if modified since last save)
    pub eeprom_dirty: bool,
    /// Target CPU type
    pub cpu_type: CpuType,
    /// Actual SRAM size (varies by CPU type)
    sram_size: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisplayType {
    Unknown,
    Ssd1306,
    Pcd8544,
}

impl Arduboy {
    /// Create a new Arduboy emulator (ATmega32u4) with all peripherals in reset state.
    pub fn new() -> Self {
        Self::new_with_cpu(CpuType::Atmega32u4)
    }

    /// Create a new emulator for the specified CPU type.
    pub fn new_with_cpu(cpu_type: CpuType) -> Self {
        let sram_size = match cpu_type {
            CpuType::Atmega32u4 => SRAM_SIZE,
            CpuType::Atmega328p => SRAM_SIZE_328P,
        };
        let data_size = REG_COUNT + IO_SIZE + sram_size;

        // Timer0: same register addresses on both chips, different interrupt vectors
        let timer0_addrs = match cpu_type {
            CpuType::Atmega32u4 => peripherals::Timer8Addrs {
                tifr: 0x35, tccr_a: 0x44, tccr_b: 0x45,
                ocr_a: 0x47, ocr_b: 0x48, timsk: 0x6E, tcnt: 0x46,
                int_ovf: peripherals::INT_TIMER0_OVF,
                int_compa: peripherals::INT_TIMER0_COMPA,
                int_compb: peripherals::INT_TIMER0_COMPB,
            },
            CpuType::Atmega328p => peripherals::Timer8Addrs {
                tifr: 0x35, tccr_a: 0x44, tccr_b: 0x45,
                ocr_a: 0x47, ocr_b: 0x48, timsk: 0x6E, tcnt: 0x46,
                int_ovf: peripherals::INT_328P_TIMER0_OVF,
                int_compa: peripherals::INT_328P_TIMER0_COMPA,
                int_compb: peripherals::INT_328P_TIMER0_COMPB,
            },
        };

        // Timer1: same register addresses, different vectors
        let timer1_addrs = match cpu_type {
            CpuType::Atmega32u4 => peripherals::Timer16Addrs {
                tifr: 0x36, tccr_a: 0x80, tccr_b: 0x81, tccr_c: 0x82,
                ocr_ah: 0x89, ocr_al: 0x88, ocr_bh: 0x8B, ocr_bl: 0x8A,
                ocr_ch: 0x8D, ocr_cl: 0x8C,
                timsk: 0x6F, tcnth: 0x85, tcntl: 0x84,
                int_ovf: peripherals::INT_TIMER1_OVF,
                int_compa: peripherals::INT_TIMER1_COMPA,
                int_compb: peripherals::INT_TIMER1_COMPB,
                int_compc: peripherals::INT_TIMER1_COMPC,
            },
            CpuType::Atmega328p => peripherals::Timer16Addrs {
                tifr: 0x36, tccr_a: 0x80, tccr_b: 0x81, tccr_c: 0x82,
                ocr_ah: 0x89, ocr_al: 0x88, ocr_bh: 0x8B, ocr_bl: 0x8A,
                ocr_ch: 0x8D, ocr_cl: 0x8C, // 328P has no OCR1C but addr harmless
                timsk: 0x6F, tcnth: 0x85, tcntl: 0x84,
                int_ovf: peripherals::INT_328P_TIMER1_OVF,
                int_compa: peripherals::INT_328P_TIMER1_COMPA,
                int_compb: peripherals::INT_328P_TIMER1_COMPB,
                int_compc: 0, // no compare C on 328P
            },
        };

        // Timer3: ATmega32u4 only
        let timer3_addrs = peripherals::Timer16Addrs {
            tifr: 0x38, tccr_a: 0x90, tccr_b: 0x91, tccr_c: 0x92,
            ocr_ah: 0x99, ocr_al: 0x98, ocr_bh: 0x9B, ocr_bl: 0x9A,
            ocr_ch: 0x9D, ocr_cl: 0x9C,
            timsk: 0x71, tcnth: 0x94, tcntl: 0x95,
            int_ovf: peripherals::INT_TIMER3_OVF,
            int_compa: peripherals::INT_TIMER3_COMPA,
            int_compb: peripherals::INT_TIMER3_COMPB,
            int_compc: peripherals::INT_TIMER3_COMPC,
        };

        // Timer2: ATmega328P only (8-bit, different addresses from Timer0)
        let timer2_addrs = peripherals::Timer8Addrs {
            tifr: 0x37, tccr_a: 0xB0, tccr_b: 0xB1,
            ocr_a: 0xB3, ocr_b: 0xB4, timsk: 0x70, tcnt: 0xB2,
            int_ovf: peripherals::INT_328P_TIMER2_OVF,
            int_compa: peripherals::INT_328P_TIMER2_COMPA,
            int_compb: peripherals::INT_328P_TIMER2_COMPB,
        };

        let mut ard = Arduboy {
            cpu: Cpu::new(),
            mem: Memory::new_with_size(data_size),
            display: Ssd1306::new(),
            timer0: peripherals::Timer8::new(timer0_addrs),
            timer1: peripherals::Timer16::new(timer1_addrs),
            timer3: peripherals::Timer16::new(timer3_addrs),
            timer4: peripherals::Timer4::new(),
            timer2: peripherals::Timer8::new(timer2_addrs),
            spi: peripherals::Spi::new(),
            pll: peripherals::Pll::new(),
            adc: peripherals::Adc::new(),
            eeprom_ctrl: peripherals::EepromCtrl::new(),
            fx_flash: peripherals::FxFlash::new(),
            spdr_in: 0,
            pin_b: 0xFF, pin_c: 0xFF, pin_d: 0xFF, pin_e: 0xFF, pin_f: 0xFF,
            spi_out: Vec::new(),
            rng_state: 0xDEAD_BEEF,
            dbg_spdr_writes: 0,
            display_type: DisplayType::Unknown,
            pcd8544: pcd8544::Pcd8544::new(),
            frame_count: 0,
            fx_cs_prev: true,
            debug: false,
            speaker_prev_pc6: false,
            speaker_last_edge: 0,
            speaker_half_period: 0,
            speaker_last_active: 0,
            speaker2_prev_pb5: false,
            speaker2_last_edge: 0,
            speaker2_half_period: 0,
            speaker2_last_active: 0,
            breakpoints: Vec::new(),
            breakpoint_hit: false,
            serial_buf: Vec::new(),
            usb_uenum: 0,
            usb_configured: false,
            audio_buf: AudioBuffer::new(),
            led_rgb: (0, 0, 0),
            led_tx: false,
            led_rx: false,
            eeprom_dirty: false,
            cpu_type,
            sram_size,
        };
        // Initialize SP to top of SRAM
        let sp = (data_size - 1) as u16;
        ard.mem.data[SPH_ADDR as usize] = (sp >> 8) as u8;
        ard.mem.data[SPL_ADDR as usize] = (sp & 0xFF) as u8;
        ard.cpu.sp = sp;

        // ATmega328P defaults to PCD8544 (Gamebuino Classic)
        if cpu_type == CpuType::Atmega328p {
            ard.display_type = DisplayType::Pcd8544;
        }

        ard
    }

    /// Load an Intel HEX file into flash memory and reset the CPU.
    ///
    /// Returns the number of bytes loaded on success.
    pub fn load_hex(&mut self, hex_str: &str) -> Result<usize, String> {
        let size = hex::parse_hex(hex_str, &mut self.mem.flash)?;
        self.reset();
        Ok(size)
    }

    /// Load FX flash data from binary. Data is placed at end of 16MB flash.
    pub fn load_fx_data(&mut self, bin: &[u8]) {
        self.fx_flash.load_data(bin);
    }

    /// Load FX flash data at a specific offset.
    pub fn load_fx_data_at(&mut self, bin: &[u8], offset: usize) {
        self.fx_flash.load_data_at(bin, offset);
    }

    /// Reset the CPU and all peripherals to power-on state.
    ///
    /// Flash and FX flash data are preserved (they represent ROM content).
    pub fn reset(&mut self) {
        self.cpu = Cpu::new();
        self.mem.data.fill(0);
        let data_size = REG_COUNT + IO_SIZE + self.sram_size;
        let sp = (data_size - 1) as u16;
        self.mem.data[SPH_ADDR as usize] = (sp >> 8) as u8;
        self.mem.data[SPL_ADDR as usize] = (sp & 0xFF) as u8;
        self.cpu.sp = sp;
        self.display = Ssd1306::new();
        self.pcd8544 = pcd8544::Pcd8544::new();
        self.display_type = DisplayType::Unknown;
        self.timer0.reset();
        self.timer1.reset();
        self.timer3.reset();
        self.timer4.reset();
        self.timer2.reset();
        self.spi.reset();
        self.pll.reset();
        self.adc.reset();
        self.eeprom_ctrl.reset();
        self.pin_b = 0xFF;
        self.pin_c = 0xFF;
        self.pin_d = 0xFF;
        self.pin_e = 0xFF;
        self.pin_f = 0xFF;
        self.spi_out.clear();
        self.spdr_in = 0;
        self.fx_cs_prev = true;
        self.speaker_prev_pc6 = false;
        self.speaker_last_edge = 0;
        self.speaker_half_period = 0;
        self.speaker_last_active = 0;
        self.speaker2_prev_pb5 = false;
        self.speaker2_last_edge = 0;
        self.speaker2_half_period = 0;
        self.speaker2_last_active = 0;
        self.breakpoint_hit = false;
        self.serial_buf.clear();
        self.usb_uenum = 0;
        self.usb_configured = false;
        self.led_rgb = (0, 0, 0);
        self.led_tx = false;
        self.led_rx = false;
        // Note: eeprom_dirty is NOT cleared on reset (tracks unsaved changes)
        // Note: FX flash data is NOT cleared on reset (persistent storage)
        // Note: breakpoints are NOT cleared on reset
    }

    /// Set button state (true = pressed)
    pub fn set_button(&mut self, btn: Button, pressed: bool) {
        // Active-low: pressed = bit cleared, released = bit set

        match self.cpu_type {
            CpuType::Atmega32u4 => {
                // --- Arduboy pin mapping (32u4) ---
                // UP=PF7, DOWN=PF4, LEFT=PF5, RIGHT=PF6, A=PE6, B=PB4
                if self.display_type != DisplayType::Pcd8544 {
                    let (pin, bit): (&mut u8, u8) = match btn {
                        Button::Up    => (&mut self.pin_f, 7),
                        Button::Down  => (&mut self.pin_f, 4),
                        Button::Left  => (&mut self.pin_f, 5),
                        Button::Right => (&mut self.pin_f, 6),
                        Button::A     => (&mut self.pin_e, 6),
                        Button::B     => (&mut self.pin_b, 4),
                    };
                    if pressed { *pin &= !(1 << bit); } else { *pin |= 1 << bit; }
                }

                // --- Gamebuino pin mapping (32u4 with PCD8544) ---
                // UP=PB5(9), DOWN=PD7(6), LEFT=PB4(8), RIGHT=PE6(7), A=PD4(4), B=PD1(2)
                if self.display_type != DisplayType::Ssd1306 {
                    let (pin2, bit2): (&mut u8, u8) = match btn {
                        Button::Up    => (&mut self.pin_b, 5),
                        Button::Down  => (&mut self.pin_d, 7),
                        Button::Left  => (&mut self.pin_b, 4),
                        Button::Right => (&mut self.pin_e, 6),
                        Button::A     => (&mut self.pin_d, 4),
                        Button::B     => (&mut self.pin_d, 1),
                    };
                    if pressed { *pin2 &= !(1 << bit2); } else { *pin2 |= 1 << bit2; }
                }
            }
            CpuType::Atmega328p => {
                // --- Gamebuino Classic pin mapping (328P) ---
                // UP=PB1(D9), DOWN=PD6(D6), LEFT=PB0(D8), RIGHT=PD7(D7)
                // A=PD4(D4), B=PD2(D2)
                let (pin, bit): (&mut u8, u8) = match btn {
                    Button::Up    => (&mut self.pin_b, 1),
                    Button::Down  => (&mut self.pin_d, 6),
                    Button::Left  => (&mut self.pin_b, 0),
                    Button::Right => (&mut self.pin_d, 7),
                    Button::A     => (&mut self.pin_d, 4),
                    Button::B     => (&mut self.pin_d, 2),
                };
                if pressed { *pin &= !(1 << bit); } else { *pin |= 1 << bit; }
            }
        }
    }

    /// Run one frame of emulation (~13.5ms = ~216000 cycles at 16MHz)
    pub fn run_frame(&mut self) {
        let cycles = (CLOCK_HZ as u64 * 135) / 10000; // 216000
        let end_tick = self.cpu.tick + cycles;
        let mut last_update = self.cpu.tick;

        // Begin sample-accurate audio recording for this frame
        self.audio_buf.begin_frame(self.cpu.tick);

        // PC sampling for stuck detection (debug only)
        let mut pc_counts: Option<std::collections::HashMap<u16, u32>> =
            if self.debug { Some(std::collections::HashMap::new()) } else { None };
        let mut last_sample = self.cpu.tick;

        while self.cpu.tick < end_tick {
            if !self.cpu.sleeping {
                let pc_byte = self.cpu.pc as usize * 2;
                if pc_byte >= self.mem.flash.len() {
                    self.cpu.pc = 0;
                }

                // Check breakpoints
                if !self.breakpoints.is_empty() && self.breakpoints.contains(&self.cpu.pc) {
                    self.breakpoint_hit = true;
                    return;
                }
                
                if let Some(ref mut counts) = pc_counts {
                    if self.cpu.tick - last_sample >= 64 {
                        last_sample = self.cpu.tick;
                        *counts.entry(self.cpu.pc).or_insert(0) += 1;
                    }
                }
                
                self.step();
            } else {
                self.cpu.tick += 4;
            }

            if self.cpu.tick - last_update >= 128 {
                last_update = self.cpu.tick;
                self.flush_spi();
                self.update_peripherals();
            }
        }
        self.update_peripherals();
        self.flush_spi();

        // End sample-accurate audio recording for this frame
        self.audio_buf.end_frame(self.cpu.tick);
        
        self.frame_count += 1;
        if let Some(pc_counts) = pc_counts {
            if self.frame_count <= 5 && !pc_counts.is_empty() {
                let mut top: Vec<_> = pc_counts.into_iter().collect();
                top.sort_by(|a, b| b.1.cmp(&a.1));
                let top5: Vec<String> = top.iter().take(5)
                    .map(|(pc, cnt)| {
                        let byte_addr = (*pc as usize) * 2;
                        let opcode = if byte_addr + 1 < self.mem.flash.len() {
                            (self.mem.flash[byte_addr] as u16) | ((self.mem.flash[byte_addr + 1] as u16) << 8)
                        } else { 0 };
                        format!("0x{:04X}(op=0x{:04X})x{}", pc, opcode, cnt)
                    })
                    .collect();
                eprintln!("  PC hotspots F{}: {}", self.frame_count, top5.join(", "));
            }
        }
    }

    /// Execute a single instruction
    fn step(&mut self) {
        let pc = self.cpu.pc as usize;
        let word = self.mem.read_program_word(pc);
        let next_word = if pc + 1 < FLASH_SIZE / 2 {
            self.mem.read_program_word(pc + 1)
        } else {
            0
        };
        let (inst, size) = opcodes::decode(word, next_word);
        let cycles = self.execute_inst(inst, size);
        self.cpu.tick += cycles as u64;
    }

    /// Execute a single instruction and return its disassembly.
    ///
    /// Used by the debugger for step-by-step execution.
    pub fn step_one(&mut self) -> String {
        let pc = self.cpu.pc;
        let word = self.mem.read_program_word(pc as usize);
        let next_word = if (pc as usize) + 1 < FLASH_SIZE / 2 {
            self.mem.read_program_word(pc as usize + 1)
        } else { 0 };
        let (inst, size) = opcodes::decode(word, next_word);
        let asm = disasm::disassemble(inst, pc);
        let cycles = self.execute_inst(inst, size);
        self.cpu.tick += cycles as u64;
        // Update peripherals after each step
        self.flush_spi();
        self.update_peripherals();
        format!("0x{:04X}: {}", pc * 2, asm)
    }

    /// Disassemble the instruction at the current PC without executing it.
    pub fn disasm_at_pc(&self) -> String {
        let pc = self.cpu.pc;
        let word = self.mem.read_program_word(pc as usize);
        let next_word = if (pc as usize) + 1 < FLASH_SIZE / 2 {
            self.mem.read_program_word(pc as usize + 1)
        } else { 0 };
        let (inst, _) = opcodes::decode(word, next_word);
        let asm = disasm::disassemble(inst, pc);
        format!("0x{:04X}: {}", pc * 2, asm)
    }

    /// Format a register dump string with R0-R31, SP, PC, SREG.
    pub fn dump_regs(&self) -> String {
        let mut s = String::new();
        for i in 0..32 {
            if i % 8 == 0 && i > 0 { s.push('\n'); }
            s.push_str(&format!("R{:2}={:02X} ", i, self.mem.data[i]));
        }
        s.push_str(&format!("\nPC={:04X} SP={:04X} SREG={} (0x{:02X})",
            self.cpu.pc * 2, self.cpu.sp,
            disasm::format_sreg(self.cpu.sreg), self.cpu.sreg));
        s.push_str(&format!("\nX={:04X} Y={:04X} Z={:04X}",
            self.mem.x(), self.mem.y(), self.mem.z()));
        s
    }

    /// Take and clear accumulated USB serial output bytes.
    pub fn take_serial_output(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.serial_buf)
    }

    /// Save EEPROM contents to a byte vector.
    pub fn save_eeprom(&self) -> Vec<u8> {
        self.mem.eeprom.clone()
    }

    /// Load EEPROM contents from a byte slice.
    pub fn load_eeprom(&mut self, data: &[u8]) {
        let len = data.len().min(EEPROM_SIZE);
        self.mem.eeprom[..len].copy_from_slice(&data[..len]);
        self.eeprom_dirty = false;
    }

    /// Get current RGB LED state as (red, green, blue).
    ///
    /// Arduboy LED pins: Red=PB6(OC1B), Green=PB7(OC1C), Blue=PB5(OC1A).
    /// Returns PWM duty or digital on/off approximation.
    pub fn get_led_state(&self) -> (u8, u8, u8) {
        self.led_rgb
    }

    /// Read from data space with peripheral hooks
    pub fn read_data(&mut self, addr: u16) -> u8 {
        let a = addr as usize;

        // GPIO PIN reads
        match addr {
            0x23 => return self.pin_b,
            0x26 => return self.pin_c,
            0x29 => return self.pin_d,
            0x2C => return self.pin_e,
            0x2F => return self.pin_f,
            _ => {}
        }

        // Timer0 reads
        if let Some(v) = self.timer0.read(addr, self.cpu.tick, &self.mem.data) {
            return v;
        }
        // Timer1 reads
        if let Some(v) = self.timer1.read(addr, self.cpu.tick, &self.mem.data) {
            return v;
        }
        // Timer3 reads
        if let Some(v) = self.timer3.read(addr, self.cpu.tick, &self.mem.data) {
            return v;
        }
        // Timer4 reads (ATmega32u4 only)
        if self.cpu_type == CpuType::Atmega32u4 {
            if let Some(v) = self.timer4.read(addr) {
                return v;
            }
        }
        // Timer2 reads (ATmega328P only)
        if self.cpu_type == CpuType::Atmega328p {
            if let Some(v) = self.timer2.read(addr, self.cpu.tick, &self.mem.data) {
                return v;
            }
        }
        // SPI reads
        if let Some(v) = self.spi.read(addr) {
            return v;
        }
        // PLL read
        if addr == 0x49 {
            return self.pll.read();
        }
        // EEPROM data read
        if addr == 0x40 {
            let ea = self.mem.data[0x41] as u16 | ((self.mem.data[0x42] as u16) << 8);
            return self.mem.eeprom.get(ea as usize).copied().unwrap_or(0xFF);
        }
        // ADC reads
        if let Some(v) = self.adc.read(addr) {
            return v;
        }

        // USB Serial register reads (ATmega32u4 only)
        if self.cpu_type == CpuType::Atmega32u4 {
            match addr {
                0xE8 => { // UEINTX - always report ready to send
                    return 0xA1;
                }
                0xE9 => return self.usb_uenum, // UENUM
                0xEE => return 0x61, // UESTA0X
                0xEF => return 0x00, // UESTA1X
                0xF2 => return 0x40, // UEBCLX
                0xF3 => return 0x00, // UEBCHX
                0xD8 => { // USBCON
                    return if self.usb_configured { 0x80 } else { 0 };
                }
                0xD9 => return 0x08, // USBSTA
                0xE3 => return 0x80, // UDADDR
                _ => {}
            }
        }

        if a < self.mem.data.len() {
            self.mem.data[a]
        } else {
            0
        }
    }

    /// Write to data space with peripheral hooks
    pub fn write_data(&mut self, addr: u16, value: u8) {
        let a = addr as usize;
        let old = if a < self.mem.data.len() { self.mem.data[a] } else { 0 };

        // GPIO DDR/PORT writes - track pin changes
        match addr {
            0x24 | 0x25 => { // DDRB, PORTB
                if a < self.mem.data.len() {
                    // Detect PB5 (speaker pin 2) transitions for GPIO-driven audio
                    if addr == 0x25 {
                        let new_pb5 = value & (1 << 5) != 0;
                        if new_pb5 != self.speaker2_prev_pb5 {
                            let tick = self.cpu.tick;
                            // Record edge in sample-accurate audio buffer
                            self.audio_buf.right.push(tick, new_pb5);
                            if self.speaker2_last_edge > 0 {
                                let half = tick.saturating_sub(self.speaker2_last_edge);
                                if half >= 400 && half <= 270000 {
                                    self.speaker2_half_period = half;
                                    self.speaker2_last_active = tick;
                                }
                            }
                            self.speaker2_last_edge = tick;
                            self.speaker2_prev_pb5 = new_pb5;
                        }
                    }
                    self.mem.data[a] = value;
                    // Track LED states from PORTB
                    // RX LED = PB0 (active-low)
                    self.led_rx = value & (1 << 0) == 0;
                    // RGB LED digital: Blue=PB5, Red=PB6, Green=PB7 (active-high)
                    self.led_rgb.2 = if value & (1 << 5) != 0 { 255 } else { 0 }; // Blue
                    self.led_rgb.0 = if value & (1 << 6) != 0 { 255 } else { 0 }; // Red
                    self.led_rgb.1 = if value & (1 << 7) != 0 { 255 } else { 0 }; // Green
                }
                return;
            }
            0x27 | 0x28 => { // DDRC, PORTC
                if a < self.mem.data.len() {
                    // Detect PC6 (speaker pin 1) transitions for GPIO-driven audio
                    if addr == 0x28 {
                        let new_pc6 = value & (1 << 6) != 0;
                        if new_pc6 != self.speaker_prev_pc6 {
                            let tick = self.cpu.tick;
                            // Record edge in sample-accurate audio buffer
                            self.audio_buf.left.push(tick, new_pc6);
                            if self.speaker_last_edge > 0 {
                                let half = tick.saturating_sub(self.speaker_last_edge);
                                // Valid audio range: ~30Hz to ~20kHz
                                // half-period: 16MHz/(2*20000)=400 to 16MHz/(2*30)=266666
                                if half >= 400 && half <= 270000 {
                                    self.speaker_half_period = half;
                                    self.speaker_last_active = tick;
                                }
                            }
                            self.speaker_last_edge = tick;
                            self.speaker_prev_pc6 = new_pc6;
                        }
                    }
                    self.mem.data[a] = value;
                }
                return;
            }
            0x2A => { // DDRD
                if a < self.mem.data.len() { self.mem.data[a] = value; }
                return;
            }
            0x2B => { // PORTD
                if a < self.mem.data.len() { self.mem.data[a] = value; }
                // TX LED = PD5 (active-low)
                self.led_tx = value & (1 << 5) == 0;
                // FX Flash CS = PD2: detect rising edge (deselect)
                if self.fx_flash.loaded {
                    let new_cs_high = value & (1 << 2) != 0;
                    if new_cs_high && !self.fx_cs_prev {
                        self.fx_flash.deselect();
                    }
                    self.fx_cs_prev = new_cs_high;
                }
                return;
            }
            0x2D | 0x2E => { // DDRE, PORTE
                if a < self.mem.data.len() { self.mem.data[a] = value; }
                return;
            }
            0x30 | 0x31 => { // DDRF, PORTF
                if a < self.mem.data.len() { self.mem.data[a] = value; }
                return;
            }
            _ => {}
        }

        // SP writes
        match addr {
            SPH_ADDR => {
                self.cpu.sp = (self.cpu.sp & 0x00FF) | ((value as u16) << 8);
                if a < self.mem.data.len() { self.mem.data[a] = value; }
                return;
            }
            SPL_ADDR => {
                self.cpu.sp = (self.cpu.sp & 0xFF00) | value as u16;
                if a < self.mem.data.len() { self.mem.data[a] = value; }
                return;
            }
            SREG_ADDR => {
                self.cpu.sreg = value;
                if a < self.mem.data.len() { self.mem.data[a] = value; }
                return;
            }
            _ => {}
        }

        // Timer0 writes
        if self.timer0.write(addr, value, old, &mut self.mem.data) { return; }
        // Timer1 writes
        if self.timer1.write(addr, value, old, &mut self.mem.data) { return; }
        // Timer3 writes
        if self.timer3.write(addr, value, old, &mut self.mem.data) { return; }
        // Timer4 writes (ATmega32u4 only)
        if self.cpu_type == CpuType::Atmega32u4 {
            if self.timer4.write(addr, value) {
                if a < self.mem.data.len() { self.mem.data[a] = value; }
                return;
            }
        }
        // Timer2 writes (ATmega328P only)
        if self.cpu_type == CpuType::Atmega328p {
            if self.timer2.write(addr, value, old, &mut self.mem.data) { return; }
        }

        // SPI writes
        if self.spi.write(addr, value) {
            // Store value in mem.data so reads return correct value
            if a < self.mem.data.len() { self.mem.data[a] = value; }
            // If SPDR written, data goes to SPI output with current DC state
            if addr == 0x4E {
                let portd = self.mem.data[0x2B];
                let portf = self.mem.data[0x31];
                let ddrd = self.mem.data[0x2A];
                
                // FX Flash CS = PD2 (active LOW)
                // Only route to flash when: data loaded + PD2 set as output + PD2 driven LOW
                let fx_cs_active = self.fx_flash.loaded
                    && (ddrd & (1 << 2) != 0)   // PD2 configured as output
                    && (portd & (1 << 2) == 0);  // PD2 driven LOW
                
                if fx_cs_active {
                    // Route to FX flash - full duplex: send MOSI, receive MISO
                    let response = self.fx_flash.transfer(value);
                    self.spdr_in = response;
                    // Store response in mem.data[SPDR] so game can read it
                    self.mem.data[0x4E] = response;
                } else {
                    // Route to display SPI
                    if self.debug && (self.dbg_spdr_writes < 30 || (self.dbg_spdr_writes >= 85 && self.dbg_spdr_writes < 100)
                        || (self.dbg_spdr_writes >= 1024 && self.dbg_spdr_writes < 1040)) {
                        let portb = self.mem.data[0x25];
                        let porte = self.mem.data[0x2E];
                        eprintln!("  SPI#{:3} val=0x{:02X} PD4={} PD6={} PF5={} PF6={} PORTB=0x{:02X} PORTD=0x{:02X} PORTE=0x{:02X} PORTF=0x{:02X}",
                            self.dbg_spdr_writes, value, 
                            (portd >> 4) & 1, (portd >> 6) & 1,
                            (portf >> 5) & 1, (portf >> 6) & 1,
                            portb, portd, porte, portf);
                    }
                    let portc = self.mem.data[0x28];
                    self.spi_out.push((value, portd, portf, portc));
                    self.spdr_in = 0xFF;
                }
                self.dbg_spdr_writes += 1;
            }
            return;
        }

        // PLL write
        if addr == 0x49 {
            self.pll.write(value);
            if a < self.mem.data.len() { self.mem.data[a] = value; }
            return;
        }

        // EEPROM control write
        if addr == 0x3F {
            let ea = self.mem.data[0x41] as u16 | ((self.mem.data[0x42] as u16) << 8);
            if value & 0x02 != 0 {
                let data_val = self.mem.data[0x40];
                if (ea as usize) < self.mem.eeprom.len() {
                    self.mem.eeprom[ea as usize] = data_val;
                    self.eeprom_dirty = true;
                }
            }
            if a < self.mem.data.len() { self.mem.data[a] = value & !2; }
            return;
        }

        // ADC writes
        if self.adc.write(addr, value, &mut self.rng_state) {
            if a < self.mem.data.len() { self.mem.data[a] = value; }
            return;
        }

        // USB Serial registers (ATmega32u4 only)
        if self.cpu_type == CpuType::Atmega32u4 {
            match addr {
            0xE9 => { // UENUM - endpoint select
                self.usb_uenum = value & 0x07;
                if a < self.mem.data.len() { self.mem.data[a] = value; }
                return;
            }
            0xF1 => { // UEDATX - write data to endpoint
                // Capture serial output from CDC endpoint (typically EP3)
                if self.usb_uenum >= 3 {
                    self.serial_buf.push(value);
                }
                return;
            }
            0xE8 => { // UEINTX - clear interrupt flags by writing 0
                if a < self.mem.data.len() { self.mem.data[a] = value; }
                return;
            }
            0xD8 => { // USBCON
                self.usb_configured = value & 0x80 != 0; // USBE bit
                if a < self.mem.data.len() { self.mem.data[a] = value; }
                return;
            }
            0xE3 => { // UDADDR
                if a < self.mem.data.len() { self.mem.data[a] = value | 0x80; } // ADDEN always set
                return;
            }
            0xE1 | 0xE2 | // UDINT, UDIEN
            0xEA | // UERST
            0xEB | // UECONX
            0xEC | // UECFG0X
            0xED | // UECFG1X
            0xF0   // UEIENX
            => {
                if a < self.mem.data.len() { self.mem.data[a] = value; }
                return;
            }
            _ => {}
            }
        }

        // Default write
        if a < self.mem.data.len() {
            self.mem.data[a] = value;
        }
    }

    /// Write a bit in data space
    pub fn write_bit(&mut self, addr: u16, bit: u8, bvalue: bool) {
        let val = self.read_data(addr);
        let new_val = if bvalue {
            val | (1 << bit)
        } else {
            val & !(1 << bit)
        };
        self.write_data(addr, new_val);
    }

    /// Flush SPI output to display
    fn flush_spi(&mut self) {
        let bytes: Vec<(u8, u8, u8, u8)> = self.spi_out.drain(..).collect();
        for (byte, portd, portf, portc) in bytes {
            // Decode DC and CS based on display type and CPU
            // Arduboy (32u4):           DC=PD4(bit4), CS=PD6(bit6) - active LOW
            // Gamebuino (32u4 PCD8544): DC=PF5(bit5), CS=PF6(bit6) - active LOW
            // Gamebuino Classic (328P): DC=PC3(bit3), CS=PC2(bit2) - active LOW
            let (is_data, cs_high) = if self.cpu_type == CpuType::Atmega328p {
                // 328P: always PCD8544, CS=PC2, DC=PC3
                (portc & (1 << 3) != 0, portc & (1 << 2) != 0)
            } else {
                match self.display_type {
                    DisplayType::Ssd1306 => {
                        (portd & (1 << 4) != 0, portd & (1 << 6) != 0)
                    }
                    DisplayType::Pcd8544 => {
                        (portf & (1 << 5) != 0, portf & (1 << 6) != 0)
                    }
                    DisplayType::Unknown => {
                        let ardu_cs_active = portd & (1 << 6) == 0;
                        let ardu_dc_cmd = portd & (1 << 4) == 0;
                        let gb_cs_active = portf & (1 << 6) == 0;
                        let gb_dc_cmd = portf & (1 << 5) == 0;

                        if self.debug && self.dbg_spdr_writes < 30 {
                            eprintln!("  DETECT: val=0x{:02X} ardu(cs={} dc_cmd={}) gb(cs={} dc_cmd={})",
                                byte, ardu_cs_active, ardu_dc_cmd, gb_cs_active, gb_dc_cmd);
                        }

                        if ardu_cs_active && ardu_dc_cmd {
                            if byte >= 0x80 {
                                self.display_type = DisplayType::Ssd1306;
                                if self.debug {
                                    eprintln!("Display auto-detected: SSD1306 (first cmd: 0x{:02X}, PD4=0 PD6=0)", byte);
                                }
                            }
                        }
                        if self.display_type == DisplayType::Unknown && gb_cs_active && gb_dc_cmd {
                            if byte == 0x21 || byte == 0x20 {
                                self.display_type = DisplayType::Pcd8544;
                                if self.debug {
                                    eprintln!("Display auto-detected: PCD8544 (first cmd: 0x{:02X}, PF5=0 PF6=0)", byte);
                                }
                            }
                        }

                        match self.display_type {
                            DisplayType::Pcd8544 => (portf & (1 << 5) != 0, portf & (1 << 6) != 0),
                            _ => (portd & (1 << 4) != 0, portd & (1 << 6) != 0),
                        }
                    }
                }
            };

            // Skip SPI bytes when display CS is HIGH (not selected)
            if cs_high {
                continue;
            }

            match self.display_type {
                DisplayType::Pcd8544 => {
                    if is_data {
                        self.pcd8544.receive_data(byte);
                    } else {
                        self.pcd8544.receive_command(byte);
                    }
                }
                _ => {
                    if is_data {
                        self.display.receive_data(byte);
                    } else {
                        self.display.receive_command(byte);
                    }
                }
            }
        }
        if self.display_type == DisplayType::Pcd8544 {
            self.pcd8544.render_to_framebuffer();
        }
    }

    /// Update all peripherals and handle interrupts
    fn update_peripherals(&mut self) {
        let ie = self.cpu.sreg & (1 << SREG_I) != 0;
        let tick = self.cpu.tick;

        // Flush SPI to display
        self.flush_spi();

        // Timer0
        self.timer0.update(tick, &mut self.mem.data);
        if ie {
            if let Some(vec_addr) = self.timer0.check_interrupt() {
                self.cpu.sleeping = false;
                self.do_interrupt(vec_addr);
                return;
            }
        }

        // Timer1
        self.timer1.update(tick, &mut self.mem.data);
        if ie {
            if let Some(vec_addr) = self.timer1.check_interrupt() {
                self.cpu.sleeping = false;
                self.do_interrupt(vec_addr);
                return;
            }
        }

        // Timer3 (ATmega32u4 only)
        if self.cpu_type == CpuType::Atmega32u4 {
            self.timer3.update(tick, &mut self.mem.data);
            if ie {
                if let Some(vec_addr) = self.timer3.check_interrupt() {
                    self.cpu.sleeping = false;
                    self.do_interrupt(vec_addr);
                    return;
                }
            }
        }

        // Timer4 (ATmega32u4 only)
        if self.cpu_type == CpuType::Atmega32u4 {
            self.timer4.update(tick, &mut self.mem.data);
            if ie {
                if let Some(vec_addr) = self.timer4.check_interrupt() {
                    self.cpu.sleeping = false;
                    self.do_interrupt(vec_addr);
                    return;
                }
            }
        }

        // Timer2 (ATmega328P only)
        if self.cpu_type == CpuType::Atmega328p {
            self.timer2.update(tick, &mut self.mem.data);
            if ie {
                if let Some(vec_addr) = self.timer2.check_interrupt() {
                    self.cpu.sleeping = false;
                    self.do_interrupt(vec_addr);
                    return;
                }
            }
        }

        // SPI
        if ie {
            if let Some(vec_addr) = self.spi.check_interrupt() {
                self.cpu.sleeping = false;
                self.do_interrupt(vec_addr);
                return;
            }
        }

        // ADC
        self.adc.update(&mut self.rng_state);
        if ie {
            if let Some(vec_addr) = self.adc.check_interrupt() {
                self.cpu.sleeping = false;
                self.do_interrupt(vec_addr);
                return;
            }
        }
    }

    /// Execute an interrupt: push PC, jump to vector
    fn do_interrupt(&mut self, vector: u16) {
        let pc = self.cpu.pc;
        // Push return address (same order as push_word/CALL)
        self.mem.data[self.cpu.sp as usize] = (pc >> 8) as u8;
        self.cpu.sp = self.cpu.sp.wrapping_sub(1);
        self.mem.data[self.cpu.sp as usize] = pc as u8;
        self.cpu.sp = self.cpu.sp.wrapping_sub(1);
        // Sync SP to memory registers
        self.mem.data[SPH_ADDR as usize] = (self.cpu.sp >> 8) as u8;
        self.mem.data[SPL_ADDR as usize] = self.cpu.sp as u8;
        // Disable interrupts
        self.cpu.sreg &= !(1 << SREG_I);
        self.mem.data[SREG_ADDR as usize] = self.cpu.sreg;
        self.cpu.pc = vector;
        self.cpu.tick += 5;
    }

    /// Get display pixel buffer as RGBA u32 slice (for minifb etc)
    pub fn framebuffer_u32(&self) -> Vec<u32> {
        match self.display_type {
            DisplayType::Pcd8544 => {
                let fb = &self.pcd8544.framebuffer;
                let mut buf = Vec::with_capacity(SCREEN_WIDTH * SCREEN_HEIGHT);
                for i in 0..(SCREEN_WIDTH * SCREEN_HEIGHT) {
                    let offset = i * 4;
                    let r = fb[offset] as u32;
                    let g = fb[offset + 1] as u32;
                    let b = fb[offset + 2] as u32;
                    buf.push((r << 16) | (g << 8) | b);
                }
                buf
            }
            _ => self.display.as_pixel_buffer(),
        }
    }

    /// Get display framebuffer RGBA bytes
    pub fn framebuffer_rgba(&self) -> &[u8] {
        match self.display_type {
            DisplayType::Pcd8544 => &self.pcd8544.framebuffer,
            _ => &self.display.framebuffer,
        }
    }

    /// Simple xorshift PRNG
    pub fn next_random(&mut self) -> u8 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 17;
        self.rng_state ^= self.rng_state << 5;
        (self.rng_state & 0xFF) as u8
    }

    /// Get current tone frequencies for stereo audio output.
    ///
    /// Returns `(left_hz, right_hz)`:
    /// - Left channel: Timer3 CTC tone → Timer4 CTC tone → GPIO PC6 bit-bang (Speaker 1)
    /// - Right channel: Timer1 CTC tone → GPIO PB5 bit-bang (Speaker 2)
    ///
    /// Priority within each channel: hardware timer > GPIO bit-bang.
    pub fn get_audio_tone(&self) -> (f32, f32) {
        let t1 = self.timer1.get_tone_hz(CLOCK_HZ);

        // Timer3/Timer4 only on 32u4
        let t3 = if self.cpu_type == CpuType::Atmega32u4 {
            self.timer3.get_tone_hz(CLOCK_HZ)
        } else { 0.0 };
        let t4 = if self.cpu_type == CpuType::Atmega32u4 {
            self.timer4.get_tone_hz(CLOCK_HZ)
        } else { 0.0 };

        // GPIO bit-bang speaker 1 (PC6): derive frequency from toggle rate
        let gpio1_hz = if self.speaker_half_period > 0 {
            let age = self.cpu.tick.saturating_sub(self.speaker_last_active);
            if age < 250_000 {
                CLOCK_HZ as f32 / (2.0 * self.speaker_half_period as f32)
            } else { 0.0 }
        } else { 0.0 };

        // GPIO bit-bang speaker 2 (PB5): derive frequency from toggle rate
        let gpio2_hz = if self.speaker2_half_period > 0 {
            let age = self.cpu.tick.saturating_sub(self.speaker2_last_active);
            if age < 250_000 {
                CLOCK_HZ as f32 / (2.0 * self.speaker2_half_period as f32)
            } else { 0.0 }
        } else { 0.0 };

        // Left: Timer3 > Timer4 > GPIO PC6
        let left = if t3 > 0.0 { t3 } else if t4 > 0.0 { t4 } else { gpio1_hz };
        // Right: Timer1 > GPIO PB5
        let right = if t1 > 0.0 { t1 } else { gpio2_hz };

        (left, right)
    }
}

impl Default for Arduboy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arduboy_creation() {
        let ard = Arduboy::new();
        assert_eq!(ard.cpu.pc, 0);
        assert_eq!(ard.cpu.sp, (DATA_SIZE - 1) as u16);
        assert_eq!(ard.cpu_type, CpuType::Atmega32u4);
    }

    #[test]
    fn test_328p_creation() {
        let ard = Arduboy::new_with_cpu(CpuType::Atmega328p);
        assert_eq!(ard.cpu.pc, 0);
        assert_eq!(ard.cpu.sp, (DATA_SIZE_328P - 1) as u16);
        assert_eq!(ard.cpu_type, CpuType::Atmega328p);
        assert_eq!(ard.display_type, DisplayType::Pcd8544);
    }

    #[test]
    fn test_button_press() {
        let mut ard = Arduboy::new();
        assert_eq!(ard.pin_f & (1 << 7), 1 << 7); // UP released
        ard.set_button(Button::Up, true);
        assert_eq!(ard.pin_f & (1 << 7), 0); // UP pressed (active low)
        ard.set_button(Button::Up, false);
        assert_eq!(ard.pin_f & (1 << 7), 1 << 7); // UP released
    }

    #[test]
    fn test_328p_button_press() {
        let mut ard = Arduboy::new_with_cpu(CpuType::Atmega328p);
        // 328P Gamebuino: UP=PB1
        assert_eq!(ard.pin_b & (1 << 1), 1 << 1);
        ard.set_button(Button::Up, true);
        assert_eq!(ard.pin_b & (1 << 1), 0);
        ard.set_button(Button::Up, false);
        assert_eq!(ard.pin_b & (1 << 1), 1 << 1);
    }

    #[test]
    fn test_load_hex() {
        let mut ard = Arduboy::new();
        let hex = ":100000000C9434000C944E000C944E000C944E00A4\n:00000001FF\n";
        let result = ard.load_hex(hex);
        assert!(result.is_ok());
        assert_eq!(ard.mem.flash[0], 0x0C);
        assert_eq!(ard.mem.flash[1], 0x94);
    }
}
