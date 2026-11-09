#include "i2c.h"

#include "gpio.h"
#include "registers.h"
#include "timer.h"

static unsigned scl_pin, sda_pin;

// Open drain: drive low or release the pin so the pull-up raises it
static void sda(bool high) {
  REG(high ? GPIO_ENABLE_CLEAR : GPIO_ENABLE_SET) = BIT(sda_pin);
}

static bool scl(bool high) {
  REG(high ? GPIO_ENABLE_CLEAR : GPIO_ENABLE_SET) = BIT(scl_pin);
  // Allow up to 1 ms of clock stretching
  for (unsigned n = 0; high && !gpio_get(scl_pin); n++) {
    if (n == 1000) return false;
    delay_us(1);
  }
  delay_us(3);
  return true;
}

static bool start(void) {
  sda(true);
  if (!scl(true)) return false;
  sda(false);
  delay_us(3);
  scl(false);
  return true;
}

static void stop(void) {
  sda(false);
  scl(true);
  sda(true);
  delay_us(3);
}

static bool write_byte(uint8_t data) {
  for (unsigned i = 0; i < 8; i++) {
    sda(data & 128);
    if (!scl(true)) return false;
    scl(false);
    data <<= 1;
  }
  sda(true);
  if (!scl(true)) return false;
  bool ack = !gpio_get(sda_pin);
  scl(false);
  return ack;
}

static bool read_byte(uint8_t* data, bool last) {
  *data = 0;
  sda(true);
  for (unsigned i = 0; i < 8; i++) {
    if (!scl(true)) return false;
    *data = (*data << 1) | gpio_get(sda_pin);
    scl(false);
  }
  sda(last);
  if (!scl(true)) return false;
  scl(false);
  sda(true);
  return true;
}

void i2c_init(unsigned clock_pin, unsigned data_pin) {
  scl_pin = clock_pin;
  sda_pin = data_pin;
  gpio_init(scl_pin, GPIO_INPUT, GPIO_PULLUP_ENABLED);
  gpio_init(sda_pin, GPIO_INPUT, GPIO_PULLUP_ENABLED);
  gpio_set(scl_pin, GPIO_LOW);
  gpio_set(sda_pin, GPIO_LOW);
  // Finish any byte interrupted by a CPU reset, then release the bus
  for (unsigned i = 0; i < 9; i++) {
    scl(false);
    scl(true);
  }
  stop();
}

bool i2c_read(uint8_t address, uint8_t reg, uint8_t* data, unsigned n) {
  bool ok = start() && write_byte(address << 1) && write_byte(reg);
  ok = ok && start() && write_byte((address << 1) | 1);
  for (unsigned i = 0; ok && i < n; i++) ok = read_byte(data + i, i == n - 1);
  stop();
  return ok;
}

bool i2c_write(uint8_t address, uint8_t reg, const uint8_t* data, unsigned n) {
  bool ok = start() && write_byte(address << 1) && write_byte(reg);
  for (unsigned i = 0; ok && i < n; i++) ok = write_byte(data[i]);
  stop();
  return ok;
}

bool i2c_read16(uint8_t address, uint16_t reg, uint8_t* data, unsigned n) {
  bool ok = start() && write_byte(address << 1);
  ok = ok && write_byte(reg >> 8) && write_byte(reg & 255);
  ok = ok && start() && write_byte((address << 1) | 1);
  for (unsigned i = 0; ok && i < n; i++) ok = read_byte(data + i, i == n - 1);
  stop();
  return ok;
}
