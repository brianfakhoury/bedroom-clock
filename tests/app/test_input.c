#include <assert.h>

#include "events.h"
#include "input.h"
#include "timer.h"
#include "touch.h"

#include "../../src/app/input.c"

static uint64_t now;
static bool touching, readable = true;

uint64_t ticks(void) { return now; }

bool touch_read(bool* down) {
  *down = touching;
  return readable;
}

static unsigned button_at(unsigned ms, unsigned events) {
  now = (uint64_t)ms * TICKS_PER_MS;
  return input_read(events);
}

static void test_touch_requires_a_new_press(void) {
  input_consume_touch();
  touching = true;
  assert(input_read(EVENT_TOUCH) == 0);
  touching = false;
  readable = false;
  assert(input_read(EVENT_TOUCH) == 0);
  readable = true;
  touching = true;
  // Failed release did not change history.
  assert(input_read(EVENT_TOUCH) == 0);
  touching = false;
  assert(input_read(EVENT_TOUCH) == 0);
  touching = true;
  assert(input_read(EVENT_TOUCH) == INPUT_TAP);
  assert(input_read(EVENT_TOUCH) == 0);
}

static void test_button_debounce_and_wake(void) {
  input_reset();
  assert(button_at(50, EVENT_KEY) == INPUT_KEY);
  assert(button_at(100, EVENT_KEY) == 0);
  assert(button_at(250, EVENT_BOOT) == 0);
  assert(button_at(251, EVENT_BOOT) == INPUT_BOOT);
  // Cancelled sleep preserves debounce history.
  input_consume_touch();
  assert(button_at(300, EVENT_KEY) == 0);
  input_reset();  // A CPU wake starts a fresh timer/debounce interval.
  assert(button_at(50, EVENT_KEY) == INPUT_KEY);
}

static void test_simultaneous_inputs(void) {
  input_reset();
  input_consume_touch();
  touching = false;
  assert(input_read(EVENT_TOUCH) == 0);
  touching = true;
  unsigned events = EVENT_TOUCH | EVENT_BOOT | EVENT_KEY;
  assert(button_at(50, events) == (INPUT_TAP | INPUT_KEY));
  assert(input_read(EVENT_USB | EVENT_TIMER) == 0);
}

int main(void) {
  test_touch_requires_a_new_press();
  test_button_debounce_and_wake();
  test_simultaneous_inputs();
}
