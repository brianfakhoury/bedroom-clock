use crate::drivers::Error;
use crate::drivers::rtc::{self, DateTime};
use crate::hal::i2c::{I2c, NoResponse};

const TIME: DateTime = DateTime { year: 2026, month: 11, day: 1, hour: 16, minute: 0, second: 0 };

#[derive(Default)]
struct Chip {
    registers: [u8; 32],
    fail_time_write: bool,
    restarts: u32,
}

impl I2c for Chip {
    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), NoResponse> {
        assert_eq!(address, 0x51);
        let (&register, values) = bytes.split_first().unwrap();
        if register == 4 && self.fail_time_write {
            return Err(NoResponse);
        }
        if register == 0 && values[0] & 0x20 == 0 {
            self.restarts += 1;
        }
        self.registers[register as usize..][..values.len()].copy_from_slice(values);
        Ok(())
    }

    fn read(&mut self, address: u8, register: &[u8], data: &mut [u8]) -> Result<(), NoResponse> {
        assert_eq!(address, 0x51);
        let &[register] = register else { panic!("expected a one-byte register") };
        data.copy_from_slice(&self.registers[register as usize..][..data.len()]);
        Ok(())
    }
}

#[test]
fn clock_output_disabled() {
    let mut chip = Chip::default();
    chip.registers[1] = 0xb0;
    assert_eq!(rtc::clock_output_disable(&mut chip), Ok(()));
    assert_eq!(chip.registers[1] & 0xb7, 0xb7);
}

#[test]
fn round_trip_and_invalid_registers() {
    let mut chip = Chip::default();
    assert_eq!(rtc::set(&mut chip, &TIME), Ok(()));
    assert_eq!(rtc::read(&mut chip), Ok(TIME));
    assert_eq!(chip.registers[8], 0); // 1 November 2026 is a Sunday
    chip.registers[4] |= 0x80; // Oscillator stopped
    assert_eq!(rtc::read(&mut chip), Err(Error::Invalid));
    chip.registers[4] &= 0x7f;
    chip.registers[0] |= 0x20; // Clock stopped
    assert_eq!(rtc::read(&mut chip), Err(Error::Invalid));
    chip.registers[0] &= !0x20;
    chip.registers[5] = 0x6a; // Not a decimal digit
    assert_eq!(rtc::read(&mut chip), Err(Error::Invalid));
}

#[test]
fn calendar_validation() {
    let mut time = DateTime { year: 2026, month: 2, day: 29, hour: 0, minute: 0, second: 0 };
    assert!(!time.is_valid());
    time.year = 2028;
    assert!(time.is_valid());
    time.second = 60;
    assert!(!time.is_valid());
    time.second = 0;
    time.month = 0;
    assert!(!time.is_valid());
    time.month = 13;
    assert!(!time.is_valid());
    assert_eq!(rtc::set(&mut Chip::default(), &time), Err(Error::Invalid));
}

#[test]
fn month_lengths_and_weekdays() {
    let lengths = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for (month, length) in (1..).zip(lengths) {
        let mut time = DateTime { year: 2026, month, day: length, hour: 0, minute: 0, second: 0 };
        assert!(time.is_valid());
        time.day += 1;
        assert!(!time.is_valid());
    }
    assert_eq!(rtc::weekday(2026, 1, 1), 4); // Thursday
    assert_eq!(rtc::weekday(2028, 2, 29), 2); // Tuesday
    assert_eq!(rtc::weekday(2026, 12, 31), 4);
}

#[test]
fn failed_write_restarts_clock() {
    let mut chip = Chip { fail_time_write: true, ..Chip::default() };
    assert_eq!(rtc::set(&mut chip, &TIME), Err(Error::Bus));
    assert_eq!(chip.restarts, 1);
    assert_eq!(chip.registers[0] & 0x20, 0);
}
