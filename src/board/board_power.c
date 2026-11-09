#include "board_power.h"

#include "battery.h"
#include "pins.h"
#include "pmic.h"
#include "power.h"
#include "registers.h"
#include "sleep.h"

bool board_power_configure(void) {
  // DCDC1 powers the board
  const uint8_t UNUSED_DCDC = PMIC_DCDC2 | PMIC_DCDC3 | PMIC_DCDC4 | PMIC_DCDC5;
  const uint8_t UNUSED_LDO = PMIC_ALDO2 | PMIC_ALDO4 | PMIC_BLDO1 | PMIC_BLDO2 |
                             PMIC_CPUSLDO | PMIC_DLDO1;
  const uint8_t UNUSED_ADC = PMIC_ADC_VBUS | PMIC_ADC_VSYS |
                             PMIC_ADC_DIE_TEMPERATURE | PMIC_ADC_GENERAL_PURPOSE;
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
             pmic_adc_disable(UNUSED_ADC | (ts_enabled ? 0 : PMIC_ADC_TS));
  return charge && rails && regulator && led && interrupts && adc;
}

// Unused peripheral clocks, including timer groups.
// Keep the system timer, GPIO, USB, flash/cache and clock/reset controls.
void unused_peripheral_clocks_disable(void) {
  static const uint32_t functional_clocks[] = {
      PCR_UART0_CLOCK,
      PCR_UART1_CLOCK,
      PCR_I2C_CLOCK,
      PCR_RMT_CLOCK,
      PCR_LEDC_CLOCK,
      PCR_TIMG0_CLOCK,
      PCR_TIMG0_WDT_CLOCK,
      PCR_TIMG1_CLOCK,
      PCR_TIMG1_WDT_CLOCK,
      PCR_TWAI0_CLOCK,
      PCR_TWAI1_CLOCK,
      PCR_I2S_TX_CLOCK,
      PCR_I2S_RX_CLOCK,
      PCR_ADC_CLOCK,
      PCR_TEMP_CLOCK,
      PCR_MCPWM_CLOCK,
      PCR_SPI2_CLOCK,
  };
  static const uint32_t register_clocks[] = {
      PCR_UART0_CONFIG,
      PCR_UART1_CONFIG,
      PCR_I2C_CONFIG,  // Board uses software I2C.
      PCR_UHCI_CONFIG,
      PCR_RMT_CONFIG,
      PCR_LEDC_CONFIG,
      PCR_TWAI0_CONFIG,
      PCR_TWAI1_CONFIG,
      PCR_I2S_CONFIG,
      PCR_ETM_CONFIG,
      PCR_PARLIO_CONFIG,
      PCR_GDMA_CONFIG,
      PCR_AES_CONFIG,
      PCR_SHA_CONFIG,
      PCR_RSA_CONFIG,
      PCR_ECC_CONFIG,
      PCR_TIMG0_CONFIG,  // Watchdogs disabled before gating.
      PCR_TIMG1_CONFIG,
      PCR_PCNT_CONFIG,
      PCR_MCPWM_CONFIG,
      PCR_SDIO_CONFIG,
      PCR_SPI2_CONFIG,  // Panel uses software QSPI.
      PCR_DS_CONFIG,
  };

  // Disable functional clocks before their register interfaces.
  const unsigned functional_count =
      sizeof(functional_clocks) / sizeof(*functional_clocks);
  for (unsigned i = 0; i < functional_count; i++)
    peripheral_functional_clock_disable(functional_clocks[i]);
  parlio_functional_clocks_disable();
  adc_register_clocks_disable();
  const unsigned register_count =
      sizeof(register_clocks) / sizeof(*register_clocks);
  for (unsigned i = 0; i < register_count; i++)
    peripheral_register_clock_disable(register_clocks[i]);
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
