use crate::common::{addr::VirtAddr, tlb::{TLBOperation, TLB}};

impl TLBOperation for TLB {

    fn flush_vaddr(vaddr: VirtAddr) {
        unsafe {
            core::arch::riscv64::sfence_vma_vaddr(vaddr.0);
        }
    }

    #[inline]
    fn flush_all() {
        unsafe {
            core::arch::riscv64::sfence_vma_all();
        }
    }
}

pub fn tlb_init() {
   todo!();
}
