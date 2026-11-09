#include "pmic.h"

#include "i2c.h"

enum {
  PMIC_ADDRESS = 0x34,
  STATUS1 = 0x00,
  CHARGER_CONTROL = 0x18,
  ADC_CONTROL = 0x30,
  CHARGE_CURRENT = 0x62,
  TERMINATION_CURRENT = 0x63,
  CHARGE_VOLTAGE = 0x64,
  BATTERY_PERCENTAGE = 0xa4,
  GAUGE_ENABLE = 0x08,
  CHARGE_ENABLE = 0x02,
  TERMINATION_ENABLE = 0x10
};

static bool read_register(uint8_t reg, uint8_t* value) {
  return i2c_read(PMIC_ADDRESS, reg, value, 1);
}

static bool update_register(uint8_t reg, uint8_t mask, uint8_t setting) {
  uint8_t value, actual;
  if (!read_register(reg, &value)) return false;
  value = (value & ~mask) | setting;
  return i2c_write(PMIC_ADDRESS, reg, &value, 1) &&
         read_register(reg, &actual) && (actual & mask) == setting;
}

bool pmic_percentage(unsigned* percentage) {
  uint8_t present, control, value;
  if (!percentage) return false;
  if (!read_register(STATUS1, &present) || !(present & 8)) return false;
  if (!read_register(CHARGER_CONTROL, &control) || !(control & GAUGE_ENABLE))
    return false;
  if (!read_register(BATTERY_PERCENTAGE, &value) || value > 100) return false;
  *percentage = value;
  return true;
}

bool pmic_charge_configure(const pmic_charge_config* config) {
  if (!config) return false;
  static const struct {
    uint16_t millivolts, maximum_ma;
    uint8_t setting;
  } voltages[] = {{4000, 1500, 1},
                  {4100, 1500, 2},
                  {4200, 1400, 3},
                  {4350, 1100, 4},
                  {4400, 1000, 5}};
  unsigned voltage = 0, maximum = 0;
  for (unsigned i = 0; i < sizeof(voltages) / sizeof(voltages[0]); i++)
    if (voltages[i].millivolts == config->voltage_mv) {
      voltage = voltages[i].setting;
      maximum = voltages[i].maximum_ma;
      break;
    }
  unsigned current = config->current_ma;
  unsigned termination = config->termination_ma;
  if (!voltage || current == 0 || current > maximum) return false;
  if (current <= 200 ? current % 25 : (current - 200) % 100) return false;
  if (termination < 25 || termination > 200 || termination % 25 ||
      termination >= current)
    return false;
  unsigned current_code =
      current <= 200 ? current / 25 : 8 + (current - 200) / 100;
  unsigned termination_code = TERMINATION_ENABLE | (termination / 25);
  uint8_t control;
  if (!read_register(CHARGER_CONTROL, &control)) return false;
  bool good = update_register(CHARGER_CONTROL, CHARGE_ENABLE, 0);
  good = good && update_register(CHARGE_VOLTAGE, 0x07, voltage);
  good = good && update_register(TERMINATION_CURRENT, 0x1f, termination_code);
  good = good && update_register(CHARGE_CURRENT, 0x1f, current_code);
  uint8_t enabled = control & CHARGE_ENABLE;
  if (good) good = update_register(CHARGER_CONTROL, CHARGE_ENABLE, enabled);
  if (!good) (void)update_register(CHARGER_CONTROL, CHARGE_ENABLE, 0);
  return good;
}

bool pmic_aldo_voltage(unsigned channel, unsigned mv) {
  if (channel < 1 || channel > 4 || mv < 500 || mv > 3500 || mv % 100)
    return false;
  return update_register(0x91 + channel, 0x1f, (mv - 500) / 100);
}

bool pmic_aldo_enable(unsigned channel, bool enabled) {
  if (channel < 1 || channel > 4) return false;
  uint8_t mask = 1u << (channel - 1);
  return update_register(0x90, mask, enabled ? mask : 0);
}

bool pmic_outputs_disable(uint8_t dcdc, uint8_t ldo, bool dldo2) {
  if (dcdc & ~0x1f) return false;
  return update_register(0x80, dcdc, 0) && update_register(0x90, ldo, 0) &&
         update_register(0x91, dldo2 ? 1 : 0, 0);
}

bool pmic_output_discharge_enable(void) {
  return update_register(0x10, 0x20, 0x20);
}

// Clear forced CCM and PWM without changing voltage or other regulators
bool pmic_dcdc1_auto(void) {
  return update_register(0x80, 0x40, 0) && update_register(0x81, 0x04, 0);
}

bool pmic_led_off(void) { return update_register(0x69, 0x37, 0x05); }

bool pmic_interrupts_disable(void) {
  const uint8_t masks[] = {0xff, 0xff, 0xdf};
  bool good = true;
  for (unsigned i = 0; i < sizeof(masks); i++) {
    // Mask first, then acknowledge latched events to release the IRQ pull-down
    bool masked = update_register(0x40 + i, masks[i], 0);
    bool cleared = i2c_write(PMIC_ADDRESS, 0x48 + i, &masks[i], 1);
    good = masked && cleared && good;
  }
  return good;
}

bool pmic_adc_disable(uint8_t channels) {
  return !(channels & ~0x3f) && update_register(ADC_CONTROL, channels, 0);
}

bool pmic_ts_charger_monitoring_enabled(bool* enabled) {
  uint8_t control;
  if (!enabled || !read_register(0x50, &control)) return false;
  *enabled = !(control & 0x10);
  return true;
}
