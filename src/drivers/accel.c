#include "accel.h"

#include "i2c.h"
#include "timer.h"

enum {
  WHO_AM_I = 0x00,
  CTRL1 = 0x02,
  CTRL2 = 0x03,
  CTRL5 = 0x06,
  CTRL7 = 0x08,
  STATUS0 = 0x2e,
  ACCEL_X_LOW = 0x35,
  AUTO_INCREMENT = 0x40,
  SENSOR_DISABLE = 0x01,
  ACCEL_ENABLE = 0x01,
  ACCEL_2G_500HZ = 0x04,
  ACCEL_READY = 0x01,
  DEVICE_ID = 0x05,
};

static uint8_t address;

static bool write_reg(uint8_t reg, uint8_t value) {
  return i2c_write(address, reg, &value, 1);
}

static bool standby(void) {
  for (unsigned attempt = 0; attempt < 3; attempt++) {
    // Attempt both writes even when one fails, and retry transient NACKs
    // Read back before claiming shutdown succeeded
    bool disabled = write_reg(CTRL7, 0);
    bool stopped = write_reg(CTRL1, AUTO_INCREMENT | SENSOR_DISABLE);
    uint8_t ctrl1, ctrl7;
    if (!disabled || !stopped) continue;
    if (!i2c_read(address, CTRL7, &ctrl7, 1)) continue;
    if (!i2c_read(address, CTRL1, &ctrl1, 1)) continue;
    if (ctrl7 == 0 && ctrl1 == (AUTO_INCREMENT | SENSOR_DISABLE)) return true;
  }
  return false;
}

bool accel_init(void) {
  uint8_t id;
  for (address = 0x6a; address <= 0x6b; address++)
    if (i2c_read(address, WHO_AM_I, &id, 1) && id == DEVICE_ID) {
      if (standby()) return true;
      break;
    }
  address = 0;
  return false;
}

static bool collect_samples(int32_t sums[3]) {
  // Auto-increment, little endian, +/-2 g at 500 Hz; gyro remains off
  if (!write_reg(CTRL1, AUTO_INCREMENT) || !write_reg(CTRL2, ACCEL_2G_500HZ) ||
      !write_reg(CTRL5, 0) || !write_reg(CTRL7, ACCEL_ENABLE))
    return false;
  delay_us(20000);
  for (unsigned sample = 0; sample < 4; sample++) {
    uint8_t status, data[6];
    delay_us(3000);
    if (!i2c_read(address, STATUS0, &status, 1)) return false;
    if (!(status & ACCEL_READY)) return false;
    if (!i2c_read(address, ACCEL_X_LOW, data, sizeof(data))) return false;
    for (unsigned i = 0; i < sizeof(data); i += 2)
      sums[i / 2] += (int16_t)(data[i] | (data[i + 1] << 8));
  }
  return true;
}

bool accel_read(int16_t xyz[3]) {
  if (!address) return false;
  int32_t sums[3] = {0};
  bool sampled = collect_samples(sums);
  bool asleep = standby();
  if (!sampled || !asleep) return false;
  for (unsigned axis = 0; axis < 3; axis++) xyz[axis] = sums[axis] / 4;
  return true;
}
