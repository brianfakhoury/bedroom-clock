//! Stand-ins for the hardware below the code under test.

use std::cell::Cell;
use std::collections::VecDeque;
use std::panic::{self, AssertUnwindSafe};

use crate::app::battery_indicator::BatteryIndicator;
use crate::app::digits::{self, Digits};
use crate::board::{Board, Events};
use crate::drivers::display::{HEIGHT, Panel, Rect, WIDTH};
use crate::drivers::rtc::DateTime;
use crate::hal::timer::{Delay, TICKS_PER_MS};

/// Adds up requested delays instead of waiting. A driver may own a shared
/// reference, leaving the total readable by the test
#[derive(Default)]
pub struct Elapsed(Cell<u32>);

impl Elapsed {
    pub fn us(&self) -> u32 {
        self.0.get()
    }
}

impl Delay for &Elapsed {
    fn delay_us(&mut self, us: u32) {
        self.0.set(self.0.get() + us);
    }
}

/// A framebuffer that enforces the panel's window rules
pub struct FakePanel {
    pub pixels: Vec<u16>,
    pub windows: u32,
    pub transferred: u32,
    pub on_count: u32,
    pub off_count: u32,
    pub level: u8,
    pub sleeping: bool,
}

impl FakePanel {
    pub fn new() -> Self {
        Self {
            pixels: vec![0; (WIDTH * HEIGHT) as usize],
            windows: 0,
            transferred: 0,
            on_count: 0,
            off_count: 0,
            level: 0,
            sleeping: false,
        }
    }

    pub fn pixel(&self, x: u32, y: u32) -> u16 {
        self.pixels[(y * WIDTH + x) as usize]
    }

    pub fn is_blank(&self) -> bool {
        self.pixels.iter().all(|&pixel| pixel == 0)
    }
}

impl Panel for FakePanel {
    fn fill(&mut self, area: Rect, color: u16) {
        self.pattern(area, |_, _| color);
    }

    fn pattern(&mut self, area: Rect, pixel: impl Fn(u32, u32) -> u16) {
        let Rect { x, y, width, height } = area;
        assert!(width > 0 && height > 0 && x + width <= WIDTH && y + height <= HEIGHT);
        assert!((x | y | width | height) & 1 == 0, "odd window {area:?}");
        self.windows += 1;
        self.transferred += width * height;
        for row in y..y + height {
            for column in x..x + width {
                self.pixels[(row * WIDTH + column) as usize] = pixel(column, row);
            }
        }
    }

    fn on(&mut self) {
        self.on_count += 1;
    }

    fn off(&mut self) {
        self.off_count += 1;
    }

    fn brightness(&mut self, value: u8) {
        self.level = value;
    }

    fn sleep(&mut self) {
        self.sleeping = true;
    }

    fn wake(&mut self) {
        self.sleeping = false;
    }
}

/// The pixels of a panel showing this time and battery ring, drawn from blank
pub fn screen(
    time: Option<(u32, u32)>,
    rotation: u32,
    battery: Option<u32>,
    ring_rotation: u32,
) -> Vec<u16> {
    let mut panel = FakePanel::new();
    Digits::new().draw(&mut panel, time, rotation, digits::RED);
    BatteryIndicator::new().draw(&mut panel, battery, ring_rotation);
    panel.pixels
}

/// An event that arrives while the application waits
pub struct Arrival {
    pub at_ms: u64,
    pub events: Events,
    /// Finger contact from this moment on
    pub touching: bool,
}

/// The payload that unwinds out of `App::run` when the board really sleeps
pub struct Asleep;

/// A board whose devices are plain values, with time that passes only while
/// the application waits
pub struct FakeBoard {
    pub panel: FakePanel,
    pub now: u64,
    pub arrivals: VecDeque<Arrival>,
    pub acceleration: Option<[i16; 3]>,
    pub battery: Option<u32>,
    pub battery_reads: u32,
    pub touching: Option<bool>,
    pub utc: Option<DateTime>,
    pub utc_reads: u32,
    pub utc_writes: u32,
    pub rtc_writable: bool,
    pub usb_input: VecDeque<u8>,
    pub usb_output: String,
    pub sleeps: u32,
    pub cancelled_sleeps: u32,
    pub watchdog_feeds: u32,
}

impl FakeBoard {
    /// Upright, 75% battery, 08:00 Pacific, no finger on the screen
    pub fn new() -> Self {
        Self {
            panel: FakePanel::new(),
            now: 0,
            arrivals: VecDeque::new(),
            acceleration: Some([0, 16384, 0]),
            battery: Some(75),
            battery_reads: 0,
            touching: Some(false),
            utc: Some(utc_at_pacific_hour(8)),
            utc_reads: 0,
            utc_writes: 0,
            rtc_writable: true,
            usb_input: VecDeque::new(),
            usb_output: String::new(),
            sleeps: 0,
            cancelled_sleeps: 0,
            watchdog_feeds: 0,
        }
    }

    pub fn at(&mut self, ms: u64) {
        self.now = ms * TICKS_PER_MS as u64;
    }

    pub fn now_ms(&self) -> u64 {
        self.now / TICKS_PER_MS as u64
    }

    pub fn arrive(&mut self, at_ms: u64, events: Events, touching: bool) {
        self.arrivals.push_back(Arrival { at_ms, events, touching });
    }

    /// Run the application until the board sleeps for real
    pub fn run_until_asleep(&mut self, run: impl FnOnce(&mut Self)) {
        let outcome = panic::catch_unwind(AssertUnwindSafe(|| run(self)));
        let payload = outcome.expect_err("the application returned instead of sleeping");
        if !payload.is::<Asleep>() {
            panic::resume_unwind(payload);
        }
    }
}

/// A winter UTC time, so Pacific time is eight hours behind
pub fn utc_at_pacific_hour(hour: u32) -> DateTime {
    DateTime { year: 2026, month: 1, day: 15, hour: (hour + 8) % 24, minute: 0, second: 0 }
}

impl Board for FakeBoard {
    type Panel = FakePanel;

    fn panel(&mut self) -> &mut FakePanel {
        &mut self.panel
    }

    fn now(&self) -> u64 {
        self.now
    }

    fn wait_until(&mut self, deadline: Option<u64>) -> Events {
        let ticks = |ms: u64| ms * TICKS_PER_MS as u64;
        let next = self.arrivals.front().map(|arrival| ticks(arrival.at_ms));
        match (next, deadline) {
            (Some(at), deadline) if deadline.is_none_or(|deadline| at <= deadline) => {
                let arrival = self.arrivals.pop_front().unwrap();
                self.now = self.now.max(at);
                self.touching = Some(arrival.touching);
                arrival.events
            }
            (_, Some(deadline)) => {
                self.now = self.now.max(deadline);
                Events { timer: true, ..Events::default() }
            }
            (_, None) => panic!("waiting forever: no deadline and no arrivals"),
        }
    }

    fn sleep(&mut self) {
        self.panel.sleep();
        self.sleeps += 1;
        if self.cancelled_sleeps > 0 {
            self.cancelled_sleeps -= 1;
        } else {
            panic::resume_unwind(Box::new(Asleep));
        }
    }

    fn feed_watchdog(&mut self) {
        self.watchdog_feeds += 1;
    }

    fn acceleration(&mut self) -> Option<[i16; 3]> {
        self.acceleration
    }

    fn battery_percentage(&mut self) -> Option<u32> {
        // Never delay the initial visible time
        assert!(self.panel.level > 0 || self.panel.on_count == 0);
        self.battery_reads += 1;
        self.battery
    }

    fn touching(&mut self) -> Option<bool> {
        self.touching
    }

    fn utc(&mut self) -> Option<DateTime> {
        self.utc_reads += 1;
        self.utc
    }

    fn set_utc(&mut self, utc: &DateTime) -> bool {
        self.utc_writes += 1;
        if self.rtc_writable {
            self.utc = Some(*utc);
        }
        self.rtc_writable
    }

    fn usb_read(&mut self) -> Option<u8> {
        self.usb_input.pop_front()
    }

    fn usb_write(&mut self, text: &str) {
        self.usb_output.push_str(text);
    }
}
