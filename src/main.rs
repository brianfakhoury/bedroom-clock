//! Touch-activated bedside clock for the Waveshare ESP32-C6 Touch AMOLED 2.16.
//!
//! Layers, each depending only on those below it:
//! `app` (clock behavior) → `board` (this board's choices) → `drivers`
//! (external chips) → `hal` (ESP32-C6 operations), started by `boot`.

#![cfg_attr(target_os = "none", no_std, no_main)]
// Host builds exist to run the tests; the hardware side goes unused there
#![cfg_attr(not(target_os = "none"), allow(dead_code))]

mod app;
mod board;
mod boot;
mod drivers;
mod hal;

#[cfg(test)]
#[path = "../tests/mod.rs"]
mod tests;

use core::cell::UnsafeCell;

use app::App;
use board::Waveshare;

/// State kept in LP RAM across deep sleep
struct Retained<T>(UnsafeCell<T>);

// SAFETY: one core and no interrupt handlers, so nothing runs concurrently
unsafe impl<T> Sync for Retained<T> {}

// The board's bus state and the clock's presentation state survive sleep;
// the external RTC keeps the time
static BOARD: Retained<Waveshare> = Retained(UnsafeCell::new(Waveshare::new()));
static APP: Retained<App> = Retained(UnsafeCell::new(App::new()));

/// Cold boot: ROM loads the firmware into memory; `start.S` runs and calls here
#[cfg(target_os = "none")]
#[unsafe(no_mangle)]
extern "C" fn main() -> ! {
    // SAFETY: the only reference; see `wake_main`
    let (board, app) = unsafe { (&mut *BOARD.0.get(), &mut *APP.0.get()) };
    board.init();
    app.init(board);
    app.run(board)
}

/// Wake from board-configured deep sleep, reached through ROM and `wake.S`.
/// The retained state lets it skip flash loading, state clearing and full
/// hardware initialization, for a much faster wake
#[cfg(target_os = "none")]
#[unsafe(no_mangle)]
extern "C" fn wake_main() -> ! {
    // SAFETY: each entry point runs on a fresh stack and never returns, so
    // the reference it takes is the only one
    let (board, app) = unsafe { (&mut *BOARD.0.get(), &mut *APP.0.get()) };
    board.resume();
    app.run(board)
}

#[cfg(not(any(target_os = "none", test)))]
fn main() {
    eprintln!("This is firmware: build it with `cargo firmware`, test it with `cargo test`.");
}
