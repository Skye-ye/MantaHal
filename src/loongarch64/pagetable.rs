use crate::addr::PhysPageNum;
use crate::bit;
use crate::loongarch64::config::mm::{PPN_MASK, PPN_OFFSET_IN_PTE};

bitflags::bitflags! {
    /// Possible flags for a page table entry.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct PTEFlags: usize {
        /// Page Valid
        const V = bit!(0);
        /// Dirty, The page has been writed.
        const D = bit!(1);
        /// Privilege level
        const PLV = bit!(2) | bit!(3);
        /// Memory access type
        const MAT = bit!(4) | bit!(5);
        /// Designates a global mapping OR Whether the page is huge page.
        const GH = bit!(6);
        /// Page is existing.
        const P = bit!(7);
        /// Page is writeable.
        const W = bit!(8);
        /// Is a Global Page if using huge page(GH bit).
        const G = bit!(12);
        /// Page is not readable.
        const NR = bit!(61);
        /// Page is not executable.
        const NX = bit!(62);
        /// Whether the privilege Level is restricted. When RPLV is 0, the PTE
        /// can be accessed by any program with privilege Level highter than PLV.
        const RPLV = bit!(63);
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize,
}

impl PageTableEntry {
    /// Create a new page table entry.
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PageTableEntry {
            bits: ppn.0 << PPN_OFFSET_IN_PTE | flags.bits(),
        }
    }

    /// Create a new empty page table entry.
    pub fn empty() -> Self {
        PageTableEntry { bits: 0 }
    }

    pub fn ppn(&self) -> PhysPageNum {
        ((self.bits >> PPN_OFFSET_IN_PTE) & PPN_MASK).into()
    }

    /// Get the flags of the page table entry.
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as usize).unwrap()
    }

    /// Check if the page table entry is valid.
    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
}
