use crate::utils::static_cell::StaticCell;

use super::config::KERNEL_HEAP_SIZE;
use buddy_system_allocator::LockedHeap;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::empty();

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

static HEAP_SPACE: StaticCell<[u8; KERNEL_HEAP_SIZE]> = StaticCell::new();

pub fn init_heap_allocator() {
    unsafe {
        // Initialize the heap space with zeros
        HEAP_SPACE.init([0; KERNEL_HEAP_SIZE]);

        // Get the pointer to the heap array
        let heap_ptr = HEAP_SPACE.get().as_ptr() as usize;
        HEAP_ALLOCATOR.lock().init(heap_ptr, KERNEL_HEAP_SIZE);
    }
}
