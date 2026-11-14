//! External chip commands: how to read or control each chip, independent of
//! board wiring and clock behavior.

pub mod accel;
pub mod audio;
pub mod display;
pub mod pmic;
pub mod rtc;
pub mod touch;

use crate::hal::i2c::NoResponse;

/// Why a chip operation failed
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Error {
    /// The chip did not respond
    Bus,
    /// A request or a reading was out of range
    Invalid,
    /// The chip acknowledged a setting that did not read back
    Unverified,
}

impl From<NoResponse> for Error {
    fn from(_: NoResponse) -> Self {
        Error::Bus
    }
}
