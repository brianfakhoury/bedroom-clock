use crate::drivers::audio;
use crate::hal::i2c::{I2c, NoResponse};

const CODEC: u8 = 0x18;
const ADC: u8 = 0x40;

struct Chips {
    registers: [[u8; 256]; 128],
    writes: u32,
    /// Transient failures of one ADC register
    adc_failures: u32,
    adc_stuck: bool,
    codec_absent: bool,
}

impl Chips {
    fn new() -> Self {
        Self {
            registers: [[0; 256]; 128],
            writes: 0,
            adc_failures: 0,
            adc_stuck: false,
            codec_absent: false,
        }
    }
}

impl I2c for Chips {
    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), NoResponse> {
        let &[register, value] = bytes else { panic!("expected one register value") };
        self.writes += 1;
        let fragile = address == ADC && register == 0x4b;
        if fragile && self.adc_failures > 0 {
            self.adc_failures -= 1;
            return Err(NoResponse);
        }
        if address == CODEC && self.codec_absent {
            return Err(NoResponse);
        }
        if !(fragile && self.adc_stuck) {
            self.registers[address as usize][register as usize] = value;
        }
        Ok(())
    }

    fn read(&mut self, address: u8, register: &[u8], data: &mut [u8]) -> Result<(), NoResponse> {
        let (&[register], [value]) = (register, data) else { panic!("expected one register") };
        if address == CODEC && self.codec_absent {
            return Err(NoResponse);
        }
        *value = self.registers[address as usize][register as usize];
        Ok(())
    }
}

#[test]
fn shutdown() {
    let mut chips = Chips::new();
    assert!(audio::disable(&mut chips).is_ok());
    assert_eq!(chips.registers[ADC as usize][0x4b], 0xff);
    assert_eq!(chips.registers[CODEC as usize][0x0d], 0xfc);
}

#[test]
fn transient_failure_retries() {
    let mut chips = Chips::new();
    chips.adc_failures = 1;
    assert!(audio::disable(&mut chips).is_ok());
    assert_eq!(chips.adc_failures, 0);
}

#[test]
fn stuck_register_does_not_block_other_chip() {
    let mut chips = Chips::new();
    chips.adc_stuck = true;
    assert!(audio::disable(&mut chips).is_err());
    assert_eq!(chips.writes, 15 + 3 * 9);
    assert_eq!(chips.registers[CODEC as usize][1], 0);
}

#[test]
fn permanent_failure_is_bounded_and_recovers() {
    let mut chips = Chips::new();
    chips.codec_absent = true;
    assert!(audio::disable(&mut chips).is_err());
    assert_eq!(chips.writes, 3 * 15 + 9);
    assert_eq!(chips.registers[ADC as usize][0x4b], 0xff);
    chips.codec_absent = false;
    assert!(audio::disable(&mut chips).is_ok());
}
