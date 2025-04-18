use super::addr::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use super::frame_allocator::{FrameTracker, frame_alloc};
use crate::bit;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::marker::PhantomData;

#[derive(Copy, Clone)]
#[repr(C)]
/// A page table entry
pub struct PageTableEntry {
    pub bits: usize,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    /// Common flags for page table entries
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
    /// floor a virtual address to a virtual page number
    fn floor(va: VirtAddr) -> VirtPageNum;
    /// convert a physical page number to a physical address
    fn ppn_to_pa(ppn: PhysPageNum) -> PhysAddr;
    /// convert a virtual page number to a virtual address
    fn vpn_to_va(vpn: VirtPageNum) -> VirtAddr;
    /// create a ppn from a token (usually from user space)
    fn ppn_from_token(token: usize) -> PhysPageNum;
    /// get the bytes array of a physical page number
    fn get_bytes_array(ppn: PhysPageNum) -> &'static mut [u8];
    /// find the page table entry without creating it
    fn find_pte<'a>(root_ppn: PhysPageNum, vpn: VirtPageNum) -> Option<&'a mut PageTableEntry>;
    /// find the page table entry, if not exist, create it
    fn find_pte_create<'a>(
        root_ppn: PhysPageNum,
        frames: &'a mut Vec<FrameTracker>,
        vpn: VirtPageNum,
    ) -> Option<&'a mut PageTableEntry>;
}

/// A page table with a given architecture implementation
pub struct PageTable<T> {
    root_ppn: PhysPageNum,
    frames: Vec<FrameTracker>,
    phantom: PhantomData<T>,
}

impl PageTableEntry {
    /// create an empty page table entry
    pub fn empty() -> Self {
        PageTableEntry { bits: 0 }
    }
}

impl<T: PTOps> PageTable<T> {
    /// create a new page table with a given architecture implementation
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();
        PageTable {
            root_ppn: frame.ppn,
            frames: vec![frame],
            phantom: PhantomData,
        }
    }

    /// create a new page table from a token (usually from user space)
    pub fn from_token(token: usize) -> Self {
        Self {
            root_ppn: T::ppn_from_token(token),
            frames: Vec::new(),
            phantom: PhantomData,
        }
    }

    /// map a virtual page number to a physical page number with given flags
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = T::find_pte_create(self.root_ppn, &mut self.frames, vpn).unwrap();
        assert!(!T::valid(pte), "vpn {:?} is mapped before mapping", vpn);
        *pte = T::new(ppn, flags | PTEFlags::V);
    }

    /// unmap a virtual page number
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = T::find_pte(self.root_ppn, vpn).unwrap();
        assert!(T::valid(pte), "vpn {:?} is not mapped", vpn);
        *pte = PageTableEntry::empty();
    }

    /// translate a virtual page number to a page table entry
    pub fn translate_vpn(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        T::find_pte(self.root_ppn, vpn).map(|pte| *pte)
    }

    /// translate a virtual address to a physical address
    pub fn translate_va(&self, va: VirtAddr) -> Option<PhysAddr> {
        T::find_pte(self.root_ppn, T::floor(va)).map(|pte| {
            let aligned_pa: PhysAddr = T::ppn_to_pa(T::ppn(pte));
            let offset = va.page_offset();
            let aligned_pa_usize: usize = aligned_pa.into();
            (aligned_pa_usize + offset).into()
        })
    }

    /// translate a byte buffer from other address spaces into kernel space
    pub fn translate_byte_buffer(
        token: usize,
        ptr: *const u8,
        len: usize,
    ) -> Vec<&'static mut [u8]> {
        let page_table = Self::from_token(token);
        let mut start = ptr as usize;
        let end = start + len;
        let mut v = Vec::new();
        while start < end {
            let start_va = VirtAddr::from(start);
            let mut vpn = T::floor(start_va);
            let ppn = T::ppn(&page_table.translate_vpn(vpn).unwrap());
            vpn.step();
            let mut end_va: VirtAddr = T::vpn_to_va(vpn);
            end_va = end_va.min(VirtAddr::from(end));
            if end_va.page_offset() == 0 {
                v.push(&mut T::get_bytes_array(ppn)[start_va.page_offset()..]);
            } else {
                v.push(&mut T::get_bytes_array(ppn)[start_va.page_offset()..end_va.page_offset()]);
            }
            start = end_va.into();
        }
        v
    }

    /// Load a string from other address spaces into kernel space without an end `\0`.
    pub fn translated_str(token: usize, ptr: *const u8) -> String {
        let page_table = Self::from_token(token);
        let mut string = String::new();
        let mut va = ptr as usize;
        loop {
            let ch: u8 = *(page_table
                .translate_va(VirtAddr::from(va))
                .unwrap()
                .get_mut());
            if ch == 0 {
                break;
            }
            string.push(ch as char);
            va += 1;
        }
        string
    }

    /// Translate a pointer from other address spaces into kernel space
    pub fn translated_ref<U>(token: usize, ptr: *const U) -> &'static U {
        let page_table = Self::from_token(token);
        page_table
            .translate_va(VirtAddr::from(ptr as usize))
            .unwrap()
            .get_ref()
    }

    /// Translate a mutable pointer from other address spaces into kernel space
    pub fn translated_refmut<U>(token: usize, ptr: *mut U) -> &'static mut U {
        let page_table = Self::from_token(token);
        let va = ptr as usize;
        page_table
            .translate_va(VirtAddr::from(va))
            .unwrap()
            .get_mut()
    }
}
