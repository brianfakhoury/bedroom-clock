use crate::board::waveshare::reset_panel;
use crate::tests::drivers::pmic::Chip;
use crate::tests::fakes::Elapsed;

#[test]
fn panel_reset_pulses_aldo3() {
    let (mut chip, elapsed) = (Chip::new(), Elapsed::default());
    assert!(reset_panel(&mut chip, &mut &elapsed).is_ok());
    assert_eq!(chip.registers[0x94] & 0x1f, 28); // 3300 mV
    assert_eq!(chip.registers[0x90] & 4, 4); // Released
    assert_eq!(chip.writes, 4);
    assert_eq!(elapsed.us(), 300_000);
}

#[test]
fn panel_reset_stops_at_the_first_failure() {
    // Each step reads, writes and reads back
    for (failed_step, delays) in [(0, 0), (1, 0), (2, 1), (3, 2)] {
        let (mut chip, elapsed) = (Chip::new(), Elapsed::default());
        chip.fail_at = failed_step * 3 + 2;
        assert!(reset_panel(&mut chip, &mut &elapsed).is_err());
        assert_eq!(chip.operations, failed_step * 3 + 2);
        assert_eq!(elapsed.us(), delays * 100_000);
    }
}
