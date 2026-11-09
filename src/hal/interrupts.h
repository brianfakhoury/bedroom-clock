#pragma once
#include <stdbool.h>
#include <stdint.h>

typedef struct {
  uint32_t gpio;
  bool usb, timer;
} interrupt_events;

// Enable falling-edge GPIO events plus USB receive and timer events
void interrupts_init(uint32_t gpio_mask);
// Wait for an event or an absolute deadline in ticks; zero waits indefinitely
interrupt_events interrupts_wait_until(uint64_t deadline);
