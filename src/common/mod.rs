use crate::utils::OnceCell;
use alloc::vec::Vec;

pub mod addr;
pub mod config;
pub mod console;
pub mod device;
pub mod frame_allocator;
pub mod pagetable;
pub mod tlb;
pub mod tls;

static CPU_ID: usize = 0;

pub(crate) static CPU_COUNT: OnceCell<usize> = OnceCell::new();

pub(crate) static MEMORY_AREAS: OnceCell<Vec<(usize, usize)>> = OnceCell::new();

pub(crate) static DEVICE_TREE_BLOB: OnceCell<Vec<u8>> = OnceCell::new();

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
