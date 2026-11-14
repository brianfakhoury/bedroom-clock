//! Host tests, compiled into the firmware crate by `src/main.rs` so they can
//! reach every layer. Run with `cargo test`.

mod app;
mod board;
mod drivers;
mod fakes;
