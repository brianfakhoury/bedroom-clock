//! AXP2101 power management chip.

use super::Error;
use crate::hal::i2c::I2c;

const ADDRESS: u8 = 0x34;
const STATUS1: u8 = 0x00;
const CHARGER_CONTROL: u8 = 0x18;
const ADC_CONTROL: u8 = 0x30;
const CHARGE_CURRENT: u8 = 0x62;
const TERMINATION_CURRENT: u8 = 0x63;
const CHARGE_VOLTAGE: u8 = 0x64;
const BATTERY_PERCENTAGE: u8 = 0xa4;
const BATTERY_PRESENT: u8 = 0x08;
const GAUGE_ENABLE: u8 = 0x08;
const CHARGE_ENABLE: u8 = 0x02;
const TERMINATION_ENABLE: u8 = 0x10;

/// Masks for the output and ADC selections below; the chip's complete lists
#[allow(dead_code)]
mod masks {
    pub const DCDC1: u8 = 1 << 0;
    pub const DCDC2: u8 = 1 << 1;
    pub const DCDC3: u8 = 1 << 2;
    pub const DCDC4: u8 = 1 << 3;
    pub const DCDC5: u8 = 1 << 4;

    pub const ALDO1: u8 = 1 << 0;
    pub const ALDO2: u8 = 1 << 1;
    pub const ALDO3: u8 = 1 << 2;
    pub const ALDO4: u8 = 1 << 3;
    pub const BLDO1: u8 = 1 << 4;
    pub const BLDO2: u8 = 1 << 5;
    pub const CPUSLDO: u8 = 1 << 6;
    pub const DLDO1: u8 = 1 << 7;

    pub const ADC_BATTERY: u8 = 1 << 0;
    pub const ADC_TS: u8 = 1 << 1;
    pub const ADC_VBUS: u8 = 1 << 2;
    pub const ADC_VSYS: u8 = 1 << 3;
    pub const ADC_DIE_TEMPERATURE: u8 = 1 << 4;
    pub const ADC_GENERAL_PURPOSE: u8 = 1 << 5;
}
pub use masks::*;

pub struct ChargeProfile {
    pub voltage_mv: u32,
    pub current_ma: u32,
    pub termination_ma: u32,
}

fn read_register(bus: &mut impl I2c, register: u8) -> Result<u8, Error> {
    let mut value = 0;
    bus.read(ADDRESS, &[register], core::slice::from_mut(&mut value))?;
    Ok(value)
}

// Change the masked bits, then read them back
fn update_register(bus: &mut impl I2c, register: u8, mask: u8, setting: u8) -> Result<(), Error> {
    let value = (read_register(bus, register)? & !mask) | setting;
    bus.write(ADDRESS, &[register, value])?;
    if read_register(bus, register)? & mask == setting { Ok(()) } else { Err(Error::Unverified) }
}

fn require(valid: bool) -> Result<(), Error> {
    if valid { Ok(()) } else { Err(Error::Invalid) }
}

/// Raw gauge estimate, 0–100
pub fn percentage(bus: &mut impl I2c) -> Result<u32, Error> {
    require(read_register(bus, STATUS1)? & BATTERY_PRESENT != 0)?;
    require(read_register(bus, CHARGER_CONTROL)? & GAUGE_ENABLE != 0)?;
    let percentage = read_register(bus, BATTERY_PERCENTAGE)?;
    require(percentage <= 100)?;
    Ok(percentage as u32)
}

/// Invalid profiles cause no writes. Configure with charging disabled, then
/// restore its enable state. On I/O failure, attempt to leave charging disabled
pub fn charge_configure(bus: &mut impl I2c, profile: &ChargeProfile) -> Result<(), Error> {
    /// Millivolts, maximum milliamps, register setting
    const VOLTAGES: [(u32, u32, u8); 5] =
        [(4000, 1500, 1), (4100, 1500, 2), (4200, 1400, 3), (4350, 1100, 4), (4400, 1000, 5)];
    let &ChargeProfile { voltage_mv, current_ma: current, termination_ma: termination } = profile;
    let &(_, maximum, voltage) =
        VOLTAGES.iter().find(|(millivolts, ..)| *millivolts == voltage_mv).ok_or(Error::Invalid)?;
    require(current != 0 && current <= maximum)?;
    require(if current <= 200 { current % 25 == 0 } else { (current - 200) % 100 == 0 })?;
    require((25..=200).contains(&termination) && termination % 25 == 0 && termination < current)?;
    let current_code = if current <= 200 { current / 25 } else { 8 + (current - 200) / 100 };
    let termination_code = TERMINATION_ENABLE | (termination / 25) as u8;

    let enabled = read_register(bus, CHARGER_CONTROL)? & CHARGE_ENABLE;
    let configured = update_register(bus, CHARGER_CONTROL, CHARGE_ENABLE, 0)
        .and_then(|()| update_register(bus, CHARGE_VOLTAGE, 0x07, voltage))
        .and_then(|()| update_register(bus, TERMINATION_CURRENT, 0x1f, termination_code))
        .and_then(|()| update_register(bus, CHARGE_CURRENT, 0x1f, current_code as u8))
        .and_then(|()| update_register(bus, CHARGER_CONTROL, CHARGE_ENABLE, enabled));
    if configured.is_err() {
        let _ = update_register(bus, CHARGER_CONTROL, CHARGE_ENABLE, 0);
    }
    configured
}

/// Channels 1–4, 500–3500 mV in 100 mV steps
pub fn aldo_voltage(bus: &mut impl I2c, channel: u8, mv: u32) -> Result<(), Error> {
    require((1..=4).contains(&channel) && (500..=3500).contains(&mv) && mv.is_multiple_of(100))?;
    update_register(bus, 0x91 + channel, 0x1f, ((mv - 500) / 100) as u8)
}

pub fn aldo_enable(bus: &mut impl I2c, channel: u8, enabled: bool) -> Result<(), Error> {
    require((1..=4).contains(&channel))?;
    let mask = 1 << (channel - 1);
    update_register(bus, 0x90, mask, if enabled { mask } else { 0 })
}

/// Combine DCDC masks and LDO masks separately; DLDO2 has its own control
pub fn outputs_disable(bus: &mut impl I2c, dcdc: u8, ldo: u8, dldo2: bool) -> Result<(), Error> {
    require(dcdc & !0x1f == 0)?;
    update_register(bus, 0x80, dcdc, 0)?;
    update_register(bus, 0x90, ldo, 0)?;
    update_register(bus, 0x91, dldo2 as u8, 0)
}

pub fn output_discharge_enable(bus: &mut impl I2c) -> Result<(), Error> {
    update_register(bus, 0x10, 0x20, 0x20)
}

/// Clear forced CCM and PWM without changing voltage or other regulators
pub fn dcdc1_auto(bus: &mut impl I2c) -> Result<(), Error> {
    update_register(bus, 0x80, 0x40, 0)?;
    update_register(bus, 0x81, 0x04, 0)
}

pub fn led_off(bus: &mut impl I2c) -> Result<(), Error> {
    update_register(bus, 0x69, 0x37, 0x05)
}

/// Mask notifications and clear pending status; protection stays on
pub fn interrupts_disable(bus: &mut impl I2c) -> Result<(), Error> {
    let mut outcome = Ok(());
    for (enable, status, mask) in [(0x40, 0x48, 0xff), (0x41, 0x49, 0xff), (0x42, 0x4a, 0xdf)] {
        // Mask first, then acknowledge latched events to release the IRQ pull-down
        let masked = update_register(bus, enable, mask, 0);
        let cleared = bus.write(ADDRESS, &[status, mask]).map_err(Error::from);
        outcome = outcome.and(masked).and(cleared);
    }
    outcome
}

/// Stop converting the given ADC channels
pub fn adc_disable(bus: &mut impl I2c, channels: u8) -> Result<(), Error> {
    require(channels & !0x3f == 0)?;
    update_register(bus, ADC_CONTROL, channels, 0)
}

/// Whether the charger uses the TS pin to monitor battery temperature
pub fn ts_charger_monitoring_enabled(bus: &mut impl I2c) -> Result<bool, Error> {
    Ok(read_register(bus, 0x50)? & 0x10 == 0)
}
