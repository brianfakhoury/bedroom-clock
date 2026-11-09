#include <assert.h>
#include <string.h>

#include "accel.h"
#include "i2c.h"
#include "timer.h"

#include "../../src/drivers/accel.c"

static uint8_t device, regs[256];
static unsigned sample_index;
static const int16_t samples[4][3] = {
    {-32768, 100, -1}, {32767, 200, -2}, {-100, 300, -3}, {97, 400, -4}};
static unsigned reads, shutdown_writes, fail_next_shutdown;
static bool fail_reads, fail_writes, fail_sample, fail_shutdown,
    ignore_shutdown;

bool i2c_write(uint8_t address, uint8_t reg, const uint8_t* data, unsigned n) {
  assert(address == device && n == 1);
  if (fail_writes) return false;
  bool shutdown = (reg == 8 && *data == 0) || (reg == 2 && *data == 0x41);
  if (shutdown) shutdown_writes++;
  if (shutdown && fail_shutdown) return false;
  if (shutdown && fail_next_shutdown) {
    fail_next_shutdown--;
    return false;
  }
  if (shutdown && ignore_shutdown) return true;
  regs[reg] = *data;
  return true;
}

bool i2c_read(uint8_t address, uint8_t reg, uint8_t* data, unsigned n) {
  reads++;
  if (fail_reads || address != device) return false;
  if (reg == 0) {
    assert(n == 1);
    *data = 5;
    return true;
  }
  if (reg == 0x2e) {
    assert(n == 1);
    *data = 1;
    return !fail_sample;
  }
  if (reg == 2 || reg == 8) {
    assert(n == 1);
    *data = regs[reg];
    return true;
  }
  assert(reg == 0x35 && n == 6);
  for (unsigned axis = 0; axis < 3; axis++) {
    uint16_t value = samples[sample_index % 4][axis];
    data[axis * 2] = value;
    data[axis * 2 + 1] = value >> 8;
  }
  sample_index++;
  return true;
}

void delay_us(uint32_t us) { (void)us; }

static void reset(void) {
  memset(regs, 0, sizeof(regs));
  fail_reads = fail_writes = fail_sample = fail_shutdown = ignore_shutdown =
      false;
  sample_index = 0;
  reads = shutdown_writes = fail_next_shutdown = 0;
  device = 0x6a;
  assert(accel_init());
}

static void assert_unavailable(void) {
  int16_t xyz[3];
  unsigned before = reads;
  assert(!accel_read(xyz) && reads == before);
}

static void test_discovery(void) {
  reset();
  for (device = 0x6a; device <= 0x6b; device++) {
    assert(accel_init());
    assert(regs[2] == 0x41 && regs[8] == 0);
    fail_reads = true;
    assert(!accel_init());
    assert_unavailable();
    fail_reads = false;
  }
  device = 0x6a;
  fail_writes = true;
  assert(!accel_init());
  assert_unavailable();
  fail_writes = false;
  device = 0;
  assert(!accel_init());
  assert_unavailable();
}

static void test_every_sample_attempt_shuts_down(void) {
  reset();
  int16_t xyz[3];
  for (unsigned i = 0; i < 100; i++) {
    fail_sample = i & 1;
    unsigned before = shutdown_writes;
    assert(accel_read(xyz) == !fail_sample);
    assert(shutdown_writes == before + 2);
    assert(regs[8] == 0 && regs[2] == 0x41);
  }
}

static void test_sample_average(void) {
  reset();
  int16_t xyz[3] = {10, 20, 30};
  assert(accel_read(xyz) && sample_index == 4);
  assert(xyz[0] == -1 && xyz[1] == 250 && xyz[2] == -2);
  fail_sample = true;
  assert(!accel_read(xyz));
  assert(xyz[0] == -1 && xyz[1] == 250 && xyz[2] == -2);
}

static void test_shutdown_retries(void) {
  reset();
  int16_t xyz[3];
  fail_next_shutdown = 2;
  assert(accel_read(xyz));
  assert(regs[8] == 0 && regs[2] == 0x41);
}

static void test_shutdown_requires_readback(void) {
  reset();
  int16_t xyz[3];
  ignore_shutdown = true;
  assert(!accel_read(xyz));  // ACK without a state change must not pass.
}

static void test_permanent_failure_is_bounded_and_recoverable(void) {
  reset();
  int16_t xyz[3];
  fail_shutdown = true;
  unsigned before = shutdown_writes;
  assert(!accel_read(xyz));
  assert(shutdown_writes == before + 6);
  // Reporting a bus failure cannot guarantee that hardware is off.
  assert(regs[8] == 1 && regs[2] == 0x40);
  fail_shutdown = false;
  assert(accel_read(xyz));
  assert(regs[8] == 0 && regs[2] == 0x41);
}

int main(void) {
  assert_unavailable();  // Before discovery, no bus transaction is attempted.
  test_discovery();
  test_every_sample_attempt_shuts_down();
  test_sample_average();
  test_shutdown_retries();
  test_shutdown_requires_readback();
  test_permanent_failure_is_bounded_and_recoverable();
}
