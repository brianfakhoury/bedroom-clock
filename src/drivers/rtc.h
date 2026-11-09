#pragma once
#include <stdbool.h>

// Gregorian date and time, years 2020–2099
typedef struct {
  unsigned year, month, day, hour, minute, second;
} datetime;

bool date_valid(const datetime* d);
// Requires a valid date; Sunday = 0
unsigned date_weekday(unsigned year, unsigned month, unsigned day);

bool rtc_clock_output_disable(void);
bool rtc_read(datetime* d);
bool rtc_set(const datetime* d);
