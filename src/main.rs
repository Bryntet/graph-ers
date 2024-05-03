#![feature(ascii_char)]
#![feature(slice_take)]
#![feature(let_chains)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

pub mod app;
pub mod helpers;
pub mod parse;

pub use app::GraphErBrain;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), eframe::Error> {
    GraphErBrain::start()
}

#[cfg(target_arch = "wasm32")]
pub fn main() {
    GraphErBrain::start()
}
