use crate::drivers::display::{Display, Panel, Rect};
use crate::hal::qspi::Qspi;
use crate::tests::fakes::Elapsed;

/// Records each chip-select transfer: its bytes, then its pixels
#[derive(Default)]
struct Bus {
    transfers: Vec<Vec<u8>>,
    pixels: Vec<u16>,
    selected: bool,
}

impl Qspi for Bus {
    fn begin(&mut self) {
        assert!(!self.selected);
        self.selected = true;
        self.transfers.push(Vec::new());
    }

    fn end(&mut self) {
        assert!(self.selected);
        self.selected = false;
    }

    fn write_byte(&mut self, value: u8) {
        assert!(self.selected);
        self.transfers.last_mut().unwrap().push(value);
    }

    fn write_quad16(&mut self, value: u16) {
        assert!(self.selected);
        self.pixels.push(value);
    }
}

type TestDisplay<'a> = Display<Bus, &'a Elapsed>;

fn commands(display: &TestDisplay) -> Vec<u8> {
    display.bus().transfers.iter().map(|transfer| transfer[2]).collect()
}

#[test]
fn pixel_transfers() {
    let elapsed = Elapsed::default();
    let mut panel = Display::new(Bus::default(), &elapsed);
    panel.fill(Rect::new(256, 2, 2, 2), 0xf123);
    let expected: [&[u8]; 4] = [
        &[2, 0, 0x2a, 0, 1, 0, 1, 1],
        &[2, 0, 0x2b, 0, 0, 2, 0, 3],
        &[2, 0, 0x2c, 0],
        &[0x32, 0, 0x3c, 0],
    ];
    assert_eq!(panel.bus().transfers, expected);
    assert_eq!(panel.bus().pixels, [0xf123; 4]);

    let mut panel = Display::new(Bus::default(), &elapsed);
    let pixel = |x, y| (x + 480 * y) as u16;
    panel.pattern(Rect::new(2, 4, 2, 2), pixel);
    assert_eq!(panel.bus().pixels, [pixel(2, 4), pixel(3, 4), pixel(2, 5), pixel(3, 5)]);
}

#[test]
fn sleep_preserves_pixels() {
    let elapsed = Elapsed::default();
    let mut panel = Display::new(Bus::default(), &elapsed);
    panel.wake(); // Already awake
    assert!(panel.bus().transfers.is_empty());
    panel.sleep();
    assert_eq!(commands(&panel), [0x28, 0x10]);
    assert_eq!(elapsed.us(), 120_000);
    panel.sleep();
    assert_eq!(commands(&panel).len(), 2);
    assert_eq!(elapsed.us(), 120_000);

    panel.wake();
    let woken = commands(&panel);
    assert_eq!(woken[2], 0x11);
    assert!(woken.len() > 3 && panel.bus().pixels.is_empty());
    assert_eq!(elapsed.us(), 120_000 + 140_000);
    panel.wake();
    assert_eq!(commands(&panel), woken);
    assert_eq!(elapsed.us(), 120_000 + 140_000);
}

#[test]
fn initialization_clears_the_dark_panel() {
    let elapsed = Elapsed::default();
    let mut panel = Display::new(Bus::default(), &elapsed);
    panel.init();
    let commands = commands(&panel);
    assert_eq!(commands[..2], [0x28, 0x11]);
    assert_eq!(commands.last(), Some(&0x28));
    assert!(!commands.contains(&0x29));
    assert_eq!(panel.bus().pixels, vec![0; 480 * 480]);
}
