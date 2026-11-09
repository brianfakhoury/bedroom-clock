"""Flash a local image or copy the computer's UTC time to the clock."""

import argparse
import subprocess
import sys
import time
from datetime import UTC, datetime
from pathlib import Path

import serial
from serial.tools.list_ports import comports


def select_port(requested):
    if requested:
        return requested
    ports = [port.device for port in comports() if port.vid == 0x303A]
    if not ports:
        raise ValueError("Connect USB, tap the screen, and press KEY to keep it awake.")
    if len(ports) != 1:
        raise ValueError("Multiple devices; specify PORT: " + ", ".join(ports))
    return ports[0]


def set_time(port, timeout=30):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            with serial.Serial(port, 115200, timeout=0.2, write_timeout=1) as link:
                link.reset_input_buffer()
                command = datetime.now(UTC).strftime("\nTIME %Y%m%d%H%M%S\n")
                link.write(command.encode("ascii"))
                reply = b""
                until = min(deadline, time.monotonic() + 1)
                while time.monotonic() < until:
                    reply = (reply + link.read(128))[-256:]
                    lines = reply.split(b"\n")[:-1]
                    if b"OK TIME" in lines:
                        return
                    if b"ERR TIME" in lines:
                        raise ValueError("The RTC rejected the time update.")
        except (OSError, serial.SerialException):
            time.sleep(0.2)
    raise ValueError("No time acknowledgement. Tap, press KEY, then run make set-time.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["flash", "set-time"])
    parser.add_argument("--port")
    parser.add_argument("--file", type=Path, default=Path("build/clock.bin"))
    args = parser.parse_args()
    try:
        port = select_port(args.port)
        if args.command == "flash":
            if not args.file.is_file():
                raise ValueError(f"Image not found: {args.file}")
            command = [sys.executable, "-m", "esptool", "--chip", "esp32c6"]
            command += ["--port", port, "--no-stub", "write_flash", "0", str(args.file)]
            subprocess.run(command, check=True)
            message = "Flashed. Setting time; tap and press KEY if the clock sleeps."
            print(message, flush=True)
        set_time(port)
        print("Time set. Press KEY to return to normal sleep.")
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"{error}\n")


if __name__ == "__main__":
    main()
