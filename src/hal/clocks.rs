//! Peripheral clock gates. Which peripherals go unused is board policy.

use super::registers::*;

/// PCR config registers with the standard register clock gate.
/// Stop watchdogs before disabling timer group register clocks
pub fn register_clock_disable(config: Reg) {
    config.clear(PCR_REGISTER_CLOCK_ENABLE);
}

/// PCR functional clock registers with the standard clock gate
pub fn functional_clock_disable(clock: Reg) {
    clock.clear(PCR_FUNCTIONAL_CLOCK_ENABLE);
}

// These peripherals use different enable bits from the standard gates

pub fn parlio_functional_clocks_disable() {
    PCR_PARLIO_RX_CLOCK.clear(PCR_PARLIO_CLOCK_ENABLE);
    PCR_PARLIO_TX_CLOCK.clear(PCR_PARLIO_CLOCK_ENABLE);
}

pub fn adc_register_clocks_disable() {
    PCR_ADC_CONFIG.clear(PCR_ADC_CLOCK_ENABLE);
}
