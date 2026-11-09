#include "display.h"

#include <stdbool.h>

#include "boot.h"
#include "qspi.h"
#include "timer.h"

enum {
  DISPLAY_OFF = 0x28,
  DISPLAY_ON = 0x29,
  SLEEP_OUT = 0x11,
  SLEEP_IN = 0x10,
  COLUMN_ADDRESS = 0x2a,
  ROW_ADDRESS = 0x2b,
  MEMORY_WRITE = 0x2c,
  MEMORY_CONTINUE = 0x3c,
  BRIGHTNESS = 0x51,
  SPI_COMMAND = 0x02,
  QSPI_PIXELS = 0x32,
};

static bool sleeping;

static void write_command(uint8_t code, const uint8_t* data, unsigned count) {
  qspi_begin();
  qspi_write_byte(SPI_COMMAND);
  qspi_write_byte(0);
  qspi_write_byte(code);
  qspi_write_byte(0);
  for (unsigned i = 0; i < count; i++) qspi_write_byte(data[i]);
  qspi_end();
}

static void command(uint8_t code) { write_command(code, 0, 0); }

static void write_register(uint8_t code, uint8_t value) {
  write_command(code, &value, 1);
}

static void draw(unsigned x, unsigned y, unsigned w, unsigned h, uint16_t color,
                 uint16_t (*pixel)(unsigned, unsigned)) {
  uint8_t columns[] = {x >> 8, x, (x + w - 1) >> 8, x + w - 1};
  write_command(COLUMN_ADDRESS, columns, sizeof(columns));
  uint8_t rows[] = {y >> 8, y, (y + h - 1) >> 8, y + h - 1};
  write_command(ROW_ADDRESS, rows, sizeof(rows));
  command(MEMORY_WRITE);
  qspi_begin();
  qspi_write_byte(QSPI_PIXELS);
  qspi_write_byte(0);
  qspi_write_byte(MEMORY_CONTINUE);
  qspi_write_byte(0);
  for (unsigned i = 0; i < w * h; i++) {
    uint16_t value = pixel ? pixel(x + i % w, y + i / w) : color;
    qspi_write_quad16(value);
  }
  qspi_end();
}

void display_rect(unsigned x, unsigned y, unsigned w, unsigned h,
                  uint16_t color) {
  draw(x, y, w, h, color, 0);
}

void display_pattern(unsigned x, unsigned y, unsigned w, unsigned h,
                     uint16_t (*pixel)(unsigned, unsigned)) {
  draw(x, y, w, h, 0, pixel);
}

static void configure(void) {
  write_register(0xfe, 0x20);
  write_register(0x19, 0x10);
  write_register(0x1c, 0xa0);
  write_register(0xfe, 0);
  write_register(0xc4, 0x80);
  write_register(0x3a, 0x55);
  write_register(0x36, 0x30);
  write_register(0x53, 0x20);
  delay_us(10000);
  write_register(0x63, 0xff);
  delay_us(10000);
  write_register(BRIGHTNESS, 0);
  write_register(0x58, 0);
}

BOOT_ONLY void display_init(void) {
  command(DISPLAY_OFF);
  command(SLEEP_OUT);
  delay_us(600000);
  configure();
  display_rect(0, 0, 480, 480, 0);
  display_off();
}

void display_on(void) { command(DISPLAY_ON); }

void display_off(void) { command(DISPLAY_OFF); }

void display_brightness(unsigned value) {
  write_register(BRIGHTNESS, (uint8_t)value);
}

void display_sleep(void) {
  if (sleeping) return;
  command(DISPLAY_OFF);
  command(SLEEP_IN);
  // CO5300 pp.143–145: opposite sleep transitions must be 120 ms apart
  // Complete the guard before CPU sleep, including immediate retaps
  delay_us(120000);
  sleeping = true;
}

void display_wake(void) {
  if (!sleeping) return;
  command(SLEEP_OUT);
  delay_us(120000);
  // Sleep Out reloads extended/test defaults. Restore the module settings
  // while dark; no reset or framebuffer clear, so retained pixels survive
  configure();
  sleeping = false;
}
