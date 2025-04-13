use crate::addr::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};

// Define RISC-V specific constants
const PAGE_SIZE_BITS: usize = 12;
const PAGE_SIZE: usize = 1 << PAGE_SIZE_BITS;
const PAGE_MASK: usize = PAGE_SIZE - 1;

// PhysAddr implementations
impl PhysAddr {
    pub fn page_number(&self) -> PhysPageNum {
        PhysPageNum(self.0 / PAGE_SIZE)
    }

    pub fn page_offset(&self) -> usize {
        self.0 & PAGE_MASK
    }

    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }
}

impl From<usize> for PhysAddr {
    fn from(v: usize) -> Self {
        Self(v)
    }
}

impl From<PhysAddr> for PhysPageNum {
    fn from(v: PhysAddr) -> Self {
        v.page_number()
    }
}

// VirtAddr implementations
impl VirtAddr {
    pub fn page_number(&self) -> VirtPageNum {
        VirtPageNum(self.0 / PAGE_SIZE)
    }

    pub fn page_offset(&self) -> usize {
        self.0 & PAGE_MASK
    }

    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }
}

impl From<usize> for VirtAddr {
    fn from(v: usize) -> Self {
        // For RISC-V, we use the address directly
        Self(v)
    }
}

impl From<VirtAddr> for VirtPageNum {
    fn from(v: VirtAddr) -> Self {
        v.page_number()
    }
}

// PhysPageNum implementations
impl PhysPageNum {
    pub fn address(&self) -> PhysAddr {
        PhysAddr(self.0 * PAGE_SIZE)
    }
}

impl From<usize> for PhysPageNum {
    fn from(v: usize) -> Self {
        Self(v)
    }
}

// VirtPageNum implementations
impl VirtPageNum {
    pub fn address(&self) -> VirtAddr {
        VirtAddr(self.0 * PAGE_SIZE)
    }
}

impl From<usize> for VirtPageNum {
    fn from(v: usize) -> Self {
        Self(v)
    }
}
