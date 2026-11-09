#include "board.h"

#include "accel.h"
#include "audio.h"
#include "board_power.h"
#include "boot.h"
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

static bool init_display(void) {
  qspi_init(PANEL_CLOCK, PANEL_DATA0, PANEL_CS);
  // ALDO3 drives LCD_RESET on this board, not the panel power input
  if (!pmic_aldo_voltage(3, 3300) || !pmic_aldo_enable(3, true)) return false;
  delay_us(100000);
  if (!pmic_aldo_enable(3, false)) return false;
  delay_us(100000);
  if (!pmic_aldo_enable(3, true)) return false;
  delay_us(100000);
  display_init();
  return true;
}

static bool init_touch(void) {
  gpio_init(TOUCH_IRQ, GPIO_INPUT, GPIO_PULLUP_ENABLED);
  gpio_init(TOUCH_RESET, GPIO_OUTPUT, GPIO_PULLUP_DISABLED);
  gpio_set(TOUCH_RESET, GPIO_LOW);
  delay_us(200000);
  gpio_set(TOUCH_RESET, GPIO_HIGH);
  delay_us(200000);
  return touch_probe();
}

static bool disable_audio(void) {
  bool disabled = audio_disable();
  // Park I2S lines low even if chip shutdown failed
  const unsigned outputs[] = {AUDIO_MCLK, AUDIO_BCLK, AUDIO_LRCLK,
                              AUDIO_DATA_OUT};
  for (unsigned i = 0; i < sizeof(outputs) / sizeof(outputs[0]); i++) {
    gpio_set(outputs[i], GPIO_LOW);
    gpio_init(outputs[i], GPIO_OUTPUT, GPIO_PULLUP_DISABLED);
  }
  return disabled;
}

BOOT_ONLY void board_init(void) {
  watchdogs_disable();
  timer_init();
  pmu_init();
  watchdog_arm();
  radios_disable();
  unused_peripheral_clocks_disable();
  i2c_init(I2C_SCL, I2C_SDA);
  gpio_init(BUTTON_BOOT, GPIO_INPUT, GPIO_PULLUP_ENABLED);
  gpio_init(BUTTON_KEY, GPIO_INPUT, GPIO_PULLUP_ENABLED);
  // Deselect the card before enabling the output, avoiding a low pulse
  gpio_set(SD_CS, GPIO_HIGH);
  gpio_init(SD_CS, GPIO_OUTPUT, GPIO_PULLUP_ENABLED);
  if (!board_power_configure()) usb_puts("ERR POWER INIT\n");
  if (!disable_audio()) usb_puts("ERR AUDIO OFF\n");
  if (!init_display()) usb_puts("ERR PANEL INIT\n");
  if (!init_touch()) usb_puts("ERR TOUCH INIT\n");
  if (!accel_init()) usb_puts("ERR ACCEL INIT\n");
  if (!rtc_clock_output_disable()) usb_puts("ERR RTC CLKOUT\n");
  events_init();
}

void board_resume(void) {
  board_power_resume();

  // TOP-off wake resets GPIO routing/enables
  // Restore buses and safe output levels without resetting external devices
  static const struct {
    uint8_t pin;
    bool level, pullup;
  } outputs[] = {
      {PANEL_CLOCK, GPIO_LOW, GPIO_PULLUP_DISABLED},
      {PANEL_DATA0, GPIO_LOW, GPIO_PULLUP_DISABLED},
      {PANEL_DATA1, GPIO_LOW, GPIO_PULLUP_DISABLED},
      {PANEL_DATA2, GPIO_LOW, GPIO_PULLUP_DISABLED},
      {PANEL_DATA3, GPIO_LOW, GPIO_PULLUP_DISABLED},
      {SD_CS, GPIO_HIGH, GPIO_PULLUP_ENABLED},
      {TOUCH_RESET, GPIO_HIGH, GPIO_PULLUP_DISABLED},
      {PANEL_CS, GPIO_HIGH, GPIO_PULLUP_DISABLED},
      {AUDIO_MCLK, GPIO_LOW, GPIO_PULLUP_DISABLED},
      {AUDIO_BCLK, GPIO_LOW, GPIO_PULLUP_DISABLED},
      {AUDIO_LRCLK, GPIO_LOW, GPIO_PULLUP_DISABLED},
      {AUDIO_DATA_OUT, GPIO_LOW, GPIO_PULLUP_DISABLED},
  };

  for (unsigned i = 0; i < sizeof(outputs) / sizeof(outputs[0]); i++) {
    gpio_set(outputs[i].pin, outputs[i].level);
    gpio_init(outputs[i].pin, GPIO_OUTPUT, outputs[i].pullup);
  }
  gpio_init(BUTTON_BOOT, GPIO_INPUT, GPIO_PULLUP_ENABLED);
  gpio_init(BUTTON_KEY, GPIO_INPUT, GPIO_PULLUP_ENABLED);
  gpio_init(TOUCH_IRQ, GPIO_INPUT, GPIO_PULLUP_ENABLED);
  // Restore output routing before releasing the retained pad levels
  gpio_release_holds();
  i2c_init(I2C_SCL, I2C_SDA);
  events_init();
}

void board_sleep(void) {
  display_sleep();
  board_power_sleep();
}
