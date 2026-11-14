use crate::app::digits::{Digits, RED};
use crate::tests::fakes::FakePanel;

const BLUE: u16 = 0x001f;

#[test]
fn centered_dimensions() {
    let (mut digits, mut panel) = (Digits::new(), FakePanel::new());
    digits.draw(&mut panel, Some((8, 58)), 0, RED);
    let lit = |x: u32, y: u32| panel.pixel(x, y) != 0;
    let columns: Vec<u32> = (0..480).filter(|&x| (0..480).any(|y| lit(x, y))).collect();
    let rows: Vec<u32> = (0..480).filter(|&y| (0..480).any(|x| lit(x, y))).collect();
    let extent = |lit: &[u32]| (lit[0], lit[lit.len() - 1] - lit[0] + 1);
    assert_eq!(extent(&columns), (72, 334));
    assert_eq!(extent(&rows), (134, 210));
}

#[test]
fn incremental_render_matches_full_render() {
    // Compare incremental transitions with a cleared-panel rendering, including
    // size boundaries, invalid time and ink changes, in all four orientations
    let (mut digits, mut panel) = (Digits::new(), FakePanel::new());
    for rotation in 0..4 {
        for hour in [9, 10, 11, 12, 1] {
            for minute in 0..60 {
                let time = (minute != 30).then_some((hour, minute));
                let ink = if minute % 2 == 1 { RED } else { BLUE };
                digits.draw(&mut panel, time, rotation, ink);
                let expected = panel.pixels.clone();
                panel.transferred = 0;
                digits.draw(&mut panel, time, rotation, ink);
                // Unchanged digits need no pixel traffic
                assert!(panel.transferred < 500);
                assert!(panel.pixels == expected);
                digits.clear(&mut panel, rotation);
                assert!(panel.is_blank());
                digits.draw(&mut panel, time, rotation, ink);
                assert!(panel.pixels == expected);
            }
        }
        digits.clear(&mut panel, rotation);
    }
}
