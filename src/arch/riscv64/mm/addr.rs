use crate::{arch::config::mm::{PAGE_MASK, PAGE_SIZE, PAGE_SIZE_BITS, PAGE_TABLE_LEVEL_NUM, PPN_WIDTH_SV39, PTES_PER_PAGE, PTE_SIZE, VA_WIDTH_SV39, VIRT_RAM_OFFSET, VPN_WIDTH_SV39}, common::{addr::{PhysAddr, PhysPageNum, VirtAddr}, pagetable::PageTableEntry}};
use crate::common::addr::VirtPageNum;

// PhysAddr implementations
impl PhysAddr {

    pub fn bits(&self) -> usize {
        self.0
    }
    #[inline]
    pub fn page_offset(&self) -> usize {
        self.0 & PAGE_MASK
    }

    #[inline]
    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }

    #[inline]
    pub fn ceil(self) -> PhysPageNum {
        PhysPageNum((self.0 + PAGE_MASK) / PAGE_SIZE)
    }

    #[inline]
    pub fn floor(self) -> PhysPageNum {
        PhysPageNum(self.0 / PAGE_SIZE)
    }

    pub fn to_vaddr(&self) -> VirtAddr {
        (self.bits() + VIRT_RAM_OFFSET).into()
    }
}

impl From<usize> for PhysAddr {
    fn from(v: usize) -> Self {
        Self(v)
    }
}


impl From<PhysAddr> for PhysPageNum {
    #[inline]
    fn from(v: PhysAddr) -> Self {
        // Address must be aligned. If not, ceil or floor it.
        assert!(v.aligned(), "Physical address must be aligned");
        v.floor()
    }
}

impl From<PhysPageNum> for PhysAddr {
    #[inline]
    fn from(v: PhysPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}

impl From<usize> for PhysPageNum {
    fn from(u: usize) -> Self {
        let tmp = u as isize >> PPN_WIDTH_SV39;
        assert!(tmp == 0 || tmp == -1);
        Self(u)
    }
}



impl PhysPageNum {
    pub(crate) const ZERO: Self = PhysPageNum(0);

    pub fn bits(&self) -> usize {
        self.0
    }

    /// Get reference to `PhysPageNum` value
    pub fn get_ref<T>(&self) -> &'static T {
        unsafe { (self.0 as *const T).as_ref().unwrap() }
    }

    /// Get mutable reference to `PhysAddr` value
    pub fn get_mut<T>(&self) -> &'static mut T {
        unsafe { (self.0 as *mut T).as_mut().unwrap() }
    }

    pub fn to_paddr(&self) -> PhysAddr {
        (*self).into()
    }

    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {

        let vaddr: VirtAddr = self.to_paddr().to_vaddr();
        unsafe {
            core::slice::from_raw_parts_mut(vaddr.bits() as *mut PageTableEntry, PTES_PER_PAGE)
        }
      
    }

}


impl VirtAddr {
    pub const fn from_usize(v: usize) -> Self {
        Self(v)
    }

    pub const fn bits(&self) -> usize {
        self.0
    }

    pub fn to_paddr(&self) -> Option<PhysAddr> {
        if self.bits() >= VIRT_RAM_OFFSET {
            Some((self.bits() - VIRT_RAM_OFFSET).into())
        } else {
           None
    }
    }

    pub fn is_null(&self) -> bool {
        self.0 == 0
    }

    /// `VirtAddr`->`VirtPageNum`
    pub fn floor(&self) -> VirtPageNum {
        VirtPageNum(self.0 / PAGE_SIZE)
    }

    /// `VirtAddr` -> rounded down to a multiple of PAGE_SIZE
    pub fn round_down(&self) -> Self {
        Self(self.0 & !PAGE_MASK)
    }

    /// `VirtAddr`->`VirtPageNum`
    pub fn ceil(&self) -> VirtPageNum {
        VirtPageNum((self.0 + PAGE_MASK) / PAGE_SIZE)
    }

    /// `VirtAddr` -> rounded up to a multiple of PAGE_SIZE
    pub fn round_up(&self) -> Self {
        Self((self.0 + PAGE_MASK) & !PAGE_MASK)
    }

    pub fn page_offset(&self) -> usize {
        self.0 & PAGE_MASK
    }

    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }

    pub const fn as_ptr(self) -> *const u8 {
        self.0 as *const u8
    }

    pub const fn as_mut_ptr(self) -> *mut u8 {
        self.0 as *mut u8
    }

    /// Get reference to `VirtAddr` value
    pub unsafe fn get_ref<T>(&self) -> &'static T {
        unsafe { (self.0 as *const T).as_ref().unwrap() }
    }

    /// Get mutable reference to `VirtAddr` value
    pub unsafe fn get_mut<T>(&self) -> &'static mut T {
        unsafe { (self.0 as *mut T).as_mut().unwrap() }
    }

    
}



impl From<usize> for VirtAddr {
    fn from(v: usize) -> Self {
        let tmp = v as isize >> VA_WIDTH_SV39;
        // NOTE: do not use assert here because syscall args passed in may be invalid
        if !(tmp == 0 || tmp == -1) {
            log::warn!("invalid virtual address {v}");
        }
        Self(v)
    }
}

impl From<usize> for VirtPageNum {
    fn from(v: usize) -> Self {
        let tmp = v >> (VPN_WIDTH_SV39 - 1);
        // NOTE: do not use assert here because syscall args passed in may be invalid
        if !(tmp == 0 || tmp == (1 << (52 - VPN_WIDTH_SV39 + 1)) - 1) {
            log::warn!("invalid virtual page number {v}");
        }
        Self(v)
    }
}

impl From<VirtAddr> for VirtPageNum {
    fn from(v: VirtAddr) -> Self {
        assert_eq!(v.page_offset(), 0);
        v.floor()
    }
}


impl From<VirtPageNum> for VirtAddr {
    fn from(v: VirtPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}

impl VirtPageNum{
    pub fn indices(&self) -> [usize; PAGE_TABLE_LEVEL_NUM] {
        let mut vpn = self.0;
        let mut indices = [0usize; PAGE_TABLE_LEVEL_NUM];
        for i in (0..PAGE_TABLE_LEVEL_NUM).rev() {
            indices[i] = vpn & (PTES_PER_PAGE - 1);
            vpn >>= 9;
        }
        indices
    }
}