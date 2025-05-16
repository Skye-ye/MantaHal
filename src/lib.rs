#![no_std]
#![no_main]
#![feature(naked_functions)]
#![allow(macro_expanded_macro_exports_accessed_by_absolute_paths)]
#![feature(stmt_expr_attributes)]
#![cfg_attr(target_arch = "riscv64", feature(riscv_ext_intrinsics))]
#![feature(used_with_arg)]

extern crate alloc;

mod addr;
mod arch;
mod config;
mod console;
mod device;
mod frame_allocator;
mod pagetable;
mod tlb;
mod utils;

use crate::utils::OnceCell;
use alloc::vec::Vec;
use fdt::Fdt;


//TODO：解决此处报错
#[unsafe(mantahal_macro::def_percpu)] 
pub(crate) static CPU_ID: usize = 0;


pub static CPU_COUNT: OnceCell<usize> = OnceCell::new();

pub static MEMORY_AREAS: OnceCell<Vec<(usize, usize)>> = OnceCell::new();

pub static DEVICE_TREE_BLOB: OnceCell<Vec<u8>> = OnceCell::new();

#[allow(dead_code)]
pub(crate) static DTB_PTR: OnceCell<usize> = OnceCell::new();

/// Returns a reference to the vector describing physical memory regions.
///
/// Each tuple in the vector represents `(start_address, end_address)` of a contiguous
/// physical memory area available for use.
///
/// # Panics
///
/// Panics if the memory areas have not been initialized (e.g., via `MEMORY_AREAS.init(...)`)
/// before this function is called. Ensure initialization occurs during early boot.
#[inline]
pub fn memory_areas() -> &'static Vec<(usize, usize)> {
    MEMORY_AREAS.get()
}

/// Returns a reference to the raw Device Tree Blob (DTB) binary data.
///
/// The DTB provides hardware configuration information from the bootloader.
///
/// # Panics
///
/// Panics if the DTB data has not been initialized (e.g., via `DEVICE_TREE_BLOB.init(...)`)
/// before this function is called. Ensure initialization occurs during early boot.
#[inline]
pub fn device_tree_blob() -> &'static Vec<u8> {
    DEVICE_TREE_BLOB.get()
}

/// Returns the total number of detected CPU cores in the system.
///
/// # Panics
///
/// Panics if the CPU count has not been initialized (e.g., via `CPU_COUNT.init(...)`)
/// before this function is called. Ensure initialization occurs during early boot.
#[inline]
pub fn cpu_count() -> usize {
    *CPU_COUNT
}


/// Get the fdt
pub fn get_fdt() -> Option<Fdt<'static>> {
    // Fdt::new(&DTB_BIN).ok()
    unsafe { Fdt::from_ptr(*DTB_PTR.get_unchecked() as *const u8).ok() }
}