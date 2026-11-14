//! The ESP32-C6 register addresses used by this firmware, checked against
//! ESP-IDF v5.4.2. All memory-mapped access goes through [`Reg`].

/// A 32-bit memory-mapped register
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Reg(u32);

pub const fn bit(n: u32) -> u32 {
    1 << n
}

impl Reg {
    /// Tests check the board's register tables by address
    #[cfg(test)]
    pub const fn address(self) -> u32 {
        self.0
    }

    /// The register `bytes` after this one, for register arrays
    pub const fn offset(self, bytes: u32) -> Reg {
        Reg(self.0 + bytes)
    }

    pub fn read(self) -> u32 {
        // SAFETY: a `Reg` is only built here, from an ESP32-C6 register address
        unsafe { (self.0 as usize as *const u32).read_volatile() }
    }

    pub fn write(self, value: u32) {
        // SAFETY: as for `read`
        unsafe { (self.0 as usize as *mut u32).write_volatile(value) }
    }

    pub fn set(self, bits: u32) {
        self.write(self.read() | bits);
    }

    pub fn clear(self, bits: u32) {
        self.write(self.read() & !bits);
    }

    /// Replace the masked field, preserving unrelated bits
    pub fn update(self, mask: u32, value: u32) {
        self.write((self.read() & !mask) | (value & mask));
    }
}

pub const GPIO_OUT_SET: Reg = Reg(0x6009_1008);
pub const GPIO_OUT_CLEAR: Reg = Reg(0x6009_100c);
pub const GPIO_ENABLE_SET: Reg = Reg(0x6009_1024);
pub const GPIO_ENABLE_CLEAR: Reg = Reg(0x6009_1028);
pub const GPIO_INPUT: Reg = Reg(0x6009_103c);
pub const GPIO_STATUS: Reg = Reg(0x6009_1044);
pub const GPIO_STATUS_CLEAR: Reg = Reg(0x6009_104c);
pub const GPIO_PIN0: Reg = Reg(0x6009_1074);
pub const GPIO_OUT_SELECT0: Reg = Reg(0x6009_1554);
pub const IOMUX_GPIO0: Reg = Reg(0x6009_0004);

// Peripheral clock/reset controls
pub const PCR_UART0_CONFIG: Reg = Reg(0x6009_6000);
pub const PCR_UART1_CONFIG: Reg = Reg(0x6009_600c);
pub const PCR_I2C_CONFIG: Reg = Reg(0x6009_6020);
pub const PCR_UHCI_CONFIG: Reg = Reg(0x6009_6028);
pub const PCR_RMT_CONFIG: Reg = Reg(0x6009_602c);
pub const PCR_LEDC_CONFIG: Reg = Reg(0x6009_6034);
pub const PCR_TWAI0_CONFIG: Reg = Reg(0x6009_605c);
pub const PCR_TWAI1_CONFIG: Reg = Reg(0x6009_6064);
pub const PCR_I2S_CONFIG: Reg = Reg(0x6009_606c);
pub const PCR_ETM_CONFIG: Reg = Reg(0x6009_6098);
pub const PCR_PARLIO_CONFIG: Reg = Reg(0x6009_60a4);
pub const PCR_GDMA_CONFIG: Reg = Reg(0x6009_60bc);
pub const PCR_AES_CONFIG: Reg = Reg(0x6009_60c8);
pub const PCR_SHA_CONFIG: Reg = Reg(0x6009_60cc);
pub const PCR_RSA_CONFIG: Reg = Reg(0x6009_60d0);
pub const PCR_ECC_CONFIG: Reg = Reg(0x6009_60d8);
pub const PCR_TIMG0_CONFIG: Reg = Reg(0x6009_603c);
pub const PCR_TIMG1_CONFIG: Reg = Reg(0x6009_6048);
pub const PCR_PCNT_CONFIG: Reg = Reg(0x6009_6094);
pub const PCR_MCPWM_CONFIG: Reg = Reg(0x6009_609c);
pub const PCR_SDIO_CONFIG: Reg = Reg(0x6009_60b0);
pub const PCR_SPI2_CONFIG: Reg = Reg(0x6009_60c0);
pub const PCR_DS_CONFIG: Reg = Reg(0x6009_60e0);
pub const PCR_UART0_CLOCK: Reg = Reg(0x6009_6004);
pub const PCR_UART1_CLOCK: Reg = Reg(0x6009_6010);
pub const PCR_I2C_CLOCK: Reg = Reg(0x6009_6024);
pub const PCR_RMT_CLOCK: Reg = Reg(0x6009_6030);
pub const PCR_LEDC_CLOCK: Reg = Reg(0x6009_6038);
pub const PCR_TIMG0_CLOCK: Reg = Reg(0x6009_6040);
pub const PCR_TIMG0_WDT_CLOCK: Reg = Reg(0x6009_6044);
pub const PCR_TIMG1_CLOCK: Reg = Reg(0x6009_604c);
pub const PCR_TIMG1_WDT_CLOCK: Reg = Reg(0x6009_6050);
pub const PCR_TWAI0_CLOCK: Reg = Reg(0x6009_6060);
pub const PCR_TWAI1_CLOCK: Reg = Reg(0x6009_6068);
pub const PCR_I2S_TX_CLOCK: Reg = Reg(0x6009_6070);
pub const PCR_I2S_RX_CLOCK: Reg = Reg(0x6009_6078);
pub const PCR_ADC_CLOCK: Reg = Reg(0x6009_6084);
pub const PCR_TEMP_CLOCK: Reg = Reg(0x6009_6088);
pub const PCR_MCPWM_CLOCK: Reg = Reg(0x6009_60a0);
pub const PCR_SPI2_CLOCK: Reg = Reg(0x6009_60c4);
pub const PCR_PARLIO_RX_CLOCK: Reg = Reg(0x6009_60a8);
pub const PCR_PARLIO_TX_CLOCK: Reg = Reg(0x6009_60ac);
pub const PCR_ADC_CONFIG: Reg = Reg(0x6009_6080);

pub const PCR_REGISTER_CLOCK_ENABLE: u32 = bit(0);
pub const PCR_FUNCTIONAL_CLOCK_ENABLE: u32 = bit(22);
pub const PCR_PARLIO_CLOCK_ENABLE: u32 = bit(18);
pub const PCR_ADC_CLOCK_ENABLE: u32 = bit(0) | bit(2);

pub const PCR_USB_CONFIG: Reg = Reg(0x6009_608c);
pub const USB_CONFIG: Reg = Reg(0x6000_f018);
pub const USB_FIFO: Reg = Reg(0x6000_f000);
pub const USB_FIFO_CONFIG: Reg = Reg(0x6000_f004);
pub const USB_IRQ_RAW: Reg = Reg(0x6000_f008);
pub const USB_IRQ_ENABLE: Reg = Reg(0x6000_f010);
pub const USB_IRQ_CLEAR: Reg = Reg(0x6000_f014);

pub const SYSTIMER_CONFIG: Reg = Reg(0x6000_a000);
pub const SYSTIMER_UPDATE0: Reg = Reg(0x6000_a004);
pub const SYSTIMER_TARGET0_HIGH: Reg = Reg(0x6000_a01c);
pub const SYSTIMER_TARGET0_LOW: Reg = Reg(0x6000_a020);
pub const SYSTIMER_TARGET0_CONFIG: Reg = Reg(0x6000_a034);
pub const SYSTIMER_VALUE0_HIGH: Reg = Reg(0x6000_a040);
pub const SYSTIMER_VALUE0_LOW: Reg = Reg(0x6000_a044);
pub const SYSTIMER_TARGET0_LOAD: Reg = Reg(0x6000_a050);
pub const SYSTIMER_IRQ_ENABLE: Reg = Reg(0x6000_a064);
pub const SYSTIMER_IRQ_RAW: Reg = Reg(0x6000_a068);
pub const SYSTIMER_IRQ_CLEAR: Reg = Reg(0x6000_a06c);
pub const PCR_SYSTIMER_CONFIG: Reg = Reg(0x6009_6054);
pub const PCR_SYSTIMER_CLOCK: Reg = Reg(0x6009_6058);

pub const PLIC_ENABLE: Reg = Reg(0x2000_1000);
pub const PLIC_TYPE: Reg = Reg(0x2000_1004);
pub const PLIC_PRIORITY1: Reg = Reg(0x2000_1014);
pub const PLIC_THRESHOLD: Reg = Reg(0x2000_1090);
pub const IRQ_MAP_GPIO: Reg = Reg(0x6001_0078);
pub const IRQ_MAP_USB: Reg = Reg(0x6001_00c0);
pub const IRQ_MAP_SYSTIMER0: Reg = Reg(0x6001_00e4);

pub const TIMG0_WDT_CONFIG: Reg = Reg(0x6000_8048);
pub const TIMG0_WDT_PROTECT: Reg = Reg(0x6000_8064);
/// Distance from a timer group 0 register to its timer group 1 twin
pub const TIMG1_OFFSET: u32 = 0x1000;
pub const EFUSE_WDT_DELAY: Reg = Reg(0x600b_0834);
pub const LP_WDT_TIMEOUT: Reg = Reg(0x600b_1c04);
pub const LP_WDT_FEED: Reg = Reg(0x600b_1c14);
pub const LP_WDT_CONFIG: Reg = Reg(0x600b_1c00);
pub const LP_WDT_PROTECT: Reg = Reg(0x600b_1c18);
pub const LP_SWD_CONFIG: Reg = Reg(0x600b_1c1c);
pub const LP_SWD_PROTECT: Reg = Reg(0x600b_1c20);

pub const LP_GPIO_MUX: Reg = Reg(0x600b_1028);
pub const LP_GPIO_HOLD: Reg = Reg(0x600b_102c);

pub const PCR_CPU_WAITI: Reg = Reg(0x6009_6114);

// Radio clock/reset controls are outside the switchable modem logic domain
pub const MODEM_CLOCKS: Reg = Reg(0x600a_9804);
pub const MODEM_CLOCKS_FORCE: Reg = Reg(0x600a_9808);
pub const MODEM_RESET: Reg = Reg(0x600a_9810);
pub const MODEM_CLOCKS1: Reg = Reg(0x600a_9814);
pub const MODEM_CLOCKS1_FORCE: Reg = Reg(0x600a_9818);
pub const MODEM_LP_CLOCKS: Reg = Reg(0x600a_f018);
pub const PMU_RADIO_POWER: Reg = Reg(0x600b_0104);

// PMU state registers repeat per mode: offset the first by a multiple of the
// mode size. HP modes are active/modem/sleep, LP modes are active/sleep
pub const PMU_HP_MODE_SIZE: u32 = 0x34;
pub const PMU_HP_DIG_POWER: Reg = Reg(0x600b_0000);
pub const PMU_HP_ICG_FUNC: Reg = Reg(0x600b_0004);
pub const PMU_HP_ICG_APB: Reg = Reg(0x600b_0008);
pub const PMU_HP_ICG_MODEM: Reg = Reg(0x600b_000c);
pub const PMU_HP_SYSCNTL: Reg = Reg(0x600b_0010);
pub const PMU_HP_CLK_POWER: Reg = Reg(0x600b_0014);
pub const PMU_HP_BIAS: Reg = Reg(0x600b_0018);
pub const PMU_HP_BACKUP: Reg = Reg(0x600b_001c);
pub const PMU_HP_BACKUP_CLK: Reg = Reg(0x600b_0020);
pub const PMU_HP_SYSCLK: Reg = Reg(0x600b_0024);
pub const PMU_HP_REGULATOR0: Reg = Reg(0x600b_0028);
pub const PMU_HP_REGULATOR1: Reg = Reg(0x600b_002c);
pub const PMU_HP_XTAL: Reg = Reg(0x600b_0030);
pub const PMU_LP_MODE_SIZE: u32 = 0x18;
pub const PMU_LP_REGULATOR0: Reg = Reg(0x600b_009c);
pub const PMU_LP_REGULATOR1: Reg = Reg(0x600b_00a0);
pub const PMU_LP_XTAL: Reg = Reg(0x600b_00a4);
pub const PMU_LP_DIG_POWER: Reg = Reg(0x600b_00a8);
pub const PMU_LP_CLK_POWER: Reg = Reg(0x600b_00ac);
pub const PMU_LP_BIAS: Reg = Reg(0x600b_00b0);
pub const PMU_IMM_MODEM_ICG: Reg = Reg(0x600b_00dc);
pub const PMU_IMM_SLEEP_SYSCLK: Reg = Reg(0x600b_00d0);
pub const PMU_WAKEUP_CNTL3: Reg = Reg(0x600b_012c);
pub const PMU_WAKEUP_CNTL5: Reg = Reg(0x600b_0134);
pub const PMU_WAKEUP_CNTL7: Reg = Reg(0x600b_013c);
/// One register per power domain
pub const PMU_DOMAIN_POWER0: Reg = Reg(0x600b_00f4);
pub const PMU_POWER_MEM_CNTL: Reg = Reg(0x600b_010c);
pub const PMU_POWER_LP_PERI: Reg = Reg(0x600b_0108);
pub const PMU_POWER_WAIT_TIMER0: Reg = Reg(0x600b_00ec);
pub const PMU_POWER_WAIT_TIMER1: Reg = Reg(0x600b_00f0);
pub const PMU_POWER_CLK_WAIT: Reg = Reg(0x600b_011c);
pub const PMU_HP_EXT_RF_PWC: Reg = Reg(0x600b_0154);

// Deep-sleep entry and always-on wake control
pub const PMU_WAKEUP_CNTL0: Reg = Reg(0x600b_0120);
pub const PMU_WAKEUP_CNTL1: Reg = Reg(0x600b_0124);
pub const PMU_WAKEUP_CNTL2: Reg = Reg(0x600b_0128);
pub const PMU_WAKEUP_CNTL4: Reg = Reg(0x600b_0130);
pub const PMU_HP_INT_CLR: Reg = Reg(0x600b_0168);
pub const LP_AON_SYS_CFG: Reg = Reg(0x600b_1034);
pub const LP_IO_PIN0: Reg = Reg(0x600b_2028);
pub const LP_IO_CLEAR: Reg = Reg(0x600b_2020);
pub const LP_IO_PAD0: Reg = Reg(0x600b_2048);
pub const LP_IO_STATUS: Reg = Reg(0x600b_2018);
pub const LP_TIMER_TARGET_HIGH: Reg = Reg(0x600b_0c04);
pub const LP_TIMER_CLEAR: Reg = Reg(0x600b_0c34);
pub const RTC_SLEEP_MODE: Reg = Reg(0x600b_1024);
pub const LPPERI_CLK_EN: Reg = Reg(0x600b_2800);
pub const LP_IO_DATE: Reg = Reg(0x600b_23fc);
pub const LP_IO_OUT_ENABLE_W1TC: Reg = Reg(0x600b_2014);

// Regulator calibration and internal analog I2C access
pub const LP_FAST_CLOCK: Reg = Reg(0x600b_0400);
pub const EFUSE_REGULATOR_BIAS: Reg = Reg(0x600b_084c);
pub const EFUSE_BLOCK2_VERSION: Reg = Reg(0x600b_0850);
pub const ANALOG_I2C_CONTROL0: Reg = Reg(0x600a_f800);
pub const ANALOG_I2C_READ_SELECT: Reg = Reg(0x600a_f81c);
pub const ANALOG_I2C_MASTER_SELECT: Reg = Reg(0x600a_f820);
