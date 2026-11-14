use crate::app::App;
use crate::board::Events;
use crate::drivers::rtc::DateTime;
use crate::tests::fakes::{FakeBoard, screen};

const NOTHING: Events = Events { boot: false, key: false, touch: false, usb: false, timer: false };
const KEY: Events = Events { key: true, ..NOTHING };
const TOUCH: Events = Events { touch: true, ..NOTHING };
const USB: Events = Events { usb: true, ..NOTHING };

/// A cold boot that runs until the board sleeps
fn boot(board: &mut FakeBoard) -> App {
    let mut app = App::new();
    app.init(board);
    board.run_until_asleep(|board| app.run(board));
    app
}

#[test]
fn boot_shows_time_then_sleeps() {
    let mut board = FakeBoard::new();
    boot(&mut board);
    assert_eq!((board.sleeps, board.now_ms()), (1, 3000));
    assert_eq!((board.panel.on_count, board.panel.off_count), (1, 1));
    assert!(board.panel.sleeping && board.watchdog_feeds > 0);
    // The panel retains the last frame for the next fade-in
    assert!(board.panel.pixels == screen(Some((8, 0)), 0, Some(75), 0));
}

#[test]
fn wake_shows_time_again() {
    let mut board = FakeBoard::new();
    let mut app = boot(&mut board);
    board.at(0); // A CPU wake restarts the timer
    board.run_until_asleep(|board| app.run(board));
    assert_eq!((board.sleeps, board.now_ms()), (2, 3000));
    assert_eq!(board.panel.on_count, 2);
}

#[test]
fn tap_restarts_the_interval() {
    let mut board = FakeBoard::new();
    // The waking contact is ignored until it lifts
    board.arrive(500, TOUCH, true);
    board.arrive(1000, TOUCH, false);
    board.arrive(2000, TOUCH, true);
    boot(&mut board);
    assert_eq!((board.sleeps, board.now_ms()), (1, 5000));
}

#[test]
fn key_toggles_hold_while_awake() {
    let mut board = FakeBoard::new();
    board.arrive(500, KEY, false);
    board.arrive(60_000, KEY, false);
    boot(&mut board);
    // Releasing an expired hold only fades out
    assert_eq!((board.sleeps, board.now_ms()), (1, 60_350));
    // The held display refreshed the time every second
    assert!(board.utc_reads >= 60);
}

#[test]
fn usb_sets_the_time_after_display_work() {
    let mut board = FakeBoard::new();
    board.usb_input.extend(b"TIME 20261107192530\n");
    board.arrive(1000, USB, false);
    boot(&mut board);
    assert_eq!(board.usb_output, "OK TIME\n");
    let expected = DateTime { year: 2026, month: 11, day: 7, hour: 19, minute: 25, second: 30 };
    assert_eq!(board.utc, Some(expected));
    assert_eq!(board.now_ms(), 3000);
}

#[test]
fn cancelled_sleep_shows_again_and_keeps_contact_consumed() {
    let mut board = FakeBoard::new();
    board.cancelled_sleeps = 1;
    // The finger that cancelled sleep is still down in the second interval
    board.arrive(3500, TOUCH, true);
    boot(&mut board);
    assert_eq!((board.sleeps, board.now_ms()), (2, 6000));
    assert_eq!(board.panel.on_count, 2);
}
