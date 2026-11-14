use crate::app::orientation::choose;

#[test]
fn cardinal_directions() {
    assert_eq!(choose([0, 16384, 0], 0), 0);
    assert_eq!(choose([0, -16384, 0], 0), 2);
    assert_eq!(choose([16384, 0, 0], 0), 3);
    assert_eq!(choose([-16384, 0, 0], 0), 1);
    assert_eq!(choose([-900, 16600, -200], 0), 0); // Physical upright calibration
}

#[test]
fn ambiguous_samples_keep_orientation() {
    let ambiguous = [[0, 0, 16384], [0, 0, -16384], [11585, 11585, 0], [0, 0, 0], [25000, 0, 0]];
    for previous in 0..4 {
        for xyz in ambiguous {
            assert_eq!(choose(xyz, previous), previous);
        }
    }
}

#[test]
fn extreme_readings_do_not_overflow() {
    for xyz in [[i16::MIN; 3], [i16::MAX; 3]] {
        assert_eq!(choose(xyz, 1), 1);
    }
}
