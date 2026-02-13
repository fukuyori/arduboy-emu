//! 16-bit Timer/Counter1 and Timer/Counter3 emulation.
//!
//! Supports CTC (Clear Timer on Compare) mode for audio tone generation.
//! Timer3 with COM3A0 (toggle OC3A on compare match) is the standard Arduboy
//! `tone()` mechanism. Timer1 can also generate tones. The [`Timer16::get_tone_hz`]
//! method derives the output frequency from register settings.
//!
//! Handles overflow and compare-match A/B interrupts.

/// Memory-mapped register addresses for a 16-bit timer instance.
#[derive(Debug, Clone)]
pub struct Timer16Addrs {
    pub tifr: u16,
    pub tccr_a: u16,
    pub tccr_b: u16,
    pub tccr_c: u16,
    pub ocr_ah: u16,
    pub ocr_al: u16,
    pub ocr_bh: u16,
    pub ocr_bl: u16,
    pub ocr_ch: u16,
    pub ocr_cl: u16,
    pub timsk: u16,
    pub tcnth: u16,
    pub tcntl: u16,
}

pub struct Timer16 {
    addrs: Timer16Addrs,
    tick: u64,
    prescale: u32,
    tcnt: u16,
    top: u16,
    ctc: bool,
    // WGM bits
    wgm: [bool; 4],
    // Clock select
    cs: u8,
    // Compare Output Mode bits (COM3A1:COM3A0, etc.)
    com_a: u8, // 0=off, 1=toggle, 2=clear, 3=set
    com_b: u8,
    com_c: u8,
    // Compare registers
    ocr_a: u16,
    ocr_b: u16,
    ocr_c: u16,
    // Force output compare
    foc_a: bool,
    foc_b: bool,
    foc_c: bool,
    // Interrupt flags (counts of pending)
    tov: u32,
    ocf_a: u32,
    ocf_b: u32,
    ocf_c: u32,
    // Interrupt enables
    toie: bool,
    ocie_a: bool,
    ocie_b: bool,
    ocie_c: bool,
    // Interrupt vector addresses (set based on which timer instance)
    int_ov: u16,
    int_compa: u16,
    int_compb: u16,
    int_compc: u16,
    old_wgm: u8,
}

impl Timer16 {
    pub fn new(addrs: Timer16Addrs) -> Self {
        // Determine which timer based on addresses
        let (int_ov, int_compa, int_compb, int_compc) = if addrs.tccr_a == 0x80 {
            // Timer1
            (super::INT_TIMER1_OVF, super::INT_TIMER1_COMPA, super::INT_TIMER1_COMPB, super::INT_TIMER1_COMPC)
        } else {
            // Timer3
            (super::INT_TIMER3_OVF, super::INT_TIMER3_COMPA, super::INT_TIMER3_COMPB, super::INT_TIMER3_COMPC)
        };
        Timer16 {
            addrs,
            tick: 0,
            prescale: 0,
            tcnt: 0,
            top: 0xFFFF,
            ctc: false,
            wgm: [false; 4],
            cs: 0,
            com_a: 0, com_b: 0, com_c: 0,
            ocr_a: 0, ocr_b: 0, ocr_c: 0,
            foc_a: false, foc_b: false, foc_c: false,
            tov: 0, ocf_a: 0, ocf_b: 0, ocf_c: 0,
            toie: false, ocie_a: false, ocie_b: false, ocie_c: false,
            int_ov, int_compa, int_compb, int_compc,
            old_wgm: 0xFF,
        }
    }

    pub fn reset(&mut self) {
        let addrs = self.addrs.clone();
        let int_ov = self.int_ov;
        let int_compa = self.int_compa;
        let int_compb = self.int_compb;
        let int_compc = self.int_compc;
        *self = Timer16::new(addrs);
        self.int_ov = int_ov;
        self.int_compa = int_compa;
        self.int_compb = int_compb;
        self.int_compc = int_compc;
    }

    fn update_state(&mut self) {
        let wgm = ((self.wgm[3] as u8) << 3) | ((self.wgm[2] as u8) << 2)
            | ((self.wgm[1] as u8) << 1) | (self.wgm[0] as u8);
        let cs = self.cs;

        if wgm != self.old_wgm {
            self.ctc = false;
            self.top = 0xFFFF;
            match wgm {
                0 => {} // Normal
                1 => self.top = 0xFF,     // PWM PC 8-bit
                2 => self.top = 0x1FF,    // PWM PC 9-bit
                3 => self.top = 0x3FF,    // PWM PC 10-bit
                4 | 12 => self.ctc = true, // CTC
                5 => self.top = 0xFF,     // Fast PWM 8-bit
                6 => self.top = 0x1FF,    // Fast PWM 9-bit
                7 => self.top = 0x3FF,    // Fast PWM 10-bit
                _ => {}
            }
            self.old_wgm = wgm;
        }

        self.prescale = match cs {
            0 => 0,
            1 => 1,
            2 => 8,
            3 => 64,
            4 => 256,
            5 => 1024,
            _ => 1,
        };
    }

    pub fn write(&mut self, addr: u16, value: u8, _old: u8, data: &mut [u8]) -> bool {
        if addr == self.addrs.tifr {
            // Writing 1 to a TIFR bit CLEARS the interrupt flag
            if value & 1 != 0 { self.tov = 0; }
            if value & 2 != 0 { self.ocf_a = 0; }
            if value & 4 != 0 { self.ocf_b = 0; }
            if value & 8 != 0 { self.ocf_c = 0; }
            return true;
        }
        if addr == self.addrs.tccr_a {
            self.com_a = (value >> 6) & 3;
            self.com_b = (value >> 4) & 3;
            self.com_c = (value >> 2) & 3;
            self.wgm[0] = value & 1 != 0;
            self.wgm[1] = value & 2 != 0;
            self.update_state();
            data[addr as usize] = value;
            return true;
        }
        if addr == self.addrs.tccr_b {
            self.wgm[2] = value & 8 != 0;
            self.wgm[3] = value & 0x10 != 0;
            self.cs = value & 7;
            self.update_state();
            data[addr as usize] = value;
            return true;
        }
        if addr == self.addrs.tccr_c {
            self.foc_a = value & 0x80 != 0;
            self.foc_b = value & 0x40 != 0;
            self.foc_c = value & 0x20 != 0;
            data[addr as usize] = value;
            return true;
        }
        if addr == self.addrs.ocr_ah {
            self.ocr_a = (self.ocr_a & 0xFF) | ((value as u16) << 8);
            data[addr as usize] = value;
            return true;
        }
        if addr == self.addrs.ocr_al {
            self.ocr_a = (self.ocr_a & 0xFF00) | value as u16;
            data[addr as usize] = value;
            return true;
        }
        if addr == self.addrs.ocr_bh {
            self.ocr_b = (self.ocr_b & 0xFF) | ((value as u16) << 8);
            data[addr as usize] = value;
            return true;
        }
        if addr == self.addrs.ocr_bl {
            self.ocr_b = (self.ocr_b & 0xFF00) | value as u16;
            data[addr as usize] = value;
            return true;
        }
        if addr == self.addrs.ocr_ch {
            self.ocr_c = (self.ocr_c & 0xFF) | ((value as u16) << 8);
            data[addr as usize] = value;
            return true;
        }
        if addr == self.addrs.ocr_cl {
            self.ocr_c = (self.ocr_c & 0xFF00) | value as u16;
            data[addr as usize] = value;
            return true;
        }
        if addr == self.addrs.tcnth {
            self.tcnt = (self.tcnt & 0xFF) | ((value as u16) << 8);
            data[addr as usize] = value;
            return true;
        }
        if addr == self.addrs.tcntl {
            self.tcnt = (self.tcnt & 0xFF00) | value as u16;
            data[addr as usize] = value;
            return true;
        }
        if addr == self.addrs.timsk {
            self.toie = value & 1 != 0;
            self.ocie_a = value & 2 != 0;
            self.ocie_b = value & 4 != 0;
            self.ocie_c = value & 8 != 0;
            data[addr as usize] = value;
            return true;
        }
        false
    }

    pub fn read(&mut self, addr: u16, tick: u64, _data: &[u8]) -> Option<u8> {
        if addr == self.addrs.tifr {
            return Some(
                ((self.tov.min(1)) as u8)
                | (((self.ocf_a.min(1)) as u8) << 1)
                | (((self.ocf_b.min(1)) as u8) << 2)
            );
        }
        if addr == self.addrs.tccr_c {
            return Some(0);
        }
        if addr == self.addrs.tcnth {
            self.do_update(tick);
            return Some((self.tcnt >> 8) as u8);
        }
        if addr == self.addrs.tcntl {
            self.do_update(tick);
            return Some((self.tcnt & 0xFF) as u8);
        }
        None
    }

    fn do_update(&mut self, tick: u64) {
        if self.prescale == 0 { return; }

        let ticks_since = tick.wrapping_sub(self.tick);
        let interval = (ticks_since / self.prescale as u64) as u32;
        if interval == 0 { return; }

        let old_tcnt = self.tcnt;
        let cnt = (self.tcnt as u32).wrapping_add(interval);
        self.tcnt = (cnt & 0xFFFF) as u16;
        self.tick += (interval as u64) * (self.prescale as u64);

        if self.ocie_a && old_tcnt < self.ocr_a && cnt as u16 >= self.ocr_a {
            self.ocf_a += 1;
            if self.ctc && !self.foc_a {
                self.tcnt = 0;
            }
        }
        if self.ocie_b && old_tcnt < self.ocr_b && cnt as u16 >= self.ocr_b {
            self.ocf_b += 1;
        }
        if self.ocie_c && old_tcnt < self.ocr_c && cnt as u16 >= self.ocr_c {
            self.ocf_c += 1;
        }

        self.tov += (cnt / self.top as u32).min(1);
    }

    pub fn update(&mut self, tick: u64, data: &mut [u8]) {
        self.do_update(tick);
        data[self.addrs.tcnth as usize] = (self.tcnt >> 8) as u8;
        data[self.addrs.tcntl as usize] = (self.tcnt & 0xFF) as u8;
    }

    pub fn check_interrupt(&mut self) -> Option<u16> {
        if self.ocf_a > 0 && self.ocie_a && !self.foc_a {
            self.ocf_a = 0;
            return Some(self.int_compa);
        }
        if self.ocf_b > 0 && self.ocie_b && !self.foc_b {
            self.ocf_b = 0;
            return Some(self.int_compb);
        }
        if self.ocf_c > 0 && self.ocie_c && !self.foc_c {
            self.ocf_c = 0;
            return Some(self.int_compc);
        }
        if self.tov > 0 && self.toie {
            self.tov = 0;
            return Some(self.int_ov);
        }
        None
    }

    /// Get tone frequency in Hz from CTC toggle mode.
    /// Returns 0.0 if timer is not generating a tone.
    /// Arduboy: Timer3 OC3A=PC6, Timer1 OC1A/OC1C for speaker pins.
    pub fn get_tone_hz(&self, clock: u32) -> f32 {
        // CTC mode (WGM=4 or 12) with COM_A=1 (toggle on match)
        if self.prescale == 0 || self.com_a != 1 {
            return 0.0;
        }
        let wgm = ((self.wgm[3] as u8) << 3) | ((self.wgm[2] as u8) << 2)
            | ((self.wgm[1] as u8) << 1) | (self.wgm[0] as u8);
        // CTC modes: 4 (OCRnA), 12 (ICRn - not commonly used for tone)
        if wgm != 4 && wgm != 12 {
            return 0.0;
        }
        if self.ocr_a == 0 {
            return 0.0;
        }
        clock as f32 / (2.0 * self.prescale as f32 * (self.ocr_a as f32 + 1.0))
    }
}
