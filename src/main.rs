#![feature(ascii_char)]
#![feature(slice_take)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod helpers;
mod parse;

use app::GraphErBrain;

fn main() -> Result<(), eframe::Error> {
    GraphErBrain::start()
}
