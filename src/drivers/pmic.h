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
enum {
  PMIC_DCDC1 = 1u << 0,
  PMIC_DCDC2 = 1u << 1,
  PMIC_DCDC3 = 1u << 2,
  PMIC_DCDC4 = 1u << 3,
  PMIC_DCDC5 = 1u << 4
};

enum {
  PMIC_ALDO1 = 1u << 0,
  PMIC_ALDO2 = 1u << 1,
  PMIC_ALDO3 = 1u << 2,
  PMIC_ALDO4 = 1u << 3,
  PMIC_BLDO1 = 1u << 4,
  PMIC_BLDO2 = 1u << 5,
  PMIC_CPUSLDO = 1u << 6,
  PMIC_DLDO1 = 1u << 7
};

// Combine PMIC_DCDC masks and LDO masks separately; DLDO2 has its own control
bool pmic_outputs_disable(uint8_t dcdc, uint8_t ldo, bool dldo2);
bool pmic_output_discharge_enable(void);
bool pmic_dcdc1_auto(void);

bool pmic_led_off(void);
// Mask notifications and clear pending status; protection stays on
bool pmic_interrupts_disable(void);
enum {
  PMIC_ADC_BATTERY = 1u << 0,
  PMIC_ADC_TS = 1u << 1,
  PMIC_ADC_VBUS = 1u << 2,
  PMIC_ADC_VSYS = 1u << 3,
  PMIC_ADC_DIE_TEMPERATURE = 1u << 4,
  PMIC_ADC_GENERAL_PURPOSE = 1u << 5
};

bool pmic_adc_disable(uint8_t channels);
bool pmic_ts_charger_monitoring_enabled(bool* enabled);
