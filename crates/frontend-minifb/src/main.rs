//! Arduboy emulator frontend v0.3.0.
//!
//! Provides three execution modes:
//!
//! - **GUI mode** (default): Scaled window with stereo audio, keyboard/gamepad input,
//!   dynamic scale toggle, screenshot, serial monitor.
//! - **Headless mode** (`--headless`): Automated testing with ASCII snapshots.
//! - **Step mode** (`--step`): Interactive instruction-level debugger.
//!
//! ## v0.2.0 features
//! - Disassembler, breakpoints (`--break`), step mode (`--step`)
//! - Register/SREG/SP dump (D key in GUI)
//! - SSD1306 contrast / invert (core)
//! - Window scale toggle (1–6 keys), fullscreen (F11)
//! - Screenshot (S key → BMP file)
//!
//! ## v0.3.0 features
//! - 2-channel stereo audio (Timer3→left, Timer1/GPIO→right)
//! - USB Serial output (captured to stderr with --serial)

use arduboy_core::{Arduboy, Button, SCREEN_WIDTH, SCREEN_HEIGHT};
use minifb::{Key, Window, WindowOptions, Scale, ScaleMode};
use gilrs::{Gilrs, Event as GilrsEvent, EventType, Axis, Button as GilrsButton};
use std::env;
use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};
use std::io::Write;

/// Audio output sample rate in Hz
const AUDIO_SAMPLE_RATE: u32 = 44100;
/// Square wave amplitude (0.0–1.0)
const AUDIO_VOLUME: f32 = 0.15;
/// Analog stick deadzone
const STICK_DEADZONE: f32 = 0.3;
/// Analog trigger deadzone
const TRIGGER_DEADZONE: f32 = 0.2;

// ─── Audio Sources ──────────────────────────────────────────────────────────

/// Hybrid audio source: uses sample-accurate PCM from ring buffer when
/// available (GPIO bit-bang), falls back to square wave synthesis for
/// timer-driven tones.
struct HybridAudioSource {
    ring: Arc<std::sync::Mutex<std::collections::VecDeque<f32>>>,
    freq_l: Arc<AtomicU32>,
    freq_r: Arc<AtomicU32>,
    sample_rate: u32,
    phase_l: f32,
    phase_r: f32,
    left_next: bool,
}

impl HybridAudioSource {
    fn new(
        ring: Arc<std::sync::Mutex<std::collections::VecDeque<f32>>>,
        freq_l: Arc<AtomicU32>,
        freq_r: Arc<AtomicU32>,
        sample_rate: u32,
    ) -> Self {
        HybridAudioSource {
            ring, freq_l, freq_r, sample_rate,
            phase_l: 0.0, phase_r: 0.0, left_next: true,
        }
    }
}

impl Iterator for HybridAudioSource {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        // Try to drain from sample-accurate ring buffer first
        if let Ok(mut ring) = self.ring.try_lock() {
            if !ring.is_empty() {
                return ring.pop_front();
            }
        }
        // Fallback: synthesize square wave from timer frequencies
        if self.left_next {
            self.left_next = false;
            let freq = f32::from_bits(self.freq_l.load(Ordering::Relaxed));
            if freq <= 0.0 { self.phase_l = 0.0; return Some(0.0); }
            let s = if self.phase_l < 0.5 { AUDIO_VOLUME } else { -AUDIO_VOLUME };
            self.phase_l += freq / self.sample_rate as f32;
            self.phase_l %= 1.0;
            Some(s)
        } else {
            self.left_next = true;
            let freq = f32::from_bits(self.freq_r.load(Ordering::Relaxed));
            if freq <= 0.0 { self.phase_r = 0.0; return Some(0.0); }
            let s = if self.phase_r < 0.5 { AUDIO_VOLUME } else { -AUDIO_VOLUME };
            self.phase_r += freq / self.sample_rate as f32;
            self.phase_r %= 1.0;
            Some(s)
        }
    }
}

impl rodio::Source for HybridAudioSource {
    fn current_frame_len(&self) -> Option<usize> { None }
    fn channels(&self) -> u16 { 2 }
    fn sample_rate(&self) -> u32 { self.sample_rate }
    fn total_duration(&self) -> Option<Duration> { None }
}

fn setup_audio(
    ring: Arc<std::sync::Mutex<std::collections::VecDeque<f32>>>,
    freq_l: Arc<AtomicU32>,
    freq_r: Arc<AtomicU32>,
) -> Option<(rodio::OutputStream, rodio::OutputStreamHandle, rodio::Sink)>
{
    match rodio::OutputStream::try_default() {
        Ok((stream, handle)) => {
            match rodio::Sink::try_new(&handle) {
                Ok(sink) => {
                    let source = HybridAudioSource::new(ring, freq_l, freq_r, AUDIO_SAMPLE_RATE);
                    sink.append(source);
                    Some((stream, handle, sink))
                }
                Err(e) => { eprintln!("Warning: audio sink: {}", e); None }
            }
        }
        Err(e) => { eprintln!("Warning: audio device: {}", e); None }
    }
}

// ─── Gamepad ────────────────────────────────────────────────────────────────

struct GamepadState {
    up: bool, down: bool, left: bool, right: bool,
    a: bool, b: bool,
    left_stick_x: f32, left_stick_y: f32,
}

impl GamepadState {
    fn new() -> Self {
        GamepadState {
            up: false, down: false, left: false, right: false,
            a: false, b: false, left_stick_x: 0.0, left_stick_y: 0.0,
        }
    }
    fn eff_up(&self)    -> bool { self.up    || self.left_stick_y < -STICK_DEADZONE }
    fn eff_down(&self)  -> bool { self.down  || self.left_stick_y >  STICK_DEADZONE }
    fn eff_left(&self)  -> bool { self.left  || self.left_stick_x < -STICK_DEADZONE }
    fn eff_right(&self) -> bool { self.right || self.left_stick_x >  STICK_DEADZONE }
}

fn init_gamepad(debug: bool) -> Option<Gilrs> {
    match Gilrs::new() {
        Ok(gilrs) => {
            if debug {
                let mut found = false;
                for (id, gp) in gilrs.gamepads() {
                    println!("Gamepad: [{}] \"{}\" ({})", id, gp.name(), gp.os_name());
                    found = true;
                }
                if !found { println!("No gamepad (hot-plug supported)."); }
            }
            Some(gilrs)
        }
        Err(e) => { eprintln!("Warning: gamepad: {}", e); None }
    }
}

fn poll_gamepad(gilrs: &mut Gilrs, state: &mut GamepadState, debug: bool) {
    while let Some(GilrsEvent { event, .. }) = gilrs.next_event() {
        match event {
            EventType::ButtonPressed(b, _)  => apply_button(state, b, true),
            EventType::ButtonReleased(b, _) => apply_button(state, b, false),
            EventType::AxisChanged(a, v, _) => apply_axis(state, a, v),
            EventType::Connected => {
                if debug {
                    for (_, gp) in gilrs.gamepads() {
                        if gp.is_connected() { println!("Gamepad connected: \"{}\"", gp.name()); }
                    }
                }
            }
            EventType::Disconnected => { if debug { println!("Gamepad disconnected"); } *state = GamepadState::new(); }
            _ => {}
        }
    }
}

fn apply_button(state: &mut GamepadState, btn: GilrsButton, pressed: bool) {
    match btn {
        GilrsButton::DPadUp    => state.up    = pressed,
        GilrsButton::DPadDown  => state.down  = pressed,
        GilrsButton::DPadLeft  => state.left  = pressed,
        GilrsButton::DPadRight => state.right = pressed,
        GilrsButton::South | GilrsButton::East | GilrsButton::Start => state.b = pressed,
        GilrsButton::West | GilrsButton::North |
        GilrsButton::LeftTrigger  | GilrsButton::RightTrigger |
        GilrsButton::LeftTrigger2 | GilrsButton::RightTrigger2 |
        GilrsButton::Select => state.a = pressed,
        _ => {}
    }
}

fn apply_axis(state: &mut GamepadState, axis: Axis, value: f32) {
    match axis {
        Axis::LeftStickX  => state.left_stick_x = value,
        Axis::LeftStickY  => state.left_stick_y = value,
        Axis::RightStickX => { if state.left_stick_x.abs() < 0.01 { state.left_stick_x = value; } }
        Axis::RightStickY => { if state.left_stick_y.abs() < 0.01 { state.left_stick_y = value; } }
        Axis::DPadX => { state.left = value < -STICK_DEADZONE; state.right = value > STICK_DEADZONE; }
        Axis::DPadY => { state.up = value < -STICK_DEADZONE; state.down = value > STICK_DEADZONE; }
        Axis::LeftZ | Axis::RightZ => {
            if value > TRIGGER_DEADZONE { state.a = true; }
            else if value < 0.05 { state.a = false; }
        }
        _ => {}
    }
}

// ─── Screenshot (BMP) ───────────────────────────────────────────────────────

fn save_screenshot(arduboy: &Arduboy, path: &str) -> Result<(), String> {
    let pixels = arduboy.framebuffer_u32();
    let w = SCREEN_WIDTH as u32;
    let h = SCREEN_HEIGHT as u32;
    let row_size = (w * 3 + 3) & !3;
    let pixel_data_size = row_size * h;
    let file_size = 54 + pixel_data_size;
    let mut data = Vec::with_capacity(file_size as usize);
    // BMP header
    data.extend_from_slice(b"BM");
    data.extend_from_slice(&file_size.to_le_bytes());
    data.extend_from_slice(&0u16.to_le_bytes());
    data.extend_from_slice(&0u16.to_le_bytes());
    data.extend_from_slice(&54u32.to_le_bytes());
    // DIB header
    data.extend_from_slice(&40u32.to_le_bytes());
    data.extend_from_slice(&w.to_le_bytes());
    data.extend_from_slice(&h.to_le_bytes());
    data.extend_from_slice(&1u16.to_le_bytes());
    data.extend_from_slice(&24u16.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes());
    data.extend_from_slice(&pixel_data_size.to_le_bytes());
    data.extend_from_slice(&2835u32.to_le_bytes());
    data.extend_from_slice(&2835u32.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes());
    // Pixel data (bottom-up BGR)
    for y in (0..h as usize).rev() {
        let mut row_bytes = 0u32;
        for x in 0..w as usize {
            let px = pixels[y * SCREEN_WIDTH + x];
            data.push((px & 0xFF) as u8);
            data.push(((px >> 8) & 0xFF) as u8);
            data.push(((px >> 16) & 0xFF) as u8);
            row_bytes += 3;
        }
        while row_bytes % 4 != 0 { data.push(0); row_bytes += 1; }
    }
    fs::write(path, &data).map_err(|e| format!("{}: {}", path, e))
}

// ─── Main ───────────────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Arduboy Emulator v0.3.0 - Rust");
        eprintln!("Usage: {} <file.hex> [options]", args[0]);
        eprintln!();
        eprintln!("Options:");
        eprintln!("  --headless           Run without GUI");
        eprintln!("  --frames N           Run N frames (headless/step, default 60)");
        eprintln!("  --debug              Show per-frame diagnostics");
        eprintln!("  --press N            Press A on frame N (headless)");
        eprintln!("  --snapshot F         Print display at frame F (repeatable)");
        eprintln!("  --mute               Disable audio");
        eprintln!("  --fx <file.bin>      Load FX flash data");
        eprintln!("  --break <addr>       Breakpoint at hex byte-address (repeatable)");
        eprintln!("  --step               Interactive step debugger");
        eprintln!("  --scale N            Initial scale 1-6 (default 6)");
        eprintln!("  --serial             Show USB serial output on stderr");
        eprintln!();
        eprintln!("GUI keys: Arrows=D-pad Z=A X=B 1-6=Scale F11=Fullscreen");
        eprintln!("          S=Screenshot D=RegDump M=Mute Esc=Quit");
        std::process::exit(1);
    }

    let hex_path = &args[1];
    let headless = args.iter().any(|a| a == "--headless");
    let mute = args.iter().any(|a| a == "--mute");
    let debug = args.iter().any(|a| a == "--debug");
    let step_mode = args.iter().any(|a| a == "--step");
    let serial_enabled = args.iter().any(|a| a == "--serial");

    let initial_scale: usize = args.iter()
        .position(|a| a == "--scale")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(6).max(1).min(6);

    let hex_str = fs::read_to_string(hex_path).expect("Failed to read HEX file");
    let mut arduboy = Arduboy::new();
    arduboy.debug = debug;
    let size = arduboy.load_hex(&hex_str).expect("Failed to parse HEX");
    if debug { println!("Loaded {} bytes into flash", size); }

    // Parse breakpoints
    {
        let mut i = 0;
        while i < args.len() {
            if args[i] == "--break" {
                if let Some(s) = args.get(i + 1) {
                    let s = s.trim_start_matches("0x").trim_start_matches("0X");
                    if let Ok(addr) = u16::from_str_radix(s, 16) {
                        let word_addr = addr / 2;
                        arduboy.breakpoints.push(word_addr);
                        if debug { println!("Breakpoint: 0x{:04X} (word 0x{:04X})", addr, word_addr); }
                    }
                }
                i += 2;
            } else { i += 1; }
        }
    }

    // Load FX data
    let fx_path: Option<String> = args.iter()
        .position(|a| a == "--fx")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.clone())
        .or_else(|| {
            let bin = hex_path.replace(".hex", ".bin").replace(".HEX", ".bin");
            if bin != *hex_path && std::path::Path::new(&bin).exists() { return Some(bin); }
            let dir = std::path::Path::new(hex_path).parent().unwrap_or(std::path::Path::new("."));
            let stem = std::path::Path::new(hex_path).file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let fx = dir.join(format!("{}-fx.bin", stem));
            if fx.exists() { Some(fx.to_string_lossy().into_owned()) } else { None }
        });
    if let Some(ref path) = fx_path {
        match fs::read(path) {
            Ok(bin) => {
                if debug { println!("FX data: {} ({} bytes)", path, bin.len()); }
                arduboy.load_fx_data(&bin);
            }
            Err(e) => eprintln!("Warning: FX data {}: {}", path, e),
        }
    }

    if step_mode {
        run_step_mode(&args, &mut arduboy);
    } else if headless {
        run_headless(&args, &mut arduboy, serial_enabled);
    } else {
        run_gui(&mut arduboy, mute, debug, initial_scale, serial_enabled);
    }
}

// ─── GUI Mode ───────────────────────────────────────────────────────────────

fn run_gui(arduboy: &mut Arduboy, start_muted: bool, debug: bool, initial_scale: usize, serial_enabled: bool) {
    let mut scale = initial_scale;
    let mut scaled_w = SCREEN_WIDTH * scale;
    let mut scaled_h = SCREEN_HEIGHT * scale;

    let mut window = Window::new(
        "Arduboy Emulator v0.3.0", scaled_w, scaled_h,
        WindowOptions {
            scale: Scale::X1,
            scale_mode: ScaleMode::AspectRatioStretch,
            resize: true,
            ..Default::default()
        },
    ).expect("Failed to create window");
    window.set_target_fps(60);

    let audio_ring: Arc<std::sync::Mutex<std::collections::VecDeque<f32>>> =
        Arc::new(std::sync::Mutex::new(std::collections::VecDeque::with_capacity(16384)));
    let freq_l = Arc::new(AtomicU32::new(0.0f32.to_bits()));
    let freq_r = Arc::new(AtomicU32::new(0.0f32.to_bits()));
    let mut muted = start_muted;
    let mut _audio = if !muted { setup_audio(audio_ring.clone(), freq_l.clone(), freq_r.clone()) } else { None };
    let mut pcm_buf: Vec<f32> = Vec::with_capacity(16384);

    let mut gilrs = init_gamepad(debug);
    let mut gp = GamepadState::new();
    let mut frame_count: u64 = 0;
    let start_time = Instant::now();
    let mut last_fps_time = Instant::now();
    let mut fps_frames: u64 = 0;
    let mut scaled_buf = vec![0u32; scaled_w * scaled_h];
    let mut prev_m = false;
    let mut prev_s = false;
    let mut prev_d = false;
    let mut prev_f11 = false;
    let mut fullscreen = false;
    let mut screenshot_n = 0u32;
    let mut prev_num = [false; 6];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if let Some(ref mut g) = gilrs { poll_gamepad(g, &mut gp, debug); }

        // Scale toggle (1-6)
        let num = [
            window.is_key_down(Key::Key1), window.is_key_down(Key::Key2),
            window.is_key_down(Key::Key3), window.is_key_down(Key::Key4),
            window.is_key_down(Key::Key5), window.is_key_down(Key::Key6),
        ];
        for i in 0..6 {
            if num[i] && !prev_num[i] && !fullscreen {
                scale = i + 1;
                scaled_w = SCREEN_WIDTH * scale;
                scaled_h = SCREEN_HEIGHT * scale;
                scaled_buf.resize(scaled_w * scaled_h, 0);
                window = Window::new(
                    "Arduboy Emulator v0.3.0", scaled_w, scaled_h,
                    WindowOptions { scale: Scale::X1, scale_mode: ScaleMode::AspectRatioStretch, resize: true, ..Default::default() },
                ).expect("window");
                window.set_target_fps(60);
            }
        }
        prev_num = num;

        // Fullscreen (F11)
        let f11 = window.is_key_down(Key::F11);
        if f11 && !prev_f11 {
            fullscreen = !fullscreen;
            if fullscreen {
                scaled_w = SCREEN_WIDTH * 12;
                scaled_h = SCREEN_HEIGHT * 12;
            } else {
                scaled_w = SCREEN_WIDTH * scale;
                scaled_h = SCREEN_HEIGHT * scale;
            }
            scaled_buf.resize(scaled_w * scaled_h, 0);
            let mut opts = WindowOptions { scale: Scale::X1, scale_mode: ScaleMode::AspectRatioStretch, resize: true, ..Default::default() };
            if fullscreen { opts.borderless = true; }
            window = Window::new("Arduboy Emulator v0.3.0", scaled_w, scaled_h, opts).expect("window");
            window.set_target_fps(60);
        }
        prev_f11 = f11;

        // Mute (M)
        let m = window.is_key_down(Key::M);
        if m && !prev_m {
            muted = !muted;
            if muted {
                freq_l.store(0.0f32.to_bits(), Ordering::Relaxed);
                freq_r.store(0.0f32.to_bits(), Ordering::Relaxed);
                _audio = None;
            } else {
                _audio = setup_audio(audio_ring.clone(), freq_l.clone(), freq_r.clone());
            }
        }
        prev_m = m;

        // Screenshot (S)
        let s = window.is_key_down(Key::S);
        if s && !prev_s {
            let f = format!("screenshot_{:04}.bmp", screenshot_n);
            match save_screenshot(arduboy, &f) {
                Ok(()) => { eprintln!("Screenshot: {}", f); screenshot_n += 1; }
                Err(e) => eprintln!("Screenshot error: {}", e),
            }
        }
        prev_s = s;

        // Reg dump (D)
        let d = window.is_key_down(Key::D);
        if d && !prev_d {
            eprintln!("--- Regs (frame {}) ---\n{}\nNext: {}\n---",
                frame_count, arduboy.dump_regs(), arduboy.disasm_at_pc());
        }
        prev_d = d;

        // Input
        arduboy.set_button(Button::Up,    window.is_key_down(Key::Up)    || gp.eff_up());
        arduboy.set_button(Button::Down,  window.is_key_down(Key::Down)  || gp.eff_down());
        arduboy.set_button(Button::Left,  window.is_key_down(Key::Left)  || gp.eff_left());
        arduboy.set_button(Button::Right, window.is_key_down(Key::Right) || gp.eff_right());
        arduboy.set_button(Button::A,     window.is_key_down(Key::Z)     || gp.a);
        arduboy.set_button(Button::B,     window.is_key_down(Key::X)     || gp.b);

        arduboy.run_frame();
        frame_count += 1;
        fps_frames += 1;

        if arduboy.breakpoint_hit {
            eprintln!("*** Breakpoint: {} ***\n{}", arduboy.disasm_at_pc(), arduboy.dump_regs());
            arduboy.breakpoint_hit = false;
        }

        if serial_enabled {
            let out = arduboy.take_serial_output();
            if !out.is_empty() {
                let _ = std::io::stderr().write_all(&out);
                let _ = std::io::stderr().flush();
            }
        }

        if !muted {
            let (lh, rh) = arduboy.get_audio_tone();
            // If sample-accurate GPIO audio edges were recorded this frame,
            // render them to PCM and push to ring buffer (bypassing frequency synth)
            if arduboy.audio_buf.has_audio() {
                arduboy.audio_buf.render_samples(
                    &mut pcm_buf,
                    AUDIO_SAMPLE_RATE,
                    arduboy_core::CLOCK_HZ,
                    AUDIO_VOLUME,
                );
                if let Ok(mut ring) = audio_ring.lock() {
                    // Limit buffer to avoid latency buildup
                    let max_buf = AUDIO_SAMPLE_RATE as usize / 5; // ~200ms
                    if ring.len() < max_buf {
                        ring.extend(pcm_buf.iter());
                    }
                }
                // Timer tones are mixed via frequency fallback in the source
                freq_l.store(0.0f32.to_bits(), Ordering::Relaxed);
                freq_r.store(0.0f32.to_bits(), Ordering::Relaxed);
            } else {
                // No GPIO edges: use timer frequency synthesis
                freq_l.store(lh.to_bits(), Ordering::Relaxed);
                freq_r.store(rh.to_bits(), Ordering::Relaxed);
            }
        }

        // Render
        let pixels = arduboy.framebuffer_u32();
        let cur_scale = scaled_w / SCREEN_WIDTH;
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                let c = pixels[y * SCREEN_WIDTH + x];
                for sy in 0..cur_scale {
                    let base = (y * cur_scale + sy) * scaled_w + x * cur_scale;
                    for sx in 0..cur_scale {
                        if base + sx < scaled_buf.len() { scaled_buf[base + sx] = c; }
                    }
                }
            }
        }
        window.update_with_buffer(&scaled_buf, scaled_w, scaled_h).expect("update");

        if last_fps_time.elapsed() >= Duration::from_secs(2) {
            let fps = fps_frames as f64 / last_fps_time.elapsed().as_secs_f64();
            let (lh, rh) = arduboy.get_audio_tone();
            let mut ti = String::new();
            if lh > 0.0 { ti.push_str(&format!(" L:{:.0}Hz", lh)); }
            if rh > 0.0 { ti.push_str(&format!(" R:{:.0}Hz", rh)); }
            let ms = if muted { " [MUTE]" } else { "" };
            window.set_title(&format!("Arduboy v0.3.0 - {:.0} FPS{}{} ({}x)", fps, ti, ms, cur_scale));
            fps_frames = 0;
            last_fps_time = Instant::now();
        }
    }
    if debug {
        let e = start_time.elapsed().as_secs_f64();
        println!("{} frames in {:.1}s ({:.1} FPS), {} cycles", frame_count, e, frame_count as f64 / e, arduboy.cpu.tick);
    }
}

// ─── Step Mode ──────────────────────────────────────────────────────────────

fn run_step_mode(args: &[String], arduboy: &mut Arduboy) {
    let max_steps: usize = args.iter()
        .position(|a| a == "--frames")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(100_000);

    println!("Step mode: Enter=step, N<enter>=step N, r=run to break, d=dump, q=quit");
    println!("{}", arduboy.dump_regs());
    println!("Next: {}", arduboy.disasm_at_pc());

    let stdin = std::io::stdin();
    let mut steps = 0usize;
    loop {
        let mut line = String::new();
        print!("step> ");
        let _ = std::io::stdout().flush();
        if stdin.read_line(&mut line).is_err() { break; }
        let cmd = line.trim();
        match cmd {
            "q" | "quit" => break,
            "d" | "dump" => { println!("{}", arduboy.dump_regs()); continue; }
            "r" | "run" => {
                for _ in 0..max_steps {
                    if !arduboy.breakpoints.is_empty() && arduboy.breakpoints.contains(&arduboy.cpu.pc) {
                        println!("*** Breakpoint: {} ***", arduboy.disasm_at_pc());
                        break;
                    }
                    arduboy.step_one();
                    steps += 1;
                }
                println!("{}", arduboy.dump_regs());
                println!("Next: {}", arduboy.disasm_at_pc());
                continue;
            }
            _ => {}
        }
        let n: usize = cmd.parse().unwrap_or(1);
        for i in 0..n {
            let asm = arduboy.step_one();
            steps += 1;
            if n <= 20 { println!("  {}", asm); }
            else if i == n - 1 { println!("  ... {} steps, last: {}", n, asm); }
        }
        println!("{}", arduboy.dump_regs());
        println!("Next: {}", arduboy.disasm_at_pc());
    }
    println!("Total: {} steps, {} cycles", steps, arduboy.cpu.tick);
}

// ─── Headless Mode ──────────────────────────────────────────────────────────

fn run_headless(args: &[String], arduboy: &mut Arduboy, serial_enabled: bool) {
    let frames: usize = args.iter()
        .position(|a| a == "--frames")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(60);
    let debug = args.iter().any(|a| a == "--debug");
    let press_frame: Option<usize> = args.iter()
        .position(|a| a == "--press")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok());
    let mut snapshots: Vec<usize> = Vec::new();
    {
        let mut i = 0;
        while i < args.len() {
            if args[i] == "--snapshot" {
                if let Some(f) = args.get(i + 1).and_then(|s| s.parse().ok()) { snapshots.push(f); }
                i += 2;
            } else { i += 1; }
        }
    }
    if debug {
        if let Some(pf) = press_frame { println!("Press A on frame {}", pf); }
        println!("Running {} frames...", frames);
    }
    for frame in 0..frames {
        if let Some(pf) = press_frame {
            if frame == pf { arduboy.set_button(Button::A, true); if debug { println!("  >> A pressed"); } }
            else if frame == pf + 5 { arduboy.set_button(Button::A, false); if debug { println!("  >> A released"); } }
        }
        arduboy.display.dbg_reset_counters();
        arduboy.pcd8544.dbg_reset_counters();
        arduboy.timer0.dbg_reset_counters();
        let t0 = arduboy.cpu.tick;
        let px0 = pixel_count(arduboy);
        arduboy.run_frame();
        let t1 = arduboy.cpu.tick;
        if arduboy.breakpoint_hit {
            println!("*** Break: {} (frame {}) ***\n{}", arduboy.disasm_at_pc(), frame+1, arduboy.dump_regs());
            arduboy.breakpoint_hit = false;
        }
        if serial_enabled {
            let out = arduboy.take_serial_output();
            if !out.is_empty() { let _ = std::io::stderr().write_all(&out); let _ = std::io::stderr().flush(); }
        }
        if debug {
            let lit = pixel_count(arduboy);
            let pxc = lit != px0;
            let sd = arduboy.display.dbg_data_count + arduboy.pcd8544.dbg_data_count;
            let (lh, rh) = arduboy.get_audio_tone();
            let mut ts = String::new();
            if lh > 0.0 { ts.push_str(&format!("  L:{:.0}Hz", lh)); }
            if rh > 0.0 { ts.push_str(&format!("  R:{:.0}Hz", rh)); }
            let show = frame < 15 || (frame < 100 && frame % 10 == 0) || (frame < 1000 && frame % 100 == 0)
                || frame == frames - 1 || pxc || sd > 0 || lh > 0.0 || rh > 0.0
                || press_frame.map_or(false, |pf| frame >= pf && frame < pf + 20);
            if show {
                println!("  Frame {:3}: +{:6} cyc  px={:4}  t0ovf={:3}  t0int={:3}  spi={:4}  [{}] disp={:?}{}{}",
                    frame+1, t1-t0, lit, arduboy.timer0.dbg_ovf_count, arduboy.timer0.dbg_int_fire_count,
                    sd, arduboy.timer0.dbg_info(), arduboy.display_type,
                    if pxc { "  ***PX" } else { "" }, ts);
            }
        }
        if snapshots.contains(&(frame+1)) || (debug && frame == frames-1) {
            println!("\n  === Frame {} ===", frame+1);
            print_display(arduboy);
        }
    }
    if debug { println!("\nDone. {} cycles.", arduboy.cpu.tick); }
}

fn pixel_count(arduboy: &Arduboy) -> usize {
    let fb = arduboy.framebuffer_rgba();
    (0..SCREEN_WIDTH * SCREEN_HEIGHT).filter(|&i| fb[i * 4] > 0).count()
}

fn print_display(arduboy: &Arduboy) {
    let fb = arduboy.framebuffer_rgba();
    let lit = (0..SCREEN_WIDTH * SCREEN_HEIGHT).filter(|&i| fb[i * 4] > 0).count();
    println!("  ({} px lit)", lit);
    for y in (0..SCREEN_HEIGHT).step_by(2) {
        let mut l = String::with_capacity(SCREEN_WIDTH + 4);
        l.push_str("  |");
        for x in 0..SCREEN_WIDTH {
            let t = fb[(y * SCREEN_WIDTH + x) * 4] > 128;
            let b = if y + 1 < SCREEN_HEIGHT { fb[((y+1) * SCREEN_WIDTH + x) * 4] > 128 } else { false };
            l.push(match (t, b) { (true,true)=>'█', (true,false)=>'▀', (false,true)=>'▄', _=>' ' });
        }
        l.push('|');
        println!("{}", l);
    }
}
