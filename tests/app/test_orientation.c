#include <assert.h>

#include "orientation.h"

#include "../../src/app/orientation.c"

static void chosen(int x, int y, int z, unsigned expected) {
  int16_t xyz[] = {x, y, z};
  assert(orientation_choose(xyz, 0) == expected);
}

static void test_cardinal_directions(void) {
  chosen(0, 16384, 0, 0);
  chosen(0, -16384, 0, 2);
  chosen(16384, 0, 0, 3);
  chosen(-16384, 0, 0, 1);
  chosen(-900, 16600, -200, 0);  // Physical upright calibration.
}

static void test_ambiguous_samples_keep_orientation(void) {
  int16_t ambiguous[][3] = {{0, 0, 16384},
                            {0, 0, -16384},
                            {11585, 11585, 0},
                            {0, 0, 0},
                            {25000, 0, 0}};
  for (unsigned old = 0; old < 4; old++)
    for (unsigned i = 0; i < sizeof(ambiguous) / sizeof(ambiguous[0]); i++)
      assert(orientation_choose(ambiguous[i], old) == old);
}

int main(void) {
  test_cardinal_directions();
  test_ambiguous_samples_keep_orientation();
}
