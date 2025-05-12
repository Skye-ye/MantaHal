use crate::arch::config::mm::{PAGE_TABLE_LEVELS, PTE_INDEX_BITS, PTE_INDEX_MASK};
#[cfg(feature = "debug")]
use core::fmt::{Debug, Formatter};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[repr(C)]
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[repr(C)]
pub struct VirtAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[repr(C)]
pub struct PhysPageNum(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[repr(C)]
pub struct VirtPageNum(pub usize);

// Common From implementations for all architectures
impl From<PhysAddr> for usize {
    fn from(v: PhysAddr) -> Self {
        v.0
    }
}

impl From<VirtAddr> for usize {
    fn from(v: VirtAddr) -> Self {
        v.0
    }
}

impl From<PhysPageNum> for usize {
    fn from(v: PhysPageNum) -> Self {
        v.0
    }
}

impl From<VirtPageNum> for usize {
    fn from(v: VirtPageNum) -> Self {
        v.0
    }
}

impl PhysAddr {
    pub fn get_ref<T>(&self) -> &'static T {
        unsafe { (self.0 as *const T).as_ref().unwrap() }
    }
    pub fn get_mut<T>(&self) -> &'static mut T {
        unsafe { (self.0 as *mut T).as_mut().unwrap() }
    }
}

impl VirtPageNum {
    pub fn step(&mut self) {
        self.0 += 1;
    }
}

impl PhysPageNum {
    pub fn step(&mut self) {
        self.0 += 1;
    }
}

impl PhysAddr {
    /// Converts the physical address to a raw pointer.
    fn as_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }

    /// Converts the physical address to a mutable raw pointer.
    fn as_mut_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }

    /// Unsafe: Converts the physical address to a reference of type U.
    ///
    /// # Safety
    /// The caller must ensure that:
    /// 1. The physical address points to valid, initialized memory for type `U`.
    /// 2. The address has the correct alignment for type `U`.
    /// 3. The memory is accessible (e.g., kernel direct-mapped region).
    /// 4. The resulting reference does not outlive the validity of the underlying memory.
    ///    The `'static` lifetime is a promise that the caller must uphold regarding
    ///    the actual lifetime of the physical memory.
    /// 5. No other mutable references to this location exist.
    #[inline]
    pub unsafe fn as_ref<U>(&self) -> &'static U {
        // SAFETY: Caller guarantees validity, alignment, accessibility, lifetime, and no mutable aliases.
        unsafe { &*self.as_ptr::<U>() }
    }

    /// Unsafe: Converts the physical address to a mutable reference of type U.
    ///
    /// # Safety
    /// The caller must ensure that:
    /// 1. The physical address points to valid, initialized memory for type `U`.
    /// 2. The address has the correct alignment for type `U`.
    /// 3. The memory is accessible and writable.
    /// 4. The resulting reference does not outlive the validity of the underlying memory.
    ///    The `'static` lifetime is a promise that the caller must uphold.
    /// 5. No other references (mutable or immutable) to this location exist.
    ///    This upholds Rust's borrowing rules manually.
    #[inline]
    pub unsafe fn as_mut<U>(&self) -> &'static mut U {
        // SAFETY: Caller guarantees validity, alignment, accessibility, lifetime, and exclusivity.
        unsafe { &mut *self.as_mut_ptr::<U>() }
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

/// Debug implementation for all address types
#[cfg(feature = "debug")]
impl Debug for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("VA:{:#x}", self.0))
    }
}

#[cfg(feature = "debug")]
impl Debug for VirtPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("VPN:{:#x}", self.0))
    }
}

#[cfg(feature = "debug")]
impl Debug for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("PA:{:#x}", self.0))
    }
}

#[cfg(feature = "debug")]
impl Debug for PhysPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("PPN:{:#x}", self.0))
    }
}
