#include "watchdog.h"

#include "registers.h"

static const uint32_t UNLOCK_KEY = 0x50d83aa1;

void watchdogs_disable(void) {
  // Clear watchdogs left running by ROM before configuring our own
  for (unsigned i = 0; i < 2; i++) {
    uint32_t base = TIMG0_BASE + i * 0x1000;
    REG(base + TIMG_WDT_PROTECT) = UNLOCK_KEY;
    REG(base + TIMG_WDT_CONFIG) = 0;
    REG(base + TIMG_WDT_PROTECT) = 0;
  }
  REG(LP_WDT_PROTECT) = UNLOCK_KEY;
  REG(LP_WDT_CONFIG) = 0;
  REG(LP_WDT_PROTECT) = 0;
  REG(LP_SWD_PROTECT) = UNLOCK_KEY;
  REG(LP_SWD_CONFIG) |= BIT(30);
  REG(LP_SWD_PROTECT) = 0;
}

void watchdog_arm(void) {
  // Reset a stalled CPU after roughly 8 seconds using RC_SLOW
  // Compensate for the eFuse WDT_DELAY_SEL multiplier
  unsigned shift = 1 + ((REG(EFUSE_WDT_DELAY) >> 16) & 3);
  REG(LP_WDT_PROTECT) = UNLOCK_KEY;
  REG(LP_WDT_CONFIG) = 0;
  REG(LP_WDT_TIMEOUT) = 1200000u >> shift;
  REG(LP_WDT_CONFIG) =
      BIT(31) | (4u << 28) | (7u << 13) | (7u << 16) | BIT(8) | 20u;
  REG(LP_WDT_FEED) = BIT(31);
  REG(LP_WDT_PROTECT) = 0;
}

void watchdog_disarm(void) {
  REG(LP_WDT_PROTECT) = UNLOCK_KEY;
  REG(LP_WDT_CONFIG) = 0;
  REG(LP_WDT_PROTECT) = 0;
}

void watchdog_feed(void) {
  REG(LP_WDT_PROTECT) = UNLOCK_KEY;
  REG(LP_WDT_FEED) = BIT(31);
  REG(LP_WDT_PROTECT) = 0;
}
