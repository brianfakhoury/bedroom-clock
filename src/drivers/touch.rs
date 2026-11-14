//! CST9220 touch controller (CST9217-compatible reports), 16-bit register
//! addresses. The report layout follows Waveshare's driver; the default scan
//! behavior is retained.

use super::Error;
use crate::hal::i2c::I2c;

const ADDRESS: u8 = 0x5a;
const REPORT: [u8; 2] = [0xd0, 0x00];
const REPORT_SIGNATURE: u8 = 0xab;

/// Check communication after the board resets the controller
pub fn probe(bus: &mut impl I2c) -> Result<(), Error> {
    Ok(bus.read(ADDRESS, &REPORT, &mut [0; 10])?)
}

/// Whether a finger is down
pub fn read(bus: &mut impl I2c) -> Result<bool, Error> {
    let mut report = [0; 10];
    bus.read(ADDRESS, &REPORT, &mut report)?;
    if report[6] != REPORT_SIGNATURE {
        return Err(Error::Invalid);
    }
    Ok(report[5] & 0x7f != 0 && report[0] & 15 == 6)
}
