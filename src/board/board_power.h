#pragma once
#include <stdbool.h>

bool board_power_configure(void);
void unused_peripheral_clocks_disable(void);
void board_power_resume(void);
// Returns only when a pending touch cancels sleep
void board_power_sleep(void);
