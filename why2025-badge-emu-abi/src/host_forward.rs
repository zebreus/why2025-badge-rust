use crate::runtime;
use crate::types::*;
use core::ffi::{c_char, c_int, c_uint, c_void};

#[unsafe(no_mangle)]
pub static mut stdin: *mut FILE = core::ptr::null_mut();

#[unsafe(no_mangle)]
pub static mut stdout: *mut FILE = core::ptr::null_mut();

#[unsafe(no_mangle)]
pub static mut stderr: *mut FILE = core::ptr::null_mut();

#[unsafe(no_mangle)]
pub static mut environ: *mut *mut c_char = core::ptr::null_mut();

#[used]
#[unsafe(link_section = ".init_array")]
static INIT_WRAPPED_OBJECTS: extern "C" fn() = init_wrapped_objects;

extern "C" fn init_wrapped_objects() {
    unsafe {
        stdin = runtime::resolve_next_object_value::<*mut FILE>(b"stdin\0");
        stdout = runtime::resolve_next_object_value::<*mut FILE>(b"stdout\0");
        stderr = runtime::resolve_next_object_value::<*mut FILE>(b"stderr\0");
        environ = runtime::resolve_next_object_value::<*mut *mut c_char>(b"environ\0");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn calloc(count: c_uint, size: c_uint) -> *mut c_void {
    let function: unsafe extern "C" fn(c_uint, c_uint) -> *mut c_void =
        unsafe { runtime::resolve_next_function(b"calloc\0") };
    unsafe { function(count, size) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn free(ptr: *mut c_void) {
    let function: unsafe extern "C" fn(*mut c_void) =
        unsafe { runtime::resolve_next_function(b"free\0") };
    unsafe { function(ptr) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fstat(fd: c_int, buf: *mut stat) -> c_int {
    let function: unsafe extern "C" fn(c_int, *mut stat) -> c_int =
        unsafe { runtime::resolve_next_function(b"fstat\0") };
    unsafe { function(fd, buf) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn getenv(name: *const c_char) -> *mut c_char {
    let function: unsafe extern "C" fn(*const c_char) -> *mut c_char =
        unsafe { runtime::resolve_next_function(b"getenv\0") };
    unsafe { function(name) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn getpid() -> pid_t {
    let function: unsafe extern "C" fn() -> pid_t =
        unsafe { runtime::resolve_next_function(b"getpid\0") };
    unsafe { function() }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn malloc(size: c_uint) -> *mut c_void {
    let function: unsafe extern "C" fn(c_uint) -> *mut c_void =
        unsafe { runtime::resolve_next_function(b"malloc\0") };
    unsafe { function(size) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn open(pathname: *const c_char, flags: c_int, mut args: ...) -> c_int {
    let function: unsafe extern "C" fn(*const c_char, c_int, ...) -> c_int =
        unsafe { runtime::resolve_next_function(b"open\0") };

    if flags & (libc::O_CREAT | libc::O_TMPFILE) != 0 {
        let mode: mode_t = unsafe { args.arg() };
        unsafe { function(pathname, flags, mode) }
    } else {
        unsafe { function(pathname, flags) }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn read(fd: c_int, buf: *mut c_void, nbyte: usize) -> isize {
    let function: unsafe extern "C" fn(c_int, *mut c_void, usize) -> isize =
        unsafe { runtime::resolve_next_function(b"read\0") };
    unsafe { function(fd, buf, nbyte) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn realloc(ptr: *mut c_void, size: c_uint) -> *mut c_void {
    let function: unsafe extern "C" fn(*mut c_void, c_uint) -> *mut c_void =
        unsafe { runtime::resolve_next_function(b"realloc\0") };
    unsafe { function(ptr, size) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn reallocarray(ptr: *mut c_void, nmemb: usize, size: usize) -> *mut c_void {
    let function: unsafe extern "C" fn(*mut c_void, usize, usize) -> *mut c_void =
        unsafe { runtime::resolve_next_function(b"reallocarray\0") };
    unsafe { function(ptr, nmemb, size) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn stat(path: *const c_char, buf: *mut stat) -> c_int {
    let function: unsafe extern "C" fn(*const c_char, *mut stat) -> c_int =
        unsafe { runtime::resolve_next_function(b"stat\0") };
    unsafe { function(path, buf) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn write(fd: c_int, buf: *const c_void, nbyte: usize) -> isize {
    let function: unsafe extern "C" fn(c_int, *const c_void, usize) -> isize =
        unsafe { runtime::resolve_next_function(b"write\0") };
    unsafe { function(fd, buf, nbyte) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapped_objects_initialize() {
        unsafe {
            init_wrapped_objects();
            assert!(!stdin.is_null());
            assert!(!stdout.is_null());
            assert!(!stderr.is_null());
            assert!(!environ.is_null());
        }
    }
}
