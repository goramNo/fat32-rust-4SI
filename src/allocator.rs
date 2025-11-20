use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicUsize, Ordering};

struct BumpAlloc {
    head: AtomicUsize,
}

unsafe impl Sync for BumpAlloc {}

const HEAP_SIZE: usize = 1024 * 1024;

struct Heap(UnsafeCell<[u8; HEAP_SIZE]>);

unsafe impl Sync for Heap {}

static HEAP: Heap = Heap(UnsafeCell::new([0; HEAP_SIZE]));

#[global_allocator]
static GLOBAL: BumpAlloc = BumpAlloc {
    head: AtomicUsize::new(0),
};

unsafe impl GlobalAlloc for BumpAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();

        let current = self.head.load(Ordering::SeqCst);
        let aligned = (current + (align - 1)) & !(align - 1);
        let end = aligned.saturating_add(size);

        if end > HEAP_SIZE {
            return null_mut();
        }

        self.head.store(end, Ordering::SeqCst);

        let base = unsafe { (*HEAP.0.get()).as_ptr() as *mut u8 };
        unsafe { base.add(aligned) }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // bump: rien
    }
}
