#include <assert.h>
#include <string.h>

#include "i2c.h"
#include "rtc.h"

#include "../../src/drivers/rtc.c"

static uint8_t regs[32];
static bool fail_time_write;
static unsigned restarts;

bool i2c_read(uint8_t a, uint8_t r, uint8_t* b, unsigned n) {
  assert(a == 0x51);
  memcpy(b, regs + r, n);
  return true;
}

bool i2c_write(uint8_t a, uint8_t r, const uint8_t* b, unsigned n) {
  assert(a == 0x51);
  if (r == 4 && fail_time_write) return false;
  if (r == 0 && !(*b & 0x20)) restarts++;
  memcpy(regs + r, b, n);
  return true;
}

static void test_clock_output_disabled(void) {
  regs[1] = 0xb0;
  assert(rtc_clock_output_disable());
  assert((regs[1] & 0xb7) == 0xb7);
}

static void test_round_trip_and_invalid_registers(void) {
  datetime d = {2026, 11, 1, 16, 0, 0}, got;
  assert(rtc_set(&d) && rtc_read(&got));
  assert(got.year == 2026 && got.month == 11 && got.day == 1 && got.hour == 16);
  regs[4] |= 128;
  assert(!rtc_read(&got));
  regs[4] &= 127;
  regs[0] |= 32;
  assert(!rtc_read(&got));
  regs[0] &= ~32;
  regs[5] = 0x6a;
  assert(!rtc_read(&got));
}

static void test_calendar_validation(void) {
  datetime d = {2026, 2, 29, 0, 0, 0};
  assert(!date_valid(&d));
  d.year = 2028;
  assert(date_valid(&d));
  d.second = 60;
  assert(!date_valid(&d));
  d.second = 0;
  d.month = 0;
  assert(!date_valid(&d));
}

static void test_failed_write_restarts_clock(void) {
  datetime d = {2026, 11, 1, 16, 0, 0};
  unsigned before = restarts;
  fail_time_write = true;
  assert(!rtc_set(&d));
  assert(restarts == before + 1 && !(regs[0] & 0x20));
  fail_time_write = false;
}

int main(void) {
  test_clock_output_disabled();
  test_round_trip_and_invalid_registers();
  test_calendar_validation();
  test_failed_write_restarts_clock();
}
