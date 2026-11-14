//! The built-in USB serial port.

use super::registers::*;
use super::timer::delay_us;

pub fn resume() {
    // TOP shutdown resets the USB peripheral clock and pad enable
    PCR_USB_CONFIG.write(1);
    USB_CONFIG.write((USB_CONFIG.read() | bit(14)) & !bit(8));
}

pub fn disconnect() {
    USB_CONFIG.clear(bit(14));
}

/// Stop sending if any FIFO wait exceeds about 20 ms
pub fn puts(text: &str) {
    let writable = || USB_FIFO_CONFIG.read() & 2 != 0;
    let commit = || USB_FIFO_CONFIG.write(1);

    for byte in text.bytes() {
        // A full uncommitted packet cannot drain, even after the host reconnects
        if !writable() {
            commit();
        }
        let mut timeout = 20_000;
        while !writable() {
            timeout -= 1;
            if timeout == 0 {
                commit();
                return;
            }
            delay_us(1);
        }
        USB_FIFO.write(byte as u32);
    }
    commit();
}

/// The next received byte, if any
pub fn get() -> Option<u8> {
    (USB_FIFO_CONFIG.read() & 4 != 0).then(|| USB_FIFO.read() as u8)
}
