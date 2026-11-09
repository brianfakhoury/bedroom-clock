#include "app.h"

#include "board.h"
#include "clock.h"
#include "events.h"
#include "input.h"
#include "time_sync.h"
#include "watchdog.h"

#define DEFAULT_BRIGHTNESS 128u

void app_init(void) { clock_init(DEFAULT_BRIGHTNESS); }

void app_run(void) {
  input_reset();

show:
  clock_show();
  input_consume_touch();
  while (clock_visible()) {
    unsigned events = wait_for_event_until(clock_next_update());
    unsigned input = input_read(events);
    if (input & (INPUT_TAP | INPUT_BOOT)) clock_show();
    if (input & INPUT_KEY) clock_toggle_always_on();
    clock_update();
    if (events & EVENT_USB) time_sync_process();
    watchdog_feed();
  }

  board_sleep();
  goto show;  // A pending touch cancelled sleep
}
