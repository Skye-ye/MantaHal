use crate::arch::config::mm::{
    PAGE_SIZE, PAGE_TABLE_LEVELS, PPN_MASK, PPN_OFFSET_IN_PTE, PTE_INDEX_BITS, PTE_INDEX_MASK,
};
use crate::bit;
use crate::common::addr::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use crate::common::frame_allocator::{FrameTracker, frame_alloc};
use crate::common::pagetable::PTEFlags;
use crate::common::pagetable::PTOps;
use crate::common::pagetable::PageTableEntry;
use alloc::vec::Vec;

bitflags::bitflags! {
    /// Possible flags for a page table entry.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Loongarch64PTEFlags: usize {
        /// Page Valid
        const V = bit!(0);
        /// Dirty, The page has been written.
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
        /// can be accessed by any program with privilege Level higher than PLV.
        const RPLV = bit!(63);
    }
}

pub struct Loongarch64PTImpl;

impl PTOps for Loongarch64PTImpl {
    fn new(ppn: PhysPageNum, flags: PTEFlags) -> PageTableEntry {
        let arch_flags: Loongarch64PTEFlags = flags.into();
        PageTableEntry {
            bits: ppn.0 << PPN_OFFSET_IN_PTE | arch_flags.bits(),
        }
    }

    fn ppn(pte: &PageTableEntry) -> PhysPageNum {
        ((pte.bits >> PPN_OFFSET_IN_PTE) & PPN_MASK).into()
    }

    fn flags(pte: &PageTableEntry) -> PTEFlags {
        Loongarch64PTEFlags::from_bits(pte.bits as usize)
            .unwrap()
            .into()
    }

    fn floor(va: VirtAddr) -> VirtPageNum {
        va.floor()
    }

    fn ppn_to_pa(ppn: PhysPageNum) -> PhysAddr {
        ppn.into()
    }

    fn vpn_to_va(vpn: VirtPageNum) -> VirtAddr {
        vpn.into()
    }

    fn ppn_from_token(pgdl: usize) -> PhysPageNum {
        (pgdl >> 12).into()
    }

    fn token_from_ppn(ppn: PhysPageNum) -> usize {
        let ppn_usize: usize = ppn.into();
        ppn_usize << 12
    }

    fn get_bytes_array(ppn: PhysPageNum) -> &'static mut [u8] {
        let pa: PhysAddr = ppn.into();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut u8, PAGE_SIZE) }
    }

    fn find_pte<'a>(root_ppn: PhysPageNum, vpn: VirtPageNum) -> Option<&'a mut PageTableEntry> {
        let mut ppn = root_ppn;
        let idxs = vpn.indices();
        let mut res: Option<&'a mut PageTableEntry> = None;
        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*idx];
            if i == PAGE_TABLE_LEVELS - 1 {
                res = Some(pte);
                break;
            }
            if Self::valid(pte) {
                break;
            }
            ppn = Self::ppn(pte);
        }
        res
    }

    fn find_pte_create<'a>(
        root_ppn: PhysPageNum,
        frames: &'a mut Vec<FrameTracker>,
        vpn: VirtPageNum,
    ) -> Option<&'a mut PageTableEntry> {
        let idxs = vpn.indices();
        let mut ppn = root_ppn;
        let mut res: Option<&'a mut PageTableEntry> = None;
        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*idx];
            if i == PAGE_TABLE_LEVELS - 1 {
                res = Some(pte);
                break;
            }
            if !Self::valid(pte) {
                let frame = frame_alloc().unwrap();
                *pte = Self::new(frame.ppn, PTEFlags::V);
                frames.push(frame);
            }
            ppn = Self::ppn(pte);
        }
        res
    }
}

impl From<Loongarch64PTEFlags> for PTEFlags {
    fn from(value: Loongarch64PTEFlags) -> Self {
        let mut flags = PTEFlags::V;

        // Writable flag
        if value.contains(Loongarch64PTEFlags::W) {
            flags |= PTEFlags::W;
        }

        // Executable flag (using negative logic in Loongarch64PTEFlags)
        if !value.contains(Loongarch64PTEFlags::NX) {
            flags |= PTEFlags::X;
        }

        // Readable flag (using negative logic in Loongarch64PTEFlags)
        if !value.contains(Loongarch64PTEFlags::NR) {
            flags |= PTEFlags::R;
        }

        // User mode access
        if value.contains(Loongarch64PTEFlags::PLV) {
            flags |= PTEFlags::U;
        }

        // Dirty bit
        if value.contains(Loongarch64PTEFlags::D) {
            flags |= PTEFlags::D;
        }

        // Global page
        if value.contains(Loongarch64PTEFlags::G) {
            flags |= PTEFlags::G;
        }

        flags
    }
}

impl From<PTEFlags> for Loongarch64PTEFlags {
    fn from(val: PTEFlags) -> Self {
        let mut flags = Loongarch64PTEFlags::empty();

        // Set readable by default, unless not readable
        if !val.contains(PTEFlags::R) {
            flags |= Loongarch64PTEFlags::NR;
        }

        // Set executable by default, unless not executable
        if !val.contains(PTEFlags::X) {
            flags |= Loongarch64PTEFlags::NX;
        }

        // Writable flag
        if val.contains(PTEFlags::W) {
            flags |= Loongarch64PTEFlags::W;
        }

        // Dirty bit
        if val.contains(PTEFlags::D) {
            flags |= Loongarch64PTEFlags::D;
        }

        // User privilege level
        if val.contains(PTEFlags::U) {
            flags |= Loongarch64PTEFlags::PLV;
        }

        // Global page
        if val.contains(PTEFlags::G) {
            flags |= Loongarch64PTEFlags::G;
        }

        flags
    }
}

impl VirtPageNum {
    pub fn indices(&self) -> [usize; PAGE_TABLE_LEVELS] {
        let mut indices = [0; PAGE_TABLE_LEVELS];
        let mut vpn = self.0;
        for i in (0..PAGE_TABLE_LEVELS).rev() {
            indices[i] = vpn & PTE_INDEX_MASK;
            vpn >>= PTE_INDEX_BITS;
        }
        indices
    }
}
