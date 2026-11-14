use super::gpio::{Level, Mode, Pin, Pull};
use super::registers::*;
use super::timer::delay_us;

/// One chip on a quad SPI bus
pub trait Qspi {
    fn begin(&mut self);
    fn end(&mut self);
    /// MSB first on data0
    fn write_byte(&mut self, value: u8);
    /// Four data lines, highest nibble first
    fn write_quad16(&mut self, value: u16);
}

/// Software QSPI; the board supplies the pins.
/// Four contiguous data GPIOs start at `data0`; clock and CS are separate
pub struct SoftQspi {
    clock: Pin,
    data0: Pin,
    chip_select: Pin,
    clock_mask: u32,
    data0_mask: u32,
    bus_mask: u32,
}

impl SoftQspi {
    pub const fn new(clock: Pin, data0: Pin, chip_select: Pin) -> Self {
        Self {
            clock,
            data0,
            chip_select,
            clock_mask: clock.mask(),
            data0_mask: data0.mask(),
            bus_mask: clock.mask() | (0x0f << data0.0),
        }
    }

    #[cfg_attr(target_os = "none", unsafe(link_section = ".init.code"))]
    #[inline(never)]
    pub fn init(&self) {
        self.clock.configure(Mode::Output, Pull::None);
        self.clock.set(Level::Low);
        for pin in self.data0.0..self.data0.0 + 4 {
            Pin(pin).configure(Mode::Output, Pull::None);
            Pin(pin).set(Level::Low);
        }
        self.chip_select.configure(Mode::Output, Pull::None);
        self.chip_select.set(Level::High);
    }
}

impl Qspi for SoftQspi {
    fn begin(&mut self) {
        self.chip_select.set(Level::Low);
    }

    fn end(&mut self) {
        GPIO_OUT_CLEAR.write(self.clock_mask);
        self.chip_select.set(Level::High);
    }

    fn write_byte(&mut self, mut value: u8) {
        for _ in 0..8 {
            GPIO_OUT_CLEAR.write(self.clock_mask | self.data0_mask);
            if value & 0x80 != 0 {
                GPIO_OUT_SET.write(self.data0_mask);
            }
            delay_us(1);
            GPIO_OUT_SET.write(self.clock_mask);
            delay_us(1);
            value <<= 1;
        }
        GPIO_OUT_CLEAR.write(self.clock_mask);
    }

    fn write_quad16(&mut self, value: u16) {
        for shift in [12, 8, 4, 0] {
            GPIO_OUT_CLEAR.write(self.bus_mask);
            GPIO_OUT_SET.write(((value as u32 >> shift) & 15) << self.data0.0);
            GPIO_OUT_SET.write(self.clock_mask);
        }
    }
}
