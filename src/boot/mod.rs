//! CPU entry and memory setup: what must happen before the application can
//! run on boot or wake. `link.ld` lays out memory; `start.S` and `wake.S`
//! prepare the CPU and call `main` and `wake_main` in `main.rs`.

#[cfg(target_os = "none")]
core::arch::global_asm!(include_str!("start.S"), include_str!("wake.S"), include_str!("memory.S"));

#[cfg(target_os = "none")]
unsafe extern "C" {
    /// Where ROM resumes after deep sleep, in `wake.S`
    pub fn wake_entry() -> !;
    /// End of the retained code and constants, from `link.ld`
    static _crc_end: u8;
}

/// Length of the retained code and constants, which ROM checks before resuming
#[cfg(target_os = "none")]
pub fn retained_bytes() -> u32 {
    const LP_RAM_BASE: usize = 0x5000_0000;
    ((&raw const _crc_end) as usize - LP_RAM_BASE) as u32
}

// Host test builds have no boot image

#[cfg(not(target_os = "none"))]
pub unsafe extern "C" fn wake_entry() -> ! {
    unreachable!()
}

#[cfg(not(target_os = "none"))]
pub fn retained_bytes() -> u32 {
    0
}

/// A panic stops here until the watchdog resets the CPU
#[cfg(target_os = "none")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    crate::hal::cpu::halt()
}
