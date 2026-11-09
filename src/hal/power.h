#pragma once
// Configure MCU power states after timer initialization, before radio shutdown
void pmu_init(void);
void radios_disable(void);
