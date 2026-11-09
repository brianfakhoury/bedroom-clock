#pragma once
#include <stdint.h>

// Initialize after the board configures the bus pins and resets the panel
void display_init(void);
// CO5300 requires even x, y, width and height (nonzero dimensions)
void display_rect(unsigned x, unsigned y, unsigned w, unsigned h,
                  uint16_t color);
// Same alignment rules; pixel callbacks receive screen coordinates
void display_pattern(unsigned x, unsigned y, unsigned w, unsigned h,
                     uint16_t (*pixel)(unsigned x, unsigned y));
void display_on(void);
void display_off(void);
void display_brightness(unsigned value);

// Sleep retains pixels; wake restores panel settings
void display_sleep(void);
void display_wake(void);
