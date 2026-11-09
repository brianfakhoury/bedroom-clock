# Power audit — 2026-11-07

This is a historical test record. The later code cleanup removed the simulator,
its targets, and the serial diagnostic commands mentioned below. Their recorded
results remain evidence for the power policy; they are not current tooling.

This audit uses the board schematic, manufacturer register documentation, live
register readbacks, actual tap tests, and firmware fault-injection/simulation.
A current meter is not a prerequisite for this work. Component currents below
are calculated or datasheet figures, not measured battery current. Runtime on
battery is the available end-to-end check; gauge percentage alone is not one.

## Changes retained

- The CO5300 enters Sleep In (0x10), after Display Off, on every route to MCU
  sleep. Its internal DC/DC, scanning and oscillator stop. Wake uses Sleep Out,
  the documented transition delays and restored interface configuration.
- DCDC1 no longer forces CCM/PWM; automatic mode allows light-load operation.
- GPIO0/1/2 are parked **high** after the panel sleeps and while both panel/SD
  chip selects are high. R15/R12/R17 are 10 kohm pull-ups to VCC3V3. Previously
  these lines were held low. Three low lines imply 3 × 3.3 V / 10 kohm =
  **0.99 mA on the 3.3 V rail**, or 3.27 mW; battery-side savings depend on
  conversion efficiency and battery voltage. GPIO routing and bus idle levels
  are restored before the next transaction. GPIO3/4 have no such pull-ups.
- PCF85063A CLKOUT is explicitly disabled (Control_2 COF=111), with readback.
  CLKOUT and CLKOE are unconnected in the board schematic. This leaves the
  timekeeping oscillator running and does not change the time or alarm setup.
- QMI8658 shutdown attempts both sensor/oscillator writes, retries at most
  three times, and verifies CTRL7=0 and CTRL1=0x41. A failed sample still runs
  this cleanup. A permanently inaccessible device cannot be guaranteed off;
  failure is returned instead of claiming success. There is no endless retry
  loop keeping the CPU awake.

- RC_FAST is now off in the LP sleep state; LP GPIO remains retained. A normal
  tap woke the diagnostic firmware with this configuration (GPIO cause 0x04,
  10.755 s after entry, well before the approximately 49 s recovery timer).
- Unused PMIC interrupt enables are cleared with readback. The application
  polls status and does not use the PMIC IRQ pin. This avoids an unused IRQ
  holding its external 10 kohm pull-up low; the potential load is about
  0.3 mA when asserted, not a measured continuous saving.
- Unused ADC channels are disabled. Battery ADC stays enabled; TS ADC is
  preserved when configured for charger monitoring. This board's existing
  REG50=0x12 selects TS as a general input with its current source off, so its
  unused TS conversion is disabled (REG30 changes from 0x03 to 0x01).
  Charger-temperature selection is not changed. In particular, external TS
  charger monitoring was already disabled; we must not claim it is enabled.
- Audio shutdown now reads back stable power-down bits and retries each chip
  independently, at most three times. An ACK with unchanged bits is a failure;
  a failed chip does not prevent shutdown of the other chip.

## Complete board inventory and disposition

| Load / option | Evidence and disposition |
| --- | --- |
| HP CPU, TOP, HP AON, radio, main crystal/PLL | Already powered down in standby. No networking stack or periodic wake. Unused clocks gated at boot and resume. |
| LP CPU | Does not execute a background program. |
| LP SRAM | Retains firmware/state so flash and HP SRAM can sleep. Removing retention would require a different cold-boot architecture and lose current wake/render state; not a drop-in saving. Keep the full 4 KiB stack reserve. |
| LP peripheral domain and RC_FAST | GPIO edge wake retained; RC_FAST off in sleep after successful normal-tap hardware test. EXT1 with LP_PERI off did not provide timely normal-tap wake; do not ship that configuration. See timed trials below. |
| External flash | U3 is supplied by VDD_SPI. Live eFuse 0x600b0830=0 (VDD_SPI_AS_GPIO=0), PMU VDD_SPI control 0x600b0118=0x63fc0000 (software override bit31 clear), and HP_SLEEP_DIG_POWER=0xe8200000 (VDD_SPI power-down requested). No force-on override was found. No extra deep-power-down command needed; these are register/configuration checks, not electrical measurements. |
| Panel supply | VCC3V3 is shared with MCU/touch. ALDO3 is **LCD_RESET**, not a panel supply switch. Cutting it does not remove panel supply. |
| Panel deep standby (0x4F, DSTB=1) | Tested command, hardware reset and full initialization/redraw: about 2.38 s, with the user confirming the red test square appeared. Requires RESX low >3 ms (p183). Not adopted: substantial wake latency and no documented incremental current saving versus Sleep In. |
| CST9220 touch | D11E handshake and D10F command acknowledged on this unit, but produced rapid repeated IRQs and non-coordinate reports rather than usable standby behavior. Hardware reset restored normal operation. Reject this command for production; names in comparison drivers are insufficient. |
| Touch monitor mode | Related **CST9217** manufacturer datasheet pp6–7 describes automatic lower-rate monitoring after inactivity and transition to active on touch. It lists 12 µA typical at 30 Hz (p1). This is evidence of a family capability, **not** confirmation of CST9220 firmware settings/current. The observed long-idle delay is compatible with scanning behavior but does not establish it. |
| Touch deep sleep / removing touch supply | Documented sleep requires wake command/reset; cannot detect the user's tap while asleep. Reject for this product. Periodic polling would change response/reliability and is not equivalent to uninterrupted touch wake. |
| QMI8658 | CTRL7 disables accelerometer/gyro; CTRL1 SensorDisable stops its high-speed oscillator. Already the documented power-down state. I2C remains available. Robust cleanup added above. |
| ES8311 codec | Suspend sequence powers down analog blocks (0D=FC), ADC/PGA (0E=FF), DAC (12=02), VMID and digital clocks (01=00). No MCLK/BCLK/LRCK output during sleep. |
| ES7210 microphone ADC / microphones | 4B/4C=FF disables references, microphone bias, PGA and ADCs; 40=C0 disables analog/VMID; 01=7F gates clocks; 06=07 disables DLL and internal pin pulls. Microphones are fed by MICBIAS12, so this shuts their supply down too. Verified against ES7210 register descriptions, not merely an audio mute. |
| Audio ALDO1 supply | Physical attempt to disable it made the shared I2C bus inaccessible, including PMIC/RTC. Restoring software alone could not restore the rail; required power cycling. ES8311 has mixed analog/main supplies. Back-power/clamp mechanism is plausible but unmeasured. Keep rail on with software shutdown; hardware isolation is needed before safely retrying rail removal. |
| Speaker amplifier | ALDO2 controls NS4150B CTRL, not VDD. CTRL is disabled and output discharge enabled. Amplifier VDD remains on shared VCC3V3. No independent supply gate exists. |
| RTC | Required for time. CLKOUT disabled; no MCU timer wake to poll time. RTCLDO retained. Clock frequency or oscillator load changes would risk time accuracy for a small unquantified saving. |
| SD card | CS high. Empty socket has no card load, but its pull-ups mattered regardless of card presence. An inserted card is supplied continuously and has no independent rail switch. |
| SPI pull-ups | Fixed above; avoid holding pulled-up signals low. I2C lines are released high after STOP, so their 2.2 kohm pull-ups do not form the same intentional DC load. |
| PWR button circuit | Resolved as a hardware-only load under documented controls; see the trace below. PWRON’s internal pull-up biases T1 on while the button is released, loading R11 (10 kohm). Neither GPIO18 nor a documented PMIC register can turn T1 off while preserving normal PWR-button operation. |
| Other pins | Audio clock/data outputs held low; SD/panel CS and touch RESET held high. KEY/BOOT pulled high. GPIO16/17 share IMU interrupt wiring, GPIO21 is audio ADC data: do not force these against external outputs. HP domain sleeps; arbitrary pull changes are not justified. |
| PMIC ADCs | Live REG30=03 before, 01 after policy change; REG50=12 unchanged. Battery ADC retained, unused TS conversion disabled because TS is already configured as a general input, not charger monitoring. Other unused channels were already off on this unit; policy now explicitly disables them. |
| PMIC gauge | No verified current telemetry. SOC is an estimator, not a linear energy counter and not a calibrated 1000 mAh discharge curve. Gauge model/custom battery calibration is distinct from 4.2 V/500 mA charger configuration. Do not interpret nonlinear percentage changes as proof of a firmware power leak. |
| PMIC outputs/LED | Required rails retained; unused outputs/indicator disabled. Live REG18=0A: backup charging and watchdog off, main charging and gauge on. Unused IRQ enables REG40/41/42 now zero. Charging limits, safety timers and protection selection unchanged. |
| Lower main supply voltage | Shared 3.3 V rail powers display/touch/audio/MCU/SD. Cannot undervolt solely on ESP32 minimum voltage; all connected parts/module requirements must be met. No arbitrary voltage reduction. |
| PMIC shutdown / BATFET off | Would stop MCU touch wake on this board's shared supply topology. PWR-button-only wake is not the requested interaction. |
| USB | Pads disabled before sleep. USB-connected charging behavior is not battery-only runtime. |
| Shorter/dimmer awake UI | Possible but not the priority: user reports >99.9% idle time. Ring/animation optimization cannot remove continuous off-state loads. |

## PWR circuit resolution

The schematic KEYS block connects:

- PMIC PWRON to the PWR button through RP7 (510 ohm).
- The button side of RP7 to T1's gate through R28 (1 kohm).
- T1 source to ground; T1 drain to GPIO18 and R11 (10 kohm to VCC3V3).

AXP2101 pin30 has an **internal 100 kohm pull-up to VINT** (datasheet p8).
With the button released, essentially no steady current flows into the MOSFET
gate, so RP7/R28 do not meaningfully divide this gate voltage. T1 is biased on,
providing an inverted copy of PWR-button state on GPIO18. R11 then supplies
approximately 3.3 V / 10 kohm = 0.33 mA to ground (about 1.09 mW), assuming
T1 pulls its drain close to ground as intended. This is a circuit calculation,
not a current measurement or a claim about exact battery-input current.

The documented alternatives were checked:

| Proposed firmware action | Result |
| --- | --- |
| Configure GPIO18 as input / disable its internal pull-up | Does not remove external R11 or turn off T1. This firmware does not use GPIO18 as a button input. |
| Drive GPIO18 low | Leaves the R11-to-ground path; saves nothing. |
| Drive GPIO18 high | Drives against T1's conducting drain, creating contention; not a saving. |
| Change PMIC button durations or disable long-press handling | REG22/REG27 change event behavior, not the internal pull-up or T1 gate voltage. |
| Disable the PWRON pull-up | No such runtime control is documented in the AXP2101 register map. PWRON/EN customization is not a GPIO output control. |
| Hold PWRON low | No independent MCU connection exists to drive this gate/button net. It also represents a permanently pressed PWR button and defeats normal recovery behavior. |
| Turn off R11's supply | VCC3V3 is also the MCU/touch/main peripheral supply. This loses touch-to-wake. |

**Conclusion:** there is no documented firmware-only way to eliminate this
path while retaining the existing PWR-button behavior and touch wake. It is
not an outstanding firmware candidate.

A board revision could remove the unneeded GPIO18 button-sense branch while
retaining the button-to-PMIC connection. Removing R11 alone removes this
external resistor load but leaves GPIO18 floating when T1 is off; a complete
board change must account for that input. Increasing R11 reduces the load
while retaining the sense branch, subject to GPIO leakage/noise/rise-time
validation. No physical component changes have been made or requested.

Evidence: archived Waveshare schematic KEYS block and netlist; AXP2101 pin30
(p8), power-on/off behavior (section6.5.4), REG22 and REG27; firmware pin map,
board initialization/resume and sleep holds. No current meter is needed to
establish these connections or the absence of a documented control.

## EXT1 hardware trial and rollback

Prepared the ESP-IDF v5.4.2 C6 EXT1 path: LP_AON external wake GPIO5 low,
PMU wake bit1, held input pad, LP_PERI power-down and RC_FAST off. The real
ELF passed simulated wake/sleep/configuration checks; on the physical C6
revision v0.2 **neither tap woke it**. PWR cycling recovered USB. Reverted to
LP GPIO falling-edge wake (bit2), LP_PERI retained, RC_FAST retained.
The restored build was flashed with hash verification, UTC was set, and the
user confirmed two successive single-tap wake/fade/sleep cycles work normally.

### Follow-up trials with automatic recovery

A standalone diagnostic image used an LP timer recovery wake, then stayed
awake for USB inspection. It was temporary; production has no timer wake.
The short initial manual windows were too small for prompt delivery and user
response: their failures are discarded. Later trials confirmed USB disappearance
before prompting and used a nominal 45 s timer (about 49.3 s on this RC clock).

| Trial | Observation / conclusion |
| --- | --- |
| Baseline GPIO, clocks retained; armed 19:17:51 UTC | Reconnected after 12.370 s, cause 0x04: valid normal-tap wake. |
| EXT1, LP_PERI/RC_FAST off; armed 19:18:14 UTC | User responded within the window, but reconnect took 49.311 s, cause 0x12 (timer and EXT1). No timely wake. Do not ship. |
| GPIO, RC_FAST off; armed 19:19:24 UTC | Prompt was delayed until after recovery. Discarded and repeated. |
| GPIO, RC_FAST off; armed 19:22:19 UTC | Reconnected after 10.755 s, cause 0x04. Normal tap succeeded without the recovery timer. Adopted; user subsequently confirmed two normal-firmware tap/fade/sleep cycles. |
| EXT1 with LP_PERI/RC_FAST retained; armed 19:22:47 UTC | Recovered after 49.382 s, cause 0x10. User reply arrived after recovery, so tap timing is ambiguous; do not use this as a conclusive failure. |

The repeated IRQ stream induced by D10F could wake all five diagnostic
configurations, including EXT1 with LP_PERI off. That establishes response to
repeated interrupts, not reliable capture of a normal tap. The precise reason
normal-tap EXT1 wake failed is not isolated; pulse capture/routing remains a
possible cause. LP GPIO works with RC_FAST off, so we retain that successful
path instead of shipping the failed EXT1 trial or claiming EXT1 is impossible.

The D10F experiment returned handshake 0x1E and command response 0A 0F 00 00,
then rapid IRQ toggles and D000 data CA 01 0A 0F 55 00 FF FF 00 00 instead of
the normal coordinate marker 0xAB. D109 stopped the stream and hardware reset
restored normal touch reports. No controller firmware was flashed.

Deep-standby panel recovery was separately confirmed visually. Its measured
2.38 s is the current complete reset/initialization path, not an irreducible
hardware minimum. Optimizing that path is possible, but no incremental power
figure establishes the benefit over the existing Sleep In policy.

## Verification and limits

- Final `make test all awake-check emulator-check` passed, including all 14
  emulator tests and 24 repeated sleep cycles. The complete 4 KiB retained
  stack reserve still links. `git diff --check` passed.
- Normal awake-boot firmware was restored with flash hash verification and
  UTC acknowledgement. Live readbacks confirmed automatic DCDC1 mode,
  audio FC/00/07/7F, IMU41/00 and unused clocks zero. After release, USB
  disappeared as expected and the user confirmed two single-tap wakes with
  a complete fade-out between them. No diagnostic image or recovery timer
  remains installed.
- Host tests cover 100 alternating successful/failed IMU sampling cleanups,
  transient shutdown NACK recovery, an ACK with unchanged register state,
  and bounded permanent bus failure. RTC clock-output masking preserves
  unrelated controls. Audio tests cover NACK retry, stuck readback, permanent
  failure and independent shutdown; PMIC tests cover masked ADC/IRQ writes,
  TS selection and partial bus failures.
- Real-ELF simulation exercises 24 wake/sleep cycles with rotations, KEY
  holds, RTC failures and hour-long model idle. Each sleep asserts panel
  Sleep In, IMU/audio shutdown registers, rail policy, radio/USB shutdown,
  high SPI pull-up lines, RTC CLKOUT off, and no timer/background activity.
- Simulation cannot establish real IRQ capture, oscillator behavior, leakage,
  rail voltage, or lifetime. The failed EXT1 trial demonstrates this boundary.
- Live POWER readbacks before the trial showed audio FC/00/07/7F, IMU41/00,
  automatic regulator enabled and all audited unused clock masks zero.
- Persistent I2C failure can prevent peripheral shutdown. The accelerometer
  now retries and verifies; audio startup failures are reported and its
  startup sequence now retries and verifies stable shutdown bits on both chips.
- The LP watchdog guards active firmware. It cannot repair a sleeping device
  whose touch-wake input has been lost. Long-term real tap/runtime testing
  remains relevant even without a current meter.

## Sources

Local snapshots and hashes are recorded in `docs/references/manifest.json`.
Key sources: Waveshare schematic (R12/R15/R17, T1/R11/R28, supply nets),
CO5300 pp143–146/183, QMI8658A CTRL1/CTRL7 and section7.1, PCF85063A
rev7.3 section7.2.2.4, ES7210 registers01/06/40/4B/4C, ES8311 register
and suspend documentation, ESP32-C6 TRM and pinned ESP-IDF PMU/LP_AON code.
Touch comparison sources are the pinned Waveshare, SensorLib and LilyGO
implementations; the CST9217 PDF is explicitly a related-chip reference.
