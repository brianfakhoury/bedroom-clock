//! Presentation state, orientation, time reading, fades and display
//! deadlines. The external RTC keeps the time.

use super::animation::{self, DISPLAY_DURATION_MS, FADE_IN_MS, FADE_OUT_MS};
use super::battery_indicator::BatteryIndicator;
use super::digits::{self, Digits};
use super::{local_time, orientation};
use crate::board::Board;
use crate::drivers::display::Panel;
use crate::hal::timer::TICKS_PER_MS;

const TIME_REFRESH_TICKS: u64 = 1000 * TICKS_PER_MS as u64;

pub struct Clock {
    visible: bool,
    held: bool,
    ring_pending: bool,
    brightness: u32,
    rotation: u32,
    fade_from: u32,
    level: u32,
    shown_at: u64,
    time_refreshed_at: u64,
    digits: Digits,
    ring: BatteryIndicator,
}

impl Clock {
    pub const fn new(brightness: u32) -> Self {
        Self {
            visible: false,
            held: false,
            ring_pending: false,
            brightness,
            rotation: 0,
            fade_from: 0,
            level: 0,
            shown_at: 0,
            time_refreshed_at: 0,
            digits: Digits::new(),
            ring: BatteryIndicator::new(),
        }
    }

    /// Prime the ring pixels for the first fade-in
    pub fn init(&mut self, board: &mut impl Board) {
        self.refresh_ring(board);
    }

    /// Activation restarts the fade from the current brightness
    pub fn show(&mut self, board: &mut impl Board) {
        board.panel().wake();
        let rotated = self.update_orientation(board);
        self.fade_from = if self.visible && !rotated { self.level } else { 0 };
        self.level = self.fade_from;
        self.refresh_time(board);
        self.shown_at = board.now();
    }

    /// KEY holds the display on; a dark display cannot acquire a hold
    pub fn toggle_always_on(&mut self, board: &mut impl Board) {
        self.held = !self.held && self.visible;
        if self.held || !self.visible {
            return;
        }
        // Releasing an expired hold starts fade-out without another viewing interval
        let fade_start = ((DISPLAY_DURATION_MS - FADE_OUT_MS) * TICKS_PER_MS) as u64;
        if board.now() - self.shown_at > fade_start {
            self.shown_at = board.now() - fade_start;
        }
    }

    pub fn update(&mut self, board: &mut impl Board) {
        if !self.visible {
            return;
        }
        let elapsed = self.animation_elapsed_ms(board);
        let fading_in = elapsed < FADE_IN_MS;
        let fading_out = !self.held && elapsed >= DISPLAY_DURATION_MS - FADE_OUT_MS;
        // Refresh time without restarting the fade
        if self.held && board.now() - self.time_refreshed_at >= TIME_REFRESH_TICKS {
            self.update_orientation(board);
            self.refresh_time(board);
        }
        let animation_ms = if self.held && !fading_in { FADE_IN_MS } else { elapsed };
        let level = animation::level(animation_ms, self.fade_from, self.brightness);
        if level != self.level {
            board.panel().brightness(level as u8);
            self.level = level;
        }
        // Defer ring updates until the time is fully visible
        if self.ring_pending && !fading_in && !fading_out {
            self.refresh_ring(board);
        }
        if self.held || elapsed < DISPLAY_DURATION_MS {
            return;
        }
        board.panel().off();
        self.visible = false;
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    /// Next display update in absolute timer ticks, if one is scheduled
    pub fn next_update(&self, board: &impl Board) -> Option<u64> {
        if !self.visible {
            return None;
        }
        let elapsed = self.animation_elapsed_ms(board);
        if self.held && elapsed >= FADE_IN_MS {
            return Some(self.time_refreshed_at + TIME_REFRESH_TICKS);
        }
        // Keep display timing independent of time spent on other work
        let next_ms = elapsed + animation::next_ms(elapsed);
        Some(self.shown_at + next_ms as u64 * TICKS_PER_MS as u64)
    }

    // Returns whether the orientation changed
    fn update_orientation(&mut self, board: &mut impl Board) -> bool {
        let Some(xyz) = board.acceleration() else {
            return false;
        };
        let rotation = orientation::choose(xyz, self.rotation);
        if rotation == self.rotation {
            return false;
        }
        board.panel().off();
        // Erase the digits in their old orientation before rotating
        self.digits.clear(board.panel(), self.rotation);
        self.rotation = rotation;
        true
    }

    // An invalid RTC produces dashes
    fn refresh_time(&mut self, board: &mut impl Board) {
        let time = board.utc().map(|utc| {
            let (hour, minute) = local_time::pacific(&utc);
            (if hour % 12 == 0 { 12 } else { hour % 12 }, minute)
        });
        board.panel().brightness(self.level as u8);
        self.digits.draw(board.panel(), time, self.rotation, digits::RED);
        self.ring_pending = true;
        board.panel().on();
        self.visible = true;
        self.time_refreshed_at = board.now();
    }

    // An unavailable reading hides the ring
    fn refresh_ring(&mut self, board: &mut impl Board) {
        let percentage = board.battery_percentage();
        self.ring.draw(board.panel(), percentage, self.rotation);
        self.ring_pending = false;
    }

    // Clamp long KEY holds to the animation interval
    fn animation_elapsed_ms(&self, board: &impl Board) -> u32 {
        let elapsed = board.now() - self.shown_at;
        if elapsed >= (DISPLAY_DURATION_MS * TICKS_PER_MS) as u64 {
            return DISPLAY_DURATION_MS;
        }
        elapsed as u32 / TICKS_PER_MS
    }
}
