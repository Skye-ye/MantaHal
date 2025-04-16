use super::addr::{PhysPageNum, VirtPageNum};
use super::frame_allocator::{FrameTracker, frame_alloc};
use crate::bit;
use alloc::vec;
use alloc::vec::Vec;
use core::marker::PhantomData;

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

pub trait PTOps {
    /// create a new page table entry
    fn new(ppn: PhysPageNum, flags: PTEFlags) -> PageTableEntry;
    /// get the physical page number of the page table entry
    fn ppn(pte: &PageTableEntry) -> PhysPageNum;
    /// get the flags of the page table entry
    fn flags(pte: &PageTableEntry) -> PTEFlags;
    /// check if the page table entry is valid
    fn valid(pte: &PageTableEntry) -> bool {
        Self::flags(pte).contains(PTEFlags::V)
    }
    /// check if the page table entry is dirty
    fn dirty(pte: &PageTableEntry) -> bool {
        Self::flags(pte).contains(PTEFlags::D)
    }
    /// check if the page table entry is readable
    fn readable(pte: &PageTableEntry) -> bool {
        Self::flags(pte).contains(PTEFlags::R)
    }
    /// check if the page table entry is writable
    fn writable(pte: &PageTableEntry) -> bool {
        Self::flags(pte).contains(PTEFlags::W)
    }
    /// check if the page table entry is executable
    fn executable(pte: &PageTableEntry) -> bool {
        Self::flags(pte).contains(PTEFlags::X)
    }
    /// find the page table entry without creating it
    fn find_pte<'a>(
        root_ppn: PhysPageNum,
        frames: &'a [FrameTracker],
        vpn: VirtPageNum,
    ) -> Option<&'a PageTableEntry>;
    /// find the page table entry, if not exist, create it
    fn find_pte_create<'a>(
        root_ppn: PhysPageNum,
        frames: &'a mut Vec<FrameTracker>,
        vpn: VirtPageNum,
    ) -> Option<&'a mut PageTableEntry>;
}

pub struct PageTable<T> {
    pub(crate) root_ppn: PhysPageNum,
    pub(crate) frames: Vec<FrameTracker>,
    phantom: PhantomData<T>,
}

impl<T: PTOps> PageTable<T> {
    pub fn new(_arch_impl: T) -> Self {
        let frame = frame_alloc().unwrap();
        PageTable {
            root_ppn: frame.ppn,
            frames: vec![frame],
            phantom: PhantomData,
        }
    }

    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = T::find_pte_create(self.root_ppn, &mut self.frames, vpn).unwrap();
        assert!(!T::valid(pte), "vpn {:?} is mapped before mapping", vpn);
        *pte = T::new(ppn, flags | PTEFlags::V);
    }
}
