pub const VIRT_ADDR_START: usize = 0x9000_0000_0000_0000;
pub const KERNEL_STACK_SIZE: usize = 64 * 1024;

pub const PAGE_SIZE: usize = 1 << PAGE_SIZE_BITS;
pub const PAGE_MASK: usize = PAGE_SIZE - 1;
pub const PAGE_SIZE_BITS: usize = 12;

pub const PTE_SIZE: usize = 8;
pub const PTES_PER_PAGE: usize = PAGE_SIZE / PTE_SIZE;
