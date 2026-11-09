#include "clock.h"

#include "accel.h"
#include "animation.h"
#include "battery_indicator.h"
#include "digits.h"
#include "display.h"
#include "local_time.h"
#include "orientation.h"
#include "pmic.h"
#include "rtc.h"
#include "timer.h"

// Presentation state; the external RTC keeps time
typedef struct {
  bool visible, held, ring_pending;
  unsigned brightness, rotation, fade_from, level;
  uint64_t shown_at, time_refreshed_at;
} clock_state;

static clock_state clock;

static bool update_orientation(void) {
  int16_t xyz[3];
  if (!accel_read(xyz)) return false;
  unsigned rotation = orientation_choose(xyz, clock.rotation);
  if (rotation == clock.rotation) return false;

  display_off();
  // Erase the digits in their old orientation before rotating
  digits_clear(clock.rotation);
  clock.rotation = rotation;
  return true;
}

static void refresh_time(void) {
  datetime utc;
  unsigned hour = 0, minute = 0;
  bool valid = rtc_read(&utc);
  if (valid) {
    pacific_time(&utc, &hour, &minute);
    hour %= 12;
    if (!hour) hour = 12;
  }
  display_brightness(clock.level);
  digits_draw(hour, minute, valid, clock.rotation, DIGITS_RED);
  clock.ring_pending = true;
  display_on();
  clock.visible = true;
  clock.time_refreshed_at = ticks();
}

static void refresh_ring(void) {
  unsigned percentage = 101;
  pmic_percentage(&percentage);
  battery_indicator_draw(percentage, clock.rotation);
  clock.ring_pending = false;
}

void clock_init(unsigned brightness) {
  clock = (clock_state){.brightness = brightness};
  // Prime the ring pixels for the first fade-in
  refresh_ring();
}

void clock_show(void) {
  display_wake();
  bool rotated = update_orientation();
  clock.fade_from = clock.visible && !rotated ? clock.level : 0;
  clock.level = clock.fade_from;
  refresh_time();
  clock.shown_at = ticks();
}

void clock_toggle_always_on(void) {
  clock.held = !clock.held && clock.visible;
  if (clock.held || !clock.visible) return;

  // Releasing an expired hold starts fade-out without another viewing interval
  uint64_t fade_start = (DISPLAY_DURATION_MS - FADE_OUT_MS) * TICKS_PER_MS;
  if (ticks() - clock.shown_at > fade_start)
    clock.shown_at = ticks() - fade_start;
}

// Clamp long KEY holds to the animation interval
static unsigned animation_elapsed_ms(void) {
  uint64_t elapsed = ticks() - clock.shown_at;
  if (elapsed >= DISPLAY_DURATION_MS * TICKS_PER_MS) return DISPLAY_DURATION_MS;
  return (unsigned)elapsed / TICKS_PER_MS;
}

void clock_update(void) {
  if (!clock.visible) return;
  unsigned elapsed = animation_elapsed_ms();
  bool fading_in = elapsed < FADE_IN_MS;
  bool fading_out = !clock.held && elapsed >= DISPLAY_DURATION_MS - FADE_OUT_MS;
  // Refresh time without restarting the fade
  if (clock.held && ticks() - clock.time_refreshed_at >= 1000 * TICKS_PER_MS) {
    update_orientation();
    refresh_time();
  }
  unsigned animation_ms = clock.held && !fading_in ? FADE_IN_MS : elapsed;
  unsigned level =
      animation_level(animation_ms, clock.fade_from, clock.brightness);
  if (level != clock.level) {
    display_brightness(level);
    clock.level = level;
  }
  // Defer ring updates until the time is fully visible
  if (clock.ring_pending && !fading_in && !fading_out) refresh_ring();
  if (clock.held || elapsed < DISPLAY_DURATION_MS) return;
  display_off();
  clock.visible = false;
}

uint64_t clock_next_update(void) {
  if (!clock.visible) return 0;
  unsigned elapsed = animation_elapsed_ms();
  if (clock.held && elapsed >= FADE_IN_MS)
    return clock.time_refreshed_at + 1000 * TICKS_PER_MS;
  // Keep display timing independent of time spent on other work
  uint64_t next_ms = elapsed + animation_next_ms(elapsed);
  return clock.shown_at + next_ms * TICKS_PER_MS;
}

bool clock_visible(void) { return clock.visible; }
