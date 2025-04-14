use crate::common::addr::PhysPageNum;
use crate::utils::static_cell::StaticCell;
use alloc::vec::Vec;

pub static FRAME_ALLOCATOR: StaticCell<&dyn FrameAllocatorOperation> = StaticCell::new();

pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

pub trait FrameAllocatorOperation: Send + Sync {
    /// allocate a frame
    fn alloc(&mut self) -> Option<PhysPageNum>;
    /// allocate more frames
    fn alloc_more(&mut self, pages: usize) -> Option<Vec<PhysPageNum>>;
    /// deallocate a frame
    fn dealloc(&mut self, ppn: PhysPageNum);
}

pub fn init_frame_allocator(allocator: &'static dyn FrameAllocatorOperation) {
    FRAME_ALLOCATOR.init(allocator);
}
