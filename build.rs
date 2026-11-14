//! Link the firmware with the board's memory layout.

fn main() {
    // Host builds exist only to run the tests
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("none") {
        return;
    }
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/src/boot/link.ld");
    println!("cargo::rerun-if-changed={script}");
    println!("cargo::rustc-link-arg-bins=-T{script}");
}
