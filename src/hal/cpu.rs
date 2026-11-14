//! RISC-V instructions that Rust cannot express. Host test builds omit them.

#[cfg(target_arch = "riscv32")]
use core::arch::asm;

/// Let every interrupt source end a `wait_for_interrupt`. Global interrupts
/// stay masked, so no handler runs
pub fn unmask_interrupt_sources() {
    #[cfg(target_arch = "riscv32")]
    // SAFETY: `mstatus.MIE` stays clear; sources are only observed, never taken
    unsafe {
        asm!("csrw mie, {0}", in(reg) u32::MAX)
    }
}

/// Complete pending register writes before the clocks change
pub fn flush_writes() {
    #[cfg(target_arch = "riscv32")]
    // SAFETY: a memory fence has no effect on program state
    unsafe {
        asm!("fence iorw, iorw")
    }
}

/// Pause the CPU clock until an unmasked interrupt source is pending
pub fn wait_for_interrupt() {
    flush_writes();
    #[cfg(target_arch = "riscv32")]
    // SAFETY: execution resumes at the next instruction; no handler runs
    unsafe {
        asm!("wfi")
    }
}

/// Spin until a watchdog reset or power transition takes over
pub fn halt() -> ! {
    #[allow(clippy::empty_loop)]
    loop {}
}
