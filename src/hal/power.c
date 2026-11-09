#include "power.h"

#include "registers.h"
#include "timer.h"

void radios_disable(void) {
  // Reset Wi-Fi, BLE and 802.15.4 before gating clocks and isolating power
  const uint32_t reset = BIT(8) | BIT(10) | BIT(14) | BIT(15) | BIT(16) |
                         BIT(17) | BIT(18) | BIT(24) | BIT(30);
  REG(MODEM_RESET) |= reset;
  REG(MODEM_CLOCKS) &= BIT(21);  // Preserve the data-dump mux
  REG(MODEM_CLOCKS_FORCE) = 0;
  REG(MODEM_CLOCKS1) = 0;
  REG(MODEM_CLOCKS1_FORCE) = 0;
  // Keep internal analog I2C available for the regulators
  REG(MODEM_LP_CLOCKS) &= ~(BIT(0) | BIT(1) | BIT(3));
  uint32_t power = REG(PMU_RADIO_POWER);
  // Clear power-up and reset/isolation bypass overrides
  power &= ~(BIT(2) | BIT(3) | BIT(4));
  power |= BIT(0) | BIT(1);  // Reset and isolate
  REG(PMU_RADIO_POWER) = power;
  delay_us(2);
  REG(PMU_RADIO_POWER) = power | BIT(5);  // Force power-down
}
