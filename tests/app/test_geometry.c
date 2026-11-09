#include <assert.h>

#include "geometry.h"

#include "../../src/app/geometry.c"

static void expect(rect actual, rect expected) {
  assert(actual.x == expected.x && actual.y == expected.y);
  assert(actual.width == expected.width && actual.height == expected.height);
}

static void test_rectangle_rotation(void) {
  const rect area = {64, 150, 70, 166};
  const rect expected[] = {
      {64, 150, 70, 166},
      {164, 64, 166, 70},
      {346, 164, 70, 166},
      {150, 346, 166, 70},
  };
  for (unsigned rotation = 0; rotation < 4; rotation++) {
    rect rotated = rect_rotate(area, rotation);
    expect(rotated, expected[rotation]);
    expect(rect_rotate(rotated, (4 - rotation) & 3), area);
    const rect screen = {0, 0, 480, 480};
    expect(rect_rotate(screen, rotation), screen);
  }
}

int main(void) {
  test_rectangle_rotation();
}
