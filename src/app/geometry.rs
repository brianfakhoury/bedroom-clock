use crate::drivers::display::{HEIGHT, Rect, WIDTH};

const _: () = assert!(WIDTH == HEIGHT, "quarter turns need a square screen");

/// Rotate a rectangle within the screen by clockwise quarter turns
pub fn rotate(area: Rect, rotation: u32) -> Rect {
    let Rect { x, y, width, height } = area;
    match rotation & 3 {
        1 => Rect::new(WIDTH - y - height, x, height, width),
        2 => Rect::new(WIDTH - x - width, WIDTH - y - height, width, height),
        3 => Rect::new(y, WIDTH - x - width, height, width),
        _ => area,
    }
}
