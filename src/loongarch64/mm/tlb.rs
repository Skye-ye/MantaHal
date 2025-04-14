use crate::common::addr::VirtAddr;
pub struct TLB;

/// TLB operations
impl TLB {
    /// flush the TLB entry by VirtualAddress
    #[inline]
    pub fn flush_vaddr(vaddr: VirtAddr) {
        unsafe {
            core::arch::asm!("dbar 0; invtlb 0x05, $r0, {reg}", reg = in(reg) vaddr.0);
        }
    }

    /// flush all tlb entry
    #[inline]
    pub fn flush_all() {
        unsafe {
            core::arch::asm!("dbar 0; invtlb 0x00, $r0, $r0");
        }
    }
}
