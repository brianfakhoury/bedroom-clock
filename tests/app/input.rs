use crate::app::input::{Input, Presses};
use crate::board::Events;
use crate::tests::fakes::FakeBoard;

const TOUCH: Events = Events { touch: true, boot: false, key: false, usb: false, timer: false };
const BOOT: Events = Events { boot: true, ..TOUCH_OFF };
const KEY: Events = Events { key: true, ..TOUCH_OFF };
const TOUCH_OFF: Events = Events { touch: false, ..TOUCH };

const NONE: Presses = Presses { tap: false, boot: false, key: false };
const TAP: Presses = Presses { tap: true, ..NONE };

fn button_at(input: &mut Input, board: &mut FakeBoard, ms: u64, events: Events) -> Presses {
    board.at(ms);
    input.read(board, events)
}

#[test]
fn touch_requires_a_new_press() {
    let (mut input, mut board) = (Input::new(), FakeBoard::new());
    input.consume_touch();
    board.touching = Some(true);
    assert_eq!(input.read(&mut board, TOUCH), NONE);
    board.touching = None;
    assert_eq!(input.read(&mut board, TOUCH), NONE);
    board.touching = Some(true);
    // The failed read of the release did not change history
    assert_eq!(input.read(&mut board, TOUCH), NONE);
    board.touching = Some(false);
    assert_eq!(input.read(&mut board, TOUCH), NONE);
    board.touching = Some(true);
    assert_eq!(input.read(&mut board, TOUCH), TAP);
    assert_eq!(input.read(&mut board, TOUCH), NONE);
}

#[test]
fn button_debounce_and_wake() {
    let (mut input, mut board) = (Input::new(), FakeBoard::new());
    input.reset();
    assert_eq!(button_at(&mut input, &mut board, 50, KEY), Presses { key: true, ..NONE });
    assert_eq!(button_at(&mut input, &mut board, 100, KEY), NONE);
    assert_eq!(button_at(&mut input, &mut board, 250, BOOT), NONE);
    assert_eq!(button_at(&mut input, &mut board, 251, BOOT), Presses { boot: true, ..NONE });
    // Cancelled sleep preserves debounce history
    input.consume_touch();
    assert_eq!(button_at(&mut input, &mut board, 300, KEY), NONE);
    input.reset(); // A CPU wake starts a fresh timer and debounce interval
    assert_eq!(button_at(&mut input, &mut board, 50, KEY), Presses { key: true, ..NONE });
}

#[test]
fn simultaneous_inputs() {
    let (mut input, mut board) = (Input::new(), FakeBoard::new());
    input.reset();
    input.consume_touch();
    assert_eq!(input.read(&mut board, TOUCH), NONE);
    board.touching = Some(true);
    let all = Events { touch: true, boot: true, key: true, ..Events::default() };
    assert_eq!(button_at(&mut input, &mut board, 50, all), Presses { key: true, ..TAP });
    let neither = Events { usb: true, timer: true, ..Events::default() };
    assert_eq!(input.read(&mut board, neither), NONE);
}
