//! Build, flash and set the clock. `.cargo/config.toml` names the commands:
//! `cargo firmware`, `cargo flash` and `cargo set-time`.

mod image;
mod port;
mod set_time;

use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::Duration;
use std::{env, fs};

use espflash::connection::{Connection, ResetAfterOperation, ResetBeforeOperation};
use espflash::flasher::Flasher;
use espflash::target::{Chip, DefaultProgressCallback};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const USAGE: &str = "\
Usage:
  cargo firmware                Compile the firmware and create target/clock.bin
  cargo flash [--port PORT]     Build, write the image to the clock, then set its time
  cargo set-time [--port PORT]  Copy this computer's UTC time to the clock

PORT selects a serial port when several Espressif devices are connected.";

const TARGET: &str = "riscv32imac-unknown-none-elf";
/// Retained LP RAM, less the stack reserve that `link.ld` enforces
const RETAINED_LIMIT: u32 = 16 * 1024 - 4096;

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let arguments: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let outcome = match arguments[..] {
        ["build"] => build().map(drop),
        ["flash"] => flash(None),
        ["flash", "--port", port] => flash(Some(port)),
        ["set-time"] => set_time(None),
        ["set-time", "--port", port] => set_time(Some(port)),
        _ => Err(USAGE.into()),
    };
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

/// Compile the firmware and convert it to the image that ROM loads from flash
fn build() -> Result<Vec<u8>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let target = env::var_os("CARGO_TARGET_DIR").map_or(root.join("target"), PathBuf::from);
    let map = format!("link-arg=-Map={}", target.join("clock.map").display());
    let cargo = env::var_os("CARGO").unwrap_or("cargo".into());
    let status = Command::new(cargo)
        .current_dir(root)
        .args(["rustc", "--release", "--package", "bedroom-clock", "--target", TARGET])
        .args(["--", "-C", &map])
        .status()?;
    if !status.success() {
        return Err("Firmware build failed.".into());
    }

    let elf = fs::read(target.join(TARGET).join("release/clock"))?;
    let image = image::from_elf(&elf)?;
    fs::write(target.join("clock.elf"), &elf)?;
    fs::write(target.join("clock.bin"), &image)?;

    let sizes = image::sizes(&elf)?;
    println!("{:>8}{:>8}{:>8}{:>8}{:>8}", ".init", ".text", ".rodata", ".data", ".bss");
    println!("{:8}{:8}{:8}{:8}{:8}", sizes.init, sizes.text, sizes.rodata, sizes.data, sizes.bss);
    println!("Retained memory: {} of {RETAINED_LIMIT} bytes", sizes.retained);
    let image_path = target.join("clock.bin");
    let shown = image_path.strip_prefix(root).unwrap_or(&image_path);
    println!("Created firmware image: {}", shown.display());
    Ok(image)
}

fn flash(requested: Option<&str>) -> Result<()> {
    let image = build()?;
    let port = port::select(requested, &port::connected()?)?;
    let serial = serialport::new(&port.name, 115_200).open_native()?;
    let connection = Connection::new(
        serial,
        port.usb,
        ResetAfterOperation::HardReset,
        ResetBeforeOperation::DefaultReset,
        115_200,
    );
    // ROM loader only, with read-back verification
    let mut flasher = Flasher::connect(connection, false, true, false, Some(Chip::Esp32c6), None)?;
    flasher.write_bin_to_flash(0, &image, &mut DefaultProgressCallback)?;
    drop(flasher);
    println!("Flashed. Setting time; tap and press KEY if the clock sleeps.");
    set_time(Some(&port.name))
}

fn set_time(requested: Option<&str>) -> Result<()> {
    let port = port::select(requested, &port::connected()?)?;
    let open = || {
        serialport::new(&port.name, 115_200)
            .timeout(Duration::from_millis(200))
            .open()
            .and_then(|link| link.clear(serialport::ClearBuffer::Input).map(|()| link))
            .map_err(std::io::Error::from)
    };
    set_time::set_time(open, Duration::from_secs(30))?;
    println!("Time set. Press KEY to return to normal sleep.");
    Ok(())
}
