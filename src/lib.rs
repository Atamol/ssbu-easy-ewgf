#![feature(concat_idents, proc_macro_hygiene)]
#![allow(unused_macros)]
mod easy_ewgf;
#[skyline::main(name = "ssbu_easy_ewgf")]
pub fn main() {
    easy_ewgf::install();
}

