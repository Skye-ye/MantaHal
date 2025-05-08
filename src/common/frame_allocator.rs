use crate::common::addr::PhysPageNum;
use crate::utils::OnceCell;
use alloc::vec::Vec;
use core::fmt::{Debug, Formatter};

pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

impl FrameTracker {
    pub fn new(ppn: PhysPageNum) -> Self {
        Self { ppn }
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}

#[cfg(feature = "debug")]
impl Debug for FrameTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("FrameTracker:PPN={:#x}", self.ppn.0))
    }
}

/// Trait for physical frame allocation.
/// Implementations MUST handle interior mutability (e.g., using Mutex)
/// because methods take &self but need to modify state.
pub trait FrameAlloc: Send + Sync {
    /// Allocate a frame.
    /// Takes &self, requires internal synchronization if state is modified.
    fn alloc(&self) -> Option<PhysPageNum>;

    /// Allocate multiple consecutive frames.
    /// Takes &self, requires internal synchronization if state is modified.
    fn allocate_physical_pages(&self, pages: usize) -> Option<Vec<PhysPageNum>>;

    /// Deallocate a frame.
    /// Takes &self, requires internal synchronization if state is modified.
    fn dealloc(&self, ppn: PhysPageNum);
}

// --- Global Static Allocator Reference ---
// Stores a reference to an object implementing the FrameAlloc trait.
// The object itself must handle thread-safety for mutation.
static FRAME_ALLOCATOR: OnceCell<&'static dyn FrameAlloc> = OnceCell::new();

/// Initialize the global frame allocator **once**.
///
/// # Panics
/// Panics if called more than once.
/// The provided `frame_allocator` must live for the 'static lifetime
/// and must implement thread-safe interior mutability.
pub fn init_frame_allocator(frame_allocator: &'static dyn FrameAlloc) {
    FRAME_ALLOCATOR.init(frame_allocator);
}

/// Allocate a frame using the globally initialized allocator.
///
/// Returns a `FrameTracker` which automatically deallocates the frame when dropped.
/// Returns `None` if no frames are available.
/// # Panics
/// Panics if the allocator is not initialized.
pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR.get().alloc().map(FrameTracker::new)
}

/// Allocate multiple consecutive physical frames using the global allocator.
///
/// Returns `None` if not enough consecutive frames are available.
/// # Panics
/// Panics if the allocator is not initialized.
pub fn frame_alloc_physical_pages(num: usize) -> Option<Vec<FrameTracker>> {
    if num == 0 {
        return Some(Vec::new());
    }
    FRAME_ALLOCATOR
        .get()
        .allocate_physical_pages(num)
        .map(|ppns| ppns.into_iter().map(FrameTracker::new).collect())
}

/// Deallocate a frame using the global allocator.
///
/// This is usually called automatically when a `FrameTracker` is dropped.
/// # Panics
/// Panics if the allocator is not initialized or if the ppn is invalid according
/// to the allocator's internal state.
pub fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.get().dealloc(ppn);
}
