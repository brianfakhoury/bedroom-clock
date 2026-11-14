use super::pins::{BUTTON_BOOT, BUTTON_KEY, TOUCH_IRQ};
use crate::hal::interrupts::{self, Pending};

/// What happened while waiting
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Events {
    pub boot: bool,
    pub key: bool,
    pub touch: bool,
    pub usb: bool,
    pub timer: bool,
}

/// The pins whose falling edges are events
pub const PINS: u32 = BUTTON_BOOT.mask() | BUTTON_KEY.mask() | TOUCH_IRQ.mask();

pub fn init() {
    interrupts::init(PINS);
}

pub fn wait_until(deadline: Option<u64>) -> Events {
    interrupts::wait_until(PINS, deadline).into()
}

impl From<Pending> for Events {
    fn from(pending: Pending) -> Self {
        Events {
            boot: pending.gpio & BUTTON_BOOT.mask() != 0,
            key: pending.gpio & BUTTON_KEY.mask() != 0,
            touch: pending.gpio & TOUCH_IRQ.mask() != 0,
            usb: pending.usb,
            timer: pending.timer,
        }
    }
}
