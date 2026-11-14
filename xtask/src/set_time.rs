//! Copy the computer's UTC time to the clock, which must acknowledge it.

use std::io::{self, Read, Write};
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, PartialEq, Debug)]
enum Answer {
    Accepted,
    Rejected,
}

/// Send the time until the clock's RTC answers or `timeout` passes.
/// `open` connects to the clock, which may not be listening yet
pub fn set_time<Link: Read + Write>(
    mut open: impl FnMut() -> io::Result<Link>,
    timeout: Duration,
) -> Result<(), &'static str> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let patience = deadline.min(Instant::now() + Duration::from_secs(1));
        match open().and_then(|mut link| request(&mut link, patience)) {
            Ok(Some(Answer::Accepted)) => return Ok(()),
            Ok(Some(Answer::Rejected)) => return Err("The RTC rejected the time update."),
            Ok(None) => {}
            // The port disappears while the clock sleeps or restarts
            Err(_) => sleep(Duration::from_millis(200)),
        }
    }
    Err("No time acknowledgement. Tap, press KEY, then run cargo xtask set-time.")
}

/// Send one request and wait until `patience` runs out for the answer
fn request(link: &mut (impl Read + Write), patience: Instant) -> io::Result<Option<Answer>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map_err(io::Error::other)?;
    link.write_all(format!("\nTIME {}\n", utc_digits(now.as_secs())).as_bytes())?;
    let mut reply = Vec::new();
    while Instant::now() < patience {
        let mut received = [0; 128];
        match link.read(&mut received) {
            Ok(count) => reply.extend(&received[..count]),
            Err(error) if error.kind() == io::ErrorKind::TimedOut => {}
            Err(error) => return Err(error),
        }
        if let Some(answer) = answer(&reply) {
            return Ok(Some(answer));
        }
    }
    Ok(None)
}

/// The clock's answer, once its line is complete
fn answer(reply: &[u8]) -> Option<Answer> {
    let complete = &reply[..reply.iter().rposition(|&byte| byte == b'\n')?];
    complete.split(|&byte| byte == b'\n').find_map(|line| match line {
        b"OK TIME" => Some(Answer::Accepted),
        b"ERR TIME" => Some(Answer::Rejected),
        _ => None,
    })
}

/// `YYYYMMDDhhmmss` for seconds since 1970, in UTC
fn utc_digits(seconds: u64) -> String {
    let (days, second_of_day) = (seconds / 86_400, seconds % 86_400);
    // Civil date from a day count, after Howard Hinnant's `civil_from_days`:
    // years begin in March so the leap day falls at the end of a 400-year era
    let day = days + 719_468;
    let (era, day_of_era) = (day / 146_097, day % 146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 { shifted_month + 3 } else { shifted_month - 9 };
    let year = era * 400 + year_of_era + u64::from(month <= 2);
    let (hour, minute, second) =
        (second_of_day / 3600, second_of_day / 60 % 60, second_of_day % 60);
    format!("{year:04}{month:02}{day:02}{hour:02}{minute:02}{second:02}")
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::rc::Rc;

    use super::*;

    /// A clock that sends scripted replies, then nothing. Clones share it
    #[derive(Clone, Default)]
    struct Link {
        replies: Rc<RefCell<VecDeque<&'static [u8]>>>,
        written: Rc<RefCell<Vec<u8>>>,
    }

    impl Link {
        fn replying(replies: &[&'static [u8]]) -> Self {
            let link = Link::default();
            link.replies.borrow_mut().extend(replies);
            link
        }
    }

    impl Read for Link {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            let reply = self.replies.borrow_mut().pop_front().ok_or(io::ErrorKind::TimedOut)?;
            buffer[..reply.len()].copy_from_slice(reply);
            Ok(reply.len())
        }
    }

    impl Write for Link {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.written.borrow_mut().extend(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    const SECOND: Duration = Duration::from_secs(1);

    #[test]
    fn fragmented_acknowledgement() {
        let link = Link::replying(&[b"boot\n", b"OK TI", b"ME\n"]);
        assert_eq!(set_time(|| Ok(link.clone()), SECOND), Ok(()));
        let written = link.written.borrow();
        let [b'\n', b'T', b'I', b'M', b'E', b' ', digits @ .., b'\n'] = &written[..] else {
            panic!("unexpected request {written:?}");
        };
        assert!(digits.len() == 14 && digits.iter().all(u8::is_ascii_digit));
    }

    #[test]
    fn rejection_and_missing_reply_are_errors() {
        let link = Link::replying(&[b"ERR TIME\n"]);
        let rejected = set_time(|| Ok(link.clone()), SECOND).unwrap_err();
        assert!(rejected.contains("RTC rejected"));
        // An unfinished line is not an answer
        let link = Link::replying(&[b"OK TIME"]);
        let silent = set_time(|| Ok(link.clone()), Duration::from_millis(50)).unwrap_err();
        assert!(silent.contains("No time acknowledgement"));
        let never = set_time(|| Ok(Link::default()), Duration::ZERO).unwrap_err();
        assert!(never.contains("No time acknowledgement"));
    }

    #[test]
    fn retries_until_the_port_opens() {
        let link = Link::replying(&[b"OK TIME\n"]);
        let mut attempts = 0;
        let open = || {
            attempts += 1;
            if attempts < 3 { Err(io::ErrorKind::NotFound.into()) } else { Ok(link.clone()) }
        };
        assert_eq!(set_time(open, Duration::from_secs(5)), Ok(()));
    }

    #[test]
    fn utc_calendar() {
        assert_eq!(utc_digits(0), "19700101000000");
        assert_eq!(utc_digits(951_782_399), "20000228235959");
        assert_eq!(utc_digits(951_782_400), "20000229000000");
        assert_eq!(utc_digits(1_700_000_000), "20231114221320");
        assert_eq!(utc_digits(1_794_079_530), "20261107192530");
        assert_eq!(utc_digits(4_102_444_799), "20991231235959");
    }
}
