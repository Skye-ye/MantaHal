use super::addr::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use super::frame_allocator::{FrameTracker, frame_alloc};
use crate::bit;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::marker::PhantomData;

#[derive(Copy, Clone, Debug)]
#[repr(C)]
/// A page table entry
pub struct PageTableEntry {
    pub bits: usize,
}

impl PageTableEntry {
    /// create an empty page table entry
    pub fn empty() -> Self {
        PageTableEntry { bits: 0 }
    }
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
    /// Architecture-specific PTE flags type (e.g., Loongarch64PTEFlags).
    /// Must implement From<PTEFlags> and Into<PTEFlags>.
    type ArchFlags: bitflags::Flags<Bits = usize> + From<PTEFlags> + Into<PTEFlags> + Copy;

    // --- Constants ---
    /// Page size in bytes (e.g., 4096).
    const PAGE_SIZE: usize;
    /// Number of bits for page size (e.g., 12 for 4KiB).
    const PAGE_SIZE_BITS: usize;
    /// Number of levels in the page table hierarchy.
    const PAGE_TABLE_LEVELS: usize;

    // --- PTE Array Access ---
    /// Get mutable slice of PTEs for a given physical page used as a page table.
    fn get_pte_array(ppn: PhysPageNum) -> &'static mut [PageTableEntry];

    // --- Address/Token Conversions ---
    fn va_to_vpn(va: VirtAddr) -> VirtPageNum;
    fn ppn_to_pa(ppn: PhysPageNum) -> PhysAddr;
    fn vpn_to_va(vpn: VirtPageNum) -> VirtAddr;
    fn ppn_from_token(token: usize) -> PhysPageNum;
    fn token_from_ppn(ppn: PhysPageNum) -> usize;

    // --- PTE Interpretation ---
    /// Check if a PTE is valid (points to a valid next level table or mapped page).
    fn pte_is_valid(pte: &PageTableEntry) -> bool;
    /// Extract the Physical Page Number (PPN) from a PTE.
    fn pte_to_ppn(pte: &PageTableEntry) -> PhysPageNum;
    /// Extract the architecture-specific flags from a PTE.
    fn pte_to_arch_flags(pte: &PageTableEntry) -> Self::ArchFlags;
    /// Convert PTE's arch flags to generic PTEFlags.
    fn pte_to_generic_flags(pte: &PageTableEntry) -> PTEFlags {
        Self::pte_to_arch_flags(pte).into()
    }

    // --- PTE Construction ---
    /// Create a new PTE for a leaf mapping (maps vpn -> ppn with flags).
    fn pte_new_leaf(ppn: PhysPageNum, flags: PTEFlags) -> PageTableEntry;
    /// Create a new PTE for an intermediate node (points to next level table ppn).
    fn pte_new_intermediate(ppn: PhysPageNum) -> PageTableEntry;
}

/// A page table managing virtual to physical address translation using a specific PTOps implementation.
pub struct PageTable<T: PTOps> {
    root_ppn: PhysPageNum,
    frames: Vec<FrameTracker>,
    phantom: PhantomData<T>,
}

impl<T: PTOps> PageTable<T> {
    /// create a new page table with a given architecture implementation
    pub fn new() -> Self {
        let frame = frame_alloc().expect("Failed to allocate root page table frame");
        let root_ppn = frame.ppn;
        // Zero out the root page table frame
        let ptes = T::get_pte_array(root_ppn);
        ptes.iter_mut()
            .for_each(|pte| *pte = PageTableEntry::empty());
        PageTable {
            root_ppn,
            frames: vec![frame],
            phantom: PhantomData,
        }
    }

    /// Create a PageTable instance representing an existing page table from a token.
    /// WARNING: This instance does not own the frames and cannot safely allocate
    /// new table pages (map operations might panic or error). Use primarily for lookups.
    pub fn from_token(token: usize) -> Self {
        Self {
            root_ppn: T::ppn_from_token(token),
            frames: Vec::new(), // No frame ownership
            phantom: PhantomData,
        }
    }

    /// Get the architecture-specific token representing this page table (e.g., for SATP/PGDL).
    pub fn token(&self) -> usize {
        T::token_from_ppn(self.root_ppn)
    }

    /// Get the root physical page number.
    pub fn root_ppn(&self) -> PhysPageNum {
        self.root_ppn
    }

    /// Find the leaf PTE for a virtual page number, without creating entries.
    fn find_pte(&self, vpn: VirtPageNum) -> Option<&'static mut PageTableEntry> {
        let mut current_ppn = self.root_ppn;
        let vpn_indices = vpn.indices(); // Assumes VirtPageNum::indices() exists

        for level in 0..T::PAGE_TABLE_LEVELS {
            let index = vpn_indices[level];
            // Bounds check index? get_pte_array should return fixed size slice.
            // Let hardware handle faults if index is out of range? Or check here?
            // For now, assume index is valid based on vpn.indices() logic.
            let ptes = T::get_pte_array(current_ppn);
            let pte = &mut ptes[index];

            if !T::pte_is_valid(pte) {
                return None; // Entry not valid, path stops here
            }

            if level == T::PAGE_TABLE_LEVELS - 1 {
                return Some(pte);
            } else {
                current_ppn = T::pte_to_ppn(pte);
            }
        }
        // Should only be reached if PAGE_TABLE_LEVELS is 0, which is invalid.
        unreachable!("Page walk finished without reaching final level?");
    }

    /// Find the leaf PTE for a virtual page number, creating intermediate tables if needed.
    fn find_or_create_pte(&mut self, vpn: VirtPageNum) -> Option<&'static mut PageTableEntry> {
        // Safety check: Can only create if we own frames.
        // A more robust solution might be a different type for borrowed page tables.
        // if self.frames.is_empty() {
        //     error!("Attempt to create PTE in a PageTable without frame ownership (from_token?)");
        //     return None;
        // }

        let mut current_ppn = self.root_ppn;
        let vpn_indices = vpn.indices();

        for level in 0..T::PAGE_TABLE_LEVELS {
            let index = vpn_indices[level];
            let ptes = T::get_pte_array(current_ppn);
            let pte = &mut ptes[index];

            if level == T::PAGE_TABLE_LEVELS - 1 {
                // Reached the final level. Return this PTE slot (might be invalid).
                return Some(pte);
            } else {
                // Intermediate level.
                if !T::pte_is_valid(pte) {
                    // Allocate a new frame for the next level table.
                    if self.frames.is_empty() {
                        log::error!(
                            "Cannot allocate page table frame in PageTable without frame ownership."
                        );
                        return None; // Or panic
                    }
                    let frame =
                        frame_alloc().expect("Failed to allocate intermediate page table frame");
                    let next_table_ppn = frame.ppn;
                    // Zero out the new frame
                    let next_ptes = T::get_pte_array(next_table_ppn);
                    next_ptes
                        .iter_mut()
                        .for_each(|entry| *entry = PageTableEntry::empty());

                    // Update current PTE to point to the new table using intermediate flags
                    *pte = T::pte_new_intermediate(next_table_ppn);
                    self.frames.push(frame); // Track ownership
                    current_ppn = next_table_ppn;
                } else {
                    // Valid entry, move to the next level.
                    // We could assert here that it's not unexpectedly a leaf if T::pte_is_leaf exists.
                    current_ppn = T::pte_to_ppn(pte);
                }
            }
        }
        unreachable!();
    }

    /// Map a virtual page number to a physical page number with given generic flags.
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self
            .find_or_create_pte(vpn)
            .expect("Failed to find or create PTE slot for mapping");
        assert!(!T::pte_is_valid(pte), "VPN {:?} is already mapped", vpn);
        *pte = T::pte_new_leaf(ppn, flags);
    }

    /// Unmap a virtual page number. Marks the PTE as invalid.
    /// Does not deallocate the target frame `ppn` or intermediate page tables.
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self
            .find_pte(vpn)
            .expect("Failed to find PTE for unmapping - VPN not mapped or invalid table path");

        assert!(
            T::pte_is_valid(pte),
            "VPN {:?} is not validly mapped, cannot unmap",
            vpn
        );

        *pte = PageTableEntry::empty();

        // TODO: Add TLB invalidation logic here (arch-specific)
        // e.g., Self::flush_tlb(vpn);
        // Do we need this?
    }

    /// Translate a virtual page number to its corresponding PageTableEntry (if validly mapped).
    pub fn translate_vpn(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn).map(|pte| *pte) // Copy the entry
    }

    /// Translate a virtual address to a physical address (if validly mapped).
    pub fn translate_va(&self, va: VirtAddr) -> Option<PhysAddr> {
        let vpn = T::va_to_vpn(va);
        self.find_pte(vpn).map(|pte| {
            let page_start_pa = T::ppn_to_pa(T::pte_to_ppn(pte));
            let page_offset = va.page_offset();
            let aligned_pa_usize: usize = page_start_pa.into();
            (aligned_pa_usize + page_offset).into()
        })
    }
}

pub unsafe fn translate_byte_buffer<T: PTOps>(
    pt: &PageTable<T>,
    ptr: *const u8,
    len: usize,
) -> Option<Vec<&'static mut [u8]>> {
    // Should likely be &'static [u8] if only checking R
    let mut start = ptr as usize;
    let end = start.checked_add(len)?;
    let mut result_slices = Vec::new();

    while start < end {
        let start_va = VirtAddr::from(start);
        let vpn = T::va_to_vpn(start_va);

        let pte = pt.find_pte(vpn)?; // Checks validity implicitly
        let flags = T::pte_to_generic_flags(pte);
        if !flags.contains(PTEFlags::R) {
            log::warn!(
                "Attempt to read from non-readable page: VA {:?}, Flags {:?}",
                start_va,
                flags
            );
            return None;
        }

        let page_start_pa = T::ppn_to_pa(T::pte_to_ppn(pte));
        let page_offset = start_va.page_offset();
        let current_phys_addr = usize::from(page_start_pa) + page_offset;

        let page_remaining = T::PAGE_SIZE - page_offset;
        let requested_remaining = end - start;
        let current_chunk_len = usize::min(page_remaining, requested_remaining);

        let phys_ptr = current_phys_addr as *mut u8; // Need mutable? Or const?
        // Let's assume read-only for byte buffer view:
        // let slice = slice::from_raw_parts(phys_ptr as *const u8, current_chunk_len);
        // If mutable access is needed, check PTEFlags::W as well. Assuming read-only here.
        let slice = unsafe { core::slice::from_raw_parts_mut(phys_ptr, current_chunk_len) }; // Original was mut
        result_slices.push(slice);

        start += current_chunk_len;
    }
    Some(result_slices)
}

/// Unsafe: Translate a null-terminated string. Checks validity.
pub unsafe fn translate_string<T: PTOps>(pt: &PageTable<T>, ptr: *const u8) -> Option<String> {
    let mut string = String::new();
    let mut current_va_usize = ptr as usize;
    loop {
        let va = VirtAddr::from(current_va_usize);
        let vpn = T::va_to_vpn(va);
        let pte = pt.find_pte(vpn)?; // Checks validity

        // Check Read permission
        let flags = T::pte_to_generic_flags(pte);
        if !flags.contains(PTEFlags::R) {
            log::warn!(
                "Attempt to read string from non-readable page: VA {:?}, Flags {:?}",
                va,
                flags
            );
            return None;
        }

        let page_start_pa = T::ppn_to_pa(T::pte_to_ppn(pte));
        let page_offset = va.page_offset();
        let pa: PhysAddr = (usize::from(page_start_pa) + page_offset).into();

        // Read the byte using the physical address
        let ch = unsafe { *pa.as_ref::<u8>() };

        if ch == 0 {
            break; // Null terminator
        }
        string.push(ch as char);
        current_va_usize = current_va_usize.checked_add(1)?;
    }
    Some(string)
}

/// Unsafe: Translate a raw pointer to a reference. Checks validity.
pub unsafe fn translate_ref<T: PTOps, U>(pt: &PageTable<T>, ptr: *const U) -> Option<&'static U> {
    pt.translate_va(VirtAddr::from(ptr as usize))
        .map(|pa| unsafe { pa.as_ref::<U>() })
}

/// Unsafe: Translate a raw pointer to a mutable reference. Checks validity and writability.
pub unsafe fn translate_refmut<T: PTOps, U>(
    pt: &PageTable<T>,
    ptr: *mut U,
) -> Option<&'static mut U> {
    let va = VirtAddr::from(ptr as usize);
    let vpn = T::va_to_vpn(va);
    pt.find_pte(vpn).and_then(|pte| {
        let flags = T::pte_to_generic_flags(pte);
        // Check Valid (already done by find_pte) and Writable
        if !flags.contains(PTEFlags::W) {
            log::warn!(
                "Attempt to get mutable reference to non-writable page: VA {:?}, Flags {:?}",
                va,
                flags
            );
            return None;
        }
        let page_start_pa = T::ppn_to_pa(T::pte_to_ppn(pte));
        let page_offset = va.page_offset();
        let pa: PhysAddr = (usize::from(page_start_pa) + page_offset).into();
        Some(unsafe { pa.as_mut::<U>() })
    })
}
