# Hardware and source references

Reference snapshots used to develop and verify the clock live here:

- `datasheets/`: device documentation
- `waveshare/`: board schematic
- `firmware/`: source references, grouped by upstream project

The manifest records source URLs, revisions and SHA-256 hashes. `included: true`
means the snapshot is present in this repository; other entries preserve the
provenance of earlier research whose local copies are no longer available.
Vendor documents and source code retain their original copyrights and terms.
These files are references, not firmware build inputs.

`firmware/pmu_registers.h` is our former standalone adaptation of ESP-IDF's
PMU layout. The active HAL defines only the register addresses it uses.

## Sources

- [waveshare/schematic.pdf](https://files.waveshare.com/wiki/ESP32-C6-Touch-AMOLED-2.16/ESP32-C6-Touch-AMOLED-2.16-Schematic.pdf)
- [datasheets/CO5300.pdf](https://dl.espressif.com/AE/esp-iot-solution/CO5300_Datasheet_V0.00.pdf) — CASET/RASET alignment: pages 159–161
- [datasheets/ESP32-C6-TRM.pdf](https://documentation.espressif.com/esp32-c6_technical_reference_manual_en.pdf) — version 1.2; GPIO holds and power domains
- [datasheets/CST9217.pdf](https://raw.githubusercontent.com/ScripTerasu/cst92xx-touch-driver/0e4cede38d860ba08cf98c7a69400e10e8274608/docs/CST9217.pdf) — Hynitron V1.0, related-chip monitor/sleep description; not a CST9220 register specification
- [datasheets/ES7210.pdf](https://files.waveshare.com/wiki/common/ES7210_DS.pdf) — microphone bias, analog and digital shutdown registers
- [datasheets/ES8311.pdf](https://files.waveshare.com/wiki/common/ES8311.DS.pdf)
- [datasheets/ES8311-user-guide.pdf](https://files.waveshare.com/wiki/common/ES8311.user.Guide.pdf)
- [datasheets/AXP2101.pdf](https://files.waveshare.com/wiki/common/X-power-AXP2101_SWcharge_V1.0.pdf)
- [datasheets/PCF85063A.pdf](https://www.nxp.com/docs/en/data-sheet/PCF85063A.pdf)
- [datasheets/QMI8658A.pdf](https://files.waveshare.com/upload/5/5f/QMI8658A_Datasheet_Rev_A.pdf)
- [firmware/esp-idf/components/esp_hw_support/port/esp32c6/private_include/pmu_bit_defs.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/esp_hw_support/port/esp32c6/private_include/pmu_bit_defs.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/hal/esp32c6/include/hal/lpwdt_ll.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/hal/esp32c6/include/hal/lpwdt_ll.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/hal/esp32c6/include/hal/rtc_io_ll.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/hal/esp32c6/include/hal/rtc_io_ll.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/soc/esp32c6/include/soc/pmu_icg_mapping.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/soc/esp32c6/include/soc/pmu_icg_mapping.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/soc/esp32c6/include/soc/soc_caps.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/soc/esp32c6/include/soc/soc_caps.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/soc/esp32c6/register/soc/pmu_reg.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/soc/esp32c6/register/soc/pmu_reg.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/soc/esp32c6/register/soc/lp_io_reg.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/soc/esp32c6/register/soc/lp_io_reg.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/soc/esp32c6/register/soc/lp_aon_reg.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/soc/esp32c6/register/soc/lp_aon_reg.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/soc/esp32c6/register/soc/lp_timer_reg.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/soc/esp32c6/register/soc/lp_timer_reg.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/soc/esp32c6/register/soc/lp_wdt_reg.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/soc/esp32c6/register/soc/lp_wdt_reg.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/soc/esp32c6/register/soc/pcr_reg.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/soc/esp32c6/register/soc/pcr_reg.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/soc/esp32c6/register/soc/gpio_reg.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/soc/esp32c6/register/soc/gpio_reg.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/LICENSE](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/LICENSE) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/waveshare/02_Example/XiaoZhi-v2.2.5/main/boards/waveshare/esp32-c6-touch-amoled-2.16/config.h](https://github.com/waveshareteam/ESP32-C6-Touch-AMOLED-2.16/blob/294543798f1a44e2f2c4d2976522323f2beee11d/02_Example/XiaoZhi-v2.2.5/main/boards/waveshare/esp32-c6-touch-amoled-2.16/config.h) — revision `294543798f1a44e2f2c4d2976522323f2beee11d`
- [firmware/waveshare/02_Example/ESP-IDF-v5.5.3/09_LVGL_V9_Test/main/user_config.h](https://github.com/waveshareteam/ESP32-C6-Touch-AMOLED-2.16/blob/294543798f1a44e2f2c4d2976522323f2beee11d/02_Example/ESP-IDF-v5.5.3/09_LVGL_V9_Test/main/user_config.h) — revision `294543798f1a44e2f2c4d2976522323f2beee11d`
- [firmware/waveshare/02_Example/ESP-IDF-v5.5.3/09_LVGL_V9_Test/components/port_bsp/display_bsp.cpp](https://github.com/waveshareteam/ESP32-C6-Touch-AMOLED-2.16/blob/294543798f1a44e2f2c4d2976522323f2beee11d/02_Example/ESP-IDF-v5.5.3/09_LVGL_V9_Test/components/port_bsp/display_bsp.cpp) — revision `294543798f1a44e2f2c4d2976522323f2beee11d`
- [firmware/esp-idf/components/esp_hw_support/port/esp32c6/rtc_clk.c](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/esp_hw_support/port/esp32c6/rtc_clk.c) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/esp_hw_support/port/esp32c6/pmu_sleep.c](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/esp_hw_support/port/esp32c6/pmu_sleep.c) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/esp_hw_support/port/esp32c6/pmu_param.c](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/esp_hw_support/port/esp32c6/pmu_param.c) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/esp_hw_support/port/esp32c6/private_include/pmu_param.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/esp_hw_support/port/esp32c6/private_include/pmu_param.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/hal/esp32c6/include/hal/clk_tree_ll.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/hal/esp32c6/include/hal/clk_tree_ll.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/hal/esp32c6/include/hal/regi2c_ctrl_ll.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/hal/esp32c6/include/hal/regi2c_ctrl_ll.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/soc/esp32c6/include/soc/regi2c_defs.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/soc/esp32c6/include/soc/regi2c_defs.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/esp_rom/esp32c6/include/esp32c6/rom/rtc.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/esp_rom/esp32c6/include/esp32c6/rom/rtc.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/esp_rom/esp32c6/ld/esp32c6.rom.ld](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/esp_rom/esp32c6/ld/esp32c6.rom.ld) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/esp_hw_support/sleep_modes.c](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/esp_hw_support/sleep_modes.c) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/hal/esp32c6/include/hal/pmu_ll.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/hal/esp32c6/include/hal/pmu_ll.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/hal/esp32c6/include/hal/lp_aon_ll.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/hal/esp32c6/include/hal/lp_aon_ll.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/hal/esp32c6/include/hal/lp_timer_ll.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/hal/esp32c6/include/hal/lp_timer_ll.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/soc/esp32c6/register/soc/usb_serial_jtag_reg.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/soc/esp32c6/register/soc/usb_serial_jtag_reg.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [datasheets/ESP32-C6-TRM-v1.2.pdf](https://documentation.espressif.com/esp32-c6_technical_reference_manual_en.pdf) — revision `Version 1.2`
- [firmware/esp-idf/components/esp_hw_support/port/esp32c6/pmu_init.c](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/esp_hw_support/port/esp32c6/pmu_init.c) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/soc/esp32c6/register/soc/lp_clkrst_reg.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/soc/esp32c6/register/soc/lp_clkrst_reg.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/ulp/lp_core/lp_core.c](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/ulp/lp_core/lp_core.c) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/hal/esp32c6/include/hal/lp_core_ll.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/hal/esp32c6/include/hal/lp_core_ll.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/soc/esp32c6/register/soc/lpperi_reg.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/soc/esp32c6/register/soc/lpperi_reg.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/soc/esp32c6/register/soc/lp_tee_reg.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/soc/esp32c6/register/soc/lp_tee_reg.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/soc/esp32c6/register/soc/lp_apm_reg.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/soc/esp32c6/register/soc/lp_apm_reg.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/soc/esp32c6/register/soc/reg_base.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/soc/esp32c6/register/soc/reg_base.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/esp_rom/patches/esp_rom_hp_regi2c_esp32c6.c](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/esp_rom/patches/esp_rom_hp_regi2c_esp32c6.c) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/soc/esp32c6/register/soc/pmu_struct.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/soc/esp32c6/register/soc/pmu_struct.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/soc/esp32c6/register/soc/efuse_reg.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/soc/esp32c6/register/soc/efuse_reg.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/soc/esp32c6/register/soc/i2c_ana_mst_reg.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/soc/esp32c6/register/soc/i2c_ana_mst_reg.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/hal/esp32c6/include/hal/efuse_ll.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/hal/esp32c6/include/hal/efuse_ll.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/soc/esp32c6/include/modem/modem_lpcon_struct.h](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/soc/esp32c6/include/modem/modem_lpcon_struct.h) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
- [firmware/esp-idf/components/soc/esp32c6/gpio_periph.c](https://github.com/espressif/esp-idf/blob/f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0/components/soc/esp32c6/gpio_periph.c) — revision `f5c3654a1c2d2a01f7f67def7a0dc48e691f63c0`
