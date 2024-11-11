#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_interface;

mod hermit;
mod plugin;

pub use plugin::HermitPlugin;
