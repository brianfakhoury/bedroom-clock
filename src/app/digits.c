#include "digits.h"

#include "display.h"
#include "geometry.h"

// The panel and this small cache survive CPU-off standby
static uint8_t previous_segments[4];
static uint16_t previous_ink;
static bool previous_large;

// Enlarge three-digit times by 5/4
// Coordinates stay in the upright frame; rotation is applied last
static void draw_segment(unsigned rotation, bool large, unsigned x, unsigned y,
                         unsigned w, unsigned h, uint16_t ink) {
  unsigned scale = large ? 5 : 4;
  unsigned left = large ? 72 : 65;
  unsigned top = large ? 135 : 156;
  // CO5300 address windows require even starts and even dimensions
  unsigned right = (left + (x + w) * scale / 4) & ~1u;
  unsigned bottom = (top + (y + h) * scale / 4) & ~1u;
  x = (left + x * scale / 4) & ~1u;
  y = (top + y * scale / 4) & ~1u;
  rect area = rect_rotate((rect){x, y, right - x, bottom - y}, rotation);
  display_rect(area.x, area.y, area.width, area.height, ink);
}

static void draw_digit(unsigned rotation, bool large, unsigned x, unsigned d,
                       bool valid, uint16_t ink, unsigned slot) {
  static const uint8_t segment_masks[] = {0x3f, 0x06, 0x5b, 0x4f, 0x66,
                                          0x6d, 0x7d, 0x07, 0x7f, 0x6f};
  static const uint8_t segment_rects[7][4] = {
      {12, 0, 48, 10}, {60, 12, 10, 66}, {60, 90, 10, 66}, {12, 158, 48, 10},
      {0, 90, 10, 66}, {0, 12, 10, 66},  {12, 80, 48, 10}};
  uint8_t bits = ink ? (valid ? segment_masks[d] : 0x40) : 0;
  uint8_t changed = bits ^ previous_segments[slot];
  if (ink != previous_ink) changed |= bits;
  previous_segments[slot] = bits;
  for (unsigned i = 0; i < 7; i++) {
    if (!(changed & (1u << i))) continue;
    const uint8_t* segment = segment_rects[i];
    draw_segment(rotation, large, x + segment[0], segment[1], segment[2],
                 segment[3], bits & (1u << i) ? ink : 0);
  }
}

static void draw_time(unsigned h, unsigned m, bool valid, unsigned rotation,
                      bool large, uint16_t ink) {
  unsigned offset = large ? 0 : 82;
  if (!large) draw_digit(rotation, false, 0, h / 10, valid, ink, 0);
  draw_digit(rotation, large, offset, h % 10, valid, ink, 1);
  draw_digit(rotation, large, offset + 116, m / 10, valid, ink, 2);
  draw_digit(rotation, large, offset + 198, m % 10, valid, ink, 3);
  draw_segment(rotation, large, offset + 86, 46, 10, 10, ink);
  draw_segment(rotation, large, offset + 86, 108, 10, 10, ink);
}

void digits_clear(unsigned rotation) {
  draw_time(0, 0, false, rotation, previous_large, 0);
  previous_ink = 0;
}

void digits_draw(unsigned h, unsigned m, bool valid, unsigned rotation,
                 uint16_t ink) {
  bool large = valid && h < 10;
  if (large != previous_large) digits_clear(rotation);
  draw_time(h, m, valid, rotation, large, ink);
  previous_large = large;
  previous_ink = ink;
}
