#include "usb.h"

#include "registers.h"
#include "timer.h"

void usb_resume(void) {
  // TOP shutdown resets the USB peripheral clock and pad enable
  REG(PCR_USB_CONFIG) = 1;
  REG(USB_CONFIG) = (REG(USB_CONFIG) | BIT(14)) & ~BIT(8);
}

void usb_disconnect(void) { REG(USB_CONFIG) &= ~BIT(14); }

void usb_puts(const char* s) {
  while (*s) {
    // A full uncommitted packet cannot drain, even after the host reconnects
    if (!(REG(USB_FIFO_CONFIG) & 2)) REG(USB_FIFO_CONFIG) = 1;
    unsigned timeout = 20000;
    while (!(REG(USB_FIFO_CONFIG) & 2) && --timeout) delay_us(1);
    if (!timeout) {
      REG(USB_FIFO_CONFIG) = 1;
      return;
    }
    REG(USB_FIFO) = (uint8_t)*s++;
  }
  REG(USB_FIFO_CONFIG) = 1;
}

int usb_get(void) {
  return (REG(USB_FIFO_CONFIG) & 4) ? (int)(REG(USB_FIFO) & 255) : -1;
}
