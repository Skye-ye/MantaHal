use crate::bit;
use crate::common::addr::{PhysPageNum, VirtPageNum};
use crate::common::pagetable::PTEFlags;
use crate::common::pagetable::PageTable;
use crate::common::pagetable::PageTableEntry;
use crate::common::pagetable::{PTEOps, PTOps};
use crate::config::mm::{PAGE_TABLE_LEVELS, PPN_MASK, PPN_OFFSET_IN_PTE};

bitflags::bitflags! {
    /// Possible flags for a page table entry.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Loongarch64PTEFlags: usize {
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

pub struct Loongarch64PTImpl;

impl PTEOps for Loongarch64PTImpl {
    fn new(&self, ppn: PhysPageNum, flags: PTEFlags) -> PageTableEntry {
        let arch_flags: Loongarch64PTEFlags = flags.into();
        PageTableEntry {
            bits: ppn.0 << PPN_OFFSET_IN_PTE | arch_flags.bits(),
        }
    }

    fn ppn(&self, pte: &PageTableEntry) -> PhysPageNum {
        ((pte.bits >> PPN_OFFSET_IN_PTE) & PPN_MASK).into()
    }

    fn flags(&self, pte: &PageTableEntry) -> PTEFlags {
        Loongarch64PTEFlags::from_bits(pte.bits as usize)
            .unwrap()
            .into()
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

impl PTOps for Loongarch64PTImpl {
    fn find_pte(&self, _vpn: VirtPageNum) -> Option<&PageTableEntry> {
        // This method doesn't have direct access to PageTable's root_ppn
        // It will be called by PageTable methods that do have access
        unimplemented!("Use PageTable::find_pte instead");
    }

    fn find_pte_create(&mut self, _vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        // Similarly, this requires access to PageTable's frame allocation
        unimplemented!("Use PageTable::find_pte_create instead");
    }
}

impl PageTable<Loongarch64PTImpl> {
    // Method to find a page table entry using LoongArch64-specific implementation
    pub fn find_pte(&self, vpn: VirtPageNum) -> Option<&PageTableEntry> {
        let indices = vpn.indices();
        let mut ppn = self.root_ppn;

        // Level 3 (top level)
        let pte3 = &ppn.get_pte_array()[indices[0]];
        if !self.arch_impl.valid(pte3) {
            return None;
        }
        ppn = self.arch_impl.ppn(pte3);

        // Level 2
        let pte2 = &ppn.get_pte_array()[indices[1]];
        if !self.arch_impl.valid(pte2) {
            return None;
        }
        ppn = self.arch_impl.ppn(pte2);

        // Level 1 (lowest level)
        let pte1 = &ppn.get_pte_array()[indices[2]];
        if !self.arch_impl.valid(pte1) {
            return None;
        }

        Some(pte1)
    }

    // Method to find or create a page table entry using LoongArch64-specific implementation
    pub fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let indices = vpn.indices();
        let mut ppn = self.root_ppn;

        // Level 3 (top level)
        let pte3 = &mut ppn.get_pte_array_mut()[indices[0]];
        if !self.arch_impl.valid(pte3) {
            // Need to allocate a new page table
            let frame = self.alloc_frame().unwrap();
            let frame_ppn = frame.ppn;

            // Zero initialize the new page table
            let pa = frame_ppn.0 << 12;
            unsafe {
                core::ptr::write_bytes(pa as *mut u8, 0, PAGE_SIZE);
            }

            // Set the PTE to point to the new page table
            *pte3 = self.arch_impl.new(frame_ppn, PTEFlags::V);
        }
        ppn = self.arch_impl.ppn(pte3);

        // Level 2
        let pte2 = &mut ppn.get_pte_array_mut()[indices[1]];
        if !self.arch_impl.valid(pte2) {
            let frame = self.alloc_frame().unwrap();
            let frame_ppn = frame.ppn;

            let pa = frame_ppn.0 << 12;
            unsafe {
                core::ptr::write_bytes(pa as *mut u8, 0, PAGE_SIZE);
            }

            *pte2 = self.arch_impl.new(frame_ppn, PTEFlags::V);
        }
        ppn = self.arch_impl.ppn(pte2);

        // Level 1 (lowest level)
        let pte1 = &mut ppn.get_pte_array_mut()[indices[2]];

        Some(pte1)
    }

    // Method that demonstrates how to use LoongArch64's lddir instruction
    // This is an alternative implementation that uses assembly instructions
    pub fn find_pte_assembly(&self, vpn: VirtPageNum) -> Option<PhysPageNum> {
        let va = vpn.0 << 12; // Convert VPN to virtual address
        let mut pa: usize;

        unsafe {
            asm!(
                // Set up arguments
                "move $a0, {root_ppn}",
                "move $a1, {va}",

                // Convert PPN to physical address
                "slli $a0, $a0, 12",

                // Page table walk with lddir instructions
                "lddir $a0, $a0, 3",      // Level 3 lookup
                "beqz $a0, 1f",           // If 0, entry not valid

                "lddir $a0, $a0, 2",      // Level 2 lookup
                "beqz $a0, 1f",           // If 0, entry not valid

                "lddir $a0, $a0, 1",      // Level 1 lookup
                "beqz $a0, 1f",           // If 0, entry not valid

                // Success path
                "move {pa_out}, $a0",
                "b 2f",

                // Failure path
                "1:",
                "li {pa_out}, 0",

                // End
                "2:",

                root_ppn = in(reg) self.root_ppn.0,
                va = in(reg) va,
                pa_out = out(reg) pa,
                options(nostack)
            );
        }

        if pa == 0 {
            None
        } else {
            Some(PhysPageNum(pa >> 12))
        }
    }
}
