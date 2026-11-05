use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};

// ── Allocations count ─────────────────────────────────────────────────────────

#[global_allocator]
pub static ALLOCATOR: CountingAllocator = CountingAllocator::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalAllocations {
    pub allocations: u64,
    pub deallocations: u64,
    pub reallocations: u64,
    pub allocated_bytes: u64,
    pub deallocated_bytes: u64,
}

#[inline(always)]
pub fn global_allocations_snapshot() -> GlobalAllocations {
    GlobalAllocations {
        allocations: ALLOCATOR.allocations(),
        deallocations: ALLOCATOR.deallocations(),
        reallocations: ALLOCATOR.reallocations(),
        allocated_bytes: ALLOCATOR.allocated_bytes(),
        deallocated_bytes: ALLOCATOR.deallocated_bytes(),
    }
}

// ── Allocator Counter ─────────────────────────────────────────────────────────

#[repr(align(64))]
pub struct CountingAllocator {
    allocations: AtomicU64,
    deallocations: AtomicU64,
    reallocations: AtomicU64,

    allocated_bytes: AtomicU64,
    deallocated_bytes: AtomicU64,
}

impl Default for CountingAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl CountingAllocator {
    pub const fn new() -> Self {
        Self {
            allocations: AtomicU64::new(0),
            deallocations: AtomicU64::new(0),
            reallocations: AtomicU64::new(0),

            allocated_bytes: AtomicU64::new(0),
            deallocated_bytes: AtomicU64::new(0),
        }
    }

    #[inline(always)]
    pub fn allocations(&self) -> u64 {
        self.allocations.load(Ordering::Relaxed)
    }

    #[inline(always)]
    pub fn deallocations(&self) -> u64 {
        self.deallocations.load(Ordering::Relaxed)
    }

    #[inline(always)]
    pub fn reallocations(&self) -> u64 {
        self.reallocations.load(Ordering::Relaxed)
    }

    #[inline(always)]
    pub fn allocated_bytes(&self) -> u64 {
        self.allocated_bytes.load(Ordering::Relaxed)
    }

    #[inline(always)]
    pub fn deallocated_bytes(&self) -> u64 {
        self.deallocated_bytes.load(Ordering::Relaxed)
    }
}

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.allocations.fetch_add(1, Ordering::Relaxed);
        self.allocated_bytes
            .fetch_add(layout.size() as u64, Ordering::Relaxed);

        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.deallocations.fetch_add(1, Ordering::Relaxed);
        self.deallocated_bytes
            .fetch_add(layout.size() as u64, Ordering::Relaxed);

        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        self.reallocations.fetch_add(1, Ordering::Relaxed);

        let old = layout.size() as u64;
        let new = new_size as u64;

        if new > old {
            self.allocated_bytes.fetch_add(new - old, Ordering::Relaxed);
        } else {
            self.deallocated_bytes
                .fetch_add(old - new, Ordering::Relaxed);
        }

        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

// ── Individual getters ────────────────────────────────────────────────────────

#[inline(always)]
pub fn allocations() -> u64 {
    global_allocations_snapshot().allocations
}

#[inline(always)]
pub fn deallocations() -> u64 {
    global_allocations_snapshot().deallocations
}

#[inline(always)]
pub fn reallocations() -> u64 {
    global_allocations_snapshot().reallocations
}

#[inline(always)]
pub fn allocated_bytes() -> u64 {
    global_allocations_snapshot().allocated_bytes
}

#[inline(always)]
pub fn deallocated_bytes() -> u64 {
    global_allocations_snapshot().deallocated_bytes
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::ALLOCATOR;
    use std::alloc::Layout;

    fn layout(size: usize) -> Layout {
        Layout::from_size_align(size, 8).unwrap()
    }

    // ── Snapshot ──────────────────────────────────────────────────────────────

    #[test]
    fn global_allocations_snapshot() {
        let before = super::global_allocations_snapshot();

        let block = Box::new([0u8; 64]);
        let _ = std::hint::black_box(&block);

        let after_alloc = super::global_allocations_snapshot();
        assert!(after_alloc.allocations > before.allocations);
        assert!(after_alloc.allocated_bytes >= before.allocated_bytes + 64);

        drop(block);

        let after_dealloc = super::global_allocations_snapshot();
        assert!(after_dealloc.deallocations > after_alloc.deallocations);
        assert!(after_dealloc.deallocated_bytes >= after_alloc.deallocated_bytes + 64);

        let mut vec: Vec<u8> = Vec::with_capacity(1);
        let before_realloc = super::global_allocations_snapshot();

        vec.reserve(4096);
        let _ = std::hint::black_box(&vec);

        let after_realloc = super::global_allocations_snapshot();
        assert!(after_realloc.reallocations > before_realloc.reallocations);
    }

    // ── CountingAllocator ─────────────────────────────────────────────────────

    mod counting_allocator {
        use super::super::CountingAllocator;
        use super::layout;
        use std::alloc::GlobalAlloc;

        #[test]
        fn new() {
            let allocator = CountingAllocator::new();

            assert_eq!(allocator.allocations(), 0);
            assert_eq!(allocator.deallocations(), 0);
            assert_eq!(allocator.reallocations(), 0);
            assert_eq!(allocator.allocated_bytes(), 0);
            assert_eq!(allocator.deallocated_bytes(), 0);
        }

        #[test]
        fn default() {
            let allocator = CountingAllocator::default();

            assert_eq!(allocator.allocations(), 0);
            assert_eq!(allocator.deallocations(), 0);
            assert_eq!(allocator.reallocations(), 0);
            assert_eq!(allocator.allocated_bytes(), 0);
            assert_eq!(allocator.deallocated_bytes(), 0);
        }

        #[test]
        fn allocations() {
            let allocator = CountingAllocator::new();
            let layout = layout(8);

            unsafe {
                let ptr = allocator.alloc(layout);
                assert!(!ptr.is_null());

                assert_eq!(allocator.allocations(), 1);

                allocator.dealloc(ptr, layout);
            }
        }

        #[test]
        fn deallocations() {
            let allocator = CountingAllocator::new();
            let layout = layout(8);

            unsafe {
                let ptr = allocator.alloc(layout);
                allocator.dealloc(ptr, layout);

                assert_eq!(allocator.deallocations(), 1);
            }
        }

        #[test]
        fn reallocations() {
            let allocator = CountingAllocator::new();
            let old = layout(16);

            unsafe {
                let ptr = allocator.alloc(old);
                assert!(!ptr.is_null());

                let ptr = allocator.realloc(ptr, old, 64);
                assert!(!ptr.is_null());

                assert_eq!(allocator.reallocations(), 1);

                allocator.dealloc(ptr, layout(64));
            }
        }

        #[test]
        fn allocated_bytes() {
            let allocator = CountingAllocator::new();
            let old = layout(16);

            unsafe {
                let ptr = allocator.alloc(old);
                assert!(!ptr.is_null());
                assert_eq!(allocator.allocated_bytes(), 16);

                let ptr = allocator.realloc(ptr, old, 64);
                assert!(!ptr.is_null());
                assert_eq!(allocator.allocated_bytes(), 64);

                allocator.dealloc(ptr, layout(64));
            }
        }

        #[test]
        fn deallocated_bytes() {
            let allocator = CountingAllocator::new();
            let old = layout(64);

            unsafe {
                let ptr = allocator.alloc(old);
                assert!(!ptr.is_null());

                let ptr = allocator.realloc(ptr, old, 16);
                assert!(!ptr.is_null());
                assert_eq!(allocator.deallocated_bytes(), 48);

                allocator.dealloc(ptr, layout(16));
                assert_eq!(allocator.deallocated_bytes(), 64);
            }
        }
    }

    // ── GlobalAlloc ───────────────────────────────────────────────────────────

    mod global_alloc {
        use super::super::CountingAllocator;
        use super::layout;
        use std::alloc::GlobalAlloc;

        #[test]
        fn alloc() {
            let allocator = CountingAllocator::new();
            let layout = layout(8);

            unsafe {
                let ptr = allocator.alloc(layout);
                assert!(!ptr.is_null());

                assert_eq!(allocator.allocations(), 1);
                assert_eq!(allocator.deallocations(), 0);
                assert_eq!(allocator.reallocations(), 0);
                assert_eq!(allocator.allocated_bytes(), layout.size() as u64);
                assert_eq!(allocator.deallocated_bytes(), 0);

                allocator.dealloc(ptr, layout);
            }
        }

        #[test]
        fn dealloc() {
            let allocator = CountingAllocator::new();
            let layout = layout(8);

            unsafe {
                let ptr = allocator.alloc(layout);
                allocator.dealloc(ptr, layout);

                assert_eq!(allocator.allocations(), 1);
                assert_eq!(allocator.deallocations(), 1);
                assert_eq!(allocator.allocated_bytes(), layout.size() as u64);
                assert_eq!(allocator.deallocated_bytes(), layout.size() as u64);
            }
        }

        #[test]
        fn realloc() {
            let allocator = CountingAllocator::new();
            let old = layout(16);

            unsafe {
                let ptr = allocator.alloc(old);
                assert!(!ptr.is_null());

                let ptr = allocator.realloc(ptr, old, 64);
                assert!(!ptr.is_null());

                assert_eq!(allocator.reallocations(), 1);
                assert_eq!(allocator.allocated_bytes(), 64);
                assert_eq!(allocator.deallocated_bytes(), 0);

                let ptr = allocator.realloc(ptr, layout(64), 16);
                assert!(!ptr.is_null());

                assert_eq!(allocator.reallocations(), 2);
                assert_eq!(allocator.deallocated_bytes(), 48);

                allocator.dealloc(ptr, layout(16));
                assert_eq!(allocator.deallocated_bytes(), 64);
            }
        }
    }

    // ── Free getters ──────────────────────────────────────────────────────────

    #[test]
    fn allocations() {
        let before = ALLOCATOR.allocations();

        let block = Box::new(0u8);
        let _ = std::hint::black_box(&block);

        assert!(super::allocations() > before);

        drop(block);
    }

    #[test]
    fn deallocations() {
        let block = Box::new(0u8);
        let before = ALLOCATOR.deallocations();

        drop(block);

        assert!(super::deallocations() > before);
    }

    #[test]
    fn reallocations() {
        let mut vec: Vec<u8> = Vec::with_capacity(1);
        let before = ALLOCATOR.reallocations();

        vec.reserve(4096);
        let _ = std::hint::black_box(&vec);

        assert!(super::reallocations() > before);
    }

    #[test]
    fn allocated_bytes() {
        let before = ALLOCATOR.allocated_bytes();

        let vec = vec![0u8; 1024];
        let _ = std::hint::black_box(&vec);

        assert!(super::allocated_bytes() >= before + 1024);
    }

    #[test]
    fn deallocated_bytes() {
        let vec = vec![0u8; 1024];
        let before = ALLOCATOR.deallocated_bytes();

        drop(vec);

        assert!(super::deallocated_bytes() >= before + 1024);
    }
}
