//! A dim ring around the screen edge, filled clockwise with the battery
//! estimate.

use super::geometry::rotate;
use crate::drivers::display::{Panel, Rect};

const LIT: u16 = 0x4000;
const DIM: u16 = 0x0800;
/// Length of the ring's path around the screen, in pixels
const PERIMETER: u32 = 1920;

/// Corner bounds in eight-row, even-aligned windows: left and right edge of
/// each. Skips pixels hidden by the screen corners
const CORNERS: [[u8; 2]; 9] =
    [[40, 72], [26, 48], [18, 34], [12, 24], [6, 18], [4, 12], [0, 10], [0, 6], [0, 6]];

/// What the panel currently shows, so unchanged readings redraw nothing
pub struct BatteryIndicator {
    drawn: Option<(Option<u32>, u32)>,
}

impl BatteryIndicator {
    pub const fn new() -> Self {
        Self { drawn: None }
    }

    /// Draw a gauge estimate of 0–100; an unavailable reading hides the ring
    pub fn draw(&mut self, panel: &mut impl Panel, percentage: Option<u32>, rotation: u32) {
        let percentage = percentage.filter(|&percentage| percentage <= 100);
        if self.drawn == Some((percentage, rotation)) {
            return;
        }
        self.drawn = Some((percentage, rotation));
        let offset = (2640 - rotation * 480) % PERIMETER;
        let end = percentage.map(|percentage| percentage * PERIMETER / 100);
        let pixel = |x, y| ring_pixel(x, y, offset, end);
        for turn in 0..4 {
            panel.pattern(rotate(Rect::new(72, 0, 336, 4), turn), pixel);
            for (row, &[left, right]) in CORNERS.iter().enumerate() {
                let window = Rect::new(left as u32, row as u32 * 8, (right - left) as u32, 8);
                panel.pattern(rotate(window, turn), pixel);
            }
        }
    }
}

// The ring is lit for `end` pixels along its path, starting `offset` from the
// bottom center; without an `end` it is hidden
fn ring_pixel(x: u32, y: u32, offset: u32, end: Option<u32>) -> u16 {
    let Some(end) = end else {
        return 0;
    };
    let dx = if x < 240 { x } else { 479 - x };
    let dy = if y < 240 { y } else { 479 - y };
    // Photo-estimated outer radius 72, inner radius 68: a four-pixel band
    // The screen masks the outside; only the inner boundary needs calculation
    if dx < 72 && dy < 72 {
        let (cx, cy) = (143 - 2 * dx, 143 - 2 * dy);
        if cx * cx + cy * cy <= 136 * 136 {
            return 0;
        }
    } else if dx >= 4 && dy >= 4 {
        return 0;
    }
    let horizontal = if y < 240 { x } else { 1439 - x };
    let vertical = if x < 240 { 1919 - y } else { 480 + y };
    let mut position = if dy <= dx { horizontal } else { vertical } + offset;
    if position >= PERIMETER {
        position -= PERIMETER;
    }
    if position < end { LIT } else { DIM }
}
