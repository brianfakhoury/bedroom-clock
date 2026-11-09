#pragma once
#include <stdint.h>
// Clockwise quarter turns; uncertain readings keep previous
unsigned orientation_choose(const int16_t xyz[3], unsigned previous);
