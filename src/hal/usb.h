#pragma once

void usb_resume(void);
void usb_disconnect(void);

// Stop sending if any FIFO wait exceeds about 20 ms
void usb_puts(const char* s);
// Returns -1 when no byte is available
int usb_get(void);
