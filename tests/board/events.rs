use crate::board::events::{Events, PINS};
use crate::hal::interrupts::Pending;

const NOTHING: Events = Events { boot: false, key: false, touch: false, usb: false, timer: false };

fn on_pin(pin: u32) -> Events {
    Pending { gpio: 1 << pin, ..Pending::default() }.into()
}

#[test]
fn watches_the_buttons_and_touch_interrupt() {
    assert_eq!(PINS, (1 << 5) | (1 << 9) | (1 << 10));
}

#[test]
fn names_raw_events() {
    assert_eq!(on_pin(9), Events { boot: true, ..NOTHING });
    assert_eq!(on_pin(10), Events { key: true, ..NOTHING });
    assert_eq!(on_pin(5), Events { touch: true, ..NOTHING });
    assert_eq!(on_pin(17), NOTHING);
    let usb = Pending { usb: true, ..Pending::default() };
    assert_eq!(Events::from(usb), Events { usb: true, ..NOTHING });
    let timer = Pending { timer: true, ..Pending::default() };
    assert_eq!(Events::from(timer), Events { timer: true, ..NOTHING });
    let everything = Pending { gpio: !0, usb: true, timer: true };
    let all = Events { boot: true, key: true, touch: true, usb: true, timer: true };
    assert_eq!(Events::from(everything), all);
}
