#include "events.h"

#include "interrupts.h"
#include "pins.h"

void events_init(void) {
  interrupts_init((1u << BUTTON_BOOT) | (1u << BUTTON_KEY) | (1u << TOUCH_IRQ));
}

unsigned wait_for_event_until(uint64_t deadline) {
  interrupt_events pending = interrupts_wait_until(deadline);
  unsigned events = 0;
  if (pending.gpio & (1u << BUTTON_BOOT)) events |= EVENT_BOOT;
  if (pending.gpio & (1u << BUTTON_KEY)) events |= EVENT_KEY;
  if (pending.gpio & (1u << TOUCH_IRQ)) events |= EVENT_TOUCH;
  if (pending.usb) events |= EVENT_USB;
  if (pending.timer) events |= EVENT_TIMER;
  return events;
}
