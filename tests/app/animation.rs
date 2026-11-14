use crate::app::animation::*;

#[test]
fn fade_curve() {
    for target in [1, 128, 255] {
        let mut previous = 0;
        for ms in 0..=FADE_IN_MS {
            let level = level(ms, 0, target);
            assert!(level >= previous && level <= target);
            previous = level;
        }
        assert_eq!(previous, target);
        for ms in FADE_IN_MS..=DISPLAY_DURATION_MS {
            let level = level(ms, 0, target);
            if ms < DISPLAY_DURATION_MS - FADE_OUT_MS {
                assert_eq!(level, target);
            }
            assert!(level <= previous);
            previous = level;
        }
        assert_eq!(previous, 0);
        assert_eq!(level(DISPLAY_DURATION_MS + 1, 0, target), 0);
    }
}

#[test]
fn retrigger() {
    // A tap during fade-out starts from the existing level without a jump
    for from in 0..=255 {
        assert_eq!(level(0, from, 255), from);
        assert_eq!(level(FADE_IN_MS, from, 255), 255);
    }
}

#[test]
fn deadlines() {
    assert_eq!(next_ms(0), 10);
    assert_eq!(next_ms(FADE_IN_MS), 2470);
    assert_eq!(next_ms(DISPLAY_DURATION_MS - FADE_OUT_MS), 10);
    assert_eq!(next_ms(DISPLAY_DURATION_MS), 0);
}
