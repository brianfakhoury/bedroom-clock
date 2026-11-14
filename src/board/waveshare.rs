//! Waveshare ESP32-C6 Touch AMOLED 2.16: device startup, panel and touch
//! reset, audio pin parking and sleep/wake sequencing.

use super::pins::*;
use super::{Board, Events, events, power};
use crate::drivers::accel::Accel;
use crate::drivers::display::{Display, Panel};
use crate::drivers::rtc::DateTime;
use crate::drivers::{Error, audio, pmic, rtc, touch};
use crate::hal::gpio::{self, Level::*, Mode::*, Pull};
use crate::hal::i2c::{I2c, SoftI2c};
use crate::hal::qspi::SoftQspi;
use crate::hal::timer::{self, Delay, Timer};
use crate::hal::{pmu, radio, usb, watchdog};

pub struct Waveshare {
    i2c: SoftI2c,
    display: Display<SoftQspi, Timer>,
    accel: Option<Accel>,
}

impl Waveshare {
    pub const fn new() -> Self {
        Self {
            i2c: SoftI2c::new(I2C_SCL, I2C_SDA),
            display: Display::new(SoftQspi::new(PANEL_CLOCK, PANEL_DATA0, PANEL_CS), Timer),
            accel: None,
        }
    }

    /// Cold boot: reset and configure every device. Failures are reported
    /// over USB and startup continues
    #[cfg_attr(target_os = "none", unsafe(link_section = ".init.code"))]
    #[inline(never)]
    pub fn init(&mut self) {
        watchdog::disable_all();
        timer::init();
        pmu::init();
        watchdog::arm();
        radio::disable();
        power::unused_peripheral_clocks_disable();
        self.i2c.init();
        BUTTON_BOOT.configure(Input, Pull::Up);
        BUTTON_KEY.configure(Input, Pull::Up);
        // Deselect the card before enabling the output, avoiding a low pulse
        SD_CS.set(High);
        SD_CS.configure(Output, Pull::Up);
        report(power::configure(&mut self.i2c), "ERR POWER INIT\n");
        report(disable_audio(&mut self.i2c), "ERR AUDIO OFF\n");
        report(self.init_display(), "ERR PANEL INIT\n");
        report(init_touch(&mut self.i2c), "ERR TOUCH INIT\n");
        self.accel = report(Accel::discover(&mut self.i2c), "ERR ACCEL INIT\n");
        report(rtc::clock_output_disable(&mut self.i2c), "ERR RTC CLKOUT\n");
        events::init();
    }

    /// Wake: TOP-off sleep reset GPIO routing and enables. Restore buses and
    /// safe output levels without resetting external devices
    pub fn resume(&mut self) {
        power::resume();

        static OUTPUTS: [(gpio::Pin, gpio::Level, Pull); 12] = [
            (PANEL_CLOCK, Low, Pull::None),
            (PANEL_DATA0, Low, Pull::None),
            (PANEL_DATA1, Low, Pull::None),
            (PANEL_DATA2, Low, Pull::None),
            (PANEL_DATA3, Low, Pull::None),
            (SD_CS, High, Pull::Up),
            (TOUCH_RESET, High, Pull::None),
            (PANEL_CS, High, Pull::None),
            (AUDIO_MCLK, Low, Pull::None),
            (AUDIO_BCLK, Low, Pull::None),
            (AUDIO_LRCLK, Low, Pull::None),
            (AUDIO_DATA_OUT, Low, Pull::None),
        ];
        for &(pin, level, pull) in &OUTPUTS {
            pin.set(level);
            pin.configure(Output, pull);
        }
        BUTTON_BOOT.configure(Input, Pull::Up);
        BUTTON_KEY.configure(Input, Pull::Up);
        TOUCH_IRQ.configure(Input, Pull::Up);
        // Restore output routing before releasing the retained pad levels
        gpio::release_holds();
        self.i2c.init();
        events::init();
    }

    fn init_display(&mut self) -> Result<(), Error> {
        self.display.bus().init();
        reset_panel(&mut self.i2c, &mut Timer)?;
        self.display.init();
        Ok(())
    }
}

// Startup continues past a failed device
fn report<T>(outcome: Result<T, Error>, message: &str) -> Option<T> {
    if outcome.is_err() {
        usb::puts(message);
    }
    outcome.ok()
}

/// ALDO3 drives LCD_RESET on this board, not the panel power input
pub fn reset_panel(bus: &mut impl I2c, delay: &mut impl Delay) -> Result<(), Error> {
    pmic::aldo_voltage(bus, 3, 3300)?;
    for released in [true, false, true] {
        pmic::aldo_enable(bus, 3, released)?;
        delay.delay_us(100_000);
    }
    Ok(())
}

fn init_touch(bus: &mut impl I2c) -> Result<(), Error> {
    TOUCH_IRQ.configure(Input, Pull::Up);
    TOUCH_RESET.configure(Output, Pull::None);
    TOUCH_RESET.set(Low);
    timer::delay_us(200_000);
    TOUCH_RESET.set(High);
    timer::delay_us(200_000);
    touch::probe(bus)
}

fn disable_audio(bus: &mut impl I2c) -> Result<(), Error> {
    let disabled = audio::disable(bus);
    // Park I2S lines low even if chip shutdown failed
    for pin in [AUDIO_MCLK, AUDIO_BCLK, AUDIO_LRCLK, AUDIO_DATA_OUT] {
        pin.set(Low);
        pin.configure(Output, Pull::None);
    }
    disabled
}

impl Board for Waveshare {
    type Panel = Display<SoftQspi, Timer>;

    fn panel(&mut self) -> &mut Self::Panel {
        &mut self.display
    }

    fn now(&self) -> u64 {
        timer::ticks()
    }

    fn wait_until(&mut self, deadline: Option<u64>) -> Events {
        events::wait_until(deadline)
    }

    fn sleep(&mut self) {
        self.display.sleep();
        power::sleep();
    }

    fn feed_watchdog(&mut self) {
        watchdog::feed();
    }

    fn acceleration(&mut self) -> Option<[i16; 3]> {
        self.accel.as_ref()?.read(&mut self.i2c, &mut Timer).ok()
    }

    fn battery_percentage(&mut self) -> Option<u32> {
        pmic::percentage(&mut self.i2c).ok()
    }

    fn touching(&mut self) -> Option<bool> {
        touch::read(&mut self.i2c).ok()
    }

    fn utc(&mut self) -> Option<DateTime> {
        rtc::read(&mut self.i2c).ok()
    }

    fn set_utc(&mut self, utc: &DateTime) -> bool {
        rtc::set(&mut self.i2c, utc).is_ok()
    }

    fn usb_read(&mut self) -> Option<u8> {
        usb::get()
    }

    fn usb_write(&mut self, text: &str) {
        usb::puts(text);
    }
}
