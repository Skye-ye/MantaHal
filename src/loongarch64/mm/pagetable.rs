use crate::bit;
use crate::common::addr::{PhysAddr, PhysPageNum};
use crate::common::pagetable::PageTableEntry;
use crate::common::pagetable::{CommonPTEFlags, PTE};
use crate::config::mm::{PPN_MASK, PPN_OFFSET_IN_PTE};

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
        /// Designates a global mapping.
        const G = bit!(6);
        /// Whether the page is huge page.
        const H = bit!(6);
        /// Page is existing.
        const P = bit!(7);
        /// Page is writeable.
        const W = bit!(8);
        /// Is a Global Page if using huge page(GH bit).
        const GH = bit!(12);
        /// Page is not readable.
        const NR = bit!(61);
        /// Page is not executable.
        const NX = bit!(62);
        /// Whether the privilege Level is restricted. When RPLV is 0, the PTE
        /// can be accessed by any program with privilege Level highter than PLV.
        const RPLV = bit!(63);
    }
}

impl PTE for PageTableEntry {
    type FlagsType = PTEFlags;

    /// Create a new page table entry.
    fn new(ppn: PhysPageNum, flags: Self::FlagsType) -> Self {
        PageTableEntry {
            bits: ppn.0 << PPN_OFFSET_IN_PTE | flags.bits(),
        }
    }

    /// Create a new empty page table entry.
    fn empty() -> Self {
        PageTableEntry { bits: 0 }
    }

    /// Get the page number of the page table entry.
    fn ppn(&self) -> PhysPageNum {
        ((self.bits >> PPN_OFFSET_IN_PTE) & PPN_MASK).into()
    }

    /// Get the flags of the page table entry.
    fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as usize).unwrap()
    }

    /// Check if the page table entry is valid.
    fn valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }

    /// Check if the page table entry is dirty.
    fn dirty(&self) -> bool {
        (self.flags() & PTEFlags::D) != PTEFlags::empty()
    }

    /// Check if the page table entry is readable. (readable when NR is 0)
    fn readable(&self) -> bool {
        (self.flags() & PTEFlags::NR) == PTEFlags::empty()
    }

    /// Check if the page table entry is writable.
    fn writable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }

    /// Check if the page table entry is executable. (executable when NX is 0)
    fn executable(&self) -> bool {
        (self.flags() & PTEFlags::NX) == PTEFlags::empty()
    }
}
