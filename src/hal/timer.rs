use super::registers::*;

pub const TICKS_PER_US: u32 = 16;
pub const TICKS_PER_MS: u32 = 16_000;

/// Blocking delays, so drivers can be tested without waiting
pub trait Delay {
    fn delay_us(&mut self, us: u32);
}

/// The system timer as a [`Delay`]
pub struct Timer;

impl Delay for Timer {
    fn delay_us(&mut self, us: u32) {
        delay_us(us);
    }
}

pub fn init() {
    PCR_SYSTIMER_CONFIG.write(1);
    PCR_SYSTIMER_CLOCK.write(bit(22));
    SYSTIMER_CONFIG.write(bit(30) | bit(31));
}

pub fn ticks() -> u64 {
    SYSTIMER_UPDATE0.write(bit(30));
    while SYSTIMER_UPDATE0.read() & bit(29) == 0 {}
    ((SYSTIMER_VALUE0_HIGH.read() as u64) << 32) | SYSTIMER_VALUE0_LOW.read() as u64
}

pub fn delay_us(us: u32) {
    let end = ticks() + us as u64 * TICKS_PER_US as u64;
    while ticks() < end {}
}
