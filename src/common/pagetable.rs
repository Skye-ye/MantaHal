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

    fn new(ppn: PhysPageNum, flags: Self::FlagsType) -> Self;
    fn empty() -> Self;
    fn ppn(&self) -> PhysPageNum;
    fn flags(&self) -> Self::FlagsType;
    fn valid(&self) -> bool;
    fn dirty(&self) -> bool;
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn executable(&self) -> bool;
}
