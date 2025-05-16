use crate::arch::config::mm::PAGE_SIZE_BITS;
use crate::frame_allocator::{FrameTracker, frame_alloc};
use crate::{
    arch::config::mm::{PAGE_SIZE, PAGE_TABLE_LEVELS, PPN_MASK, PPN_OFFSET_IN_PTE},
    bit,
    {
        addr::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum},
        pagetable::{PTEFlags, PTOps, PageTableEntry},
    },
};
use alloc::vec::Vec;
use riscv::register::satp::{self, Satp};

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
    type ArchFlags = Riscv64PTEFlags;

    const PAGE_SIZE: usize = PAGE_SIZE;
    const PAGE_SIZE_BITS: usize = PAGE_SIZE_BITS;
    const PAGE_TABLE_LEVELS: usize = PAGE_TABLE_LEVELS;
    fn get_pte_array(ppn: PhysPageNum) -> &'static mut [PageTableEntry] {
        const PTES_PER_PAGE: usize = PAGE_SIZE / core::mem::size_of::<PageTableEntry>();
        let va = Self::ppn_to_pa(ppn).to_vaddr().0;
        unsafe { core::slice::from_raw_parts_mut(va as *mut PageTableEntry, PTES_PER_PAGE) }
    }

    fn va_to_vpn(va: VirtAddr) -> VirtPageNum {
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

    fn pte_is_valid(pte: &PageTableEntry) -> bool {
        Self::pte_to_arch_flags(pte).contains(Riscv64PTEFlags::V)
    }
    fn pte_to_ppn(pte: &PageTableEntry) -> PhysPageNum {
        ((pte.bits >> PPN_OFFSET_IN_PTE) & PPN_MASK).into()
    }

    fn pte_to_arch_flags(pte: &PageTableEntry) -> Self::ArchFlags {
        Riscv64PTEFlags::from_bits(pte.bits & ((1 << PPN_OFFSET_IN_PTE) - 1)).unwrap()
    }

    fn pte_new_leaf(ppn: PhysPageNum, flags: PTEFlags) -> PageTableEntry {
        let arch_flags: Riscv64PTEFlags = flags.into();
        // Combine PPN shifted with arch flags
        PageTableEntry {
            bits: (ppn.0 << PPN_OFFSET_IN_PTE) | arch_flags.bits(),
        }
    }
    fn pte_new_intermediate(ppn: PhysPageNum) -> PageTableEntry {
        // 中间页表项标志位：V=1（有效），其他权限位保持 0
        let flags = Riscv64PTEFlags::V;

        PageTableEntry {
            // PPN 左移 10 位后与标志位组合
            bits: (ppn.0 <<PPN_OFFSET_IN_PTE) | flags.bits(),
        }
    }

    fn switch_page_table(page_table_token: usize) {
        unsafe {
            satp::write(Satp::from_bits(page_table_token));
            core::arch::riscv64::sfence_vma_all();
        }
    }
}
