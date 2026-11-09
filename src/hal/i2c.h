#pragma once
#include <stdbool.h>
#include <stdint.h>

void i2c_init(unsigned clock_pin, unsigned data_pin);
// Device addresses are 7-bit
bool i2c_read(uint8_t address, uint8_t reg, uint8_t* data, unsigned n);
// Read bytes using a 16-bit register address
bool i2c_read16(uint8_t address, uint16_t reg, uint8_t* data, unsigned n);
bool i2c_write(uint8_t address, uint8_t reg, const uint8_t* data, unsigned n);
