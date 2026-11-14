use crate::app::geometry::rotate;
use crate::drivers::display::Rect;

#[test]
fn rectangle_rotation() {
    let area = Rect::new(64, 150, 70, 166);
    let expected = [
        Rect::new(64, 150, 70, 166),
        Rect::new(164, 64, 166, 70),
        Rect::new(346, 164, 70, 166),
        Rect::new(150, 346, 166, 70),
    ];
    let screen = Rect::new(0, 0, 480, 480);
    for (rotation, expected) in (0..).zip(expected) {
        let rotated = rotate(area, rotation);
        assert_eq!(rotated, expected);
        assert_eq!(rotate(rotated, (4 - rotation) & 3), area);
        assert_eq!(rotate(screen, rotation), screen);
    }
}
