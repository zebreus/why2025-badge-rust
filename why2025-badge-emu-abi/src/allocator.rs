use core::alloc::{GlobalAlloc, Layout};
use core::ffi::c_void;
use core::ptr;

struct LibcAllocator;

#[global_allocator]
static GLOBAL_ALLOCATOR: LibcAllocator = LibcAllocator;

unsafe impl GlobalAlloc for LibcAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size().max(1);
        let align = layout.align();

        if align <= core::mem::size_of::<usize>() {
            unsafe { libc::malloc(size).cast::<u8>() }
        } else {
            let mut out: *mut c_void = ptr::null_mut();
            let rc = unsafe { libc::posix_memalign(&mut out, align, size) };
            if rc == 0 {
                out.cast::<u8>()
            } else {
                ptr::null_mut()
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        unsafe {
            libc::free(ptr.cast::<c_void>());
        }
    }

    unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
        unsafe { libc::realloc(ptr.cast::<c_void>(), new_size.max(1)).cast::<u8>() }
    }
}
