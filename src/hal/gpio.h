#pragma once
#include <stdbool.h>

enum { GPIO_INPUT, GPIO_OUTPUT };

enum { GPIO_LOW, GPIO_HIGH };

enum { GPIO_PULLUP_DISABLED, GPIO_PULLUP_ENABLED };

void gpio_init(unsigned pin, bool output, bool pullup);
void gpio_set(unsigned pin, bool high);
bool gpio_get(unsigned pin);
// Release retained output levels after restoring GPIO routing
void gpio_release_holds(void);
