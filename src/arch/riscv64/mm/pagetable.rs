use alloc::vec::Vec;
use riscv::register::satp::{self, Satp};
use crate::common::frame_allocator::{FrameTracker, frame_alloc};
use crate::{
    arch::config::mm::{PAGE_SIZE, PPN_MASK, PPN_OFFSET_IN_PTE,PAGE_TABLE_LEVEL_NUM},
    bit,
    common::{
        addr::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum},
        pagetable::{PTEFlags, PTOps, PageTableEntry},
    },
};

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    /// Common flags for page table entries
    pub struct Riscv64PTEFlags: usize {
        const V = bit!(0);    // Valid
        const R = bit!(1);    // Readable
        const W = bit!(2);    // Writable
        const X = bit!(3);    // Executable
        const U = bit!(4);    // User mode access
        const G = bit!(5);    // Global
        const A = bit!(6);    // Accessed
        const D = bit!(7);    // Dirty
        const COW = bit!(8);  // Copy On Write
    }
}

impl From<Riscv64PTEFlags> for PTEFlags {
    fn from(value: Riscv64PTEFlags) -> Self {
        let mut flags = PTEFlags::V;

        // Writable flag
        if value.contains(Riscv64PTEFlags::W) {
            flags |= PTEFlags::W;
        }

        // Executable flag
        if value.contains(Riscv64PTEFlags::X) {
            flags |= PTEFlags::X;
        }

        // Readable flag
        if value.contains(Riscv64PTEFlags::R) {
            flags |= PTEFlags::R;
        }

        // User mode access
        if value.contains(Riscv64PTEFlags::U) {
            flags |= PTEFlags::U;
        }

        // Dirty bit
        if value.contains(Riscv64PTEFlags::D) {
            flags |= PTEFlags::D;
        }

        // Global page
        if value.contains(Riscv64PTEFlags::G) {
            flags |= PTEFlags::G;
        }

        flags
    }
}

impl From<PTEFlags> for Riscv64PTEFlags {
    fn from(val: PTEFlags) -> Self {
        let mut flags = Riscv64PTEFlags::empty();

        //Readable flag
        if val.contains(PTEFlags::R) {
            flags |= Riscv64PTEFlags::R;
        }

        // Executable flag
        if val.contains(PTEFlags::X) {
            flags |= Riscv64PTEFlags::X;
        }

        // Writable flag
        if val.contains(PTEFlags::W) {
            flags |= Riscv64PTEFlags::W;
        }

        // Dirty bit
        if val.contains(PTEFlags::D) {
            flags |= Riscv64PTEFlags::D;
        }

        // User privilege level
        if val.contains(PTEFlags::U) {
            flags |= Riscv64PTEFlags::U;
        }

        // Global page
        if val.contains(PTEFlags::G) {
            flags |= Riscv64PTEFlags::G;
        }

        flags
    }
}

pub struct Riscv64PTImpl;

impl PTOps for Riscv64PTEFlags {
    fn new(ppn: PhysPageNum, flags: PTEFlags) -> PageTableEntry {
        let arch_flags: Riscv64PTEFlags = flags.into();
        PageTableEntry {
            bits: ppn.0 << PPN_OFFSET_IN_PTE | arch_flags.bits(),
        }
    }

    fn ppn(pte: &PageTableEntry) -> PhysPageNum {
        ((pte.bits >> PPN_OFFSET_IN_PTE) & PPN_MASK).into()
    }

    fn flags(pte: &PageTableEntry) -> PTEFlags {
        Riscv64PTEFlags::from_bits(pte.bits as usize)
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
        let vaddr: VirtAddr = ppn.to_paddr().to_vaddr();
        unsafe { core::slice::from_raw_parts_mut(vaddr.bits() as *mut u8, PAGE_SIZE) }
    }

    fn find_pte<'a>(root_ppn: PhysPageNum, vpn: VirtPageNum) -> Option<&'a mut PageTableEntry> {
        let mut ppn = root_ppn;
        let idxs = vpn.indices();
        for (i, idx) in idxs.into_iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[idx];
            if !Self::valid(pte) {
                return None;
            }
            if i == PAGE_TABLE_LEVEL_NUM - 1 {
                return Some(pte);
            }
            ppn = Self::ppn(pte);
        }
        return None;
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
            if i == PAGE_TABLE_LEVEL_NUM - 1 {
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

    fn switch_page_table(page_table_token: usize) {
        unsafe {
            satp::write(Satp::from_bits(page_table_token));
            core::arch::riscv64::sfence_vma_all();
        }
    }
}
