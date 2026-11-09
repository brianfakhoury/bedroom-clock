#pragma once
#include <stdint.h>

// Four contiguous data GPIOs starting at data0_pin, separate clock and CS
void qspi_init(unsigned clock_pin, unsigned data0_pin, unsigned cs_pin);
void qspi_begin(void);
void qspi_end(void);
// MSB first on data0; quad transfers send the highest nibble first
void qspi_write_byte(uint8_t value);
void qspi_write_quad16(uint16_t value);
