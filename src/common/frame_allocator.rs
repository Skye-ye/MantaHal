use crate::common::addr::PhysPageNum;
use crate::utils::static_cell::StaticCell;
use alloc::vec::Vec;

pub static FRAME_ALLOCATOR: StaticCell<&mut dyn FrameAlloc> = StaticCell::new();

pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

pub trait FrameAlloc: Send + Sync {
    /// allocate a frame
    fn alloc(&mut self) -> Option<PhysPageNum>;
    /// allocate multiple consecutive frames
    fn allocate_physical_pages(&mut self, pages: usize) -> Option<Vec<PhysPageNum>>;
    /// deallocate a frame
    fn dealloc(&mut self, ppn: PhysPageNum);
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

/// Initialize the frame allocator
pub fn init_frame_allocator(allocator: &'static mut dyn FrameAlloc) {
    FRAME_ALLOCATOR.init(allocator);
}

/// Allocate a frame
pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR.get_mut().alloc().map(FrameTracker::new)
}

/// Deallocate a frame
pub fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.get_mut().dealloc(ppn);
}
