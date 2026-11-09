#pragma once
#include <stdint.h>
#define DISPLAY_DURATION_MS 3000u
#define FADE_IN_MS 180u
#define FADE_OUT_MS 350u
uint8_t animation_level(unsigned elapsed_ms, unsigned from, unsigned target);
// Delay until the next frame in milliseconds; zero when finished
unsigned animation_next_ms(unsigned elapsed_ms);
