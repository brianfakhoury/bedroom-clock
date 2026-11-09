#pragma once
#include <stdbool.h>
#include <stdint.h>

bool accel_init(void);
// Wake, average four samples, then verify standby
// Failure leaves xyz unchanged
bool accel_read(int16_t xyz[3]);
