#![windows_subsystem = "windows"]
pub mod error;
pub mod extractors;
pub mod gui;
#[macro_use]
pub mod utils;

fn main() {
    gui::run().expect("Failed to launch GUI");
}