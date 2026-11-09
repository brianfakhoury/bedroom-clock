#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <string.h>

#include "battery_indicator.h"
#include "display.h"

#include "../../src/app/battery_indicator.c"
#include "../../src/app/geometry.c"

static uint16_t pixels[480][480], reference[480][480];
static unsigned writes, transferred;

void display_pattern(unsigned x, unsigned y, unsigned w, unsigned h,
                     uint16_t (*pixel)(unsigned, unsigned)) {
  assert(w && h && x + w <= 480 && y + h <= 480);
  assert(!((x | y | w | h) & 1));
  writes++;
  transferred += w * h;
  for (unsigned j = y; j < y + h; j++)
    for (unsigned i = x; i < x + w; i++) {
      assert(i < 72 || i >= 408 || j < 72 || j >= 408);
      pixels[j][i] = pixel(i, j);
    }
}

static void test_fill_and_cache(void) {
  battery_indicator_draw(101, 0);
  unsigned prior = 0;
  for (unsigned p = 0; p <= 100; p++) {
    transferred = 0;
    battery_indicator_draw(p, 0);
    assert(transferred < 15000);
    unsigned lit = 0;
    for (unsigned y = 0; y < 480; y++)
      for (unsigned x = 0; x < 480; x++) lit += pixels[y][x] == 0x4000;
    assert(lit >= prior);
    if (!p) assert(!lit);
    prior = lit;
    writes = 0;
    battery_indicator_draw(p, 0);
    assert(!writes);
  }
}

static void test_stroke_geometry(void) {
  battery_indicator_draw(100, 0);
  // Check the complete visible band against the two circle boundaries,
  // independently of the renderer's narrow transfer-window table.
  for (unsigned y = 0; y < 480; y++) {
    for (unsigned x = 0; x < 480; x++) {
      unsigned dx = x < 240 ? x : 479 - x;
      unsigned dy = y < 240 ? y : 479 - y;
      bool band;
      if (dx < 72 && dy < 72) {
        unsigned cx = 143 - 2 * dx, cy = 143 - 2 * dy;
        unsigned distance = cx * cx + cy * cy;
        if (distance > 144 * 144) continue;  // Physical screen mask.
        band = distance > 136 * 136;
      } else {
        band = dx < 4 || dy < 4;
      }
      assert(pixels[y][x] == (band ? 0x4000 : 0));
    }
  }
}

static void test_clockwise_fill_and_rotation(void) {
  battery_indicator_draw(25, 0);
  assert(pixels[479][230] == 0x4000 && pixels[479][250] == 0x0800);
  assert(pixels[300][0] == 0x4000 && pixels[200][0] == 0x0800);
  memcpy(reference, pixels, sizeof(pixels));
  for (unsigned r = 1; r <= 4; r++) {
    battery_indicator_draw(25, r & 3);
    for (unsigned y = 0; y < 480; y++)
      for (unsigned x = 0; x < 480; x++)
        assert(pixels[y][x] == reference[479 - x][y]);
    memcpy(reference, pixels, sizeof(pixels));
  }
}

static void test_unavailable_reading_hides_ring(void) {
  battery_indicator_draw(75, 0);
  battery_indicator_draw(101, 0);
  for (unsigned y = 0; y < 480; y++)
    for (unsigned x = 0; x < 480; x++) assert(!pixels[y][x]);
}

int main(void) {
  test_fill_and_cache();
  test_stroke_geometry();
  test_clockwise_fill_and_rotation();
  test_unavailable_reading_hides_ring();
}
