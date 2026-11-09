#include <assert.h>
#include <setjmp.h>

#include "board_power.h"
#include "pmic.h"
#include "registers.h"
#include "sleep.h"

// Observe the selected clock gates without accessing MCU registers.
static uint32_t clock_registers[64];

static uint32_t* clock_register(uint32_t address) {
  assert(address >= 0x60096000 && address < 0x60096100 && !(address & 3));
  return &clock_registers[(address - 0x60096000) / 4];
}

#undef REG
#define REG(address) (*clock_register(address))
#include "../../src/board/board_power.c"

enum {
  CHARGE,
  DISCHARGE,
  AUDIO_RAIL,
  UNUSED_RAILS,
  REGULATOR,
  LED,
  IRQ,
  TS,
  ADC,
  COUNT
};

static unsigned calls;
static int failure;
static bool ts_enabled, ready;
static unsigned preparations, sleeps, resumes;
static jmp_buf asleep;
char _crc_end[1];

void wake_entry(void) {}

static bool called(unsigned operation) {
  assert(!(calls & (1u << operation)));
  calls |= 1u << operation;
  return (int)operation != failure;
}

bool pmic_charge_configure(const pmic_charge_config* c) {
  assert(c->voltage_mv == 4200 && c->current_ma == 500 &&
         c->termination_ma == 50);
  return called(CHARGE);
}

bool pmic_output_discharge_enable(void) { return called(DISCHARGE); }

bool pmic_aldo_enable(unsigned channel, bool enabled) {
  assert(channel == 1 && enabled);
  return called(AUDIO_RAIL);
}

bool pmic_outputs_disable(uint8_t dcdc, uint8_t ldo, bool dldo2) {
  assert(dcdc == 0x1e && ldo == 0xfa && dldo2);
  return called(UNUSED_RAILS);
}

bool pmic_dcdc1_auto(void) { return called(REGULATOR); }

bool pmic_led_off(void) { return called(LED); }

bool pmic_interrupts_disable(void) { return called(IRQ); }

bool pmic_ts_charger_monitoring_enabled(bool* enabled) {
  if (!called(TS)) return false;
  *enabled = ts_enabled;
  return true;
}

bool pmic_adc_disable(uint8_t channels) {
  assert(channels == (ts_enabled ? 0x3c : 0x3e));
  return called(ADC);
}

static void enable_clocks(void) {
  for (unsigned i = 0; i < 64; i++) clock_registers[i] = ~0u;
}

static void check_clocks(void) {
  assert(REG(0x60096000) == (~0u & ~BIT(0)));   // UART0 register clock.
  assert(REG(0x60096004) == (~0u & ~BIT(22)));  // UART0 functional clock.
  assert(REG(0x60096080) == (~0u & ~(BIT(0) | BIT(2))));  // ADC gates.
  assert(REG(0x6009608c) == ~0u);  // USB must stay available.
}

bool sleep_prepare(unsigned pin) {
  assert(pin == 5);
  preparations++;
  if (!ready) enable_clocks();  // Cancelled sleep restores MCU execution first.
  return ready;
}

void sleep_resume(unsigned pin) {
  assert(pin == 5);
  enable_clocks();
  resumes++;
}

void sleep_enter(void (*entry)(void), unsigned retained_bytes,
                 uint32_t high_outputs, uint32_t held_outputs) {
  (void)retained_bytes;
  assert(ready && entry == wake_entry && high_outputs == 0x07);
  assert(held_outputs == (0x1f | BIT(6) | BIT(11) | BIT(15) | BIT(19) |
                          BIT(20) | BIT(22) | BIT(23)));
  sleeps++;
  longjmp(asleep, 1);
}

static void test_power_configuration(void) {
  failure = -1;
  for (unsigned ts = 0; ts < 2; ts++) {
    calls = 0;
    ts_enabled = ts;
    assert(board_power_configure() && calls == (1u << COUNT) - 1);
  }
  calls = 0;
  failure = CHARGE;
  assert(!board_power_configure() && calls == (1u << COUNT) - 1);
  calls = 0;
  failure = TS;
  assert(!board_power_configure());
  assert(calls == (((1u << COUNT) - 1) & ~BIT(ADC)));
}

static void test_clock_policy_on_boot_wake_and_cancel(void) {
  enable_clocks();
  unused_peripheral_clocks_disable();
  check_clocks();
  board_power_sleep();
  assert(preparations == 1 && sleeps == 0);
  check_clocks();
  ready = true;
  if (!setjmp(asleep)) {
    board_power_sleep();
    assert(false);
  }
  assert(sleeps == 1);
  board_power_resume();
  assert(resumes == 1);
  check_clocks();
}

int main(void) {
  test_power_configuration();
  test_clock_policy_on_boot_wake_and_cancel();
}
