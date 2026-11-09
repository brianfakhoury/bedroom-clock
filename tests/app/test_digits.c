#include <assert.h>
#include <string.h>

#include "digits.h"
#include "display.h"

#include "../../src/app/digits.c"
#include "../../src/app/geometry.c"

static uint16_t pixels[480][480], expected[480][480];
static unsigned pixels_written;

void display_rect(unsigned x, unsigned y, unsigned w, unsigned h,
                  uint16_t color) {
  assert(x + w <= 480 && y + h <= 480);
  assert(!((x | y | w | h) & 1));
  pixels_written += w * h;
  for (unsigned row = y; row < y + h; row++)
    for (unsigned col = x; col < x + w; col++) pixels[row][col] = color;
}

static void test_centered_dimensions(void) {
  digits_clear(0);
  memset(pixels, 0, sizeof(pixels));
  digits_draw(8, 58, true, 0, DIGITS_RED);
  unsigned left = 480, right = 0, top = 480, bottom = 0;
  for (unsigned y = 0; y < 480; y++)
    for (unsigned x = 0; x < 480; x++)
      if (pixels[y][x]) {
        if (x < left) left = x;
        if (x > right) right = x;
        if (y < top) top = y;
        if (y > bottom) bottom = y;
      }
  assert(right - left + 1 == 334 && bottom - top + 1 == 210);
  assert(left == 72 && top == 134);
}

static void test_incremental_render_matches_full_render(void) {
  // Compare incremental transitions with a cleared-panel rendering, including
  // size boundaries, invalid time and ink changes, in all four orientations.
  digits_clear(0);
  const unsigned hours[] = {9, 10, 11, 12, 1};
  for (unsigned rotation = 0; rotation < 4; rotation++) {
    for (unsigned h = 0; h < sizeof(hours) / sizeof(hours[0]); h++) {
      for (unsigned m = 0; m < 60; m++) {
        bool valid = m != 30;
        uint16_t ink = m % 2 ? DIGITS_RED : 0x001f;
        digits_draw(hours[h], m, valid, rotation, ink);
        memcpy(expected, pixels, sizeof(pixels));
        pixels_written = 0;
        digits_draw(hours[h], m, valid, rotation, ink);
        // Unchanged digits need no pixel traffic.
        assert(pixels_written < 500);
        assert(memcmp(pixels, expected, sizeof(pixels)) == 0);
        digits_clear(rotation);
        for (unsigned y = 0; y < 480; y++)
          for (unsigned x = 0; x < 480; x++) assert(!pixels[y][x]);
        digits_draw(hours[h], m, valid, rotation, ink);
        assert(memcmp(pixels, expected, sizeof(pixels)) == 0);
      }
    }
    digits_clear(rotation);
  }
}

int main(void) {
  test_centered_dimensions();
  test_incremental_render_matches_full_render();
}
