#pragma once
#include <stdbool.h>

// Check communication after the board resets the controller
bool touch_probe(void);
// A failed read clears down
bool touch_read(bool* down);
