use super::registers::*;

/// A GPIO, by number
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pin(pub u8);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Input,
    Output,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Pull {
    None,
    Up,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Low,
    High,
}

impl Pin {
    pub const fn mask(self) -> u32 {
        bit(self.0 as u32)
    }

    pub fn configure(self, mode: Mode, pull: Pull) {
        let pullup = if pull == Pull::Up { bit(8) } else { 0 };
        IOMUX_GPIO0.offset(self.0 as u32 * 4).write(bit(12) | bit(9) | pullup | (2 << 10));
        GPIO_OUT_SELECT0.offset(self.0 as u32 * 4).write(128 | bit(9));
        let enable = if mode == Mode::Output { GPIO_ENABLE_SET } else { GPIO_ENABLE_CLEAR };
        enable.write(self.mask());
    }

    pub fn set(self, level: Level) {
        let output = if level == Level::High { GPIO_OUT_SET } else { GPIO_OUT_CLEAR };
        output.write(self.mask());
    }

    pub fn is_high(self) -> bool {
        GPIO_INPUT.read() & self.mask() != 0
    }
}

/// Release retained output levels after restoring GPIO routing
pub fn release_holds() {
    LP_GPIO_HOLD.write(0);
}
