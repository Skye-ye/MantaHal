use crate::common::addr::VirtAddr;
use crate::common::tlb::{TLBOperation, Tlb};
use crate::loongarch64::config::mm::PAGE_SIZE_BITS;
use loongArch64::register::{stlbps, tlbidx, tlbrehi, tlbrentry};

/// TLB operations
impl TLBOperation for Tlb {
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

pub fn tlb_init() {
    Tlb::flush_all();
    tlbidx::set_ps(PAGE_SIZE_BITS);
    stlbps::set_ps(PAGE_SIZE_BITS);
    tlbrehi::set_ps(PAGE_SIZE_BITS);
}

#[inline]
pub fn set_tlb_refill(tlbrentry: usize) {
    tlbrentry::set_tlbrentry(tlbrentry & 0xFFFF_FFFF_FFFF);
}
