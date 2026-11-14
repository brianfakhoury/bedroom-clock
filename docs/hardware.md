# Hardware

The board is the Waveshare ESP32-C6 Touch AMOLED 2.16. Schematics, datasheets
and pinned vendor sources are indexed in [references](references/README.md).

| Device | Function |
| --- | --- |
| ESP32-C6 | Processor, USB and sleep control |
| PCF85063A | UTC time and calendar |
| CO5300 | 480 × 480 AMOLED |
| CST9220 | Touch detection |
| QMI8658 | Orientation sampling |
| AXP2101 | Supplies and battery charging |
| ES8311 / ES7210 | Unused audio devices, shut down through I²C |

## Wiring and display

`src/board/pins.rs` names the GPIO connections. Panel CS is GPIO15, touch IRQ
is GPIO5, and touch reset is GPIO11. Some vendor examples swap CS and IRQ;
the schematic and XiaoZhi configuration agree with this firmware.

CO5300 windows require even start coordinates and dimensions (pp159–161).
The panel sleeps with Display Off followed by Sleep In. Both sleep transitions
wait 120 ms; wake restores module configuration without clearing retained
pixels. Deeper standby requires reset and full initialization; it is not used.

Touch uses address 0x5A and reads ten-byte reports from 0xD000. Waveshare uses
CST9217-compatible driver names for this CST9220 module. The normal reset
configuration is retained: the experimental D10F command caused rapid IRQs.
The QMI8658 is discovered at 0x6A or 0x6B and shut down after every sample.

## Supplies

- DCDC1 powers the shared 3.3 V rail and uses automatic switching mode.
- ALDO1 powers audio analog/ADC supplies. Keep it on: disabling it made the
  shared I²C bus inaccessible. Both audio chips instead use software shutdown.
- ALDO2 controls the speaker amplifier; it is off with output discharge enabled.
- ALDO3 drives panel reset, not panel power.
- RTCLDO keeps the RTC running. Other unused outputs are disabled.

The sleep handoff holds shared SPI GPIO0/1/2 high to avoid current through
three external 10 kohm pull-ups. The PWR button sense circuit has another
resistor load that cannot be removed through a documented firmware control.
The [power audit](power-audit.md) records the circuit trace and device trials.

## Battery

The installed MakerHawk 1000 mAh pack uses a 4200 mV / 500 mA / 50 mA
termination profile in `src/board/battery.rs`, based on the supplied retailer
specification. No original cell-manufacturer datasheet has been found.
The connector names MX1.25/GH1.25 do not by themselves establish compatibility;
the schematic labels pin 1 positive and pin 2 ground.

Startup disables charging while applying and verifying the profile, then
restores its previous enable state. A failure requests charging off; a failed
bus cannot guarantee hardware state. Normal wake does not reconfigure charging.
Precharge, timers and protection selection remain unchanged. The two-wire pack
does not provide cell-temperature measurement; existing REG50=0x12 configures
TS as a general input. Its unused ADC conversion is disabled.

The ring uses the PMIC's uncalibrated fuel-gauge estimate. Charger settings are
not gauge calibration, and uneven percentage drops do not prove changing power
draw. The PMIC driver now exposes only the percentage needed by the UI, not a
telemetry console. Historical readbacks remain in the power audit.

## Recovery

Keep the enclosure clear of BOOT and PWR. BOOT held during reset enters the ROM
downloader, where USB can enumerate without the clock application running.
A brief PWR press may be needed to start on battery.

A PMIC state can survive an ESP32 USB reset. The tested recovery sequence was:
unplug USB, hold PWR for 10–12 seconds, release for 5 seconds, hold PWR for
about 2 seconds, then reconnect USB. This recovered the shared bus after the
audio-rail experiment without opening the enclosure. Losing all power can
invalidate the RTC; wake, press KEY and run `cargo xtask set-time` afterward.
