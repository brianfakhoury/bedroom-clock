//! MCU power states. Startup-only: this code is lost during deep sleep.

use super::registers::*;
use super::timer::delay_us;

const ACTIVE: u32 = 0;
const MODEM: u32 = 1;
const SLEEP: u32 = 2;

// Regulators use the internal analog bus, not the external I2C pins
#[cfg_attr(target_os = "none", unsafe(link_section = ".init.code"))]
#[inline(never)]
fn regulator_bit(register: u32, position: u32, set: bool) {
    MODEM_LP_CLOCKS.set(bit(2)); // Analog I2C master clock
    ANALOG_I2C_READ_SELECT.write(0x00ff_fbff); // Select DIG_REG read path
    let master_offset = if ANALOG_I2C_MASTER_SELECT.read() & bit(12) != 0 { 0 } else { 4 };
    let control = ANALOG_I2C_CONTROL0.offset(master_offset);
    let wait = || while control.read() & bit(25) != 0 {};
    wait();
    control.write(0x6d | (register << 8));
    wait();
    let mut value = (control.read() >> 16) & 255;
    value = if set { value | bit(position) } else { value & !bit(position) };
    control.write(0x6d | (register << 8) | (value << 16) | bit(24));
    wait();
}

fn active_bias(fuse: u32, fallback: u32) -> u32 {
    match fuse {
        0 => fallback,
        30.. => 31,
        _ => fuse + 2,
    }
}

#[cfg_attr(target_os = "none", unsafe(link_section = ".init.code"))]
#[inline(never)]
fn configure_hp_states(bias: u32) {
    // Keep Espressif's backup settings with SRAM save/restore disabled
    const BACKUP: [u32; 3] = [0x0101_00a0, 0x0010_0010, 0x1280_0200];
    for (mode, backup) in (ACTIVE..=SLEEP).zip(BACKUP) {
        let state = |register: Reg| register.offset(mode * PMU_HP_MODE_SIZE);
        let sleep = mode == SLEEP;
        let active = mode == ACTIVE;
        state(PMU_HP_DIG_POWER).write(match mode {
            SLEEP => 0xe820_0000,
            ACTIVE => 0,
            _ => bit(29),
        });
        state(PMU_HP_ICG_FUNC).write(match mode {
            ACTIVE => !0,
            SLEEP => 0,
            _ => bit(20),
        });
        state(PMU_HP_ICG_APB).write(match mode {
            ACTIVE => !0,
            SLEEP => 0,
            _ => bit(9),
        });
        state(PMU_HP_ICG_MODEM).update(
            3 << 30,
            match mode {
                ACTIVE => 2 << 30,
                SLEEP => 0,
                _ => 1 << 30,
            },
        );
        let awake_clocks = bit(28) | bit(29) | bit(30);
        state(PMU_HP_CLK_POWER).write(if sleep { bit(26) | bit(27) } else { awake_clocks });
        state(PMU_HP_XTAL).update(bit(31), if sleep { 0 } else { bit(31) });
        // UART wake, pad selection, watchdog pause and CPU stall; no global holds
        let mut system_control = if active { 0 } else { bit(24) | bit(28) | bit(29) };
        system_control |= if sleep { bit(27) } else { 0 };
        state(PMU_HP_SYSCNTL).update(0x3f << 24, system_control);
        // Enable the awake system clock; modem selects PLL; sleep gates it
        let mut system_clock = if sleep { 0 } else { bit(27) };
        system_clock |= if active { 0 } else { bit(28) | bit(29) };
        system_clock |= if mode == MODEM { bit(30) } else { 0 };
        state(PMU_HP_SYSCLK).update(0x3f << 26, system_clock);
        let mut bias_control = if active { bit(25) } else { 0 };
        bias_control |= if sleep { bit(30) | bit(31) } else { 0 };
        state(PMU_HP_BIAS).update(0x7f << 25, bias_control);
        // Clear sleep regulator overrides; use calibrated bias only while awake
        let regulator = if sleep { 0 } else { bit(18) | (bias << 27) };
        state(PMU_HP_REGULATOR0).update(0xffff_0000, regulator);
        state(PMU_HP_REGULATOR1).update(0xffff_ff00, 0);
        state(PMU_HP_BACKUP).write(backup);
        state(PMU_HP_BACKUP_CLK).write(!0);
        PMU_IMM_MODEM_ICG.set(bit(31));
        PMU_IMM_SLEEP_SYSCLK.set(bit(28));
        PMU_WAKEUP_CNTL3.update(3 << 16, 2 << 16);
    }
}

#[cfg_attr(target_os = "none", unsafe(link_section = ".init.code"))]
#[inline(never)]
fn configure_lp_states(active_bias: u32, sleep_bias: u32, attenuation: u32) {
    for (mode, bias) in [active_bias, sleep_bias].into_iter().enumerate() {
        let state = |register: Reg| register.offset(mode as u32 * PMU_LP_MODE_SIZE);
        let sleeping = mode == 1;
        state(PMU_LP_REGULATOR0).update(0xffe0_0000, bit(22) | (bias << 27));
        state(PMU_LP_REGULATOR1).update(0xf000_0000, 0);
        // Retain LP GPIO for latched touch edges; RC_FAST is needed only awake
        state(PMU_LP_DIG_POWER).write(if sleeping { bit(30) } else { 0 });
        state(PMU_LP_CLK_POWER).write(if sleeping { 0 } else { bit(30) });
    }
    PMU_LP_XTAL.offset(PMU_LP_MODE_SIZE).update(bit(31), 0);
    let sleep_bias_control = (attenuation << 26) | bit(30) | bit(31);
    PMU_LP_BIAS.offset(PMU_LP_MODE_SIZE).update(0x7f << 25, sleep_bias_control);
}

// Nominal RC periods in Q19 microseconds, used for power-transition delays.
// Evaluated at build time

const fn slow_cycles(us: u32) -> u32 {
    (((us as u64) << 19) / 3_495_253) as u32
}

const fn fast_cycles(us: u32) -> u32 {
    (((us as u64) << 19) / 29_959) as u32
}

/// Configure MCU power states after timer initialization, before radio shutdown
#[cfg_attr(target_os = "none", unsafe(link_section = ".init.code"))]
#[inline(never)]
pub fn init() {
    PMU_LP_CLK_POWER.set(bit(30));
    delay_us(50);
    LP_FAST_CLOCK.clear(bit(2)); // LP fast clock: RC_FAST, independent of XTAL
    delay_us(3);

    PMU_HP_EXT_RF_PWC.set(bit(26));
    PMU_HP_EXT_RF_PWC.set(bit(27));
    regulator_bit(5, 7, true); // Hand LP regulator control to PMU
    regulator_bit(7, 7, true); // Hand HP regulator control to PMU
    regulator_bit(13, 2, false); // Release software LP regulator force-on
    regulator_bit(13, 3, false); // Release software HP regulator force-on

    let version = EFUSE_BLOCK2_VERSION.read();
    let calibrated = (version >> 30) * 100 + ((version >> 27) & 7) >= 3;
    let fuse = EFUSE_REGULATOR_BIAS.read();
    let hp_active_bias = if calibrated { active_bias(fuse & 31, 25) } else { 25 };
    let lp_active_bias = if calibrated { active_bias((fuse >> 5) & 31, 26) } else { 26 };
    let lp_sleep_bias = if calibrated { (((fuse >> 19) & 15) + 23) & 31 } else { 23 };
    let sleep_attenuation = if calibrated { ((fuse >> 16) & 7) + 8 } else { 12 };
    configure_hp_states(hp_active_bias);
    configure_lp_states(lp_active_bias, lp_sleep_bias, sleep_attenuation);

    // Hand TOP, AON, CPU and radio power to PMU; keep VDD_SPI defaults
    for domain in [0, 1, 2, 4] {
        PMU_DOMAIN_POWER0.offset(domain * 4).clear(63);
    }
    PMU_POWER_MEM_CNTL.clear(15 << 24);
    PMU_POWER_LP_PERI.clear(63);

    PMU_WAKEUP_CNTL3.update(0xffff, const { (slow_cycles(450) << 8) | slow_cycles(450) });
    PMU_WAKEUP_CNTL7.update(0xffff_0000, const { fast_cycles(154) << 16 });
    PMU_WAKEUP_CNTL5.update(0xff00_0000, const { slow_cycles(500) << 24 });
    const SETTLE: u32 = fast_cycles(2);
    PMU_POWER_WAIT_TIMER0.update(0xffff_c000, (SETTLE << 23) | (SETTLE << 14));
    PMU_POWER_WAIT_TIMER1.update(0xffff_0000, (SETTLE << 23) | (SETTLE << 16));
    PMU_POWER_CLK_WAIT.write(const { fast_cycles(250) | (fast_cycles(50) << 16) });
    // No modem wake source is enabled, so its scheduling target is unused
    PMU_WAKEUP_CNTL5.clear(0xf_ffff);
}
