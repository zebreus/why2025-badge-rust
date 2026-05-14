use core::ffi::{c_char, c_int, c_uint, c_void};
use core::{ptr, slice};

type size_t = c_uint;

#[unsafe(no_mangle)]
pub extern "C" fn strlen(value: *const c_char) -> size_t {
    let mut len = 0usize;
    unsafe {
        while *value.add(len) != 0 {
            len += 1;
        }
    }
    len as size_t
}

#[unsafe(no_mangle)]
pub extern "C" fn memcmp(left: *const c_void, right: *const c_void, count: size_t) -> c_int {
    let left = left.cast::<u8>();
    let right = right.cast::<u8>();

    for offset in 0..count as usize {
        let lhs = unsafe { *left.add(offset) };
        let rhs = unsafe { *right.add(offset) };

        if lhs != rhs {
            return lhs as c_int - rhs as c_int;
        }
    }

    0
}

#[unsafe(no_mangle)]
pub extern "C" fn memcpy(dst: *mut c_void, src: *const c_void, count: size_t) -> *mut c_void {
    let dst_bytes = dst.cast::<u8>();
    let src_bytes = src.cast::<u8>();

    unsafe {
        for offset in 0..count as usize {
            *dst_bytes.add(offset) = *src_bytes.add(offset);
        }
    }
    dst
}

#[unsafe(no_mangle)]
pub extern "C" fn memmove(dst: *mut c_void, src: *const c_void, count: size_t) -> *mut c_void {
    let dst_bytes = dst.cast::<u8>();
    let src_bytes = src.cast::<u8>();
    let count = count as usize;

    unsafe {
        if (dst_bytes as usize) <= (src_bytes as usize)
            || (dst_bytes as usize) >= (src_bytes as usize).saturating_add(count)
        {
            for offset in 0..count {
                *dst_bytes.add(offset) = *src_bytes.add(offset);
            }
        } else {
            for offset in (0..count).rev() {
                *dst_bytes.add(offset) = *src_bytes.add(offset);
            }
        }
    }
    dst
}

#[unsafe(no_mangle)]
pub extern "C" fn memset(dst: *mut c_void, value: c_int, count: size_t) -> *mut c_void {
    let dst_bytes = dst.cast::<u8>();
    let value = value as u8;

    unsafe {
        for offset in 0..count as usize {
            *dst_bytes.add(offset) = value;
        }
    }
    dst
}

#[unsafe(no_mangle)]
pub extern "C" fn memchr(value: *const c_void, needle: c_int, count: size_t) -> *mut c_void {
    let bytes = unsafe { slice::from_raw_parts(value.cast::<u8>(), count as usize) };
    let needle = needle as u8;

    for (offset, byte) in bytes.iter().copied().enumerate() {
        if byte == needle {
            return unsafe { value.cast::<u8>().add(offset).cast_mut().cast::<c_void>() };
        }
    }

    ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "C" fn bzero(dst: *mut c_void, count: size_t) {
    let _ = memset(dst, 0, count);
}

#[unsafe(no_mangle)]
pub extern "C" fn explicit_bzero(dst: *mut c_void, count: size_t) {
    let dst_bytes = dst.cast::<u8>();

    unsafe {
        for offset in 0..count as usize {
            ptr::write_volatile(dst_bytes.add(offset), 0);
        }
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strlen_counts_to_nul() {
        assert_eq!(strlen(c"badge".as_ptr()), 5);
    }

    #[test]
    fn memory_helpers_copy_compare_and_find() {
        let source = *b"abcde";
        let mut destination = [0u8; 5];

        let copied = memcpy(
            destination.as_mut_ptr().cast::<c_void>(),
            source.as_ptr().cast::<c_void>(),
            source.len() as size_t,
        );

        assert_eq!(copied, destination.as_mut_ptr().cast::<c_void>());
        assert_eq!(destination, source);
        assert_eq!(
            memcmp(
                destination.as_ptr().cast::<c_void>(),
                source.as_ptr().cast::<c_void>(),
                source.len() as size_t,
            ),
            0,
        );

        let found = memchr(
            destination.as_ptr().cast::<c_void>(),
            b'c' as c_int,
            destination.len() as size_t,
        );
        assert_eq!(found, unsafe {
            destination.as_ptr().add(2).cast_mut().cast::<c_void>()
        });
    }

    #[test]
    fn memmove_handles_overlap() {
        let mut buffer = *b"abcdef";
        let src = buffer.as_ptr();
        let dst = unsafe { buffer.as_mut_ptr().add(2) };

        let moved = memmove(dst.cast::<c_void>(), src.cast::<c_void>(), 4);

        assert_eq!(moved, dst.cast::<c_void>());
        assert_eq!(&buffer, b"ababcd");
    }

    #[test]
    fn explicit_bzero_clears_bytes() {
        let mut buffer = *b"secret";
        explicit_bzero(buffer.as_mut_ptr().cast::<c_void>(), buffer.len() as size_t);
        assert_eq!(&buffer, b"\0\0\0\0\0\0");
    }
}
