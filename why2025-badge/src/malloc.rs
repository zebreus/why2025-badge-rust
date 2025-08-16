use core::{alloc::{GlobalAlloc, Layout}, ffi::c_void};

/// Memory Allocator using `malloc(3)` and `free(3)`.
///
/// Usage:
/// ```rust
/// use why2025_badge::malloc::Malloc;
///
/// #[global_allocator]
/// static ALLOC: Malloc = Malloc;
/// ```
pub struct Malloc;

unsafe impl GlobalAlloc for Malloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe {
            why2025_badge_sys::malloc(layout.size() as u32) as *mut u8
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        unsafe {
            why2025_badge_sys::free(ptr as *mut c_void);
        }
    }
}

