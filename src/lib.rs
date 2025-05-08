#![no_std]
#![no_main]
#![feature(naked_functions)]
#![allow(macro_expanded_macro_exports_accessed_by_absolute_paths)]
#![feature(stmt_expr_attributes)]
extern crate alloc;

mod common;
mod utils;

define_arch_mods_and_api![("riscv64", riscv64), ("loongarch64", loongarch64),];

pub trait HAL {
    /// initialize the hardware
    fn init();
    /// shutdown the hardware
    fn shutdown();
}
