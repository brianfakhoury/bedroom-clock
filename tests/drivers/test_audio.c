#include <assert.h>
#include <string.h>

#include "audio.h"
#include "i2c.h"

#include "../../src/drivers/audio.c"

static uint8_t regs[128][256];
static unsigned writes, failures;
static bool stuck, nack;

bool i2c_write(uint8_t address, uint8_t reg, const uint8_t* value, unsigned n) {
  assert(n == 1);
  writes++;
  if (address == 0x40 && reg == 0x4b && failures) {
    failures--;
    return false;
  }
  if (address == 0x18 && nack) return false;
  if (!(address == 0x40 && reg == 0x4b && stuck)) regs[address][reg] = *value;
  return true;
}

bool i2c_read(uint8_t address, uint8_t reg, uint8_t* value, unsigned n) {
  assert(n == 1);
  if (address == 0x18 && nack) return false;
  *value = regs[address][reg];
  return true;
}

static void reset(void) {
  memset(regs, 0, sizeof(regs));
  writes = failures = 0;
  stuck = nack = false;
}

static void test_shutdown(void) {
  reset();
  assert(audio_disable());
  assert(regs[0x40][0x4b] == 0xff && regs[0x18][0x0d] == 0xfc);
}

static void test_transient_failure_retries(void) {
  reset();
  failures = 1;
  assert(audio_disable() && failures == 0);
}

static void test_stuck_register_does_not_block_other_chip(void) {
  reset();
  stuck = true;
  assert(!audio_disable() && writes == 15 + 3 * 9);
  assert(regs[0x18][1] == 0);
}

static void test_permanent_failure_is_bounded_and_recovers(void) {
  reset();
  nack = true;
  assert(!audio_disable() && writes == 3 * 15 + 9);
  assert(regs[0x40][0x4b] == 0xff);
  nack = false;
  assert(audio_disable());
}

int main(void) {
  test_shutdown();
  test_transient_failure_retries();
  test_stuck_register_does_not_block_other_chip();
  test_permanent_failure_is_bounded_and_recovers();
}
