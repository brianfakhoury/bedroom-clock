use crate::app::local_time::pacific;
use crate::drivers::rtc::DateTime;

fn local(month: u32, day: u32, hour: u32, minute: u32) -> (u32, u32) {
    pacific(&DateTime { year: 2026, month, day, hour, minute, second: 0 })
}

#[test]
fn dst_boundaries() {
    assert_eq!(local(3, 8, 9, 59), (1, 59));
    assert_eq!(local(3, 8, 10, 0), (3, 0));
    assert_eq!(local(11, 1, 8, 59), (1, 59));
    assert_eq!(local(11, 1, 9, 0), (1, 0));
    assert_eq!(local(11, 1, 10, 0), (2, 0));
}

#[test]
fn utc_day_rollover() {
    assert_eq!(local(1, 1, 0, 0), (16, 0));
    assert_eq!(local(7, 1, 0, 0), (17, 0));
}
