#include "gpio.h"

#include "registers.h"

void gpio_init(unsigned pin, bool output, bool pullup) {
  uint32_t mux = BIT(12) | BIT(9) | (pullup ? BIT(8) : 0) | (2u << 10);
  REG(IOMUX_GPIO0 + pin * 4) = mux;
  REG(GPIO_OUT_SELECT0 + pin * 4) = 128 | BIT(9);
  REG(output ? GPIO_ENABLE_SET : GPIO_ENABLE_CLEAR) = BIT(pin);
}

void gpio_set(unsigned pin, bool high) {
  REG(high ? GPIO_OUT_SET : GPIO_OUT_CLEAR) = BIT(pin);
}

bool gpio_get(unsigned pin) { return (REG(GPIO_INPUT_REG) & BIT(pin)) != 0; }

void gpio_release_holds(void) { REG(LP_GPIO_HOLD) = 0; }
