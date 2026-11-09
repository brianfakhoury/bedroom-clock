#include "timer.h"

#include "registers.h"

void timer_init(void) {
  REG(PCR_SYSTIMER_CONFIG) = 1;
  REG(PCR_SYSTIMER_CLOCK) = BIT(22);
  REG(SYSTIMER_CONFIG) = BIT(30) | BIT(31);
}

uint64_t ticks(void) {
  REG(SYSTIMER_UPDATE0) = BIT(30);
  while (!(REG(SYSTIMER_UPDATE0) & BIT(29))) {
  }
  return ((uint64_t)REG(SYSTIMER_VALUE0_HIGH) << 32) | REG(SYSTIMER_VALUE0_LOW);
}

void delay_us(uint32_t us) {
  uint64_t end = ticks() + (uint64_t)us * TICKS_PER_US;
  while (ticks() < end) {
  }
}
