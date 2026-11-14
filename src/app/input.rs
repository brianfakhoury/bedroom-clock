//! Touch press detection, waking-contact consumption and button debounce.

use crate::board::{Board, Events};
use crate::hal::timer::TICKS_PER_MS;

const BUTTON_DEBOUNCE_MS: u32 = 200;

/// New presses since the last read
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Presses {
    pub tap: bool,
    pub boot: bool,
    pub key: bool,
}

pub struct Input {
    was_touching: bool,
    last_button: Option<u64>,
}

impl Input {
    pub const fn new() -> Self {
        Self { was_touching: false, last_button: None }
    }

    /// Reset debounce history after CPU wake, which restarts the timer
    pub fn reset(&mut self) {
        self.last_button = None;
    }

    /// The contact that started this display session is already handled
    pub fn consume_touch(&mut self) {
        self.was_touching = true;
    }

    /// Translate hardware events into new presses
    pub fn read(&mut self, board: &mut impl Board, events: Events) -> Presses {
        let mut presses =
            Presses { tap: events.touch && self.touch_pressed(board), ..Presses::default() };
        if !(events.boot || events.key) {
            return presses;
        }
        let now = board.now();
        if let Some(last) = self.last_button
            && now - last <= (BUTTON_DEBOUNCE_MS * TICKS_PER_MS) as u64
        {
            return presses;
        }
        self.last_button = Some(now);
        presses.key = events.key;
        presses.boot = !events.key;
        presses
    }

    // A failed read leaves the contact history unchanged
    fn touch_pressed(&mut self, board: &mut impl Board) -> bool {
        let Some(touching) = board.touching() else {
            return false;
        };
        let pressed = touching && !self.was_touching;
        self.was_touching = touching;
        pressed
    }
}
