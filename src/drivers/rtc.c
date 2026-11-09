#include "rtc.h"

#include "boot.h"
#include "i2c.h"

enum { RTC_ADDRESS = 0x51, STOP = 0x20, HOUR_12 = 0x02 };

static unsigned days_in_month(unsigned y, unsigned m) {
  static const uint8_t days[] = {31, 28, 31, 30, 31, 30,
                                 31, 31, 30, 31, 30, 31};
  return days[m - 1] + (m == 2 && y % 4 == 0);
}

bool date_valid(const datetime* d) {
  if (d->year < 2020 || d->year > 2099) return false;
  if (d->month < 1 || d->month > 12) return false;
  if (d->day < 1 || d->day > days_in_month(d->year, d->month)) return false;
  return d->hour < 24 && d->minute < 60 && d->second < 60;
}

unsigned date_weekday(unsigned y, unsigned m, unsigned d) {
  static const uint8_t t[] = {0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4};
  if (m < 3) y--;
  return (y + y / 4 - y / 100 + y / 400 + t[m - 1] + d) % 7;
}

static uint8_t to_bcd(unsigned v) { return (v / 10) * 16 + v % 10; }

static unsigned from_bcd(uint8_t v) { return (v >> 4) * 10 + (v & 15); }

static bool valid_bcd(uint8_t v) { return (v & 15) < 10 && (v >> 4) < 10; }

BOOT_ONLY bool rtc_clock_output_disable(void) {
  uint8_t control;
  if (!i2c_read(RTC_ADDRESS, 1, &control, 1)) return false;
  // COF=111 disables the unused CLKOUT. Write ones to AF/TF to preserve
  // their flags, including an event arriving between the read and write
  control |= 0x4f;
  return i2c_write(RTC_ADDRESS, 1, &control, 1) &&
         i2c_read(RTC_ADDRESS, 1, &control, 1) && (control & 7) == 7;
}

bool rtc_read(datetime* d) {
  uint8_t control, data[7];
  if (!i2c_read(RTC_ADDRESS, 0, &control, 1)) return false;
  if (control & (STOP | HOUR_12)) return false;
  if (!i2c_read(RTC_ADDRESS, 4, data, 7) || (data[0] & 128)) return false;
  data[0] &= 127;
  data[1] &= 127;
  data[2] &= 63;
  data[3] &= 63;
  data[5] &= 31;
  for (unsigned i = 0; i < 7; i++)
    if (i != 4 && !valid_bcd(data[i])) return false;
  *d = (datetime){2000 + from_bcd(data[6]), from_bcd(data[5]),
                  from_bcd(data[3]),        from_bcd(data[2]),
                  from_bcd(data[1]),        from_bcd(data[0])};
  return date_valid(d);
}

bool rtc_set(const datetime* d) {
  if (!date_valid(d)) return false;
  uint8_t control;
  if (!i2c_read(RTC_ADDRESS, 0, &control, 1)) return false;
  control = (control & ~(STOP | HOUR_12)) | STOP;
  if (!i2c_write(RTC_ADDRESS, 0, &control, 1)) return false;
  unsigned weekday = date_weekday(d->year, d->month, d->day);
  uint8_t data[] = {to_bcd(d->second),     to_bcd(d->minute), to_bcd(d->hour),
                    to_bcd(d->day),        weekday,           to_bcd(d->month),
                    to_bcd(d->year - 2000)};
  bool ok = i2c_write(RTC_ADDRESS, 4, data, 7);
  // Restart even if writing the time failed
  control &= ~STOP;
  bool running = i2c_write(RTC_ADDRESS, 0, &control, 1);
  return ok && running;
}
