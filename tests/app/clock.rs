use crate::app::battery_indicator::BatteryIndicator;
use crate::app::clock::Clock;
use crate::hal::timer::TICKS_PER_MS;
use crate::tests::fakes::{FakeBoard, FakePanel, screen, utc_at_pacific_hour};

const SIDEWAYS: [i16; 3] = [16384, 0, 0];

fn ms(ms: u64) -> Option<u64> {
    Some(ms * TICKS_PER_MS as u64)
}

/// A primed clock at full brightness
fn started() -> (Clock, FakeBoard) {
    let (mut clock, mut board) = (Clock::new(255), FakeBoard::new());
    clock.init(&mut board);
    board.battery_reads = 0;
    (clock, board)
}

#[test]
fn initialization_primes_ring() {
    let (mut clock, mut board) = (Clock::new(128), FakeBoard::new());
    clock.init(&mut board);
    assert!(!clock.visible());
    assert_eq!((board.battery_reads, board.panel.on_count), (1, 0));
    assert!(board.panel.pixels == ring_only(Some(75)));

    let (mut clock, mut board) = (Clock::new(128), FakeBoard::new());
    board.battery = None;
    clock.init(&mut board);
    assert!(board.panel.is_blank());
    clock.show(&mut board);
    board.at(180);
    clock.update(&mut board);
    assert_eq!(board.panel.level, 128);
}

/// The primed ring before any time is drawn
fn ring_only(battery: Option<u32>) -> Vec<u16> {
    let mut panel = FakePanel::new();
    BatteryIndicator::new().draw(&mut panel, battery, 0);
    panel.pixels
}

#[test]
fn lifecycle_and_retrigger() {
    let (mut clock, mut board) = started();
    clock.update(&mut board);
    assert!(clock.next_update(&board).is_none() && !clock.visible());
    // A dark display cannot acquire a hold; it must still fade out below
    clock.toggle_always_on(&mut board);
    clock.show(&mut board);
    assert!(clock.visible());
    assert_eq!((board.panel.on_count, board.panel.level), (1, 0));
    assert!(board.panel.pixels == screen(Some((8, 0)), 0, Some(75), 0));
    assert_eq!(board.battery_reads, 0);
    board.at(100);
    clock.update(&mut board);
    assert_eq!(board.battery_reads, 0);
    board.at(180);
    clock.update(&mut board);
    assert_eq!(board.panel.level, 255);
    assert_eq!(board.battery_reads, 1);
    assert_eq!(clock.next_update(&board), ms(2650));
    board.at(2800);
    clock.update(&mut board);
    let level = board.panel.level;
    assert!(level > 0 && level < 255);
    clock.show(&mut board);
    assert_eq!(board.panel.level, level); // Retrigger without a jump
    board.at(2980);
    clock.update(&mut board);
    assert_eq!(board.panel.level, 255);
    board.at(5800);
    clock.update(&mut board);
    assert!(!clock.visible());
    assert_eq!((board.panel.level, board.panel.off_count), (0, 1));
    assert!(clock.next_update(&board).is_none());
    // Retain the filled ring for the next fade-in
    assert!(board.panel.pixels == screen(Some((8, 0)), 0, Some(75), 0));
    clock.update(&mut board);
    assert_eq!(board.panel.off_count, 1);
}

#[test]
fn retap_at_timeout_precedes_update() {
    let (mut clock, mut board) = started();
    clock.show(&mut board);
    board.at(3000);
    clock.show(&mut board); // Input is handled before the expired frame is updated
    clock.update(&mut board);
    assert!(clock.visible());
    assert_eq!(board.panel.off_count, 0);
    assert_eq!(clock.next_update(&board), ms(3010));
}

#[test]
fn rotation_and_sensor_failures() {
    let (mut clock, mut board) = started();
    board.acceleration = Some(SIDEWAYS);
    clock.show(&mut board);
    // The digits turn at once, dark; the ring follows after the fade-in
    assert!(board.panel.pixels == screen(Some((8, 0)), 3, Some(75), 0));
    assert_eq!((board.panel.off_count, board.panel.level), (1, 0));
    board.acceleration = None;
    board.utc = None;
    clock.show(&mut board);
    assert!(board.panel.pixels == screen(None, 3, Some(75), 0));
    assert_eq!(board.panel.off_count, 1);
    board.at(180);
    clock.update(&mut board);
    assert!(board.panel.pixels == screen(None, 3, Some(75), 3));
}

#[test]
fn twelve_hour_boundaries() {
    let (mut clock, mut board) = started();
    for (hour, expected) in [(0, 12), (1, 1), (11, 11), (12, 12), (13, 1), (23, 11)] {
        board.utc = Some(utc_at_pacific_hour(hour));
        clock.show(&mut board);
        assert!(board.panel.pixels == screen(Some((expected, 0)), 0, Some(75), 0), "{hour}");
    }
}

#[test]
fn expired_hold_fades_without_restarting_interval() {
    let (mut clock, mut board) = started();
    board.at(10_000);
    clock.show(&mut board);
    clock.toggle_always_on(&mut board);
    board.at(10_180);
    clock.update(&mut board);
    assert_eq!(board.panel.level, 255);
    assert_eq!(clock.next_update(&board), ms(11_000));
    board.utc = Some(utc_at_pacific_hour(4));
    board.acceleration = Some(SIDEWAYS);
    board.at(86_410_000); // A day-long hold still refreshes and releases normally
    clock.update(&mut board);
    assert!(clock.visible());
    assert!(board.panel.pixels == screen(Some((4, 0)), 3, Some(75), 3));
    assert_eq!(board.panel.level, 255);
    clock.toggle_always_on(&mut board);
    clock.update(&mut board);
    assert_eq!(board.panel.level, 255);
    board.at(86_410_175);
    clock.update(&mut board);
    assert!(board.panel.level > 0 && board.panel.level < 255);
    board.at(86_410_350);
    clock.update(&mut board);
    assert!(!clock.visible());
}

#[test]
fn early_hold_release_preserves_deadline() {
    let (mut clock, mut board) = started();
    board.at(20_000);
    clock.show(&mut board);
    board.at(20_180);
    clock.update(&mut board);
    clock.toggle_always_on(&mut board);
    board.at(21_000);
    clock.toggle_always_on(&mut board);
    assert_eq!(clock.next_update(&board), ms(22_650));
    assert!(board.panel.pixels == screen(Some((8, 0)), 0, Some(75), 0));
}

#[test]
fn failed_battery_read_hides_ring() {
    let (mut clock, mut board) = started();
    clock.show(&mut board);
    board.at(180);
    clock.update(&mut board);
    board.battery = None;
    let reads_before = board.battery_reads;
    clock.show(&mut board);
    assert_eq!(board.battery_reads, reads_before);
    board.at(360);
    clock.update(&mut board);
    assert_eq!(board.battery_reads, reads_before + 1);
    assert!(board.panel.pixels == screen(Some((8, 0)), 0, None, 0));
}
