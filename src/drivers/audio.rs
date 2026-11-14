//! ES8311 codec and ES7210 microphone ADC, which this clock only shuts down.
//! Startup-only: this code and its tables are lost during deep sleep.

use super::Error;
use crate::hal::i2c::I2c;

const CODEC_ADDRESS: u8 = 0x18;
const ADC_ADDRESS: u8 = 0x40;

// Power-down register sequences from Espressif's ES8311 suspend and ES7210
// stop drivers bundled with Waveshare's 2.16 Audio_Test example

/// Register, value
#[cfg_attr(target_os = "none", unsafe(link_section = ".init.data"))]
static CODEC_SEQUENCE: [[u8; 2]; 15] = [
    [0x32, 0],
    [0x17, 0],
    [0x0e, 0xff],
    [0x12, 2],
    [0x14, 0],
    [0x0d, 0xfa],
    [0x15, 0],
    [0x02, 0x10],
    [0x00, 0],
    [0x00, 0x1f],
    [0x01, 0x30],
    [0x01, 0],
    [0x45, 0],
    [0x0d, 0xfc],
    [0x02, 0],
];

#[cfg_attr(target_os = "none", unsafe(link_section = ".init.data"))]
static ADC_SEQUENCE: [[u8; 2]; 9] = [
    [0x47, 0xff],
    [0x48, 0xff],
    [0x49, 0xff],
    [0x4a, 0xff],
    [0x4b, 0xff],
    [0x4c, 0xff],
    [0x40, 0xc0],
    [0x01, 0x7f],
    [0x06, 7],
];

// Read only stable power/clock bits; reserved and reset bits are excluded

/// Register, mask, expected value
#[cfg_attr(target_os = "none", unsafe(link_section = ".init.data"))]
static CODEC_CHECKS: [[u8; 3]; 4] =
    [[0x0d, 0xfb, 0xf8], [0x0e, 0x60, 0x60], [0x12, 2, 2], [0x01, 0x3f, 0]];

#[cfg_attr(target_os = "none", unsafe(link_section = ".init.data"))]
static ADC_CHECKS: [[u8; 3]; 5] =
    [[0x4b, 0xfc, 0xfc], [0x4c, 0xfc, 0xfc], [0x40, 0xc3, 0xc0], [0x01, 0x7f, 0x7f], [0x06, 7, 7]];

/// Power down both chips, retrying each independently
#[cfg_attr(target_os = "none", unsafe(link_section = ".init.code"))]
#[inline(never)]
pub fn disable(bus: &mut impl I2c) -> Result<(), Error> {
    let mut codec = Err(Error::Bus);
    let mut adc = Err(Error::Bus);
    for _ in 0..3 {
        if codec.is_err() {
            codec = power_down(bus, CODEC_ADDRESS, &CODEC_SEQUENCE, &CODEC_CHECKS);
        }
        if adc.is_err() {
            adc = power_down(bus, ADC_ADDRESS, &ADC_SEQUENCE, &ADC_CHECKS);
        }
        if codec.and(adc).is_ok() {
            break;
        }
    }
    codec.and(adc)
}

// Attempt every write and every check, even after a failure
#[cfg_attr(target_os = "none", unsafe(link_section = ".init.code"))]
#[inline(never)]
fn power_down(
    bus: &mut impl I2c,
    address: u8,
    sequence: &[[u8; 2]],
    checks: &[[u8; 3]],
) -> Result<(), Error> {
    let mut outcome = Ok(());
    for write in sequence {
        if bus.write(address, write).is_err() {
            outcome = Err(Error::Bus);
        }
    }
    for &[register, mask, expected] in checks {
        let mut value = 0;
        if bus.read(address, &[register], core::slice::from_mut(&mut value)).is_err() {
            outcome = Err(Error::Bus);
        } else if value & mask != expected {
            outcome = outcome.and(Err(Error::Unverified));
        }
    }
    outcome
}
