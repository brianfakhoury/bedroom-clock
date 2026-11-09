#include <assert.h>

#include "accel.h"
#include "battery_indicator.h"
#include "clock.h"
#include "digits.h"
#include "display.h"
#include "local_time.h"
#include "pmic.h"
#include "rtc.h"
#include "timer.h"

#include "../../src/app/clock.c"
#include "../../src/app/animation.c"
#include "../../src/app/orientation.c"

static uint64_t now;
static bool valid_rtc, valid_accel, valid_battery;
static unsigned battery_drawn, battery_reads;
static int16_t acceleration[3];
static unsigned panel_level, on_count, off_count, erase_count;
static bool drawn_valid;
static unsigned drawn_rotation, drawn_hour, local_hour;

uint64_t ticks(void) { return now; }

bool accel_read(int16_t xyz[3]) {
  for (unsigned i = 0; i < 3; i++) xyz[i] = acceleration[i];
  return valid_accel;
}

bool rtc_read(datetime* d) {
  *d = (datetime){2026, 11, 1, 16, 0, 0};
  return valid_rtc;
}

void pacific_time(const datetime* d, unsigned* h, unsigned* m) {
  (void)d;
  *h = local_hour;
  *m = 0;
}

void display_wake(void) {}

void display_on(void) { on_count++; }

void display_off(void) { off_count++; }

void display_brightness(unsigned value) { panel_level = value; }

void digits_draw(unsigned h, unsigned m, bool valid, unsigned rotation,
                 uint16_t ink) {
  (void)m;
  (void)ink;
  drawn_hour = h;
  drawn_valid = valid;
  drawn_rotation = rotation;
}

void digits_clear(unsigned rotation) {
  assert(rotation == drawn_rotation);
  erase_count++;
}

bool pmic_percentage(unsigned* percentage) {
  // Never delay the initial visible time.
  assert(panel_level > 0 || on_count == 0);
  battery_reads++;
  if (!valid_battery) return false;
  *percentage = 75;
  return true;
}

void battery_indicator_draw(unsigned percentage, unsigned rotation) {
  assert(rotation < 4);
  battery_drawn = percentage;
}

static void at(unsigned ms) { now = (uint64_t)ms * TICKS_PER_MS; }

static void reset(void) {
  now = 0;
  valid_rtc = valid_accel = valid_battery = true;
  battery_drawn = battery_reads = 0;
  acceleration[0] = acceleration[2] = 0;
  acceleration[1] = 16384;
  panel_level = on_count = off_count = erase_count = 0;
  drawn_valid = false;
  drawn_rotation = drawn_hour = 0;
  local_hour = 8;
  clock_init(255);
  battery_drawn = battery_reads = 0;
}

static void test_initialization_primes_ring(void) {
  reset();
  clock_init(128);
  assert(!clock_visible());
  assert(battery_drawn == 75 && battery_reads == 1 && on_count == 0);
  valid_battery = false;
  clock_init(128);
  assert(battery_drawn == 101);
  clock_show();
  at(180);
  clock_update();
  assert(panel_level == 128);
}

static void test_lifecycle_and_retrigger(void) {
  reset();
  clock_update();
  assert(clock_next_update() == 0 && !clock_visible());
  clock_toggle_always_on();
  // A dark display cannot acquire a hold; it must still fade out below.
  clock_show();
  assert(clock_visible() && on_count == 1 && panel_level == 0 && drawn_valid);
  assert(!battery_reads);
  at(100);
  clock_update();
  assert(!battery_reads);
  at(180);
  clock_update();
  assert(panel_level == 255);
  assert(battery_reads == 1);
  assert(clock_next_update() == 2650 * TICKS_PER_MS);
  assert(battery_drawn == 75);
  at(2800);
  clock_update();
  assert(panel_level > 0 && panel_level < 255);
  unsigned level = panel_level;
  clock_show();
  assert(panel_level == level);  // Retrigger without jump.
  at(2980);
  clock_update();
  assert(panel_level == 255);
  at(5800);
  clock_update();
  assert(!clock_visible() && panel_level == 0);
  assert(off_count == 1 && clock_next_update() == 0);
  assert(battery_drawn == 75);  // Retain the filled ring for the next fade-in.
  clock_update();
  assert(off_count == 1);
}

static void test_retap_at_timeout_precedes_update(void) {
  reset();
  clock_show();
  at(3000);
  clock_show();  // Input is handled before the expired frame is updated.
  clock_update();
  assert(clock_visible() && off_count == 0);
  assert(clock_next_update() == now + 10 * TICKS_PER_MS);
}

static void test_rotation_and_sensor_failures(void) {
  reset();
  acceleration[0] = 16384;
  acceleration[1] = 0;
  clock_show();
  assert(drawn_rotation == 3 && erase_count == 1);
  assert(panel_level == 0);
  valid_accel = false;
  valid_rtc = false;
  clock_show();
  assert(!drawn_valid && drawn_rotation == 3 && erase_count == 1);
  valid_rtc = true;
}

static void test_twelve_hour_boundaries(void) {
  reset();
  const unsigned hours[] = {0, 1, 11, 12, 13, 23};
  const unsigned expected[] = {12, 1, 11, 12, 1, 11};
  for (unsigned i = 0; i < sizeof(hours) / sizeof(hours[0]); i++) {
    local_hour = hours[i];
    clock_show();
    assert(drawn_hour == expected[i]);
  }
}

static void test_expired_hold_fades_without_restarting_interval(void) {
  reset();
  at(10000);
  clock_show();
  clock_toggle_always_on();
  at(10180);
  clock_update();
  assert(panel_level == 255);
  assert(clock_next_update() == 11000 * TICKS_PER_MS);
  local_hour = 4;
  acceleration[0] = 16384;
  acceleration[1] = 0;
  at(86410000);  // A day-long hold still refreshes and releases normally.
  clock_update();
  assert(clock_visible() && drawn_hour == 4 && drawn_rotation == 3);
  assert(erase_count == 1);
  assert(panel_level == 255);
  clock_toggle_always_on();
  clock_update();
  assert(panel_level == 255);
  at(86410175);
  clock_update();
  assert(panel_level > 0 && panel_level < 255);
  at(86410350);
  clock_update();
  assert(!clock_visible());
}

static void test_early_hold_release_preserves_deadline(void) {
  reset();
  at(20000);
  clock_show();
  at(20180);
  clock_update();
  clock_toggle_always_on();
  at(21000);
  clock_toggle_always_on();
  assert(clock_next_update() == 22650 * TICKS_PER_MS);
  assert(battery_drawn == 75);
}

static void test_failed_battery_read_hides_ring(void) {
  reset();
  clock_show();
  at(180);
  clock_update();
  valid_battery = false;
  unsigned reads_before = battery_reads;
  clock_show();
  assert(battery_reads == reads_before);
  now += 180 * TICKS_PER_MS;
  clock_update();
  assert(battery_drawn == 101 && battery_reads == reads_before + 1);
}

int main(void) {
  test_initialization_primes_ring();
  test_lifecycle_and_retrigger();
  test_retap_at_timeout_precedes_update();
  test_rotation_and_sensor_failures();
  test_twelve_hour_boundaries();
  test_expired_hold_fades_without_restarting_interval();
  test_early_hold_release_preserves_deadline();
  test_failed_battery_read_hides_ring();
}
