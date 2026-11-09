# Architecture

The external RTC owns UTC. One HP-core event loop presents the time on demand;
there is no RTOS, heap, network stack, LP program or periodic standby wake.

## Responsibilities

- `app/main.c`: cold boot initializes the board and app; wake restores hardware.
  Both enter `app_run()`.
- `app/app.c`: `app_init()` initializes presentation state once. `app_run()`
  owns input, display updates and the request to sleep. A cancelled sleep
  repeats the viewing interval without resetting debounce history.
- `app/input.c`: touch press detection, waking-contact consumption and button debounce.
- `app/clock.c`: private presentation state, initialization, orientation, time reading, fades
  and display deadlines. Activation restarts the fade; held refreshes preserve it.
- `app/digits.c` and `battery_indicator.c`: element geometry and redraw caches.
- `app/geometry.c`: pure rectangle rotation. UI elements transform their geometry
  and call the display driver directly; pixel callbacks receive screen coordinates.
- `app/orientation.c`: choose viewing orientation from accelerometer readings.
- `app/animation.c` and `local_time.c`: fade and time calculations.
- `app/time_sync.c`: one bounded USB request, `TIME YYYYMMDDhhmmss` in UTC.
- `board/board.c`: device startup, panel/touch reset, audio pin parking and
  sleep/wake sequencing. Device resets complete before driver initialization.
- `drivers`: register operations for each external device.
- `board/events.c`: maps raw GPIO/USB/timer events to board event names.
- `board/board_power.c`: charger/rail policy, unused peripheral clocks, touch wake
  pin, parked/held outputs and ROM resume entry.
- `hal/i2c.c`: software I²C; the board supplies SCL/SDA pins at initialization.
- `hal/qspi.c`: software QSPI transfers; the board supplies bus pins.
  `drivers/display.c` owns panel commands, pixel windows and sleep transitions.
- `hal/interrupts.c`: interrupt setup, deadline waiting and raw event capture.
- `hal/sleep.c`: GPIO wake handoff, cancellation recovery and MCU deep-sleep entry.
- `hal`: MCU GPIO, timers, USB, watchdogs and power setup. It has no dependency
  on board pin assignments or board interfaces. Board power policy uses direct
  register writes for its unused-clock list; MCU sleep mechanics stay in HAL.
- `boot`: reset/wake assembly, memory routines and linker layout.
- `hal/registers.h`: the register addresses used by this firmware. PMU setup
  uses masked writes to preserve unrelated fields, without a full device struct.

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
4 KiB stack reserve. Startup-only hardware routines use disposable HP SRAM.
The LP CPU does not execute this code; the HP CPU uses retained memory on wake.

HP CPU/TOP/AON, radio, flash supply, main crystal/PLL and RC_FAST sleep. LP GPIO
remains powered for touch detection. The external RTC maintains wall time.
The LP watchdog guards active work but pauses during legitimate waits; it
cannot recover a permanently lost touch-wake signal.

Device initialization verifies charger settings and shutdown writes. Audio
and IMU shutdown use bounded retries; a persistent bus failure is reported or
returned, not treated as successful shutdown. Charge parameters belong in
`board/battery.h`, not the PMIC driver. The board keeps the audio supply on
because removing it broke the shared I²C bus. See the [power audit](power-audit.md).

USB reads are bounded per event-loop pass. Only validated time updates reach
the RTC, and the host requires an acknowledgement. USB is disconnected during
standby. Drawing and I²C use software buses; these have been kept separate from
application behavior so hardware changes remain local.
