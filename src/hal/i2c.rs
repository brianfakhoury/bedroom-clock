use super::gpio::{Level, Mode, Pin, Pull};
use super::registers::*;
use super::timer::delay_us;

/// The device did not acknowledge, or held the clock low for too long
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct NoResponse;

/// An I²C controller. Device addresses are 7-bit
pub trait I2c {
    /// Write bytes, normally a register address followed by its values
    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), NoResponse>;
    /// Write a register address of one or two bytes, then read from it
    fn read(&mut self, address: u8, register: &[u8], data: &mut [u8]) -> Result<(), NoResponse>;
}

/// Software I²C; the board supplies the pins
pub struct SoftI2c {
    scl: Pin,
    sda: Pin,
}

impl SoftI2c {
    pub const fn new(scl: Pin, sda: Pin) -> Self {
        Self { scl, sda }
    }

    pub fn init(&self) {
        self.scl.configure(Mode::Input, Pull::Up);
        self.sda.configure(Mode::Input, Pull::Up);
        self.scl.set(Level::Low);
        self.sda.set(Level::Low);
        // Finish any byte interrupted by a CPU reset, then release the bus
        for _ in 0..9 {
            self.clock_low();
            let _ = self.clock_high();
        }
        self.stop();
    }

    // Open drain: drive low or release the pin so the pull-up raises it
    fn drive(pin: Pin, high: bool) {
        let enable = if high { GPIO_ENABLE_CLEAR } else { GPIO_ENABLE_SET };
        enable.write(pin.mask());
    }

    fn data(&self, high: bool) {
        Self::drive(self.sda, high);
    }

    fn clock_low(&self) {
        Self::drive(self.scl, false);
        delay_us(3);
    }

    fn clock_high(&self) -> Result<(), NoResponse> {
        Self::drive(self.scl, true);
        // Allow up to 1 ms of clock stretching
        let mut waited_us = 0;
        while !self.scl.is_high() {
            if waited_us == 1000 {
                return Err(NoResponse);
            }
            delay_us(1);
            waited_us += 1;
        }
        delay_us(3);
        Ok(())
    }

    fn start(&self) -> Result<(), NoResponse> {
        self.data(true);
        self.clock_high()?;
        self.data(false);
        delay_us(3);
        self.clock_low();
        Ok(())
    }

    fn stop(&self) {
        self.data(false);
        let _ = self.clock_high();
        self.data(true);
        delay_us(3);
    }

    fn write_byte(&self, mut byte: u8) -> Result<(), NoResponse> {
        for _ in 0..8 {
            self.data(byte & 0x80 != 0);
            self.clock_high()?;
            self.clock_low();
            byte <<= 1;
        }
        self.data(true);
        self.clock_high()?;
        let acknowledged = !self.sda.is_high();
        self.clock_low();
        if acknowledged { Ok(()) } else { Err(NoResponse) }
    }

    fn read_byte(&self, last: bool) -> Result<u8, NoResponse> {
        let mut byte = 0;
        self.data(true);
        for _ in 0..8 {
            self.clock_high()?;
            byte = (byte << 1) | self.sda.is_high() as u8;
            self.clock_low();
        }
        // Acknowledge every byte except the last
        self.data(last);
        self.clock_high()?;
        self.clock_low();
        self.data(true);
        Ok(byte)
    }

    fn send(&self, address: u8, bytes: &[u8]) -> Result<(), NoResponse> {
        self.start()?;
        self.write_byte(address << 1)?;
        bytes.iter().try_for_each(|&byte| self.write_byte(byte))
    }

    fn receive(&self, address: u8, data: &mut [u8]) -> Result<(), NoResponse> {
        self.start()?;
        self.write_byte((address << 1) | 1)?;
        let last = data.len().saturating_sub(1);
        for (i, byte) in data.iter_mut().enumerate() {
            *byte = self.read_byte(i == last)?;
        }
        Ok(())
    }
}

impl I2c for SoftI2c {
    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), NoResponse> {
        let sent = self.send(address, bytes);
        self.stop();
        sent
    }

    fn read(&mut self, address: u8, register: &[u8], data: &mut [u8]) -> Result<(), NoResponse> {
        let received = self.send(address, register).and_then(|()| self.receive(address, data));
        self.stop();
        received
    }
}
