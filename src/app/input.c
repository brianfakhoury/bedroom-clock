#include "input.h"

#include "events.h"
#include "timer.h"
#include "touch.h"

#define BUTTON_DEBOUNCE_MS 200u

static bool was_touching;
static uint64_t last_button_ticks;

void input_reset(void) { last_button_ticks = 0; }

void input_consume_touch(void) { was_touching = true; }

static bool touch_pressed(void) {
  bool touching;
  if (!touch_read(&touching)) return false;
  bool pressed = touching && !was_touching;
  was_touching = touching;
  return pressed;
}

unsigned input_read(unsigned events) {
  unsigned input = 0;
  if ((events & EVENT_TOUCH) && touch_pressed()) input |= INPUT_TAP;
  if (!(events & EVENT_BUTTON)) return input;
  uint64_t now = ticks();
  if (last_button_ticks &&
      now - last_button_ticks <= BUTTON_DEBOUNCE_MS * TICKS_PER_MS)
    return input;

  last_button_ticks = now;
  return input | (events & EVENT_KEY ? INPUT_KEY : INPUT_BOOT);
}
