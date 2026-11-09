#include "qspi.h"

#include "boot.h"
#include "gpio.h"
#include "registers.h"
#include "timer.h"

static unsigned chip_select, data_shift;
static uint32_t clock_mask, data0_mask, bus_mask;

BOOT_ONLY void qspi_init(unsigned clock_pin, unsigned data0_pin,
                         unsigned cs_pin) {
  chip_select = cs_pin;
  data_shift = data0_pin;
  clock_mask = BIT(clock_pin);
  data0_mask = BIT(data0_pin);
  bus_mask = clock_mask | (0x0fu << data0_pin);
  gpio_init(clock_pin, GPIO_OUTPUT, GPIO_PULLUP_DISABLED);
  gpio_set(clock_pin, GPIO_LOW);
  for (unsigned pin = data0_pin; pin < data0_pin + 4; pin++) {
    gpio_init(pin, GPIO_OUTPUT, GPIO_PULLUP_DISABLED);
    gpio_set(pin, GPIO_LOW);
  }
  gpio_init(cs_pin, GPIO_OUTPUT, GPIO_PULLUP_DISABLED);
  gpio_set(cs_pin, GPIO_HIGH);
}

void qspi_begin(void) { gpio_set(chip_select, GPIO_LOW); }

void qspi_end(void) {
  REG(GPIO_OUT_CLEAR) = clock_mask;
  gpio_set(chip_select, GPIO_HIGH);
}

void qspi_write_byte(uint8_t value) {
  for (unsigned i = 0; i < 8; i++) {
    REG(GPIO_OUT_CLEAR) = clock_mask | data0_mask;
    if (value & 128) REG(GPIO_OUT_SET) = data0_mask;
    delay_us(1);
    REG(GPIO_OUT_SET) = clock_mask;
    delay_us(1);
    value <<= 1;
  }
  REG(GPIO_OUT_CLEAR) = clock_mask;
}

void qspi_write_quad16(uint16_t value) {
  for (int shift = 12; shift >= 0; shift -= 4) {
    REG(GPIO_OUT_CLEAR) = bus_mask;
    REG(GPIO_OUT_SET) = ((value >> shift) & 15) << data_shift;
    REG(GPIO_OUT_SET) = clock_mask;
  }
}
