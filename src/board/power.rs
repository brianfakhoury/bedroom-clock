//! Charger and rail policy, unused peripheral clocks, the touch wake pin,
//! parked and held outputs, and the ROM resume entry.

use super::battery;
use super::pins::*;
use crate::boot;
use crate::drivers::{Error, pmic};
use crate::hal::i2c::I2c;
use crate::hal::registers::*;
use crate::hal::{clocks, sleep};

// DCDC1 powers the board
const UNUSED_DCDC: u8 = pmic::DCDC2 | pmic::DCDC3 | pmic::DCDC4 | pmic::DCDC5;
// Keep audio ALDO1 on: cutting it makes the shared I2C bus unresponsive
// Disable amplifier CTRL (ALDO2); keep panel reset and RTC powered
const UNUSED_LDO: u8 =
    pmic::ALDO2 | pmic::ALDO4 | pmic::BLDO1 | pmic::BLDO2 | pmic::CPUSLDO | pmic::DLDO1;
const UNUSED_ADC: u8 =
    pmic::ADC_VBUS | pmic::ADC_VSYS | pmic::ADC_DIE_TEMPERATURE | pmic::ADC_GENERAL_PURPOSE;

/// Apply the charger and rail policy. Every step is attempted
pub fn configure(bus: &mut impl I2c) -> Result<(), Error> {
    let charge = pmic::charge_configure(bus, &battery::CHARGE);
    let rails = pmic::output_discharge_enable(bus)
        .and_then(|()| pmic::aldo_enable(bus, 1, true))
        .and_then(|()| pmic::outputs_disable(bus, UNUSED_DCDC, UNUSED_LDO, true));
    let regulator = pmic::dcdc1_auto(bus);
    let led = pmic::led_off(bus);
    let interrupts = pmic::interrupts_disable(bus);
    // Keep battery sensing; keep TS only if charger protection uses it
    let adc = pmic::ts_charger_monitoring_enabled(bus).and_then(|ts_enabled| {
        pmic::adc_disable(bus, UNUSED_ADC | if ts_enabled { 0 } else { pmic::ADC_TS })
    });
    charge.and(rails).and(regulator).and(led).and(interrupts).and(adc)
}

// Unused peripheral clocks, including timer groups.
// Keep the system timer, GPIO, USB, flash/cache and clock/reset controls

pub static UNUSED_FUNCTIONAL_CLOCKS: [Reg; 17] = [
    PCR_UART0_CLOCK,
    PCR_UART1_CLOCK,
    PCR_I2C_CLOCK,
    PCR_RMT_CLOCK,
    PCR_LEDC_CLOCK,
    PCR_TIMG0_CLOCK,
    PCR_TIMG0_WDT_CLOCK,
    PCR_TIMG1_CLOCK,
    PCR_TIMG1_WDT_CLOCK,
    PCR_TWAI0_CLOCK,
    PCR_TWAI1_CLOCK,
    PCR_I2S_TX_CLOCK,
    PCR_I2S_RX_CLOCK,
    PCR_ADC_CLOCK,
    PCR_TEMP_CLOCK,
    PCR_MCPWM_CLOCK,
    PCR_SPI2_CLOCK,
];

pub static UNUSED_REGISTER_CLOCKS: [Reg; 23] = [
    PCR_UART0_CONFIG,
    PCR_UART1_CONFIG,
    PCR_I2C_CONFIG, // Board uses software I2C
    PCR_UHCI_CONFIG,
    PCR_RMT_CONFIG,
    PCR_LEDC_CONFIG,
    PCR_TWAI0_CONFIG,
    PCR_TWAI1_CONFIG,
    PCR_I2S_CONFIG,
    PCR_ETM_CONFIG,
    PCR_PARLIO_CONFIG,
    PCR_GDMA_CONFIG,
    PCR_AES_CONFIG,
    PCR_SHA_CONFIG,
    PCR_RSA_CONFIG,
    PCR_ECC_CONFIG,
    PCR_TIMG0_CONFIG, // Watchdogs disabled before gating
    PCR_TIMG1_CONFIG,
    PCR_PCNT_CONFIG,
    PCR_MCPWM_CONFIG,
    PCR_SDIO_CONFIG,
    PCR_SPI2_CONFIG, // Panel uses software QSPI
    PCR_DS_CONFIG,
];

pub fn unused_peripheral_clocks_disable() {
    // Disable functional clocks before their register interfaces
    UNUSED_FUNCTIONAL_CLOCKS.iter().copied().for_each(clocks::functional_clock_disable);
    clocks::parlio_functional_clocks_disable();
    clocks::adc_register_clocks_disable();
    UNUSED_REGISTER_CLOCKS.iter().copied().for_each(clocks::register_clock_disable);
}

// R15/R12/R17 pull shared SPI lines up through 10 kohm; park them high
pub const HIGH_OUTPUTS: u32 = PANEL_CLOCK.mask() | PANEL_DATA0.mask() | PANEL_DATA1.mask();
// Hold outputs only: global holds interfere with USB; touch/I2C stay inputs
pub const HELD_OUTPUTS: u32 = HIGH_OUTPUTS
    | PANEL_DATA2.mask()
    | PANEL_DATA3.mask()
    | SD_CS.mask()
    | TOUCH_RESET.mask()
    | PANEL_CS.mask()
    | AUDIO_MCLK.mask()
    | AUDIO_BCLK.mask()
    | AUDIO_LRCLK.mask()
    | AUDIO_DATA_OUT.mask();

pub fn resume() {
    sleep::resume(TOUCH_IRQ);
    unused_peripheral_clocks_disable();
}

/// Returns only when a pending touch cancels sleep
pub fn sleep() {
    if !sleep::prepare(TOUCH_IRQ) {
        unused_peripheral_clocks_disable();
        return;
    }
    // ROM checks the retained code before resuming through wake.S
    sleep::enter(boot::wake_entry, boot::retained_bytes(), HIGH_OUTPUTS, HELD_OUTPUTS)
}
