#pragma once
#include <stdbool.h>
#include <stdint.h>
void clock_init(unsigned brightness);
void clock_show(void);
void clock_toggle_always_on(void);
void clock_update(void);
bool clock_visible(void);
// Next display update in absolute timer ticks; zero means none scheduled
uint64_t clock_next_update(void);
