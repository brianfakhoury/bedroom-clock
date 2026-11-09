#pragma once
#include <stdbool.h>
#include <stdint.h>

// Arm falling-edge wake on GPIO 0–7
// Return false on a pending edge, with awake hardware already restored
bool sleep_prepare(unsigned wake_pin);
// Restore MCU clocks and the wake pin after deep sleep or cancelled preparation
void sleep_resume(unsigned wake_pin);
// After successful preparation, enter deep sleep without returning
// ROM resumes at entry after checking retained_bytes starting at LP RAM base
void sleep_enter(void (*entry)(void), unsigned retained_bytes,
                 uint32_t high_outputs, uint32_t held_outputs);
