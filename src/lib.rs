#![no_std]
#![no_main]
#![feature(naked_functions)]

extern crate bitflags;

mod addr;
mod utils;

arch_modules![("riscv64", riscv64), ("loongarch64", loongarch64)];
