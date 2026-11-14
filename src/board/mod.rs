//! Choices for this board: pin assignments, battery settings, which devices
//! stay powered, and their startup and sleep order.

pub mod battery;
pub mod events;
pub mod pins;
pub mod power;
pub mod waveshare;

pub use events::Events;
pub use waveshare::Waveshare;

use crate::drivers::display::Panel;
use crate::drivers::rtc::DateTime;

/// Everything the application asks of the hardware
pub trait Board {
    type Panel: Panel;

    fn panel(&mut self) -> &mut Self::Panel;
    /// Monotonic time in timer ticks
    fn now(&self) -> u64;
    /// Wait for an event or the absolute deadline in ticks; without a
    /// deadline, wait indefinitely
    fn wait_until(&mut self, deadline: Option<u64>) -> Events;
    /// Sleep the panel and processor. Returns only if a pending touch cancels
    /// sleep; otherwise the next wake restarts the application
    fn sleep(&mut self);
    /// Promise that the application is still making progress
    fn feed_watchdog(&mut self);

    /// Averaged x, y, z in counts of 1/16384 g
    fn acceleration(&mut self) -> Option<[i16; 3]>;
    /// Raw gauge estimate, 0–100
    fn battery_percentage(&mut self) -> Option<u32>;
    /// Whether a finger is on the screen
    fn touching(&mut self) -> Option<bool>;
    fn utc(&mut self) -> Option<DateTime>;
    /// Returns whether the clock accepted the time
    fn set_utc(&mut self, utc: &DateTime) -> bool;

    /// The next byte from the USB host, if any
    fn usb_read(&mut self) -> Option<u8>;
    fn usb_write(&mut self, text: &str);
}
