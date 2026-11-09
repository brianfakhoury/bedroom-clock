#pragma once
#include "pmic.h"

// Installed MakerHawk 1000 mAh pack (B0DPZVBKMY)
// Standard charge: 0.5C to 4.2 V, terminate at 0.05C
static const pmic_charge_config board_battery_charge = {
    .voltage_mv = 4200,
    .current_ma = 500,
    .termination_ma = 50,
};
