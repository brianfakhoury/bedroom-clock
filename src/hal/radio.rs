use super::registers::*;
use super::timer::delay_us;

/// Reset Wi-Fi, BLE and 802.15.4, then gate their clocks and isolate power
pub fn disable() {
    const RESET: u32 =
        bit(8) | bit(10) | bit(14) | bit(15) | bit(16) | bit(17) | bit(18) | bit(24) | bit(30);
    MODEM_RESET.set(RESET);
    MODEM_CLOCKS.write(MODEM_CLOCKS.read() & bit(21)); // Preserve the data-dump mux
    MODEM_CLOCKS_FORCE.write(0);
    MODEM_CLOCKS1.write(0);
    MODEM_CLOCKS1_FORCE.write(0);
    // Keep internal analog I2C available for the regulators
    MODEM_LP_CLOCKS.clear(bit(0) | bit(1) | bit(3));
    let mut power = PMU_RADIO_POWER.read();
    // Clear power-up and reset/isolation bypass overrides
    power &= !(bit(2) | bit(3) | bit(4));
    power |= bit(0) | bit(1); // Reset and isolate
    PMU_RADIO_POWER.write(power);
    delay_us(2);
    PMU_RADIO_POWER.write(power | bit(5)); // Force power-down
}
