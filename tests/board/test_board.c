#include <assert.h>
#include <string.h>

#include "accel.h"
#include "audio.h"
#include "board.h"
#include "board_power.h"
#include "display.h"
#include "events.h"
#include "gpio.h"
#include "i2c.h"
#include "pins.h"
#include "pmic.h"
#include "power.h"
#include "qspi.h"
#include "rtc.h"
#include "timer.h"
#include "touch.h"
#include "usb.h"
#include "watchdog.h"

#include "../../src/board/board.c"

enum { POWER, AUDIO, PANEL, TOUCH, IMU, RTC, EVENTS, COUNT };

static unsigned calls, errors, panel_step, touch_step, audio_low, audio_outputs;
static unsigned sleep_steps;
static int failure, panel_failure;
#define AUDIO_PINS ((1u << 19) | (1u << 20) | (1u << 22) | (1u << 23))

static bool called(unsigned operation) {
  assert(!(calls & (1u << operation)));
  calls |= 1u << operation;
  return (int)operation != failure;
}

bool board_power_configure(void) { return called(POWER); }

bool audio_disable(void) { return called(AUDIO); }

void qspi_init(unsigned clock, unsigned data, unsigned cs) {
  assert(clock == 0 && data == 1 && cs == 15 && panel_step++ == 0);
}

bool pmic_aldo_voltage(unsigned channel, unsigned mv) {
  assert(channel == 3 && mv == 3300 && panel_step == 1);
  return (int)panel_step++ != panel_failure;
}

bool pmic_aldo_enable(unsigned channel, bool enabled) {
  assert(channel == 3);
  assert(panel_step == 2 || panel_step == 4 || panel_step == 6);
  assert(enabled == (panel_step != 4));
  return (int)panel_step++ != panel_failure;
}

void display_init(void) {
  assert(panel_step == 8);
  called(PANEL);
}

bool touch_probe(void) {
  assert(touch_step == 6);
  return called(TOUCH);
}

bool accel_init(void) { return called(IMU); }

bool rtc_clock_output_disable(void) { return called(RTC); }

void events_init(void) {
  assert(calls & (1u << RTC));
  called(EVENTS);
}

void usb_puts(const char* message) {
  assert(strncmp(message, "ERR ", 4) == 0);
  errors++;
}

static unsigned startup_step;

void watchdogs_disable(void) { assert(startup_step++ == 0); }

void timer_init(void) { assert(startup_step++ == 1); }

void pmu_init(void) { assert(startup_step++ == 2); }

void watchdog_arm(void) {
  assert(startup_step++ == 3);
  startup_step = 0;
}

void radios_disable(void) {}

void unused_peripheral_clocks_disable(void) {}

void board_power_resume(void) {}

void gpio_release_holds(void) {}

void display_sleep(void) { assert(sleep_steps++ == 0); }

void board_power_sleep(void) { assert(sleep_steps++ == 1); }

void i2c_init(unsigned clock, unsigned data) {
  assert(clock == 7 && data == 8);
}

void gpio_init(unsigned pin, bool output, bool pullup) {
  if ((1u << pin) & AUDIO_PINS) {
    assert(output && !pullup && (audio_low & (1u << pin)));
    audio_outputs |= 1u << pin;
  }
  if (pin == TOUCH_IRQ) {
    assert(!output && pullup && touch_step++ == 0);
  }
  if (pin == TOUCH_RESET) {
    assert(output && !pullup && touch_step++ == 1);
  }
}

void gpio_set(unsigned pin, bool high) {
  if ((1u << pin) & AUDIO_PINS) {
    assert(!high && (calls & (1u << AUDIO)));
    audio_low |= 1u << pin;
  }
  if (pin == TOUCH_RESET) {
    assert(touch_step == (high ? 4u : 2u));
    touch_step++;
  }
}

void delay_us(uint32_t us) {
  if (us == 200000) {
    assert(touch_step == 3 || touch_step == 5);
    touch_step++;
  } else {
    assert(us == 100000);
    assert(panel_step == 3 || panel_step == 5 || panel_step == 7);
    panel_step++;
  }
}

static void start(int fail, int reset_failure) {
  calls = errors = panel_step = touch_step = audio_low = audio_outputs = 0;
  failure = fail;
  panel_failure = reset_failure;
  board_init();
  assert(audio_outputs == AUDIO_PINS);
}

int main(void) {
  start(-1, -1);
  assert(calls == (1u << COUNT) - 1 && errors == 0);
  const int failures[] = {POWER, AUDIO, TOUCH};
  for (unsigned i = 0; i < sizeof(failures) / sizeof(*failures); i++) {
    start(failures[i], -1);
    assert(calls == (1u << COUNT) - 1 && errors == 1);
  }
  const int reset_failures[] = {1, 2, 4, 6};
  const unsigned count = sizeof(reset_failures) / sizeof(*reset_failures);
  for (unsigned i = 0; i < count; i++) {
    start(-1, reset_failures[i]);
    assert(panel_step == (unsigned)reset_failures[i] + 1);
    assert(calls == (((1u << COUNT) - 1) & ~(1u << PANEL)) && errors == 1);
  }
  board_sleep();
  assert(sleep_steps == 2);
}
