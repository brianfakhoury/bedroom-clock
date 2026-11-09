#pragma once
#include <stdint.h>

#define EVENT_BOOT 1u
#define EVENT_USB 2u
#define EVENT_TIMER 4u
#define EVENT_TOUCH 8u
#define EVENT_KEY 16u
#define EVENT_BUTTON (EVENT_BOOT | EVENT_KEY)

void events_init(void);
// Wait for an event or the absolute timer deadline; zero means no deadline
unsigned wait_for_event_until(uint64_t deadline);
