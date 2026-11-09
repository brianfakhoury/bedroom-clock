"""Setup must reject corrupt downloads and recover incomplete installations."""

import hashlib
import io
import tarfile
from pathlib import Path
from unittest.mock import patch

import pytest

from tools import setup


def test_corrupt_archive_removed_without_replacing_installation(tmp_path):
    destination = tmp_path / "riscv32-esp-elf"
    destination.mkdir()
    (destination / "old").touch()
    archive = tmp_path / f"riscv32-esp-elf-{setup.VERSION}-test.tar.xz"
    archive.write_bytes(b"corrupt")
    with pytest.raises(ValueError):
        setup.install(tmp_path, "test", "0" * 64)
    assert not archive.exists()
    assert (destination / "old").exists()


def test_incomplete_install_replaced_and_repeat_skipped(tmp_path):
    destination = tmp_path / "riscv32-esp-elf"
    destination.mkdir()
    (destination / "partial").touch()
    archive = tmp_path / f"riscv32-esp-elf-{setup.VERSION}-test.tar.xz"
    with tarfile.open(archive, "w:xz") as bundle:
        item = tarfile.TarInfo("riscv32-esp-elf/complete")
        item.size = 2
        bundle.addfile(item, io.BytesIO(b"ok"))
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    with patch.object(setup, "compiler_works", return_value=True):
        setup.install(tmp_path, "test", digest)
        assert not (destination / "partial").exists()
        assert (destination / "complete").exists()
        unexpected = AssertionError("unexpected extraction")
        with patch.object(setup.tarfile, "open", side_effect=unexpected):
            setup.install(tmp_path, "test", digest)


def test_failed_download_leaves_no_archive(tmp_path):
    with (
        patch.object(setup, "urlopen", return_value=io.BytesIO(b"corrupt")),
        pytest.raises(ValueError, match="checksum mismatch"),
    ):
        setup.install(tmp_path, "test", "0" * 64)
    assert list(tmp_path.iterdir()) == []


def test_wrong_compiler_version_is_rejected():
    with (
        patch.object(setup.os, "access", return_value=True),
        patch.object(setup.subprocess, "check_output", return_value="gcc 14.2.0"),
    ):
        assert not setup.compiler_works(Path("unused"))
