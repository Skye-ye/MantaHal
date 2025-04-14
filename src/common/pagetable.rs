use super::addr::PhysPageNum;
use crate::bit;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct CommonPTEFlags: usize {
        const R = bit!(0);    // Readable
        const W = bit!(1);    // Writable
        const X = bit!(2);    // Executable
        const U = bit!(3);    // User mode access
        const D = bit!(4);    // Dirty
        const A = bit!(5);    // Accessed
        const G = bit!(6);    // Global
    }
}

pub trait PTE {
    type FlagsType;

    /// create a new page table entry
    fn new(ppn: PhysPageNum, flags: Self::FlagsType) -> Self;
    /// create a empty page table entry
    fn empty() -> Self;
    /// get the physical page number of the page table entry
    fn ppn(&self) -> PhysPageNum;
    /// get the flags of the page table entry
    fn flags(&self) -> Self::FlagsType;
    /// check if the page table entry is valid
    fn valid(&self) -> bool;
    /// check if the page table entry is dirty
    fn dirty(&self) -> bool;
    /// check if the page table entry is readable
    fn readable(&self) -> bool;
    /// check if the page table entry is writable
    fn writable(&self) -> bool;
    /// check if the page table entry is executable
    fn executable(&self) -> bool;
}
