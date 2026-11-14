//! The LP watchdog guards active work; `interrupts::wait_until` pauses it
//! while waiting.

use super::registers::*;

const UNLOCK_KEY: u32 = 0x50d8_3aa1;

fn unlocked(protect: Reg, change: impl FnOnce()) {
    protect.write(UNLOCK_KEY);
    change();
    protect.write(0);
}

/// Clear watchdogs left running by ROM before configuring our own
pub fn disable_all() {
    for group in [0, TIMG1_OFFSET] {
        unlocked(TIMG0_WDT_PROTECT.offset(group), || TIMG0_WDT_CONFIG.offset(group).write(0));
    }
    unlocked(LP_WDT_PROTECT, || LP_WDT_CONFIG.write(0));
    unlocked(LP_SWD_PROTECT, || LP_SWD_CONFIG.set(bit(30)));
}

/// Reset a stalled CPU after roughly 8 seconds using RC_SLOW
pub fn arm() {
    // Compensate for the eFuse WDT_DELAY_SEL multiplier
    let shift = 1 + ((EFUSE_WDT_DELAY.read() >> 16) & 3);
    unlocked(LP_WDT_PROTECT, || {
        LP_WDT_CONFIG.write(0);
        LP_WDT_TIMEOUT.write(1_200_000 >> shift);
        LP_WDT_CONFIG.write(bit(31) | (4 << 28) | (7 << 13) | (7 << 16) | bit(8) | 20);
        LP_WDT_FEED.write(bit(31));
    });
}

pub fn disarm() {
    unlocked(LP_WDT_PROTECT, || LP_WDT_CONFIG.write(0));
}

pub fn feed() {
    unlocked(LP_WDT_PROTECT, || LP_WDT_FEED.write(bit(31)));
}
