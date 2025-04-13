pub const VIRT_ADDR_START: usize = 0x9000_0000_0000_0000;
pub const KERNEL_STACK_SIZE: usize = 64 * 1024;

pub const PAGE_SIZE: usize = 1 << PAGE_SIZE_BITS;
pub const PAGE_MASK: usize = PAGE_SIZE - 1;
pub const PAGE_SIZE_BITS: usize = 12;

pub const PTE_SIZE: usize = 8;
pub const PTES_PER_PAGE: usize = PAGE_SIZE / PTE_SIZE;

pub const PA_LEN: usize = 48;
pub const PA_MASK: usize = (1 << PA_LEN) - 1;
pub const PPN_LEN: usize = PA_LEN - PAGE_SIZE_BITS;
pub const PPN_MASK: usize = (1 << PPN_LEN) - 1;

pub const VA_LEN: usize = 48;
pub const VA_MASK: usize = (1 << VA_LEN) - 1;
pub const VPN_LEN: usize = VA_LEN - PAGE_SIZE_BITS;
pub const VPN_MASK: usize = (1 << VPN_LEN) - 1;

pub const PPN_OFFSET_IN_PTE: usize = 12;
