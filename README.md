# Bedroom Clock

I needed a silent clock that is completely dark until it is tapped, when it quickly shows the time in red light (minimizes sleep disruption). I built this touch-activated bedside clock firmware for the [Waveshare ESP32-C6 Touch AMOLED 2.16](https://www.waveshare.com/esp32-c6-touch-amoled-2.16.htm), written in bare-metal Rust and RISC-V assembly. It takes advantage of lighting individual pixels on an AMOLED to simply show digits and not an entire backlit screen. It also takes advantage of the embedded processor to fully shut down the CPU between taps while booting back up in milliseconds.

You tap to show the time for three seconds. Red digits fade in and out, rotate with the board, and use 12-hour Pacific time with daylight saving (hardcoded for simplicity). A dim red border shows the battery estimate. You can press KEY (one of three buttons on the device) while awake to toggle the display on (which keeps the usb port active if needed). The RTC (realtime clock) keeps UTC while the processor and display sleep. There is no networking, RTOS, heap, or background polling.

## Build and flash

Install [rustup](https://rustup.rs). The pinned compiler and its RISC-V target install themselves on first use; the firmware itself depends on no crates.

```sh
cargo firmware
cargo test
cargo flash
```

Before flashing an installed clock, connect USB, tap, and press KEY to keep USB available. For initial installation or downloader recovery, hold BOOT while connecting USB with the board powered off, then release BOOT.

`cargo flash` builds and installs local source, verifies the flash, and sets the RTC from the computer. Startup shows the time, fades out, and sleeps, just like a touch wake. If it sleeps before time setting finishes, tap and press KEY when prompted; press KEY again to resume normal sleep.

The tool selects a single connected Espressif device. Use `--port /dev/cu.…` to choose explicitly, particularly when multiple boards are connected.

## Set the time

The time is set automatically when flashing, but you can set it again (disconnecting all power can lose the time): Connect USB, tap, press KEY, then run:

```sh
cargo set-time
```

Press KEY again when finished. The clock shows dashes if its RTC is invalid.

## Source layout

| Directory | Owns |
| --- | --- |
| `src/app` | Clock behavior and appearance: what to draw, how input changes it, and when to sleep |
| `src/board` | Choices for this board: pin assignments, battery settings, which devices stay powered, and their startup and sleep order |
| `src/drivers` | External chip commands: how to read or control each chip, independent of board wiring and clock behavior |
| `src/hal` | ESP32-C6 operations: how to use its pins, buses, timers, interrupts and power controls |
| `src/boot` | CPU entry and memory setup: what must happen before the app can run on boot or wake |
| `tests` | Host tests for every layer above the HAL, run against stand-in hardware |
| `xtask` | The build tool: firmware image creation, flashing and time setting |
