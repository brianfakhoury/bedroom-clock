#pragma once
#include <stdbool.h>
#include <stdint.h>
#define DIGITS_RED 0xf800u
void digits_draw(unsigned hour, unsigned minute, bool valid, unsigned rotation,
                 uint16_t ink);
// Erase the cached digits before changing orientation
void digits_clear(unsigned rotation);
