//! One bounded USB request: `TIME YYYYMMDDhhmmss\n`, in UTC.

use crate::board::Board;
use crate::drivers::rtc::DateTime;

const LINE_LENGTH: usize = 19;

pub struct TimeSync {
    line: [u8; LINE_LENGTH],
    /// Bytes received; more than a full line marks the line as overlong
    used: usize,
}

impl TimeSync {
    pub const fn new() -> Self {
        Self { line: [0; LINE_LENGTH], used: 0 }
    }

    /// Read what the host sent, up to one line or a bounded number of bytes
    pub fn process(&mut self, board: &mut impl Board) {
        // Bound host work so malformed input cannot starve the display or sleep
        for _ in 0..64 {
            match board.usb_read() {
                None => return,
                Some(b'\r') => {}
                Some(b'\n') => {
                    let valid = self.used == LINE_LENGTH
                        && parse(&self.line).is_some_and(|time| board.set_utc(&time));
                    if self.used != 0 {
                        board.usb_write(if valid { "OK TIME\n" } else { "ERR TIME\n" });
                    }
                    self.used = 0;
                    return;
                }
                Some(byte) => match self.line.get_mut(self.used) {
                    Some(slot) => {
                        *slot = byte;
                        self.used += 1;
                    }
                    // Reject the entire overlong line
                    None => self.used = LINE_LENGTH + 1,
                },
            }
        }
    }
}

fn parse(line: &[u8; LINE_LENGTH]) -> Option<DateTime> {
    let digits = line.strip_prefix(b"TIME ")?;
    let mut pairs = [0; 7];
    for (pair, text) in pairs.iter_mut().zip(digits.chunks_exact(2)) {
        for &character in text {
            if !character.is_ascii_digit() {
                return None;
            }
            *pair = *pair * 10 + (character - b'0') as u32;
        }
    }
    let [century, year, month, day, hour, minute, second] = pairs;
    let time = DateTime { year: century * 100 + year, month, day, hour, minute, second };
    time.is_valid().then_some(time)
}
