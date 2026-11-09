#include "local_time.h"

#include "rtc.h"

void pacific_time(const datetime* d, unsigned* h, unsigned* m) {
  // America/Los_Angeles, current US rules (since 2007). RTC always holds UTC
  unsigned march = 8 + (7 - date_weekday(d->year, 3, 8)) % 7;
  unsigned november = 1 + (7 - date_weekday(d->year, 11, 1)) % 7;
  bool after_spring = d->day > march || (d->day == march && d->hour >= 10);
  bool before_fall = d->day < november || (d->day == november && d->hour < 9);
  bool dst = (d->month > 3 && d->month < 11) ||
             (d->month == 3 && after_spring) || (d->month == 11 && before_fall);
  *h = (d->hour + 24 - (dst ? 7 : 8)) % 24;
  *m = d->minute;
}
