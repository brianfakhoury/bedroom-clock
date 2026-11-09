#include <assert.h>
#include <setjmp.h>

#include "app.h"
#include "board.h"
#include "clock.h"
#include "events.h"
#include "input.h"
#include "time_sync.h"
#include "watchdog.h"

#include "../../src/app/app.c"

static jmp_buf asleep;
static bool visible, held;
static unsigned initializations, activations, waits, updates, usb, sleeps;
static bool press_key, cancel_sleep;
static unsigned input_resets, contacts_consumed;

void clock_init(unsigned brightness) {
  initializations++;
  assert(brightness == 128);
  visible = held = false;
}

void clock_show(void) {
  visible = true;
  activations++;
}

bool clock_visible(void) { return visible; }

uint64_t clock_next_update(void) {
  assert(visible);
  return 1;
}

void input_reset(void) { input_resets++; }

void input_consume_touch(void) { contacts_consumed++; }

unsigned wait_for_event_until(uint64_t deadline) {
  assert(deadline && visible && activations > 0);
  // First frame precedes all waiting.
  assert(updates == waits && usb == waits / 2);
  waits++;
  return waits % 2 ? EVENT_TOUCH : EVENT_USB | EVENT_TIMER;
}

unsigned input_read(unsigned events) {
  assert(events == (waits % 2 ? EVENT_TOUCH : EVENT_USB | EVENT_TIMER));
  if (press_key && waits % 4 == 1) return INPUT_KEY;
  if (waits % 4 == 3) return INPUT_TAP;
  if (held && waits % 4 == 0) return INPUT_KEY;
  return 0;
}

void clock_toggle_always_on(void) {
  held = !held;
  assert(held == (waits % 4 == 1));
}

void clock_update(void) {
  assert(waits == updates + 1 && usb == updates / 2);
  if (press_key) assert(held == (waits % 4 != 0));
  unsigned session = (waits - 1) / 4;
  unsigned shows = waits % 4 == 1 || waits % 4 == 2 ? 1 : 2;
  assert(activations == session * 2 + shows);
  if (waits % 4 == 0) {
    assert(!held);
    visible = false;
  }
  updates++;
}

void time_sync_process(void) {
  assert(waits % 2 == 0 && updates == waits);  // USB event, after display work.
  assert(usb + 1 == waits / 2);
  usb++;
}

void watchdog_feed(void) {}

void board_sleep(void) {
  sleeps++;
  assert(!visible && input_resets == 1 && contacts_consumed == sleeps);
  if (cancel_sleep && sleeps == 1) return;
  longjmp(asleep, 1);
}

static void reset(void) {
  activations = waits = updates = usb = sleeps = 0;
  press_key = cancel_sleep = false;
  input_resets = contacts_consumed = 0;
  visible = held = false;
  initializations = 0;
}

static void test_boot_shows_time_then_sleeps(void) {
  reset();
  app_init();
  assert(initializations == 1 && activations == 0 && waits == 0);
  if (!setjmp(asleep)) app_run();
  assert(initializations == 1 && !visible && activations == 2 && usb == 2);
}

static void test_key_toggles_hold_while_awake(void) {
  reset();
  press_key = true;
  app_init();
  assert(initializations == 1 && activations == 0 && waits == 0);
  if (!setjmp(asleep)) app_run();
  assert(initializations == 1 && !visible && activations == 2 && usb == 2);
}

static void test_cancelled_sleep_preserves_input_history(void) {
  reset();
  cancel_sleep = true;
  if (!setjmp(asleep)) app_run();
  assert(sleeps == 2 && input_resets == 1 && contacts_consumed == 2);
  assert(initializations == 0 && activations == 4 && usb == 4);
}

int main(void) {
  test_boot_shows_time_then_sleeps();
  test_key_toggles_hold_while_awake();
  test_cancelled_sleep_preserves_input_history();
}
