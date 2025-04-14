use crate::bit;
use crate::common::addr::PhysPageNum;
use crate::common::pagetable::CommonPTEFlags;
use crate::common::pagetable::PTE;
use crate::common::pagetable::PageTableEntry;
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

    fn new(ppn: PhysPageNum, flags: Self::FlagsType) -> Self {
        PageTableEntry {
            bits: ppn.0 << PPN_OFFSET_IN_PTE | flags.bits(),
        }
    }

    fn empty() -> Self {
        PageTableEntry { bits: 0 }
    }

    fn ppn(&self) -> PhysPageNum {
        ((self.bits >> PPN_OFFSET_IN_PTE) & PPN_MASK).into()
    }

    fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as usize).unwrap()
    }

    fn valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }

    fn dirty(&self) -> bool {
        (self.flags() & PTEFlags::D) != PTEFlags::empty()
    }

    fn readable(&self) -> bool {
        (self.flags() & PTEFlags::NR) == PTEFlags::empty()
    }

    fn writable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }

    fn executable(&self) -> bool {
        (self.flags() & PTEFlags::NX) == PTEFlags::empty()
    }
}

impl From<CommonPTEFlags> for PTEFlags {
    fn from(value: CommonPTEFlags) -> Self {
        let mut flags = PTEFlags::V;

        // Writable flag
        if value.contains(CommonPTEFlags::W) {
            flags |= PTEFlags::W;
        }

        // Executable flag (using negative logic in PTEFlags)
        if !value.contains(CommonPTEFlags::X) {
            flags |= PTEFlags::NX;
        }

        // Readable flag (using negative logic in PTEFlags)
        if !value.contains(CommonPTEFlags::R) {
            flags |= PTEFlags::NR;
        }

        // User mode access
        if value.contains(CommonPTEFlags::U) {
            flags |= PTEFlags::PLV;
        }

        // Dirty bit
        if value.contains(CommonPTEFlags::D) {
            flags |= PTEFlags::D;
        }

        // Global page
        if value.contains(CommonPTEFlags::G) {
            flags |= PTEFlags::G;
        }

        flags
    }
}

impl From<PTEFlags> for CommonPTEFlags {
    fn from(val: PTEFlags) -> Self {
        let mut flags = CommonPTEFlags::empty();

        // Set readable by default, unless not readable
        if !val.contains(PTEFlags::NR) {
            flags |= CommonPTEFlags::R;
        }

        // Set executable by default, unless not executable
        if !val.contains(PTEFlags::NX) {
            flags |= CommonPTEFlags::X;
        }

        // Writable flag
        if val.contains(PTEFlags::W) {
            flags |= CommonPTEFlags::W;
        }

        // Dirty bit
        if val.contains(PTEFlags::D) {
            flags |= CommonPTEFlags::D;
        }

        // User privilege level
        if val.contains(PTEFlags::PLV) {
            flags |= CommonPTEFlags::U;
        }

        // Global page
        if val.contains(PTEFlags::G) {
            flags |= CommonPTEFlags::G;
        }

        flags
    }
}
