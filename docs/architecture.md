# Architecture

The external RTC owns UTC. One HP-core event loop presents the time on demand;
there is no RTOS, heap, network stack, LP program or periodic standby wake.

## Responsibilities

- `main.rs`: cold boot initializes the board and app; wake restores hardware.
  Both enter `App::run`. The board and app live in two retained statics.
- `app/mod.rs`: `App::init` prepares the display once. `App::run` owns input,
  display updates and the request to sleep. A cancelled sleep repeats the
  viewing interval without resetting debounce history.
- `app/input.rs`: touch press detection, waking-contact consumption and button debounce.
- `app/clock.rs`: private presentation state, initialization, orientation, time reading, fades
  and display deadlines. Activation restarts the fade; held refreshes preserve it.
- `app/digits.rs` and `battery_indicator.rs`: element geometry and redraw caches.
- `app/geometry.rs`: pure rectangle rotation. UI elements transform their geometry
  and draw on the panel directly; pixel functions receive screen coordinates.
- `app/orientation.rs`: choose viewing orientation from accelerometer readings.
- `app/animation.rs` and `local_time.rs`: fade and time calculations.
- `app/time_sync.rs`: one bounded USB request, `TIME YYYYMMDDhhmmss` in UTC.
- `board/mod.rs`: the `Board` trait, everything the application asks of the
  hardware. The application is written against it and nothing below it.
- `board/waveshare.rs`: this board's `Board`. Device startup, panel/touch
  reset, audio pin parking and sleep/wake sequencing. Device resets complete
  before driver initialization.
- `board/events.rs`: maps raw GPIO/USB/timer events to board event names.
- `board/power.rs`: charger/rail policy, unused peripheral clocks, touch wake
  pin, parked/held outputs and ROM resume entry.
- `drivers`: register operations for each external device, written against
  the HAL's `I2c`, `Qspi` and `Delay` traits. `drivers/display.rs` owns panel
  commands, pixel windows and sleep transitions behind the `Panel` trait.
- `hal/i2c.rs`: software I²C; the board supplies SCL/SDA pins.
- `hal/qspi.rs`: software QSPI transfers; the board supplies bus pins.
- `hal/interrupts.rs`: interrupt setup, deadline waiting and raw event capture.
- `hal/sleep.rs`: GPIO wake handoff, cancellation recovery and MCU deep-sleep entry.
- `hal`: MCU GPIO, timers, USB, watchdogs and power setup. It has no dependency
  on board pin assignments or board interfaces. The board supplies the list of
  unused clocks; MCU sleep mechanics stay in HAL.
- `hal/registers.rs`: the register addresses used by this firmware, and the
  `Reg` type through which every register is read and written. PMU setup uses
  masked writes to preserve unrelated fields, without a full device struct.
- `hal/cpu.rs`: the three RISC-V instructions that Rust cannot express.
- `boot`: reset/wake assembly, memory routines, linker layout and the panic
  handler, which waits for the watchdog.

## Seams and safety

Each layer meets the one below through a trait where a test needs to stand in
for hardware, and through plain functions elsewhere. Drivers take the bus they
use as an argument, so one `SoftI2c` serves five chips without shared state.
Traits are resolved at build time; there is no dynamic dispatch.

`tests/` mirrors `src/` and is compiled into the firmware crate for host
builds only. Driver tests supply a register-level chip in place of the bus;
application tests supply a `FakeBoard` whose panel is a framebuffer and whose
time passes only while the application waits. The code that sequences real
pins and registers (`board/waveshare.rs`, most of `hal`) runs only on the MCU.

`unsafe` is confined to register access in `Reg`, the instructions in
`hal/cpu.rs`, the ROM calls in `hal/sleep.rs`, the linker symbols in `boot`,
the two retained statics in `main.rs`, and the attributes that export entry
points and place startup-only code. Everything else is checked by the
compiler: an absent accelerometer is an `Option`, a missing deadline is
`None`, and a device result cannot be ignored silently.

## Wake, display, sleep

Cold boot initializes hardware and presentation state, then shows the time and
sleeps just like a touch wake. Touch wake preserves presentation state. A touch interrupt wakes the HP core through the LP GPIO
falling-edge latch. Wake restores timers, pin routing and I²C, without resetting
the external devices. The application exits panel sleep, samples orientation,
reads the RTC, then draws and fades in. An invalid RTC produces dashes.

The awake loop draws the first frame before waiting. Each event handles input,
updates the display, then services USB time setting. When the display becomes
hidden, the app asks the board to sleep the panel and processor.
A cancelled sleep starts another viewing interval without resetting debounce
history; the triggering contact is already consumed.

The interval starts after frame preparation: 180 ms fade-in, 2470 ms hold,
350 ms fade-out. A retap starts another interval from the current brightness.
KEY hold refreshes time and orientation once per second. Releasing an expired
hold starts fade-out immediately. Absolute deadlines prevent slow work from
extending the display interval.

The battery ring is primed at cold boot and retained by the panel, so it fades
with the digits. Each activation reads the gauge after fade-in; KEY hold also
refreshes it once per second. Changed readings redraw only the ring. Invalid
readings hide it. No battery work runs during standby.

After fade-out, the panel enters Sleep In before MCU standby. A touch pending
during the pin handoff cancels sleep and reactivates the display. Otherwise ROM
resumes through `boot/wake.S` with a fresh stack and retained application state.

## Memory and power

Runtime code and state live in 16 KiB retained LP SRAM, with a linker-enforced
4 KiB stack reserve. Startup-only hardware routines use disposable HP SRAM:
they are placed in `.init.code` and `.init.data` by attribute, and must never
be called after the first sleep. `cargo xtask build` reports retained use.
The LP CPU does not execute this code; the HP CPU uses retained memory on wake.

HP CPU/TOP/AON, radio, flash supply, main crystal/PLL and RC_FAST sleep. LP GPIO
remains powered for touch detection. The external RTC maintains wall time.
The LP watchdog guards active work but pauses during legitimate waits; it
cannot recover a permanently lost touch-wake signal.

Device initialization verifies charger settings and shutdown writes. Audio
and IMU shutdown use bounded retries; a persistent bus failure is reported or
returned, not treated as successful shutdown. Charge parameters belong in
`board/battery.rs`, not the PMIC driver. The board keeps the audio supply on
because removing it broke the shared I²C bus. See the [power audit](power-audit.md).

USB reads are bounded per event-loop pass. Only validated time updates reach
the RTC, and the host requires an acknowledgement. USB is disconnected during
standby. Drawing and I²C use software buses; these have been kept separate from
application behavior so hardware changes remain local.
