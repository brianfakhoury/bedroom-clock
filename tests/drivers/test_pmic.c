#include <assert.h>
#include <string.h>

#include "i2c.h"
#include "pmic.h"

#include "../../src/drivers/pmic.c"

static uint8_t registers[256];
static unsigned writes, operations, fail_at;
static bool corrupt_readback;

bool i2c_read(uint8_t addr, uint8_t reg, uint8_t* out, unsigned n) {
  assert(addr == 0x34);
  if (++operations == fail_at) return false;
  memcpy(out, registers + reg, n);
  if (corrupt_readback && writes && reg == 0x64) out[0] ^= 1;
  return true;
}

bool i2c_write(uint8_t addr, uint8_t reg, const uint8_t* data, unsigned n) {
  assert(addr == 0x34);
  writes++;
  if (++operations == fail_at) return false;
  // Profile writes must only occur after charging is disabled.
  if (reg >= 0x62 && reg <= 0x64) assert(!(registers[0x18] & 2));
  if (reg >= 0x48 && reg <= 0x4a)
    registers[reg] &= ~*data;
  else
    memcpy(registers + reg, data, n);
  return true;
}

static void reset(void) {
  memset(registers, 0, sizeof(registers));
  registers[0] = 8;
  registers[0x18] = 0x0a;
  registers[0x30] = 3;
  registers[0x62] = 0xe8;
  registers[0x63] = 0xf5;
  registers[0x64] = 0xa3;
  registers[0xa4] = 87;
  registers[0x61] = 5;
  registers[0x65] = 2;
  registers[0x67] = 0xe6;
  writes = operations = fail_at = 0;
  corrupt_readback = false;
}

static const pmic_charge_config config = {4200, 500, 50};

static void test_charge_profile(void) {
  reset();
  assert(pmic_charge_configure(&config));
  assert(registers[0x18] == 0x0a);
  assert(registers[0x64] == 0xa3);
  assert(registers[0x63] == 0xf2);
  assert(registers[0x62] == 0xeb);
  assert(registers[0x61] == 5 && registers[0x65] == 2 &&
         registers[0x67] == 0xe6);
  reset();
  registers[0x18] &= ~2;
  assert(pmic_charge_configure(&config) && registers[0x18] == 8);
}

static void test_charge_limits(void) {
  const unsigned voltages[] = {4000, 4100, 4200, 4350, 4400};
  const unsigned limits[] = {1500, 1500, 1400, 1100, 1000};
  for (unsigned i = 0; i < 5; i++) {
    pmic_charge_config profile = {voltages[i], limits[i], 50};
    reset();
    assert(pmic_charge_configure(&profile));
    assert((registers[0x64] & 7) == i + 1);
    assert((registers[0x62] & 31) == 8 + (limits[i] - 200) / 100);
    profile.current_ma += 100;
    reset();
    assert(!pmic_charge_configure(&profile) && operations == 0);
  }
  const unsigned currents[] = {50, 175, 200, 300};
  const unsigned settings[] = {2, 7, 8, 9};
  for (unsigned i = 0; i < 4; i++) {
    pmic_charge_config profile = {4200, currents[i], 25};
    reset();
    assert(pmic_charge_configure(&profile));
    assert((registers[0x62] & 31) == settings[i]);
  }
}

static void test_invalid_charge_profile(void) {
  const pmic_charge_config invalid[] = {
      {4200, 501, 50},  {4300, 500, 50},  {4200, 500, 0},   {4200, 0, 50},
      {4200, 1500, 50}, {4350, 1200, 50}, {4400, 1100, 50}, {4200, 500, 51}};
  for (unsigned i = 0; i < sizeof(invalid) / sizeof(*invalid); i++) {
    reset();
    assert(!pmic_charge_configure(&invalid[i]) && operations == 0);
  }
  assert(!pmic_charge_configure(NULL));
}

static void test_charge_write_failures(void) {
  reset();
  assert(pmic_charge_configure(&config));
  unsigned configuration_operations = operations;
  for (unsigned failure = 1; failure <= configuration_operations; failure++) {
    reset();
    fail_at = failure;
    assert(!pmic_charge_configure(&config));
    // Initial read failure leaves hardware untouched; later failures disable
    // charge.
    if (failure == 1)
      assert(writes == 0);
    else
      assert(!(registers[0x18] & 2));
  }
  reset();
  corrupt_readback = true;
  assert(!pmic_charge_configure(&config) && !(registers[0x18] & 2));
}

static void test_percentage(void) {
  reset();
  unsigned percentage = 123;
  assert(pmic_percentage(&percentage) && percentage == 87 && writes == 0);
  for (unsigned failure = 1; failure <= 3; failure++) {
    reset();
    fail_at = failure;
    percentage = 123;
    assert(!pmic_percentage(&percentage) && percentage == 123 && writes == 0);
  }
  reset();
  registers[0] = 0;
  assert(!pmic_percentage(&percentage));
  reset();
  registers[0x18] &= ~8;
  assert(!pmic_percentage(&percentage));
  const unsigned percentages[] = {0, 100, 101, 255};
  for (unsigned i = 0; i < sizeof(percentages) / sizeof(*percentages); i++) {
    reset();
    registers[0xa4] = percentages[i];
    percentage = 123;
    assert(pmic_percentage(&percentage) == (percentages[i] <= 100));
    assert(percentage == (percentages[i] <= 100 ? percentages[i] : 123));
  }
  assert(!pmic_percentage(NULL));
}

static void test_output_masks(void) {
  reset();
  assert(!pmic_aldo_voltage(0, 3300));
  assert(!pmic_aldo_voltage(3, 3350));
  assert(!pmic_aldo_enable(5, true));
  assert(!pmic_outputs_disable(0x80, 0, false));
  assert(writes == 0);
  registers[0x94] = 0xe0;
  assert(pmic_aldo_voltage(3, 3300) && registers[0x94] == 0xfc);
  registers[0x90] = 1;
  assert(pmic_aldo_enable(3, true) && registers[0x90] == 5);
  assert(pmic_aldo_enable(3, false) && registers[0x90] == 1);
  registers[0x80] = 0xff;
  registers[0x90] = 0xff;
  registers[0x91] = 0xff;
  assert(pmic_outputs_disable(PMIC_DCDC2 | PMIC_DCDC3 | PMIC_DCDC4 | PMIC_DCDC5,
                              PMIC_ALDO2 | PMIC_ALDO4 | PMIC_BLDO1 | PMIC_BLDO2 |
                                  PMIC_CPUSLDO | PMIC_DLDO1,
                              true));
  assert(registers[0x80] == 0xe1 && registers[0x90] == 5 &&
         registers[0x91] == 0xfe);
}

static void test_automatic_regulator(void) {
  reset();
  registers[0x80] = 0xff;
  registers[0x81] = 0xff;
  assert(pmic_dcdc1_auto());
  assert(registers[0x80] == 0xbf && registers[0x81] == 0xfb);
  for (unsigned fail = 1; fail <= 6; fail++) {
    reset();
    fail_at = fail;
    assert(!pmic_dcdc1_auto());
  }
}

static void test_interrupts_disable(void) {
  reset();
  const uint8_t masks[] = {0xff, 0xff, 0xdf};
  for (unsigned i = 0; i < 3; i++) {
    registers[0x40 + i] = masks[i];
    registers[0x48 + i] = masks[i];
  }
  assert(pmic_interrupts_disable());
  for (unsigned i = 0; i < 3; i++) {
    assert(registers[0x40 + i] == 0);
    assert(registers[0x48 + i] == 0);
  }
  assert(registers[0x18] == 0x0a && registers[0x30] == 3);
  reset();
  fail_at = 1;
  registers[0x41] = 0xff;
  registers[0x42] = 0xdf;
  assert(!pmic_interrupts_disable());
  assert(registers[0x41] == 0 && registers[0x42] == 0);
}

static void test_adc_and_temperature_selection(void) {
  reset();
  registers[0x30] = 0xff;
  const uint8_t unused = PMIC_ADC_VBUS | PMIC_ADC_VSYS |
                         PMIC_ADC_DIE_TEMPERATURE | PMIC_ADC_GENERAL_PURPOSE;
  assert(pmic_adc_disable(unused) && registers[0x30] == 0xc3);
  assert(pmic_adc_disable(unused | PMIC_ADC_TS) && registers[0x30] == 0xc1);
  assert(!pmic_adc_disable(0x80));
  bool enabled = true;
  registers[0x50] = 0x12;
  assert(pmic_ts_charger_monitoring_enabled(&enabled) && !enabled);
  registers[0x50] = 0x0a;
  assert(pmic_ts_charger_monitoring_enabled(&enabled) && enabled);
  assert(!pmic_ts_charger_monitoring_enabled(NULL));
  fail_at = operations + 1;
  assert(!pmic_ts_charger_monitoring_enabled(&enabled) && enabled);
}

int main(void) {
  test_charge_profile();
  test_charge_limits();
  test_invalid_charge_profile();
  test_charge_write_failures();
  test_percentage();
  test_output_masks();
  test_automatic_regulator();
  test_interrupts_disable();
  test_adc_and_temperature_selection();
}
