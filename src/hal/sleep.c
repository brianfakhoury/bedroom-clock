#include "sleep.h"

#include "gpio.h"
#include "registers.h"
#include "timer.h"
#include "usb.h"
#include "watchdog.h"

// Fixed ROM functions from ESP-IDF's esp32c6.rom.ld (see docs/references)
static void (*const rom_set_wake_entry)(void (*)(void), unsigned) =
    (void (*)(void (*)(void), unsigned))0x400000a4;
static unsigned (*const rom_get_wake_entry)(void) =
    (unsigned (*)(void))0x400000a8;

void sleep_resume(unsigned wake_pin) {
  REG(LP_TIMER_TARGET_HIGH) = 0;
  REG(LP_TIMER_CLEAR) = BIT(31);
  REG(LP_GPIO_MUX) &= ~BIT(wake_pin);
  REG(LP_IO_PIN0 + wake_pin * 4) = BIT(3);
  REG(LP_IO_CLEAR) = BIT(wake_pin);
  REG(RTC_SLEEP_MODE) &= ~1u;
  watchdogs_disable();
  timer_init();
  usb_resume();
  watchdog_arm();
  gpio_init(wake_pin, GPIO_INPUT, GPIO_PULLUP_ENABLED);
}

static void arm_wake_pin(unsigned wake_pin) {
  REG(LP_TIMER_TARGET_HIGH) = 0;
  REG(LP_TIMER_CLEAR) = BIT(31);
  REG(LPPERI_CLK_EN) |= BIT(27);
  REG(LP_IO_DATE) |= BIT(31);
  REG(LP_IO_OUT_ENABLE_W1TC) = BIT(wake_pin);
  REG(LP_IO_PAD0 + wake_pin * 4) = BIT(9) | BIT(8) | BIT(4) | BIT(3);
  REG(LP_IO_CLEAR) = BIT(wake_pin);
  REG(LP_IO_PIN0 + wake_pin * 4) = BIT(10) | (2u << 7) | BIT(3);
  REG(LP_GPIO_MUX) |= BIT(wake_pin);
}

bool sleep_prepare(unsigned wake_pin) {
  arm_wake_pin(wake_pin);

  // Cancel sleep if an edge arrived before or during the switch to LP GPIO
  if ((REG(GPIO_STATUS) | REG(LP_IO_STATUS)) & BIT(wake_pin)) {
    REG(GPIO_STATUS_CLEAR) = BIT(wake_pin);
    sleep_resume(wake_pin);
    return false;
  }
  return true;
}

void sleep_enter(void (*entry)(void), unsigned retained_bytes,
                 uint32_t high_outputs, uint32_t held_outputs) {
  REG(RTC_SLEEP_MODE) |= 1u;
  REG(LP_AON_SYS_CFG) &= ~BIT(30);
  // ROM uses this address and checksum to resume from LP RAM after deep sleep
  rom_set_wake_entry(entry, retained_bytes);
  // Let the watchdog reset if ROM rejects the saved wake entry
  if (rom_get_wake_entry() != (unsigned)entry)
    while (true) {
    }
  REG(PMU_WAKEUP_CNTL1) = 0;
  REG(PMU_WAKEUP_CNTL2) = BIT(2);
  REG(PMU_HP_INT_CLR) = BIT(31) | BIT(30);
  REG(PMU_WAKEUP_CNTL4) = BIT(31);
  usb_disconnect();
  delay_us(100000);
  REG(GPIO_OUT_SET) = high_outputs;
  REG(LP_GPIO_HOLD) = held_outputs;
  watchdog_feed();
  __asm__ volatile("fence iorw, iorw" ::: "memory");
  REG(PMU_WAKEUP_CNTL0) = BIT(31);
  // Deep-sleep wake restarts through ROM, not this stack
  while (true) {
  }
}
