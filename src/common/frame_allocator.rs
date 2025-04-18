use crate::common::addr::PhysPageNum;
use crate::utils::static_cell::StaticCell;
use alloc::vec::Vec;
use core::fmt::{Debug, Formatter};

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

#[cfg(feature = "debug")]
impl Debug for FrameTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("FrameTracker:PPN={:#x}", self.ppn.0))
    }
}

pub struct StackFrameAllocator {
    current: usize,
    end: usize,
    recycled: Vec<usize>,
}

impl StackFrameAllocator {
    pub fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }

    fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
        self.current = l.0;
        self.end = r.0;
    }
}

impl FrameAlloc for StackFrameAllocator {
    fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else if self.current == self.end {
            None
        } else {
            self.current += 1;
            Some((self.current - 1).into())
        }
    }

    fn allocate_physical_pages(&mut self, pages: usize) -> Option<Vec<PhysPageNum>> {
        if self.current + pages > self.end {
            None
        } else {
            let start = self.current;
            self.current += pages;

            let mut result = Vec::with_capacity(pages);
            for i in 0..pages {
                result.push((start + i).into());
            }

            Some(result)
        }
    }

    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.0;
        // validity check
        if ppn >= self.current || self.recycled.iter().any(|&v| v == ppn) {
            panic!("Frame ppn={:#x} has not been allocated!", ppn);
        }
        // recycle
        self.recycled.push(ppn);
    }
}

type FrameAllocatorImpl = StackFrameAllocator;

pub static FRAME_ALLOCATOR: StaticCell<FrameAllocatorImpl> = StaticCell::new();

/// Initialize the frame allocator
pub fn init_frame_allocator() {
    FRAME_ALLOCATOR.init(StackFrameAllocator::new());
    // TODO: set the range of the frame allocator
    let l = PhysPageNum::from(0x80000000);
    let r = PhysPageNum::from(0x80000000 + 1024 * 1024 * 1024);
    FRAME_ALLOCATOR.get_mut().init(l, r);
}

/// Allocate a frame
pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR.get_mut().alloc().map(FrameTracker::new)
}

/// Allocate multiple consecutive frames
pub fn frame_alloc_physical_pages(num: usize) -> Option<Vec<FrameTracker>> {
    FRAME_ALLOCATOR
        .get_mut()
        .allocate_physical_pages(num)
        .map(|x| x.iter().map(|&t| FrameTracker::new(t)).collect())
}

/// Deallocate a frame
pub fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.get_mut().dealloc(ppn);
}
