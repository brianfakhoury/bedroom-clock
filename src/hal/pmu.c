#include <stdbool.h>

#include "boot.h"
#include "power.h"
#include "registers.h"
#include "timer.h"

enum { ACTIVE, MODEM, SLEEP };

// Regulators use the internal analog bus, not the external I2C pins
static BOOT_ONLY void regulator_bit(unsigned reg, unsigned bit, bool set) {
  REG(MODEM_LP_CLOCKS) |= BIT(2);             // Analog I2C master clock
  REG(ANALOG_I2C_READ_SELECT) = 0x00fffbffu;  // Select DIG_REG read path
  unsigned master_offset = (REG(ANALOG_I2C_MASTER_SELECT) & BIT(12)) ? 0 : 4;
  uint32_t ctrl = ANALOG_I2C_CONTROL0 + master_offset;
  while (REG(ctrl) & BIT(25)) {
  }
  REG(ctrl) = 0x6d | (reg << 8);
  while (REG(ctrl) & BIT(25)) {
  }
  unsigned value = (REG(ctrl) >> 16) & 255;
  value = set ? value | BIT(bit) : value & ~BIT(bit);
  REG(ctrl) = 0x6d | (reg << 8) | (value << 16) | BIT(24);
  while (REG(ctrl) & BIT(25)) {
  }
}

static BOOT_ONLY unsigned active_bias(unsigned fuse, unsigned fallback) {
  if (!fuse) return fallback;
  return fuse > 29 ? 31 : fuse + 2;
}

static inline void update(uint32_t address, uint32_t mask, uint32_t value) {
  REG(address) = (REG(address) & ~mask) | (value & mask);
}

static BOOT_ONLY void configure_hp_states(unsigned bias) {
  // Keep Espressif's backup settings with SRAM save/restore disabled
  const uint32_t backup[] = {0x010100a0, 0x00100010, 0x12800200};
  for (unsigned mode = ACTIVE; mode <= SLEEP; ++mode) {
    bool sleep = mode == SLEEP;
    bool active = mode == ACTIVE;
    REG(PMU_HP_DIG_POWER(mode)) = sleep ? 0xe8200000u : (active ? 0 : BIT(29));
    REG(PMU_HP_ICG_FUNC(mode)) = active ? ~0u : (sleep ? 0 : BIT(20));
    REG(PMU_HP_ICG_APB(mode)) = active ? ~0u : (sleep ? 0 : BIT(9));
    update(PMU_HP_ICG_MODEM(mode), 3u << 30, (active ? 2u : !sleep) << 30);
    uint32_t awake_clocks = BIT(28) | BIT(29) | BIT(30);
    REG(PMU_HP_CLK_POWER(mode)) = sleep ? BIT(26) | BIT(27) : awake_clocks;
    update(PMU_HP_XTAL(mode), BIT(31), sleep ? 0 : BIT(31));
    // UART wake, pad selection, watchdog pause and CPU stall; no global holds
    uint32_t system_control = active ? 0 : BIT(24) | BIT(28) | BIT(29);
    system_control |= sleep ? BIT(27) : 0;
    update(PMU_HP_SYSCNTL(mode), 0x3fu << 24, system_control);
    // Enable the awake system clock; modem selects PLL; sleep gates it
    uint32_t system_clock = sleep ? 0 : BIT(27);
    system_clock |= active ? 0 : BIT(28) | BIT(29);
    system_clock |= mode == MODEM ? BIT(30) : 0;
    update(PMU_HP_SYSCLK(mode), 0x3fu << 26, system_clock);
    uint32_t bias_control = active ? BIT(25) : 0;
    bias_control |= sleep ? BIT(30) | BIT(31) : 0;
    update(PMU_HP_BIAS(mode), 0x7fu << 25, bias_control);
    // Clear sleep regulator overrides; use calibrated bias only while awake
    uint32_t regulator = sleep ? 0 : BIT(18) | (bias << 27);
    update(PMU_HP_REGULATOR0(mode), 0xffff0000u, regulator);
    update(PMU_HP_REGULATOR1(mode), 0xffffff00u, 0);
    REG(PMU_HP_BACKUP(mode)) = backup[mode];
    REG(PMU_HP_BACKUP_CLK(mode)) = ~0u;
    REG(PMU_IMM_MODEM_ICG) |= BIT(31);
    REG(PMU_IMM_SLEEP_SYSCLK) |= BIT(28);
    update(PMU_WAKEUP_CNTL3, 3u << 16, 2u << 16);
  }
}

static BOOT_ONLY void configure_lp_states(unsigned active_bias,
                                          unsigned sleep_bias,
                                          unsigned attenuation) {
  for (unsigned mode = 0; mode < 2; ++mode) {
    bool sleeping = mode == 1;
    unsigned bias = sleeping ? sleep_bias : active_bias;
    update(PMU_LP_REGULATOR0(mode), 0xffe00000u, BIT(22) | (bias << 27));
    update(PMU_LP_REGULATOR1(mode), 0xf0000000u, 0);
    // Retain LP GPIO for latched touch edges; RC_FAST is needed only awake
    REG(PMU_LP_DIG_POWER(mode)) = sleeping ? BIT(30) : 0;
    REG(PMU_LP_CLK_POWER(mode)) = sleeping ? 0 : BIT(30);
  }
  update(PMU_LP_XTAL(1), BIT(31), 0);
  update(PMU_LP_BIAS(1), 0x7fu << 25, (attenuation << 26) | BIT(30) | BIT(31));
}

// Nominal RC periods in Q19 microseconds, used for power-transition delays
// Inline to evaluate constant divisions at build time
static inline __attribute__((always_inline)) unsigned slow_cycles(unsigned us) {
  return ((uint64_t)us << 19) / 3495253;
}

static inline __attribute__((always_inline)) unsigned fast_cycles(unsigned us) {
  return ((uint64_t)us << 19) / 29959;
}

BOOT_ONLY void pmu_init(void) {
  REG(PMU_LP_CLK_POWER(0)) |= BIT(30);
  delay_us(50);
  REG(LP_FAST_CLOCK) &= ~BIT(2);  // LP fast clock: RC_FAST, independent of XTAL
  delay_us(3);

  REG(PMU_HP_EXT_RF_PWC) |= BIT(26);
  REG(PMU_HP_EXT_RF_PWC) |= BIT(27);
  regulator_bit(5, 7, true);    // Hand LP regulator control to PMU
  regulator_bit(7, 7, true);    // Hand HP regulator control to PMU
  regulator_bit(13, 2, false);  // Release software LP regulator force-on
  regulator_bit(13, 3, false);  // Release software HP regulator force-on

  unsigned version = REG(EFUSE_BLOCK2_VERSION);
  bool calibrated = ((version >> 30) * 100 + ((version >> 27) & 7)) >= 3;
  unsigned fuse = REG(EFUSE_REGULATOR_BIAS);
  unsigned hp_active_bias = calibrated ? active_bias(fuse & 31, 25) : 25;
  unsigned lp_active_bias = calibrated ? active_bias((fuse >> 5) & 31, 26) : 26;
  unsigned lp_sleep_bias = calibrated ? (((fuse >> 19) & 15) + 23) & 31 : 23;
  unsigned sleep_attenuation = calibrated ? ((fuse >> 16) & 7) + 8 : 12;
  configure_hp_states(hp_active_bias);
  configure_lp_states(lp_active_bias, lp_sleep_bias, sleep_attenuation);

  // Hand TOP, AON, CPU and radio power to PMU; keep VDD_SPI defaults
  const unsigned domains[] = {0, 1, 2, 4};
  for (unsigned i = 0; i < 4; ++i) REG(PMU_DOMAIN_POWER(domains[i])) &= ~63u;
  REG(PMU_POWER_MEM_CNTL) &= ~(15u << 24);
  REG(PMU_POWER_LP_PERI) &= ~63u;

  update(PMU_WAKEUP_CNTL3, 0xffffu, (slow_cycles(450) << 8) | slow_cycles(450));
  update(PMU_WAKEUP_CNTL7, 0xffff0000u, fast_cycles(154) << 16);
  update(PMU_WAKEUP_CNTL5, 0xff000000u, slow_cycles(500) << 24);
  unsigned settle = fast_cycles(2);
  update(PMU_POWER_WAIT_TIMER0, 0xffffc000u, (settle << 23) | (settle << 14));
  update(PMU_POWER_WAIT_TIMER1, 0xffff0000u, (settle << 23) | (settle << 16));
  REG(PMU_POWER_CLK_WAIT) = fast_cycles(250) | (fast_cycles(50) << 16);
  // No modem wake source is enabled, so its scheduling target is unused
  REG(PMU_WAKEUP_CNTL5) &= ~0xfffffu;
}
