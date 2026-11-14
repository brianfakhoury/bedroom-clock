use crate::app::time_sync::TimeSync;
use crate::drivers::rtc::DateTime;
use crate::tests::fakes::FakeBoard;

/// Deliver a request and return the reply
fn send(sync: &mut TimeSync, board: &mut FakeBoard, text: &str) -> String {
    board.usb_input.extend(text.bytes());
    while !board.usb_input.is_empty() {
        sync.process(board);
    }
    std::mem::take(&mut board.usb_output)
}

#[test]
fn valid_utc() {
    let (mut sync, mut board) = (TimeSync::new(), FakeBoard::new());
    assert_eq!(send(&mut sync, &mut board, "TIME 20261107192530\r\n"), "OK TIME\n");
    let expected = DateTime { year: 2026, month: 11, day: 7, hour: 19, minute: 25, second: 30 };
    assert_eq!(board.utc, Some(expected));
}

#[test]
fn invalid_requests_leave_rtc_unchanged() {
    let (mut sync, mut board) = (TimeSync::new(), FakeBoard::new());
    for request in [
        "TIME 20260229192530\n",
        "TIME 2026110719253\n",
        "TIME 202611071925300\n",
        "TIME 2026110719253X\n",
        "OTHER20261107192530\n",
        "TIME 20261107242530\n",
    ] {
        assert_eq!(send(&mut sync, &mut board, request), "ERR TIME\n", "{request}");
        assert_eq!(board.utc_writes, 0);
    }
}

#[test]
fn fragmented_request() {
    let (mut sync, mut board) = (TimeSync::new(), FakeBoard::new());
    assert_eq!(send(&mut sync, &mut board, "TIME 2028"), "");
    assert_eq!(send(&mut sync, &mut board, "0229010203\n"), "OK TIME\n");
}

#[test]
fn blank_lines_get_no_reply() {
    let (mut sync, mut board) = (TimeSync::new(), FakeBoard::new());
    assert_eq!(send(&mut sync, &mut board, "\n\r\n"), "");
}

#[test]
fn overflow_is_bounded_and_recovers() {
    let (mut sync, mut board) = (TimeSync::new(), FakeBoard::new());
    board.usb_input.extend("x".repeat(128).bytes().chain(*b"\n"));
    sync.process(&mut board);
    assert_eq!(board.usb_input.len(), 65); // Host cannot monopolize the event loop
    sync.process(&mut board);
    sync.process(&mut board);
    assert_eq!(std::mem::take(&mut board.usb_output), "ERR TIME\n");
    // Recover after overflow
    assert_eq!(send(&mut sync, &mut board, "TIME 20261107192530\n"), "OK TIME\n");
}

#[test]
fn rtc_failure() {
    let (mut sync, mut board) = (TimeSync::new(), FakeBoard::new());
    board.rtc_writable = false;
    assert_eq!(send(&mut sync, &mut board, "TIME 20261107192530\n"), "ERR TIME\n");
}
