use super::addr::{PhysPageNum, VirtPageNum};
use super::frame_allocator::{FrameTracker, frame_alloc};
use crate::bit;
use alloc::vec;
use alloc::vec::Vec;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct PTEFlags: usize {
        const V = bit!(0);    // Valid
        const R = bit!(1);    // Readable
        const W = bit!(2);    // Writable
        const X = bit!(3);    // Executable
        const U = bit!(4);    // User mode access
        const D = bit!(5);    // Dirty
        const A = bit!(6);    // Accessed
        const G = bit!(7);    // Global
    }
}

pub trait PTEOps {
    /// create a new page table entry
    fn new(&self, ppn: PhysPageNum, flags: PTEFlags) -> PageTableEntry;
    /// get the physical page number of the page table entry
    fn ppn(&self, pte: &PageTableEntry) -> PhysPageNum;
    /// get the flags of the page table entry
    fn flags(&self, pte: &PageTableEntry) -> PTEFlags;
    /// check if the page table entry is valid
    fn valid(&self, pte: &PageTableEntry) -> bool {
        self.flags(pte).contains(PTEFlags::V)
    }
    /// check if the page table entry is dirty
    fn dirty(&self, pte: &PageTableEntry) -> bool {
        self.flags(pte).contains(PTEFlags::D)
    }
    /// check if the page table entry is readable
    fn readable(&self, pte: &PageTableEntry) -> bool {
        self.flags(pte).contains(PTEFlags::R)
    }
    /// check if the page table entry is writable
    fn writable(&self, pte: &PageTableEntry) -> bool {
        self.flags(pte).contains(PTEFlags::W)
    }
    /// check if the page table entry is executable
    fn executable(&self, pte: &PageTableEntry) -> bool {
        self.flags(pte).contains(PTEFlags::X)
    }
}

pub struct PageTable<T: PTOps = DefaultPTOps> {
    pub(crate) root_ppn: PhysPageNum,
    pub(crate) frames: Vec<FrameTracker>,
    pub(crate) arch_impl: T,
}

pub trait PTOps {
    /// find the page table entry without creating it
    fn find_pte(&self, vpn: VirtPageNum) -> Option<&PageTableEntry>;
    /// find the page table entry and create it if it doesn't exist
    fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry>;
}

pub struct DefaultPTOps;

impl PTOps for DefaultPTOps {
    fn find_pte(&self, _vpn: VirtPageNum) -> Option<&PageTableEntry> {
        // This implementation will be used when no architecture-specific one is provided
        panic!("No architecture-specific implementation available for PTOps::find_pte")
    }

    fn find_pte_create(&mut self, _vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        // This implementation will be used when no architecture-specific one is provided
        panic!("No architecture-specific implementation available for PTOps::find_pte_create")
    }
}

impl<T: PTEOps + PTOps> PageTable<T> {
    pub fn new(arch_impl: T) -> Self {
        let frame = frame_alloc().unwrap();
        PageTable {
            root_ppn: frame.ppn,
            frames: vec![frame],
            arch_impl,
        }
    }

    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.arch_impl.find_pte_create(vpn).unwrap();
        assert!(
            !self.arch_impl.valid(pte),
            "vpn {:?} is mapped before mapping",
            vpn
        );
        *pte = self.arch_impl.new(ppn, flags | PTEFlags::V);
    }
}
