//! ESP32-C6 operations: pins, buses, timers, interrupts and power controls.
//! Nothing here knows the board's pin assignments.

pub mod clocks;
pub mod cpu;
pub mod gpio;
pub mod i2c;
pub mod interrupts;
pub mod pmu;
pub mod qspi;
pub mod radio;
pub mod registers;
pub mod sleep;
pub mod timer;
pub mod usb;
pub mod watchdog;
