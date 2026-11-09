"""Time setting needs an acknowledgement from the RTC, not just an open port."""

import re
from types import SimpleNamespace
from unittest.mock import MagicMock, patch

import pytest

from tools import clock


def test_fragmented_acknowledgement():
    connection = MagicMock()
    connection.__enter__.return_value = connection
    connection.read.side_effect = [b"boot\n", b"OK TI", b"ME\n"]
    with patch.object(clock.serial, "Serial", return_value=connection):
        clock.set_time("clock")
    assert re.fullmatch(rb"\nTIME \d{14}\n", connection.write.call_args.args[0])


def test_failed_write_and_missing_reply_are_errors():
    connection = MagicMock()
    connection.__enter__.return_value = connection
    connection.read.return_value = b"ERR TIME\n"
    with (
        patch.object(clock.serial, "Serial", return_value=connection),
        pytest.raises(ValueError, match="RTC rejected"),
    ):
        clock.set_time("clock")
    with pytest.raises(ValueError, match="No time acknowledgement"):
        clock.set_time("clock", timeout=0)


def test_port_selection_requires_one_matching_device():
    ports = [
        SimpleNamespace(vid=0x1234, device="other"),
        SimpleNamespace(vid=0x303A, device="clock"),
    ]
    with patch.object(clock, "comports", return_value=ports):
        assert clock.select_port(None) == "clock"
        assert clock.select_port("explicit") == "explicit"
    for found in [ports[:1], ports + ports[1:]]:
        with (
            patch.object(clock, "comports", return_value=found),
            pytest.raises(ValueError),
        ):
            clock.select_port(None)
