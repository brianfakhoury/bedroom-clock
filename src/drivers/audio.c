#include "audio.h"

#include "boot.h"
#include "i2c.h"

// Power-down register sequences from Espressif's ES8311 suspend and ES7210
// stop drivers bundled with Waveshare's 2.16 Audio_Test example
static BOOT_ONLY bool write_sequence(uint8_t address,
                                     const uint8_t (*sequence)[2], unsigned n) {
  bool good = true;
  for (unsigned i = 0; i < n; i++)
    if (!i2c_write(address, sequence[i][0], &sequence[i][1], 1)) good = false;
  return good;
}

static BOOT_ONLY bool verify_registers(uint8_t address,
                                       const uint8_t (*checks)[3], unsigned n) {
  bool good = true;
  for (unsigned i = 0; i < n; i++) {
    uint8_t value;
    if (!i2c_read(address, checks[i][0], &value, 1) ||
        (value & checks[i][1]) != checks[i][2])
      good = false;
  }
  return good;
}

BOOT_ONLY bool audio_disable(void) {
  static const uint8_t codec[][2] BOOT_DATA = {
      {0x32, 0},    {0x17, 0}, {0x0e, 0xff}, {0x12, 2},    {0x14, 0},
      {0x0d, 0xfa}, {0x15, 0}, {0x02, 0x10}, {0x00, 0},    {0x00, 0x1f},
      {0x01, 0x30}, {0x01, 0}, {0x45, 0},    {0x0d, 0xfc}, {0x02, 0},
  };
  static const uint8_t adc[][2] BOOT_DATA = {
      {0x47, 0xff}, {0x48, 0xff}, {0x49, 0xff}, {0x4a, 0xff}, {0x4b, 0xff},
      {0x4c, 0xff}, {0x40, 0xc0}, {0x01, 0x7f}, {0x06, 7},
  };
  // Read only stable power/clock bits; reserved and reset bits are excluded
  static const uint8_t codec_checks[][3] BOOT_DATA = {
      {0x0d, 0xfb, 0xf8},
      {0x0e, 0x60, 0x60},
      {0x12, 2, 2},
      {0x01, 0x3f, 0},
  };
  static const uint8_t adc_checks[][3] BOOT_DATA = {
      {0x4b, 0xfc, 0xfc}, {0x4c, 0xfc, 0xfc}, {0x40, 0xc3, 0xc0},
      {0x01, 0x7f, 0x7f}, {0x06, 7, 7},
  };
  const unsigned codec_count = sizeof(codec) / sizeof(*codec);
  const unsigned adc_count = sizeof(adc) / sizeof(*adc);
  const unsigned codec_reads = sizeof(codec_checks) / sizeof(*codec_checks);
  const unsigned adc_reads = sizeof(adc_checks) / sizeof(*adc_checks);
  bool codec_off = false, adc_off = false;
  for (unsigned attempt = 0; attempt < 3; attempt++) {
    if (!codec_off) {
      bool written = write_sequence(0x18, codec, codec_count);
      bool verified = verify_registers(0x18, codec_checks, codec_reads);
      codec_off = written && verified;
    }
    if (!adc_off) {
      bool written = write_sequence(0x40, adc, adc_count);
      bool verified = verify_registers(0x40, adc_checks, adc_reads);
      adc_off = written && verified;
    }
    if (codec_off && adc_off) break;
  }
  return codec_off && adc_off;
}
