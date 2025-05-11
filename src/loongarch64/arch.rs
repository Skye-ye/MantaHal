use crate::{
    arch::config::mm::VIRT_ADDR_START,
    common::{DEVICE_TREE_BLOB, MEMORY_AREAS},
};

use alloc::vec;
use alloc::vec::Vec;

pub fn arch_init() {
    DEVICE_TREE_BLOB.init(Vec::new());
    MEMORY_AREAS.init(vec![(VIRT_ADDR_START | 0x9000_0000, 0x2000_0000)]);
}

#[inline]
pub fn hart_id() -> usize {
    loongArch64::register::cpuid::read().core_id()
}
