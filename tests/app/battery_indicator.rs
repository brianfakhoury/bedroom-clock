use crate::app::battery_indicator::BatteryIndicator;
use crate::drivers::display::{Panel, Rect};
use crate::tests::fakes::FakePanel;

const LIT: u16 = 0x4000;
const DIM: u16 = 0x0800;

/// A panel that also rejects ring pixels outside the screen's border
struct Border(FakePanel);

impl Panel for Border {
    fn pattern(&mut self, area: Rect, pixel: impl Fn(u32, u32) -> u16) {
        let central = |start: u32, length: u32| start >= 72 && start + length <= 408;
        assert!(!(central(area.x, area.width) && central(area.y, area.height)));
        self.0.pattern(area, pixel);
    }

    fn fill(&mut self, _: Rect, _: u16) {}
    fn on(&mut self) {}
    fn off(&mut self) {}
    fn brightness(&mut self, _: u8) {}
    fn sleep(&mut self) {}
    fn wake(&mut self) {}
}

fn drawn(percentage: Option<u32>, rotation: u32) -> FakePanel {
    let mut panel = Border(FakePanel::new());
    BatteryIndicator::new().draw(&mut panel, percentage, rotation);
    panel.0
}

#[test]
fn fill_and_cache() {
    let (mut ring, mut panel) = (BatteryIndicator::new(), Border(FakePanel::new()));
    ring.draw(&mut panel, None, 0);
    let mut prior = 0;
    for percentage in 0..=100 {
        panel.0.transferred = 0;
        ring.draw(&mut panel, Some(percentage), 0);
        assert!(panel.0.transferred < 15000);
        let lit = panel.0.pixels.iter().filter(|&&pixel| pixel == LIT).count();
        assert!(lit >= prior);
        assert!(percentage != 0 || lit == 0);
        prior = lit;
        panel.0.windows = 0;
        ring.draw(&mut panel, Some(percentage), 0);
        assert_eq!(panel.0.windows, 0);
    }
}

#[test]
fn stroke_geometry() {
    let panel = drawn(Some(100), 0);
    // Check the complete visible band against the two circle boundaries,
    // independently of the renderer's narrow transfer-window table
    for y in 0..480 {
        for x in 0..480 {
            let dx = if x < 240 { x } else { 479 - x };
            let dy = if y < 240 { y } else { 479 - y };
            let band = if dx < 72 && dy < 72 {
                let (cx, cy) = (143 - 2 * dx, 143 - 2 * dy);
                let distance = cx * cx + cy * cy;
                if distance > 144 * 144 {
                    continue; // Physical screen mask
                }
                distance > 136 * 136
            } else {
                dx < 4 || dy < 4
            };
            assert_eq!(panel.pixel(x, y), if band { LIT } else { 0 }, "at {x},{y}");
        }
    }
}

#[test]
fn clockwise_fill_and_rotation() {
    let mut reference = drawn(Some(25), 0);
    assert_eq!((reference.pixel(230, 479), reference.pixel(250, 479)), (LIT, DIM));
    assert_eq!((reference.pixel(0, 300), reference.pixel(0, 200)), (LIT, DIM));
    for rotation in [1, 2, 3, 0] {
        let panel = drawn(Some(25), rotation);
        for y in 0..480 {
            for x in 0..480 {
                assert_eq!(panel.pixel(x, y), reference.pixel(y, 479 - x));
            }
        }
        reference = panel;
    }
}

#[test]
fn unavailable_reading_hides_ring() {
    let (mut ring, mut panel) = (BatteryIndicator::new(), Border(FakePanel::new()));
    ring.draw(&mut panel, Some(75), 0);
    ring.draw(&mut panel, None, 0);
    assert!(panel.0.is_blank());
    // An out-of-range estimate is as good as none
    ring.draw(&mut panel, Some(75), 0);
    ring.draw(&mut panel, Some(101), 0);
    assert!(panel.0.is_blank());
}
