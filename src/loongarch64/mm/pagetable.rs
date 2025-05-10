use crate::bit;
use crate::common::addr::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use crate::common::pagetable::PTEFlags;
use crate::common::pagetable::PTOps;
use crate::common::pagetable::PageTableEntry;
use crate::loongarch64::config::mm::{
    PAGE_SIZE, PAGE_SIZE_BITS, PAGE_TABLE_LEVELS, PPN_MASK, PPN_OFFSET_IN_PTE,
};

bitflags::bitflags! {
    /// Possible flags for a page table entry.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Loongarch64PTEFlags: usize {
        /// Page Valid
        const V = bit!(0);
        /// Dirty, The page has been written.
        const D = bit!(1);
        /// Privilege level
        const PLV = (bit!(2)) | (bit!(3));
        /// Memory access type
        const MAT = (bit!(4)) | (bit!(5));
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

// --- From/Into Conversions (Remain the same) ---
impl From<Loongarch64PTEFlags> for PTEFlags {
    fn from(value: Loongarch64PTEFlags) -> Self {
        let mut flags = PTEFlags::empty(); // Start empty

        // Only consider it generically Valid if Loongarch V is set
        if value.contains(Loongarch64PTEFlags::V) {
            flags |= PTEFlags::V;
        } else {
            return PTEFlags::empty();
        }

        // Readable if NR is NOT set
        if !value.contains(Loongarch64PTEFlags::NR) {
            flags |= PTEFlags::R;
        }

        // Writable if W is set
        if value.contains(Loongarch64PTEFlags::W) {
            flags |= PTEFlags::W;
        }

        // Executable if NX is NOT set
        if !value.contains(Loongarch64PTEFlags::NX) {
            flags |= PTEFlags::X;
        }

        // User mode if PLV indicates user (e.g., PLV == 3)
        // Assuming PLV=3 means User. Adjust if needed.
        const PLV_USER_BITS: usize = (bit!(2)) | (bit!(3)); // PLV = 3
        if value.bits() & ((bit!(2)) | (bit!(3))) == PLV_USER_BITS {
            flags |= PTEFlags::U;
        }

        // Dirty if D is set
        if value.contains(Loongarch64PTEFlags::D) {
            flags |= PTEFlags::D;
        }

        // Global if G is set (assuming G is bit 6 and not H)
        if value.contains(Loongarch64PTEFlags::G) {
            flags |= PTEFlags::G;
        }

        flags
    }
}

impl From<PTEFlags> for Loongarch64PTEFlags {
    fn from(val: PTEFlags) -> Self {
        // Loongarch defaults: Present, Cached Memory, PLV=Kernel (0)
        // Adjust MAT and PLV based on common usage. Assume Kernel, Cacheable unless U is set.
        // TODO: Check if MAT is correct
        let mut flags = Loongarch64PTEFlags::P | Loongarch64PTEFlags::MAT; // MAT=1 (Cached)

        // Must be valid if any permission is set (R/W/X/U)
        if val.intersects(PTEFlags::R | PTEFlags::W | PTEFlags::X | PTEFlags::U) {
            flags |= Loongarch64PTEFlags::V;
        }

        // Set NR unless R is requested
        if !val.contains(PTEFlags::R) {
            flags |= Loongarch64PTEFlags::NR;
        }

        // Set W if requested
        if val.contains(PTEFlags::W) {
            flags |= Loongarch64PTEFlags::W;
        }

        // Set NX unless X is requested
        if !val.contains(PTEFlags::X) {
            flags |= Loongarch64PTEFlags::NX;
        }

        // Set PLV if U is requested (Assume PLV=3 for User)
        if val.contains(PTEFlags::U) {
            const PLV_USER_BITS: usize = (bit!(2)) | (bit!(3)); // PLV = 3
            flags |= Loongarch64PTEFlags::from_bits_retain(PLV_USER_BITS); // Set PLV=3
        }

        // Set D if requested
        if val.contains(PTEFlags::D) {
            flags |= Loongarch64PTEFlags::D;
        }

        // Set G if requested
        if val.contains(PTEFlags::G) {
            flags |= Loongarch64PTEFlags::G;
        }

        flags
    }
}

pub struct Loongarch64PTImpl;

impl PTOps for Loongarch64PTImpl {
    type ArchFlags = Loongarch64PTEFlags;

    const PAGE_SIZE: usize = PAGE_SIZE;
    const PAGE_SIZE_BITS: usize = PAGE_SIZE_BITS;
    const PAGE_TABLE_LEVELS: usize = PAGE_TABLE_LEVELS;

    fn get_pte_array(ppn: PhysPageNum) -> &'static mut [PageTableEntry] {
        // Assuming PAGE_SIZE is divisible by size_of::<PageTableEntry>()
        const PTES_PER_PAGE: usize = PAGE_SIZE / core::mem::size_of::<PageTableEntry>();
        let pa = Self::ppn_to_pa(ppn).0;
        unsafe { core::slice::from_raw_parts_mut(pa as *mut PageTableEntry, PTES_PER_PAGE) }
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
        Self::pte_to_arch_flags(pte).contains(Loongarch64PTEFlags::V)
    }

    fn pte_to_ppn(pte: &PageTableEntry) -> PhysPageNum {
        ((pte.bits >> PPN_OFFSET_IN_PTE) & PPN_MASK).into()
    }

    fn pte_to_arch_flags(pte: &PageTableEntry) -> Self::ArchFlags {
        // Extract only the flag bits (mask out PPN)
        // Assuming flags are in the lower bits + high bits (NR, NX, RPLV)
        // Need a mask for ALL flag bits combined.
        // PPN mask covers bits PPN_OFFSET_IN_PTE upwards.
        // Flags are bits below PPN_OFFSET_IN_PTE and bits 61, 62, 63.
        const FLAGS_MASK_LOW: usize = (1 << PPN_OFFSET_IN_PTE) - 1;
        const FLAGS_MASK_HIGH: usize = Loongarch64PTEFlags::NR.bits()
            | Loongarch64PTEFlags::NX.bits()
            | Loongarch64PTEFlags::RPLV.bits();
        const FLAGS_MASK: usize = FLAGS_MASK_LOW | FLAGS_MASK_HIGH;

        Loongarch64PTEFlags::from_bits_retain(pte.bits & FLAGS_MASK)
    }

    fn pte_new_leaf(ppn: PhysPageNum, flags: PTEFlags) -> PageTableEntry {
        // Convert generic flags to arch flags using From impl
        let arch_flags: Loongarch64PTEFlags = flags.into();
        // Combine PPN shifted with arch flags
        PageTableEntry {
            bits: (ppn.0 << PPN_OFFSET_IN_PTE) | arch_flags.bits(),
        }
    }

    fn pte_new_intermediate(ppn: PhysPageNum) -> PageTableEntry {
        // Intermediate nodes just need to be Valid (V=1) and Present (P=1)? Maybe MAT?
        // Pointing to the next level table ppn. Permissions usually don't apply.
        // Using minimal flags: V=1, P=1. MAT=Cached?
        // TODO: Check if MAT is correct
        let arch_flags = Loongarch64PTEFlags::V | Loongarch64PTEFlags::P | Loongarch64PTEFlags::MAT; // MAT=1 Cached
        PageTableEntry {
            bits: (ppn.0 << PPN_OFFSET_IN_PTE) | arch_flags.bits(),
        }
    }
}
