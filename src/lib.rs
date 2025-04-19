#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(alloc_error_handler)]
#![allow(macro_expanded_macro_exports_accessed_by_absolute_paths)]
extern crate alloc;
extern crate bitflags;

mod common;
mod utils;

arch_modules![("riscv64", riscv64), ("loongarch64", loongarch64)];

pub mod arch {
    #[cfg(target_arch = "loongarch64")]
    pub use crate::loongarch64::*;
    #[cfg(target_arch = "riscv64")]
    pub use crate::riscv64::*;
}

pub trait HAL {
    /// initialize the hardware
    fn init();
    /// shutdown the hardware
    fn shutdown();
}
