#include "touch.h"

#include "i2c.h"

// CST9220 (CST9217-compatible reports), 16-bit register addresses
// Report layout follows Waveshare's driver; retain default scan behavior
bool touch_read(bool* down) {
  uint8_t data[10];
  *down = false;
  if (!i2c_read16(0x5a, 0xd000, data, sizeof(data)) || data[6] != 0xab)
    return false;
  *down = (data[5] & 0x7f) != 0 && (data[0] & 15) == 6;
  return true;
}

bool touch_probe(void) {
  uint8_t data[10];
  return i2c_read16(0x5a, 0xd000, data, sizeof(data));
}
