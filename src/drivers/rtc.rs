//! PCF85063A real-time clock, which keeps UTC.

use super::Error;
use crate::hal::i2c::I2c;

const ADDRESS: u8 = 0x51;
const CONTROL_1: u8 = 0x00;
const CONTROL_2: u8 = 0x01;
const SECONDS: u8 = 0x04;
const STOP: u8 = 0x20;
const HOUR_12: u8 = 0x02;
const OSCILLATOR_STOPPED: u8 = 0x80;

/// Gregorian date and time, years 2020–2099
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DateTime {
    pub year: u32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

impl DateTime {
    pub fn is_valid(&self) -> bool {
        (2020..=2099).contains(&self.year)
            && (1..=12).contains(&self.month)
            && (1..=days_in_month(self.year, self.month)).contains(&self.day)
            && self.hour < 24
            && self.minute < 60
            && self.second < 60
    }
}

// Every fourth year is a leap year from 2020 to 2099
fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        2 => 28 + year.is_multiple_of(4) as u32,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Day of the week for a valid date; Sunday = 0
pub fn weekday(mut year: u32, month: u32, day: u32) -> u32 {
    const OFFSETS: [u8; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    if month < 3 {
        year -= 1;
    }
    let offset = OFFSETS[(month as usize + 11) % 12] as u32;
    (year + year / 4 - year / 100 + year / 400 + offset + day) % 7
}

fn to_bcd(value: u32) -> u8 {
    ((value / 10) * 16 + value % 10) as u8
}

fn from_bcd(value: u8) -> u32 {
    (value >> 4) as u32 * 10 + (value & 15) as u32
}

fn is_bcd(value: u8) -> bool {
    value & 15 < 10 && value >> 4 < 10
}

fn read_register(bus: &mut impl I2c, register: u8) -> Result<u8, Error> {
    let mut value = 0;
    bus.read(ADDRESS, &[register], core::slice::from_mut(&mut value))?;
    Ok(value)
}

/// Stop driving the unused CLKOUT pin
#[cfg_attr(target_os = "none", unsafe(link_section = ".init.code"))]
#[inline(never)]
pub fn clock_output_disable(bus: &mut impl I2c) -> Result<(), Error> {
    // COF=111 disables CLKOUT. Write ones to AF/TF to preserve their flags,
    // including an event arriving between the read and write
    let control = read_register(bus, CONTROL_2)? | 0x4f;
    bus.write(ADDRESS, &[CONTROL_2, control])?;
    if read_register(bus, CONTROL_2)? & 7 == 7 { Ok(()) } else { Err(Error::Unverified) }
}

pub fn read(bus: &mut impl I2c) -> Result<DateTime, Error> {
    if read_register(bus, CONTROL_1)? & (STOP | HOUR_12) != 0 {
        return Err(Error::Invalid);
    }
    let mut data = [0; 7];
    bus.read(ADDRESS, &[SECONDS], &mut data)?;
    let [seconds, minutes, hours, days, _weekday, months, years] = data;
    if seconds & OSCILLATOR_STOPPED != 0 {
        return Err(Error::Invalid);
    }
    let fields = [years, months & 31, days & 63, hours & 63, minutes & 127, seconds & 127];
    if !fields.into_iter().all(is_bcd) {
        return Err(Error::Invalid);
    }
    let [year, month, day, hour, minute, second] = fields.map(from_bcd);
    let time = DateTime { year: 2000 + year, month, day, hour, minute, second };
    if time.is_valid() { Ok(time) } else { Err(Error::Invalid) }
}

pub fn set(bus: &mut impl I2c, time: &DateTime) -> Result<(), Error> {
    if !time.is_valid() {
        return Err(Error::Invalid);
    }
    let control = read_register(bus, CONTROL_1)? & !HOUR_12;
    bus.write(ADDRESS, &[CONTROL_1, control | STOP])?;
    let written = bus.write(
        ADDRESS,
        &[
            SECONDS,
            to_bcd(time.second),
            to_bcd(time.minute),
            to_bcd(time.hour),
            to_bcd(time.day),
            weekday(time.year, time.month, time.day) as u8,
            to_bcd(time.month),
            to_bcd(time.year - 2000),
        ],
    );
    // Restart even if writing the time failed
    let running = bus.write(ADDRESS, &[CONTROL_1, control & !STOP]);
    Ok(written.and(running)?)
}
