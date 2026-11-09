#pragma once

// Waveshare ESP32-C6-AMOLED-2.16 only. CS/IRQ differ from some vendor examples
enum {
  PANEL_CLOCK = 0,
  PANEL_DATA0 = 1,
  PANEL_DATA1 = 2,
  PANEL_DATA2 = 3,
  PANEL_DATA3 = 4,
  PANEL_CS = 15,
  TOUCH_IRQ = 5,
  SD_CS = 6,
  I2C_SCL = 7,
  I2C_SDA = 8,
  BUTTON_BOOT = 9,
  BUTTON_KEY = 10,
  TOUCH_RESET = 11,
  AUDIO_MCLK = 19,
  AUDIO_BCLK = 20,
  AUDIO_LRCLK = 22,
  AUDIO_DATA_OUT = 23,
};
