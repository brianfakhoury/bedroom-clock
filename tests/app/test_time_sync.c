#include <assert.h>
#include <string.h>

#include "i2c.h"
#include "rtc.h"
#include "time_sync.h"
#include "usb.h"

#include "../../src/app/time_sync.c"
#include "../../src/drivers/rtc.c"

static const char* input;
static char reply[32];
static uint8_t registers[32];
static unsigned writes;
static bool fail_write;

int usb_get(void) { return *input ? (unsigned char)*input++ : -1; }

void usb_puts(const char* text) { strcpy(reply, text); }

bool i2c_read(uint8_t address, uint8_t reg, uint8_t* data, unsigned n) {
  assert(address == 0x51);
  memcpy(data, registers + reg, n);
  return true;
}

bool i2c_write(uint8_t address, uint8_t reg, const uint8_t* data, unsigned n) {
  assert(address == 0x51);
  writes++;
  if (fail_write) return false;
  memcpy(registers + reg, data, n);
  return true;
}

static void send(const char* text) {
  reply[0] = 0;
  input = text;
  while (*input) time_sync_process();
}

static void test_valid_utc(void) {
  send("TIME 20261107192530\r\n");
  assert(strcmp(reply, "OK TIME\n") == 0);
  datetime date;
  assert(rtc_read(&date));
  assert(date.year == 2026 && date.month == 11 && date.day == 7);
  assert(date.hour == 19 && date.minute == 25 && date.second == 30);
}

static void test_invalid_requests_leave_rtc_unchanged(void) {
  const char* invalid[] = {"TIME 20260229192530\n",  "TIME 2026110719253\n",
                           "TIME 202611071925300\n", "TIME 2026110719253X\n",
                           "OTHER20261107192530\n",  "TIME 20261107242530\n"};
  for (unsigned i = 0; i < sizeof(invalid) / sizeof(*invalid); i++) {
    unsigned before = writes;
    send(invalid[i]);
    assert(strcmp(reply, "ERR TIME\n") == 0 && writes == before);
  }
}

static void test_fragmented_request(void) {
  send("TIME 2028");
  assert(!reply[0]);
  send("0229010203\n");
  assert(strcmp(reply, "OK TIME\n") == 0);
}

static void test_overflow_is_bounded_and_recovers(void) {
  char flood[130];
  memset(flood, 'x', sizeof(flood));
  flood[128] = '\n';
  flood[129] = 0;
  input = flood;
  time_sync_process();
  assert(input == flood + 64);  // Host cannot monopolize the event loop.
  time_sync_process();
  time_sync_process();
  assert(strcmp(reply, "ERR TIME\n") == 0);
  send("TIME 20261107192530\n");
  assert(strcmp(reply, "OK TIME\n") == 0);  // Recover after overflow.
}

static void test_rtc_failure(void) {
  fail_write = true;
  send("TIME 20261107192530\n");
  assert(strcmp(reply, "ERR TIME\n") == 0);
  fail_write = false;
}

int main(void) {
  test_valid_utc();
  test_invalid_requests_leave_rtc_unchanged();
  test_fragmented_request();
  test_overflow_is_bounded_and_recovers();
  test_rtc_failure();
}
