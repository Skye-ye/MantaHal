use crate::common::addr::VirtAddr;
use crate::common::tlb::{TLB, TLBOperation};

/// TLB operations
impl TLBOperation for TLB {
    #[inline]
    fn flush_vaddr(vaddr: VirtAddr) {
        unsafe {
            core::arch::asm!("dbar 0; invtlb 0x05, $r0, {reg}", reg = in(reg) vaddr.0);
        }
    }

    #[inline]
    fn flush_all() {
        unsafe {
            core::arch::asm!("dbar 0; invtlb 0x00, $r0, $r0");
        }
    }
}
