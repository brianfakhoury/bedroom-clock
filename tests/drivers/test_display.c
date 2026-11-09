#include <assert.h>
#include <string.h>

#include "display.h"
#include "qspi.h"
#include "timer.h"

#include "../../src/drivers/display.c"

static uint8_t transfers[32][8];
static unsigned lengths[32], count, pixels;
static uint16_t values[4];
static uint32_t elapsed;

void qspi_begin(void) { assert(count < 32); }

void qspi_write_byte(uint8_t value) {
  assert(lengths[count] < 8);
  transfers[count][lengths[count]++] = value;
}

void qspi_write_quad16(uint16_t value) {
  if (pixels < 4) values[pixels] = value;
  pixels++;
}

void qspi_end(void) { count++; }

void delay_us(uint32_t us) { elapsed += us; }

static void reset(void) {
  memset(transfers, 0, sizeof(transfers));
  memset(lengths, 0, sizeof(lengths));
  count = pixels = elapsed = 0;
}

static uint16_t pixel(unsigned x, unsigned y) { return x + 480 * y; }

static void test_pixel_transfers(void) {
  reset();
  display_rect(256, 2, 2, 2, 0xf123);
  const uint8_t column[] = {2, 0, 0x2a, 0, 1, 0, 1, 1};
  const uint8_t row[] = {2, 0, 0x2b, 0, 0, 2, 0, 3};
  const uint8_t write[] = {2, 0, 0x2c, 0};
  const uint8_t stream[] = {0x32, 0, 0x3c, 0};
  assert(count == 4 && pixels == 4);
  assert(!memcmp(transfers[0], column, sizeof(column)));
  assert(!memcmp(transfers[1], row, sizeof(row)));
  assert(!memcmp(transfers[2], write, sizeof(write)));
  assert(!memcmp(transfers[3], stream, sizeof(stream)));
  for (unsigned i = 0; i < 4; i++) assert(values[i] == 0xf123);
  reset();
  display_pattern(2, 4, 2, 2, pixel);
  assert(pixels == 4);
  assert(values[0] == pixel(2, 4) && values[1] == pixel(3, 4));
  assert(values[2] == pixel(2, 5) && values[3] == pixel(3, 5));
}

static void test_sleep_preserves_pixels(void) {
  reset();
  display_sleep();
  assert(count == 2 && elapsed == 120000);
  assert(transfers[0][2] == 0x28 && transfers[1][2] == 0x10);
  display_sleep();
  assert(count == 2 && elapsed == 120000);
  reset();
  display_wake();
  assert(transfers[0][2] == 0x11 && elapsed == 140000);
  assert(pixels == 0 && count > 1);
  unsigned commands = count;
  display_wake();
  assert(count == commands && elapsed == 140000);
}

int main(void) {
  test_pixel_transfers();
  test_sleep_preserves_pixels();
}
