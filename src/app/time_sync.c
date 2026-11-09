#include "time_sync.h"

#include "rtc.h"
#include "usb.h"

// USB request: TIME YYYYMMDDhhmmss\n, in UTC
static char line[19];
static unsigned used;

static bool set_time(void) {
  const char prefix[] = "TIME ";
  for (unsigned i = 0; i < 5; i++)
    if (line[i] != prefix[i]) return false;
  unsigned pair[7] = {0};
  for (unsigned i = 0; i < 14; i++) {
    char c = line[5 + i];
    if (c < '0' || c > '9') return false;
    pair[i / 2] = pair[i / 2] * 10 + c - '0';
  }
  datetime date = {
      pair[0] * 100 + pair[1], pair[2], pair[3], pair[4], pair[5], pair[6]};
  return date_valid(&date) && rtc_set(&date);
}

void time_sync_process(void) {
  // Bound host work so malformed input cannot starve the display or sleep
  for (unsigned budget = 0; budget < 64; budget++) {
    int c = usb_get();
    if (c < 0) return;
    if (c == '\r') continue;
    if (c == '\n') {
      bool valid = used == sizeof(line) && set_time();
      if (used) usb_puts(valid ? "OK TIME\n" : "ERR TIME\n");
      used = 0;
      return;
    }
    if (used < sizeof(line))
      line[used++] = (char)c;
    else
      used = sizeof(line) + 1;  // Reject the entire overlong line
  }
}
