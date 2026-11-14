//! GPIO wake handoff, cancellation recovery and MCU deep-sleep entry.

use super::gpio::{Mode, Pin, Pull};
use super::registers::*;
use super::timer::delay_us;
use super::{cpu, timer, usb, watchdog};

/// Where ROM resumes after deep sleep
pub type WakeEntry = unsafe extern "C" fn() -> !;

// Fixed ROM functions from ESP-IDF's esp32c6.rom.ld (see docs/references)
const ROM_SET_WAKE_ENTRY: usize = 0x4000_00a4;
const ROM_GET_WAKE_ENTRY: usize = 0x4000_00a8;

/// Arm falling-edge wake on a GPIO from 0 to 7.
/// Returns false on a pending edge, with awake hardware already restored
pub fn prepare(wake_pin: Pin) -> bool {
    arm_wake_pin(wake_pin);

    // Cancel sleep if an edge arrived before or during the switch to LP GPIO
    if (GPIO_STATUS.read() | LP_IO_STATUS.read()) & wake_pin.mask() != 0 {
        GPIO_STATUS_CLEAR.write(wake_pin.mask());
        resume(wake_pin);
        return false;
    }
    true
}

/// Restore MCU clocks and the wake pin after deep sleep or cancelled preparation
pub fn resume(wake_pin: Pin) {
    LP_TIMER_TARGET_HIGH.write(0);
    LP_TIMER_CLEAR.write(bit(31));
    LP_GPIO_MUX.clear(wake_pin.mask());
    LP_IO_PIN0.offset(wake_pin.0 as u32 * 4).write(bit(3));
    LP_IO_CLEAR.write(wake_pin.mask());
    RTC_SLEEP_MODE.clear(1);
    watchdog::disable_all();
    timer::init();
    usb::resume();
    watchdog::arm();
    wake_pin.configure(Mode::Input, Pull::Up);
}

fn arm_wake_pin(wake_pin: Pin) {
    LP_TIMER_TARGET_HIGH.write(0);
    LP_TIMER_CLEAR.write(bit(31));
    LPPERI_CLK_EN.set(bit(27));
    LP_IO_DATE.set(bit(31));
    LP_IO_OUT_ENABLE_W1TC.write(wake_pin.mask());
    LP_IO_PAD0.offset(wake_pin.0 as u32 * 4).write(bit(9) | bit(8) | bit(4) | bit(3));
    LP_IO_CLEAR.write(wake_pin.mask());
    LP_IO_PIN0.offset(wake_pin.0 as u32 * 4).write(bit(10) | (2 << 7) | bit(3));
    LP_GPIO_MUX.set(wake_pin.mask());
}

/// After successful preparation, enter deep sleep.
/// ROM resumes at `entry` after checking `retained_bytes` from the LP RAM base
pub fn enter(entry: WakeEntry, retained_bytes: u32, high_outputs: u32, held_outputs: u32) -> ! {
    // SAFETY: mask ROM provides these functions at fixed addresses
    let (set_wake_entry, get_wake_entry) = unsafe {
        (
            core::mem::transmute::<usize, extern "C" fn(WakeEntry, u32)>(ROM_SET_WAKE_ENTRY),
            core::mem::transmute::<usize, extern "C" fn() -> u32>(ROM_GET_WAKE_ENTRY),
        )
    };

    RTC_SLEEP_MODE.set(1);
    LP_AON_SYS_CFG.clear(bit(30));
    // ROM uses this address and checksum to resume from LP RAM after deep sleep
    set_wake_entry(entry, retained_bytes);
    // Let the watchdog reset if ROM rejects the saved wake entry
    if get_wake_entry() != entry as usize as u32 {
        cpu::halt();
    }
    PMU_WAKEUP_CNTL1.write(0);
    PMU_WAKEUP_CNTL2.write(bit(2));
    PMU_HP_INT_CLR.write(bit(31) | bit(30));
    PMU_WAKEUP_CNTL4.write(bit(31));
    usb::disconnect();
    delay_us(100_000);
    GPIO_OUT_SET.write(high_outputs);
    LP_GPIO_HOLD.write(held_outputs);
    watchdog::feed();
    cpu::flush_writes();
    PMU_WAKEUP_CNTL0.write(bit(31));
    // Deep-sleep wake restarts through ROM, not this stack
    cpu::halt()
}
