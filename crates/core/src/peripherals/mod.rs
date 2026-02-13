//! ATmega32u4 peripheral emulation.
//!
//! Contains hardware peripherals needed to run Arduboy and Gamebuino games:
//!
//! - [`Timer8`] — 8-bit Timer/Counter0 (frame rate timing, millis())
//! - [`Timer16`] — 16-bit Timer/Counter1 and Timer/Counter3 (audio tone generation)
//! - [`Timer4`] — 10-bit high-speed Timer/Counter4 (PWM audio, LED control)
//! - [`Spi`] — SPI master controller (display and FX flash communication)
//! - [`Adc`] — Analog-to-digital converter (random seed, battery sensing)
//! - [`Pll`] — PLL frequency synthesizer (USB clock, fast PWM)
//! - [`EepromCtrl`] — EEPROM read/write controller (save data)
//! - [`FxFlash`] — W25Q128 16 MB external SPI flash (Arduboy FX game data)

mod timer8;
mod timer16;
mod timer4;
mod spi;
mod eeprom;
mod adc;
mod pll;
pub mod fx_flash;

pub use timer8::{Timer8, Timer8Addrs};
pub use timer16::{Timer16, Timer16Addrs};
pub use timer4::Timer4;
pub use spi::Spi;
pub use eeprom::EepromCtrl;
pub use adc::Adc;
pub use pll::Pll;
pub use fx_flash::FxFlash;

// ATmega32u4 interrupt vector addresses (word addresses from datasheet)
// These are already word addresses - do NOT divide by 2
pub const INT_TIMER0_COMPA: u16 = 0x002A;
pub const INT_TIMER0_COMPB: u16 = 0x002C;
pub const INT_TIMER0_OVF: u16 = 0x002E;
pub const INT_TIMER1_COMPA: u16 = 0x0022;
pub const INT_TIMER1_COMPB: u16 = 0x0024;
pub const INT_TIMER1_COMPC: u16 = 0x0026;
pub const INT_TIMER1_OVF: u16 = 0x0028;
pub const INT_TIMER3_COMPA: u16 = 0x0040;
pub const INT_TIMER3_COMPB: u16 = 0x0042;
pub const INT_TIMER3_COMPC: u16 = 0x0044;
pub const INT_TIMER3_OVF: u16 = 0x0046;
pub const INT_SPI: u16 = 0x0030;
pub const INT_ADC: u16 = 0x003A;
