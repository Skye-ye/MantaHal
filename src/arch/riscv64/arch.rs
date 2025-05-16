use crate::CPU_ID;
use crate::{
    arch::config::mm::VIRT_ADDR_START, DEVICE_TREE_BLOB, DTB_PTR, MEMORY_AREAS
};
use core::slice;
use alloc::vec;
use alloc::vec::Vec;
use fdt::Fdt;

pub fn arch_init() {
    let mut buffer = Vec::new();
    if let Ok(fdt) = unsafe { Fdt::from_ptr(*DTB_PTR as *const u8) } {
        unsafe {
            buffer.extend_from_slice(slice::from_raw_parts(
                *DTB_PTR as *const u8,
                fdt.total_size(),
            ));
        }
    }
    DEVICE_TREE_BLOB.init(buffer);
    let mut mem_area = Vec::new();
    if let Ok(fdt) = Fdt::new(&DEVICE_TREE_BLOB) {
        log::info!("There has {} CPU(s)", fdt.cpus().count());
        fdt.memory().regions().for_each(|x| {
            log::info!(
                "memory region {:#X} - {:#X}",
                x.starting_address as usize,
                x.starting_address as usize + x.size.unwrap()
            );
            mem_area.push((
                x.starting_address as usize | VIRT_ADDR_START,
                x.size.unwrap_or(0),
            ));
        });
    } else {
        mem_area.push((0x8000_0000 | VIRT_ADDR_START, 0x1000_0000));
    }
    MEMORY_AREAS.init(mem_area);
}

#[inline]
pub fn hart_id() -> usize {
    CPU_ID.read_current()
    
}
