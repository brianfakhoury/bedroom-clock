#include <assert.h>

#include "events.h"
#include "interrupts.h"
#include "pins.h"

#include "../../src/board/events.c"

static interrupt_events pending;
static uint64_t expected_deadline;

void interrupts_init(uint32_t gpio_mask) {
  assert(gpio_mask == ((1u << 5) | (1u << 9) | (1u << 10)));
}

interrupt_events interrupts_wait_until(uint64_t deadline) {
  assert(deadline == expected_deadline);
  return pending;
}

int main(void) {
  events_init();
  const unsigned pins[] = {BUTTON_BOOT, BUTTON_KEY, TOUCH_IRQ};
  const unsigned events[] = {EVENT_BOOT, EVENT_KEY, EVENT_TOUCH};
  expected_deadline = 12345;
  for (unsigned i = 0; i < 3; i++) {
    pending = (interrupt_events){.gpio = 1u << pins[i]};
    assert(wait_for_event_until(expected_deadline) == events[i]);
  }
  pending = (interrupt_events){.usb = true};
  assert(wait_for_event_until(expected_deadline) == EVENT_USB);
  pending = (interrupt_events){.timer = true};
  assert(wait_for_event_until(expected_deadline) == EVENT_TIMER);
  pending = (interrupt_events){.gpio = ~0u, .usb = true, .timer = true};
  expected_deadline = 0;
  assert(wait_for_event_until(0) ==
         (EVENT_BUTTON | EVENT_TOUCH | EVENT_USB | EVENT_TIMER));
  pending = (interrupt_events){.gpio = 1u << 17};
  assert(wait_for_event_until(0) == 0);
}
