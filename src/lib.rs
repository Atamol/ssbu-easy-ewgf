#![feature(proc_macro_hygiene)]
#![allow(unused_macros)]

mod hid;
mod memory;
mod state;
mod easy_ewgf;

#[skyline::main(name = "ssbu_easy_ewgf")]
pub fn main() {
    easy_ewgf::install();
}

