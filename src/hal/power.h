#pragma once
#include <stdint.h>

// Configure MCU power states after timer initialization, before radio shutdown
void pmu_init(void);
void radios_disable(void);

// PCR config registers with the standard register clock gate.
// Stop watchdogs before disabling timer group register clocks.
void peripheral_register_clock_disable(uint32_t register_address);
// PCR functional clock registers with the standard clock gate.
void peripheral_functional_clock_disable(uint32_t register_address);
// These peripherals use different enable bits from the standard gates.
void parlio_functional_clocks_disable(void);
void adc_register_clocks_disable(void);
