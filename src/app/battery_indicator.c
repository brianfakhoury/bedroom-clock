#include "battery_indicator.h"

#include <stdint.h>

#include "display.h"
#include "geometry.h"

static unsigned level = 101, offset, end;

static uint16_t pixel(unsigned x, unsigned y) {
  if (level > 100) return 0;
  unsigned dx = x < 240 ? x : 479 - x;
  unsigned dy = y < 240 ? y : 479 - y;
  // Photo-estimated outer radius 72, inner radius 68: a four-pixel band
  // The screen masks the outside; only the inner boundary needs calculation
  if (dx < 72 && dy < 72) {
    unsigned cx = 143 - 2 * dx, cy = 143 - 2 * dy;
    if (cx * cx + cy * cy <= 136 * 136) return 0;
  } else if (dx >= 4 && dy >= 4)
    return 0;
  unsigned horizontal = y < 240 ? x : 1439 - x;
  unsigned vertical = x < 240 ? 1919 - y : 480 + y;
  unsigned position = (dy <= dx ? horizontal : vertical) + offset;
  if (position >= 1920) position -= 1920;
  return position < end ? 0x4000 : 0x0800;
}

void battery_indicator_draw(unsigned percentage, unsigned rotation) {
  static unsigned previous = ~0u;
  if (percentage > 100) percentage = 101;
  unsigned key = percentage * 4 + rotation;
  if (previous == key) return;
  previous = key;
  level = percentage;
  offset = (2640 - rotation * 480) % 1920;
  end = level * 1920 / 100;
  // Corner bounds in eight-row, even-aligned windows
  // Skip pixels hidden by the screen corners
  static const uint8_t corners[][2] = {
      {40, 72}, {26, 48}, {18, 34}, {12, 24}, {6, 18},
      {4, 12},  {0, 10},  {0, 6},   {0, 6},
  };
  for (unsigned turn = 0; turn < 4; turn++) {
    rect area = rect_rotate((rect){72, 0, 336, 4}, turn);
    display_pattern(area.x, area.y, area.width, area.height, pixel);
    for (unsigned row = 0; row < 9; row++) {
      unsigned left = corners[row][0], width = corners[row][1] - left;
      area = rect_rotate((rect){left, row * 8, width, 8}, turn);
      display_pattern(area.x, area.y, area.width, area.height, pixel);
    }
  }
}
