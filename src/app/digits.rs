//! Seven-segment time, redrawn one changed segment at a time.

use super::geometry::rotate;
use crate::drivers::display::{Panel, Rect};

pub const RED: u16 = 0xf800;

const DASH: u8 = 0x40;
/// Lit segments per digit: bit 0 is the top, then clockwise; bit 6 is the middle
const SEGMENT_MASKS: [u8; 10] = [0x3f, 0x06, 0x5b, 0x4f, 0x66, 0x6d, 0x7d, 0x07, 0x7f, 0x6f];
/// Each segment within a digit, in the same order
const SEGMENT_RECTS: [Rect; 7] = [
    Rect::new(12, 0, 48, 10),
    Rect::new(60, 12, 10, 66),
    Rect::new(60, 90, 10, 66),
    Rect::new(12, 158, 48, 10),
    Rect::new(0, 90, 10, 66),
    Rect::new(0, 12, 10, 66),
    Rect::new(12, 80, 48, 10),
];

/// What the panel currently shows. The panel and this small cache survive
/// CPU-off standby
pub struct Digits {
    segments: [u8; 4],
    ink: u16,
    large: bool,
}

impl Digits {
    pub const fn new() -> Self {
        Self { segments: [0; 4], ink: 0, large: false }
    }

    /// Draw hour and minute in the given ink; no time draws dashes
    pub fn draw(
        &mut self,
        panel: &mut impl Panel,
        time: Option<(u32, u32)>,
        rotation: u32,
        ink: u16,
    ) {
        // Enlarge three-digit times
        let large = time.is_some_and(|(hour, _)| hour < 10);
        if large != self.large {
            self.clear(panel, rotation);
        }
        self.draw_time(panel, time, rotation, large, ink);
        self.large = large;
        self.ink = ink;
    }

    /// Erase the cached digits before changing orientation
    pub fn clear(&mut self, panel: &mut impl Panel, rotation: u32) {
        self.draw_time(panel, None, rotation, self.large, 0);
        self.ink = 0;
    }

    fn draw_time(
        &mut self,
        panel: &mut impl Panel,
        time: Option<(u32, u32)>,
        rotation: u32,
        large: bool,
        ink: u16,
    ) {
        let frame = Frame { rotation, large };
        let segments = match time {
            _ if ink == 0 => [0; 4], // Erasing
            None => [DASH; 4],
            Some((hour, minute)) => [hour / 10, hour % 10, minute / 10, minute % 10]
                .map(|digit| SEGMENT_MASKS[digit as usize % 10]),
        };
        let offset = if large { 0 } else { 82 };
        let columns = [0, offset, offset + 116, offset + 198];
        let repaint = ink != self.ink;
        let digits = columns.into_iter().zip(segments).zip(&mut self.segments);
        // A large time has no leading digit
        for ((x, lit), shown) in digits.skip(large as usize) {
            let changed = (lit ^ *shown) | if repaint { lit } else { 0 };
            *shown = lit;
            for (i, segment) in SEGMENT_RECTS.iter().enumerate() {
                if changed & (1 << i) != 0 {
                    let color = if lit & (1 << i) != 0 { ink } else { 0 };
                    frame.fill(panel, Rect { x: x + segment.x, ..*segment }, color);
                }
            }
        }
        // Colon
        for y in [46, 108] {
            frame.fill(panel, Rect::new(offset + 86, y, 10, 10), ink);
        }
    }
}

/// Where upright digit coordinates land on the panel
#[derive(Clone, Copy)]
struct Frame {
    rotation: u32,
    large: bool,
}

impl Frame {
    // Coordinates stay in the upright frame; rotation is applied last
    fn fill(self, panel: &mut impl Panel, upright: Rect, ink: u16) {
        let Rect { x, y, width, height } = upright;
        // Enlarge three-digit times by 5/4
        let (scale, left, top) = if self.large { (5, 72, 135) } else { (4, 65, 156) };
        // CO5300 address windows require even starts and even dimensions
        let right = (left + (x + width) * scale / 4) & !1;
        let bottom = (top + (y + height) * scale / 4) & !1;
        let x = (left + x * scale / 4) & !1;
        let y = (top + y * scale / 4) & !1;
        panel.fill(rotate(Rect::new(x, y, right - x, bottom - y), self.rotation), ink);
    }
}
