#pragma once

#define INPUT_TAP 1u
#define INPUT_BOOT 2u
#define INPUT_KEY 4u

// Reset debounce history after CPU wake
void input_reset(void);
// The contact that started this display session is already handled
void input_consume_touch(void);
// Translate hardware events into new presses
unsigned input_read(unsigned events);
