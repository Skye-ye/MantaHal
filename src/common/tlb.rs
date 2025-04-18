use crate::common::addr::VirtAddr;
pub struct TLB;

pub trait TLBOperation {
    /// flush the TLB entry by VirtualAddress
    fn flush_vaddr(vaddr: VirtAddr);
    /// flush all tlb entry
    fn flush_all();
}
