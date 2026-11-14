use crate::drivers::rtc::{DateTime, weekday};

/// Hour and minute in America/Los_Angeles, under current US rules (since
/// 2007). The RTC always holds UTC
pub fn pacific(utc: &DateTime) -> (u32, u32) {
    // Second Sunday of March and first Sunday of November
    let march = 8 + (7 - weekday(utc.year, 3, 8)) % 7;
    let november = 1 + (7 - weekday(utc.year, 11, 1)) % 7;
    let after_spring = utc.day > march || (utc.day == march && utc.hour >= 10);
    let before_fall = utc.day < november || (utc.day == november && utc.hour < 9);
    let dst = match utc.month {
        3 => after_spring,
        11 => before_fall,
        month => month > 3 && month < 11,
    };
    ((utc.hour + 24 - if dst { 7 } else { 8 }) % 24, utc.minute)
}
