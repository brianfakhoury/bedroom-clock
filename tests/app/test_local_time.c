#include <assert.h>

#include "i2c.h"
#include "local_time.h"

#include "../../src/app/local_time.c"
#include "../../src/drivers/rtc.c"

// Time-zone conversion must not touch the RTC bus.
bool i2c_read(uint8_t a, uint8_t r, uint8_t* b, unsigned n) {
  (void)a;
  (void)r;
  (void)b;
  (void)n;
  assert(false);
  return false;
}

bool i2c_write(uint8_t a, uint8_t r, const uint8_t* b, unsigned n) {
  (void)a;
  (void)r;
  (void)b;
  (void)n;
  assert(false);
  return false;
}

static void local(unsigned mon, unsigned day, unsigned hour, unsigned min,
                  unsigned expect) {
  datetime d = {2026, mon, day, hour, min, 0};
  unsigned h, m;
  pacific_time(&d, &h, &m);
  assert(h == expect && m == min);
}

static void test_dst_boundaries(void) {
  local(3, 8, 9, 59, 1);
  local(3, 8, 10, 0, 3);
  local(11, 1, 8, 59, 1);
  local(11, 1, 9, 0, 1);
  local(11, 1, 10, 0, 2);
}

static void test_utc_day_rollover(void) {
  local(1, 1, 0, 0, 16);
  local(7, 1, 0, 0, 17);
}

int main(void) {
  test_dst_boundaries();
  test_utc_day_rollover();
}
