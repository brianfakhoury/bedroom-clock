//! Clock behavior and appearance: what to draw, how input changes it, and
//! when to sleep.

pub mod animation;
pub mod battery_indicator;
pub mod clock;
pub mod digits;
pub mod geometry;
pub mod input;
pub mod local_time;
pub mod orientation;
pub mod time_sync;

use crate::board::Board;
use clock::Clock;
use input::Input;
use time_sync::TimeSync;

const DEFAULT_BRIGHTNESS: u32 = 128;

pub struct App {
    clock: Clock,
    input: Input,
    time_sync: TimeSync,
}

impl App {
    pub const fn new() -> Self {
        Self {
            clock: Clock::new(DEFAULT_BRIGHTNESS),
            input: Input::new(),
            time_sync: TimeSync::new(),
        }
    }

    /// Prepare the display once, at cold boot
    pub fn init(&mut self, board: &mut impl Board) {
        self.clock.init(board);
    }

    /// Show the time, then sleep; every later wake starts here again
    pub fn run(&mut self, board: &mut impl Board) -> ! {
        self.input.reset();
        loop {
            self.view(board);
            // Returns only if a pending touch cancelled sleep: start another
            // viewing interval without resetting debounce history
            board.sleep();
        }
    }

    // One viewing interval: handle input and update the display until hidden
    fn view(&mut self, board: &mut impl Board) {
        self.clock.show(board);
        self.input.consume_touch();
        while self.clock.visible() {
            let events = board.wait_until(self.clock.next_update(board));
            let presses = self.input.read(board, events);
            if presses.tap || presses.boot {
                self.clock.show(board);
            }
            if presses.key {
                self.clock.toggle_always_on(board);
            }
            self.clock.update(board);
            if events.usb {
                self.time_sync.process(board);
            }
            board.feed_watchdog();
        }
    }
}
