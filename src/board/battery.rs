use crate::drivers::pmic::ChargeProfile;

/// Installed MakerHawk 1000 mAh pack (B0DPZVBKMY).
/// Standard charge: 0.5C to 4.2 V, terminate at 0.05C
pub const CHARGE: ChargeProfile =
    ChargeProfile { voltage_mv: 4200, current_ma: 500, termination_ma: 50 };
