use crate::board::power::*;
use crate::tests::drivers::pmic::Chip;

/// A PMIC with every output, interrupt and ADC channel enabled
fn busy_chip(ts_monitoring: bool) -> Chip {
    let mut chip = Chip::new();
    for register in [0x30, 0x40, 0x41, 0x42, 0x80, 0x81, 0x90, 0x91] {
        chip.registers[register] = 0xff;
    }
    chip.registers[0x50] = if ts_monitoring { 0x0a } else { 0x12 };
    chip
}

#[test]
fn power_configuration() {
    for ts_monitoring in [false, true] {
        let mut chip = busy_chip(ts_monitoring);
        assert!(configure(&mut chip).is_ok());
        // The battery's charge profile, with charging still enabled
        assert_eq!(chip.registers[0x64] & 7, 3);
        assert_eq!(chip.registers[0x62] & 31, 11);
        assert_eq!(chip.registers[0x63] & 31, 0x12);
        assert_eq!(chip.registers[0x18] & 2, 2);
        // Only DCDC1, audio ALDO1 and panel-reset ALDO3 stay on, in automatic mode
        assert_eq!(chip.registers[0x80], 0xa1);
        assert_eq!(chip.registers[0x81], 0xfb);
        assert_eq!(chip.registers[0x90], 0x05);
        assert_eq!(chip.registers[0x91], 0xfe);
        assert_eq!(chip.registers[0x10] & 0x20, 0x20);
        assert_eq!(chip.registers[0x69] & 0x37, 0x05);
        assert_eq!(chip.registers[0x40..0x43], [0, 0, 0x20]);
        // Keep battery sensing; keep TS only if charger protection uses it
        assert_eq!(chip.registers[0x30], if ts_monitoring { 0xc3 } else { 0xc1 });
    }
}

#[test]
fn every_step_is_attempted_after_a_failure() {
    let mut chip = busy_chip(false);
    chip.fail_at = 1; // The charge profile
    assert!(configure(&mut chip).is_err());
    assert_eq!((chip.registers[0x80], chip.registers[0x90]), (0xa1, 0x05));
    assert_eq!(chip.registers[0x30], 0xc1);

    let mut complete = busy_chip(false);
    assert!(configure(&mut complete).is_ok());
    let mut chip = busy_chip(false);
    chip.fail_at = complete.operations - 3; // The TS monitoring check
    assert!(configure(&mut chip).is_err());
    assert_eq!(chip.registers[0x30], 0xff); // ADC selection needs that answer
}

#[test]
fn clock_policy() {
    // Original board policy, expressed as offsets to catch mapping changes
    const REGISTER_OFFSETS: [u32; 23] = [
        0x00, 0x0c, 0x20, 0x28, 0x2c, 0x34, 0x5c, 0x64, 0x6c, 0x98, 0xa4, 0xbc, 0xc8, 0xcc, 0xd0,
        0xd8, 0x3c, 0x48, 0x94, 0x9c, 0xb0, 0xc0, 0xe0,
    ];
    const FUNCTIONAL_OFFSETS: [u32; 17] = [
        0x04, 0x10, 0x24, 0x30, 0x38, 0x40, 0x44, 0x4c, 0x50, 0x60, 0x68, 0x70, 0x78, 0x84, 0x88,
        0xa0, 0xc4,
    ];
    let offsets = |registers: &[crate::hal::registers::Reg]| -> Vec<u32> {
        registers.iter().map(|register| register.address() - 0x6009_6000).collect()
    };
    assert_eq!(offsets(&UNUSED_REGISTER_CLOCKS), REGISTER_OFFSETS);
    assert_eq!(offsets(&UNUSED_FUNCTIONAL_CLOCKS), FUNCTIONAL_OFFSETS);
    // USB (0x8c) and the system timer (0x54, 0x58) must stay enabled
    for kept in [0x8c, 0x54, 0x58] {
        assert!(!REGISTER_OFFSETS.contains(&kept) && !FUNCTIONAL_OFFSETS.contains(&kept));
    }
}

#[test]
fn sleep_outputs() {
    assert_eq!(HIGH_OUTPUTS, 0x07);
    let held =
        0x1f | (1 << 6) | (1 << 11) | (1 << 15) | (1 << 19) | (1 << 20) | (1 << 22) | (1 << 23);
    assert_eq!(HELD_OUTPUTS, held);
}
