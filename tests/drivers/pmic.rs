use crate::drivers::Error;
use crate::drivers::pmic::{self, ChargeProfile};
use crate::hal::i2c::{I2c, NoResponse};

pub struct Chip {
    pub registers: [u8; 256],
    pub writes: u32,
    pub operations: u32,
    /// The operation, counted from one, that gets no response
    pub fail_at: u32,
    corrupt_readback: bool,
}

impl Chip {
    /// A battery at 87% with the gauge and charging enabled
    pub fn new() -> Self {
        let mut registers = [0; 256];
        for (register, value) in [
            (0x00, 0x08),
            (0x18, 0x0a),
            (0x30, 0x03),
            (0x61, 0x05),
            (0x62, 0xe8),
            (0x63, 0xf5),
            (0x64, 0xa3),
            (0x65, 0x02),
            (0x67, 0xe6),
            (0xa4, 87),
        ] {
            registers[register] = value;
        }
        Self { registers, writes: 0, operations: 0, fail_at: 0, corrupt_readback: false }
    }

    fn respond(&mut self) -> Result<(), NoResponse> {
        self.operations += 1;
        if self.operations == self.fail_at { Err(NoResponse) } else { Ok(()) }
    }
}

impl I2c for Chip {
    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), NoResponse> {
        let &[register, value] = bytes else { panic!("expected one register value") };
        assert_eq!(address, 0x34);
        self.writes += 1;
        self.respond()?;
        match register {
            // Profile writes must only occur after charging is disabled
            0x62..=0x64 => assert_eq!(self.registers[0x18] & 2, 0),
            // Interrupt status bits clear when written with ones
            0x48..=0x4a => {
                self.registers[register as usize] &= !value;
                return Ok(());
            }
            _ => {}
        }
        self.registers[register as usize] = value;
        Ok(())
    }

    fn read(&mut self, address: u8, register: &[u8], data: &mut [u8]) -> Result<(), NoResponse> {
        let (&[register], [value]) = (register, data) else { panic!("expected one register") };
        assert_eq!(address, 0x34);
        self.respond()?;
        *value = self.registers[register as usize];
        if self.corrupt_readback && self.writes > 0 && register == 0x64 {
            *value ^= 1;
        }
        Ok(())
    }
}

const PROFILE: ChargeProfile =
    ChargeProfile { voltage_mv: 4200, current_ma: 500, termination_ma: 50 };

fn profile(voltage_mv: u32, current_ma: u32, termination_ma: u32) -> ChargeProfile {
    ChargeProfile { voltage_mv, current_ma, termination_ma }
}

#[test]
fn charge_profile() {
    let mut chip = Chip::new();
    assert_eq!(pmic::charge_configure(&mut chip, &PROFILE), Ok(()));
    assert_eq!(chip.registers[0x18], 0x0a);
    assert_eq!(chip.registers[0x64], 0xa3);
    assert_eq!(chip.registers[0x63], 0xf2);
    assert_eq!(chip.registers[0x62], 0xeb);
    // Precharge, timers and protection selection remain unchanged
    assert_eq!((chip.registers[0x61], chip.registers[0x65], chip.registers[0x67]), (5, 2, 0xe6));

    let mut chip = Chip::new();
    chip.registers[0x18] &= !2;
    assert_eq!(pmic::charge_configure(&mut chip, &PROFILE), Ok(()));
    assert_eq!(chip.registers[0x18], 8); // Charging stays disabled
}

#[test]
fn charge_limits() {
    let limits = [(4000, 1500), (4100, 1500), (4200, 1400), (4350, 1100), (4400, 1000)];
    for (setting, (voltage, limit)) in (1..).zip(limits) {
        let mut chip = Chip::new();
        assert_eq!(pmic::charge_configure(&mut chip, &profile(voltage, limit, 50)), Ok(()));
        assert_eq!(chip.registers[0x64] & 7, setting);
        assert_eq!((chip.registers[0x62] & 31) as u32, 8 + (limit - 200) / 100);
        let mut chip = Chip::new();
        let excessive = profile(voltage, limit + 100, 50);
        assert_eq!(pmic::charge_configure(&mut chip, &excessive), Err(Error::Invalid));
        assert_eq!(chip.operations, 0);
    }
    for (current, setting) in [(50, 2), (175, 7), (200, 8), (300, 9)] {
        let mut chip = Chip::new();
        assert_eq!(pmic::charge_configure(&mut chip, &profile(4200, current, 25)), Ok(()));
        assert_eq!(chip.registers[0x62] & 31, setting);
    }
}

#[test]
fn invalid_charge_profile() {
    for (voltage, current, termination) in [
        (4200, 501, 50),
        (4300, 500, 50),
        (4200, 500, 0),
        (4200, 0, 50),
        (4200, 1500, 50),
        (4350, 1200, 50),
        (4400, 1100, 50),
        (4200, 500, 51),
    ] {
        let mut chip = Chip::new();
        let invalid = profile(voltage, current, termination);
        assert_eq!(pmic::charge_configure(&mut chip, &invalid), Err(Error::Invalid));
        assert_eq!(chip.operations, 0);
    }
}

#[test]
fn charge_write_failures() {
    let mut chip = Chip::new();
    assert_eq!(pmic::charge_configure(&mut chip, &PROFILE), Ok(()));
    for failure in 1..=chip.operations {
        let mut chip = Chip::new();
        chip.fail_at = failure;
        assert!(pmic::charge_configure(&mut chip, &PROFILE).is_err());
        // An initial read failure leaves hardware untouched; later failures
        // disable charging
        if failure == 1 {
            assert_eq!(chip.writes, 0);
        } else {
            assert_eq!(chip.registers[0x18] & 2, 0, "failure {failure}");
        }
    }
    let mut chip = Chip::new();
    chip.corrupt_readback = true;
    assert_eq!(pmic::charge_configure(&mut chip, &PROFILE), Err(Error::Unverified));
    assert_eq!(chip.registers[0x18] & 2, 0);
}

#[test]
fn percentage() {
    let mut chip = Chip::new();
    assert_eq!(pmic::percentage(&mut chip), Ok(87));
    assert_eq!(chip.writes, 0);
    for failure in 1..=3 {
        let mut chip = Chip::new();
        chip.fail_at = failure;
        assert_eq!(pmic::percentage(&mut chip), Err(Error::Bus));
        assert_eq!(chip.writes, 0);
    }
    let mut chip = Chip::new();
    chip.registers[0] = 0; // No battery
    assert_eq!(pmic::percentage(&mut chip), Err(Error::Invalid));
    let mut chip = Chip::new();
    chip.registers[0x18] &= !8; // Gauge disabled
    assert_eq!(pmic::percentage(&mut chip), Err(Error::Invalid));
    for reading in [0, 100, 101, 255] {
        let mut chip = Chip::new();
        chip.registers[0xa4] = reading;
        let expected = if reading <= 100 { Ok(reading as u32) } else { Err(Error::Invalid) };
        assert_eq!(pmic::percentage(&mut chip), expected);
    }
}

#[test]
fn output_masks() {
    let mut chip = Chip::new();
    assert_eq!(pmic::aldo_voltage(&mut chip, 0, 3300), Err(Error::Invalid));
    assert_eq!(pmic::aldo_voltage(&mut chip, 3, 3350), Err(Error::Invalid));
    assert_eq!(pmic::aldo_enable(&mut chip, 5, true), Err(Error::Invalid));
    assert_eq!(pmic::outputs_disable(&mut chip, 0x80, 0, false), Err(Error::Invalid));
    assert_eq!(chip.operations, 0);
    chip.registers[0x94] = 0xe0;
    assert_eq!(pmic::aldo_voltage(&mut chip, 3, 3300), Ok(()));
    assert_eq!(chip.registers[0x94], 0xfc);
    chip.registers[0x90] = 1;
    assert_eq!(pmic::aldo_enable(&mut chip, 3, true), Ok(()));
    assert_eq!(chip.registers[0x90], 5);
    assert_eq!(pmic::aldo_enable(&mut chip, 3, false), Ok(()));
    assert_eq!(chip.registers[0x90], 1);
    chip.registers[0x80] = 0xff;
    chip.registers[0x90] = 0xff;
    chip.registers[0x91] = 0xff;
    let dcdc = pmic::DCDC2 | pmic::DCDC3 | pmic::DCDC4 | pmic::DCDC5;
    let ldo = pmic::ALDO2 | pmic::ALDO4 | pmic::BLDO1 | pmic::BLDO2 | pmic::CPUSLDO | pmic::DLDO1;
    assert_eq!(pmic::outputs_disable(&mut chip, dcdc, ldo, true), Ok(()));
    assert_eq!((chip.registers[0x80], chip.registers[0x90], chip.registers[0x91]), (0xe1, 5, 0xfe));
}

#[test]
fn automatic_regulator() {
    let mut chip = Chip::new();
    chip.registers[0x80] = 0xff;
    chip.registers[0x81] = 0xff;
    assert_eq!(pmic::dcdc1_auto(&mut chip), Ok(()));
    assert_eq!((chip.registers[0x80], chip.registers[0x81]), (0xbf, 0xfb));
    for failure in 1..=6 {
        let mut chip = Chip::new();
        chip.fail_at = failure;
        assert_eq!(pmic::dcdc1_auto(&mut chip), Err(Error::Bus));
    }
}

#[test]
fn interrupts_disable() {
    let pending = [(0x40, 0x48, 0xff), (0x41, 0x49, 0xff), (0x42, 0x4a, 0xdf)];
    let mut chip = Chip::new();
    for (enable, status, mask) in pending {
        chip.registers[enable] = mask;
        chip.registers[status] = mask;
    }
    assert_eq!(pmic::interrupts_disable(&mut chip), Ok(()));
    for (enable, status, _) in pending {
        assert_eq!((chip.registers[enable], chip.registers[status]), (0, 0));
    }
    assert_eq!((chip.registers[0x18], chip.registers[0x30]), (0x0a, 3));

    // A failure on one register does not stop the others
    let mut chip = Chip::new();
    chip.fail_at = 1;
    chip.registers[0x41] = 0xff;
    chip.registers[0x42] = 0xdf;
    assert_eq!(pmic::interrupts_disable(&mut chip), Err(Error::Bus));
    assert_eq!((chip.registers[0x41], chip.registers[0x42]), (0, 0));
}

#[test]
fn adc_and_temperature_selection() {
    let mut chip = Chip::new();
    chip.registers[0x30] = 0xff;
    let unused =
        pmic::ADC_VBUS | pmic::ADC_VSYS | pmic::ADC_DIE_TEMPERATURE | pmic::ADC_GENERAL_PURPOSE;
    assert_eq!(pmic::adc_disable(&mut chip, unused), Ok(()));
    assert_eq!(chip.registers[0x30], 0xc3);
    assert_eq!(pmic::adc_disable(&mut chip, unused | pmic::ADC_TS), Ok(()));
    assert_eq!(chip.registers[0x30], 0xc1);
    assert_eq!(pmic::adc_disable(&mut chip, 0x80), Err(Error::Invalid));
    chip.registers[0x50] = 0x12;
    assert_eq!(pmic::ts_charger_monitoring_enabled(&mut chip), Ok(false));
    chip.registers[0x50] = 0x0a;
    assert_eq!(pmic::ts_charger_monitoring_enabled(&mut chip), Ok(true));
    chip.fail_at = chip.operations + 1;
    assert_eq!(pmic::ts_charger_monitoring_enabled(&mut chip), Err(Error::Bus));
}
