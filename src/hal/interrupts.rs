use super::registers::*;
use super::timer::ticks;
use super::{cpu, watchdog};

/// Raw events captured by one wait
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Pending {
    /// Pins that saw a falling edge
    pub gpio: u32,
    pub usb: bool,
    pub timer: bool,
}

/// Enable falling-edge events on `gpio_mask`, plus USB receive and timer events
pub fn init(gpio_mask: u32) {
    // WFI pauses the CPU clock without entering light sleep
    // Global interrupts stay masked so events are handled here, not in an ISR
    PCR_CPU_WAITI.clear(bit(3));
    PLIC_ENABLE.write(0);
    PLIC_TYPE.write(0);
    PLIC_PRIORITY1.write(1);
    PLIC_THRESHOLD.write(0);
    IRQ_MAP_GPIO.write(1);
    IRQ_MAP_USB.write(1);
    IRQ_MAP_SYSTIMER0.write(1);
    USB_IRQ_ENABLE.write(bit(2)); // USB received packet only
    for pin in (0..32).filter(|&pin| gpio_mask & bit(pin) != 0) {
        GPIO_PIN0.offset(pin * 4).write(bit(13) | (2 << 7));
    }
    GPIO_STATUS_CLEAR.write(gpio_mask);
    SYSTIMER_IRQ_ENABLE.write(1);
    PLIC_ENABLE.write(bit(1));
    cpu::unmask_interrupt_sources();
}

/// Wait for an event on `gpio_mask`, USB or the timer.
/// The deadline is absolute, in ticks; without one, wait indefinitely
pub fn wait_until(gpio_mask: u32, deadline: Option<u64>) -> Pending {
    let expired = || deadline.is_some_and(|deadline| ticks() >= deadline);
    let usb_unread = || USB_FIFO_CONFIG.read() & 4 != 0;

    SYSTIMER_CONFIG.clear(bit(24));
    SYSTIMER_IRQ_CLEAR.write(1);
    if let Some(deadline) = deadline {
        if ticks() >= deadline {
            return Pending { timer: true, ..Pending::default() };
        }
        SYSTIMER_TARGET0_CONFIG.write(0);
        SYSTIMER_TARGET0_HIGH.write((deadline >> 32) as u32);
        SYSTIMER_TARGET0_LOW.write(deadline as u32);
        SYSTIMER_TARGET0_LOAD.write(1);
        SYSTIMER_CONFIG.set(bit(24));
    }
    // Check unread USB bytes explicitly; other events remain latched for WFI
    if !(usb_unread() || expired()) {
        watchdog::disarm();
        cpu::wait_for_interrupt();
        watchdog::arm();
    }
    let mut pending =
        Pending { gpio: GPIO_STATUS.read() & gpio_mask, usb: usb_unread(), timer: expired() };
    GPIO_STATUS_CLEAR.write(pending.gpio);
    if USB_IRQ_RAW.read() & bit(2) != 0 {
        USB_IRQ_CLEAR.write(bit(2));
        pending.usb = true;
    }
    if SYSTIMER_IRQ_RAW.read() & 1 != 0 {
        SYSTIMER_IRQ_CLEAR.write(1);
        pending.timer = true;
    }
    SYSTIMER_CONFIG.clear(bit(24));
    pending
}
