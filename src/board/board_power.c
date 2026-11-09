#include "board_power.h"

#include "battery.h"
#include "pins.h"
#include "pmic.h"
#include "registers.h"
#include "sleep.h"

bool board_power_configure(void) {
  const uint8_t UNUSED_DCDC = 0x1e;  // DCDC2–5; DCDC1 powers the board
  const uint8_t UNUSED_LDO = 0xfa;   // ALDO2/4, BLDO1/2, CPUSLDO, DLDO1
  const uint8_t UNUSED_ADC = 0x3c;   // All channels except battery and TS
  const uint8_t TS_ADC = 0x02;
  // Keep audio ALDO1 on: cutting it makes the shared I2C bus unresponsive
  // Disable amplifier CTRL (ALDO2); keep panel reset and RTC powered
  bool charge = pmic_charge_configure(&board_battery_charge);
  bool rails = pmic_output_discharge_enable() && pmic_aldo_enable(1, true);
  rails = rails && pmic_outputs_disable(UNUSED_DCDC, UNUSED_LDO, true);
  bool regulator = pmic_dcdc1_auto();
  bool led = pmic_led_off();
  bool interrupts = pmic_interrupts_disable();
  // Keep battery sensing; keep TS only if charger protection uses it
  bool ts_enabled;
  bool adc = pmic_ts_charger_monitoring_enabled(&ts_enabled) &&
             pmic_adc_disable(UNUSED_ADC | (ts_enabled ? 0 : TS_ADC));
  return charge && rails && regulator && led && interrupts && adc;
}

// Unused peripheral clocks, including timer groups
// Keep the system timer, GPIO, USB, flash/cache and clock/reset controls
static const uint32_t register_clocks[] = {
    0x60096000u,  // UART0
    0x6009600cu,  // UART1
    0x60096020u,  // Hardware I2C (the clock uses software I2C)
    0x60096028u,  // UHCI
    0x6009602cu,  // RMT
    0x60096034u,  // LEDC
    0x6009605cu,  // TWAI0
    0x60096064u,  // TWAI1
    0x6009606cu,  // I2S
    0x60096098u,  // ETM
    0x600960a4u,  // Parallel IO
    0x600960bcu,  // GDMA
    0x600960c8u,  // AES
    0x600960ccu,  // SHA
    0x600960d0u,  // RSA
    0x600960d8u,  // ECC
    0x6009603cu,  // Timer group 0 (watchdogs disabled before gating)
    0x60096048u,  // Timer group 1
    0x60096094u,  // Pulse counter
    0x6009609cu,  // Motor PWM
    0x600960b0u,  // SDIO slave
    0x600960c0u,  // SPI2 (panel uses software QSPI)
    0x600960e0u,  // Digital signature
};
static const uint32_t functional_clocks[] = {
    0x60096004u, 0x60096010u,               // UART0/1
    0x60096024u, 0x60096030u, 0x60096038u,  // I2C, RMT, LEDC
    0x60096040u, 0x60096044u, 0x6009604cu,
    0x60096050u,               // Timer groups/timer WDTs
    0x60096060u, 0x60096068u,  // TWAI0/1
    0x60096070u, 0x60096078u,  // I2S TX/RX
    0x60096084u, 0x60096088u,  // SAR ADC, temperature sensor
    0x600960a0u, 0x600960c4u,  // Motor PWM, SPI2
};

void unused_peripheral_clocks_disable(void) {
  const unsigned functional_count =
      sizeof(functional_clocks) / sizeof(*functional_clocks);
  for (unsigned i = 0; i < functional_count; i++)
    REG(functional_clocks[i]) &= ~BIT(22);
  REG(0x600960a8u) &= ~BIT(18);  // Parallel RX functional clock
  REG(0x600960acu) &= ~BIT(18);  // Parallel TX functional clock
  // ADC functional/register interfaces
  REG(0x60096080u) &= ~(BIT(0) | BIT(2));

  const unsigned register_count =
      sizeof(register_clocks) / sizeof(*register_clocks);
  for (unsigned i = 0; i < register_count; i++)
    REG(register_clocks[i]) &= ~BIT(0);
}

extern void wake_entry(void);
extern char _crc_end[];

void board_power_resume(void) {
  sleep_resume(TOUCH_IRQ);
  unused_peripheral_clocks_disable();
}

void board_power_sleep(void) {
  if (!sleep_prepare(TOUCH_IRQ)) {
    unused_peripheral_clocks_disable();
    return;
  }

  // R15/R12/R17 pull shared SPI lines up through 10 kohm; park them high
  const uint32_t high_outputs =
      BIT(PANEL_CLOCK) | BIT(PANEL_DATA0) | BIT(PANEL_DATA1);
  // Hold outputs only: global holds interfere with USB; touch/I2C stay inputs
  const uint32_t held_outputs =
      high_outputs | BIT(PANEL_DATA2) | BIT(PANEL_DATA3) | BIT(SD_CS) |
      BIT(TOUCH_RESET) | BIT(PANEL_CS) | BIT(AUDIO_MCLK) | BIT(AUDIO_BCLK) |
      BIT(AUDIO_LRCLK) | BIT(AUDIO_DATA_OUT);
  // ROM checks the retained code before resuming through wake.S
  unsigned retained_bytes = (uintptr_t)_crc_end - 0x50000000u;
  sleep_enter(wake_entry, retained_bytes, high_outputs, held_outputs);
}
