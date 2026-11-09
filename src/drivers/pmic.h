#pragma once
#include <stdbool.h>
#include <stdint.h>

typedef struct {
  unsigned voltage_mv;
  unsigned current_ma;
  unsigned termination_ma;
} pmic_charge_config;

// Raw gauge estimate, 0–100; false leaves the output unchanged
bool pmic_percentage(unsigned* percentage);
// Invalid inputs cause no writes. Configure with charging disabled, then
// restore its enable state. On I/O failure, attempt to leave charging disabled
bool pmic_charge_configure(const pmic_charge_config* config);

bool pmic_aldo_voltage(unsigned channel, unsigned mv);
bool pmic_aldo_enable(unsigned channel, bool enabled);
// Enable-register masks: DCDC1..5, ALDO1..4/BLDO1..2/CPUSLDO/DLDO1
bool pmic_outputs_disable(uint8_t dcdc, uint8_t ldo, bool dldo2);
bool pmic_output_discharge_enable(void);
bool pmic_dcdc1_auto(void);

bool pmic_led_off(void);
// Mask notifications and clear pending status; protection stays on
bool pmic_interrupts_disable(void);
bool pmic_adc_disable(uint8_t channels);
bool pmic_ts_charger_monitoring_enabled(bool* enabled);
