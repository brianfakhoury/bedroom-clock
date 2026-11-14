//! Waveshare ESP32-C6-AMOLED-2.16 only. CS/IRQ differ from some vendor examples.

use crate::hal::gpio::Pin;

pub const PANEL_CLOCK: Pin = Pin(0);
pub const PANEL_DATA0: Pin = Pin(1);
pub const PANEL_DATA1: Pin = Pin(2);
pub const PANEL_DATA2: Pin = Pin(3);
pub const PANEL_DATA3: Pin = Pin(4);
pub const PANEL_CS: Pin = Pin(15);
pub const TOUCH_IRQ: Pin = Pin(5);
pub const SD_CS: Pin = Pin(6);
pub const I2C_SCL: Pin = Pin(7);
pub const I2C_SDA: Pin = Pin(8);
pub const BUTTON_BOOT: Pin = Pin(9);
pub const BUTTON_KEY: Pin = Pin(10);
pub const TOUCH_RESET: Pin = Pin(11);
pub const AUDIO_MCLK: Pin = Pin(19);
pub const AUDIO_BCLK: Pin = Pin(20);
pub const AUDIO_LRCLK: Pin = Pin(22);
pub const AUDIO_DATA_OUT: Pin = Pin(23);
