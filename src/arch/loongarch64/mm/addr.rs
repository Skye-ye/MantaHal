//! Address space for LoongArch64
//!
//! This module implements basic address space operations for LoongArch64.
use crate::addr::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use crate::arch::loongarch64::config::mm::{
    PA_MASK, PAGE_MASK, PAGE_SIZE, PAGE_SIZE_BITS, PPN_MASK, VA_MASK, VPN_MASK,
};

// PhysAddr implementations
impl PhysAddr {
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
}

impl From<usize> for PhysAddr {
    #[inline]
    fn from(v: usize) -> Self {
        Self(v & PA_MASK)
    }
}

impl From<usize> for PhysPageNum {
    #[inline]
    fn from(v: usize) -> Self {
        Self(v & PPN_MASK)
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

// VirtAddr implementations
impl VirtAddr {
    #[inline]
    pub fn page_offset(&self) -> usize {
        self.0 & PAGE_MASK
    }

    #[inline]
    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }

    #[inline]
    pub fn ceil(self) -> VirtPageNum {
        VirtPageNum((self.0 + PAGE_MASK) / PAGE_SIZE)
    }

    #[inline]
    pub fn floor(self) -> VirtPageNum {
        VirtPageNum(self.0 / PAGE_SIZE)
    }
}

impl From<usize> for VirtAddr {
    #[inline]
    fn from(v: usize) -> Self {
        Self(v & VA_MASK)
    }
}

impl From<usize> for VirtPageNum {
    #[inline]
    fn from(v: usize) -> Self {
        Self(v & VPN_MASK)
    }
}

impl From<VirtAddr> for VirtPageNum {
    #[inline]
    fn from(v: VirtAddr) -> Self {
        // Address must be aligned. If not, ceil or floor it.
        assert!(v.aligned(), "Virtual address must be aligned");
        v.floor()
    }
}

impl From<VirtPageNum> for VirtAddr {
    #[inline]
    fn from(v: VirtPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}
