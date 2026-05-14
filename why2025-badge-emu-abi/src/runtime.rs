use core::cell::UnsafeCell;
use core::ffi::{c_char, c_int, c_long, c_void};
use core::mem;

struct PthreadKey(UnsafeCell<libc::pthread_key_t>);
struct PthreadOnce(UnsafeCell<libc::pthread_once_t>);

unsafe impl Sync for PthreadKey {}
unsafe impl Sync for PthreadOnce {}

static ERRNO_KEY: PthreadKey = PthreadKey(UnsafeCell::new(0));
static ERRNO_KEY_ONCE: PthreadOnce = PthreadOnce(UnsafeCell::new(libc::PTHREAD_ONCE_INIT));

unsafe extern "C" fn free_errno_slot(value: *mut c_void) {
    if !value.is_null() {
        unsafe {
            libc::free(value);
        }
    }
}

extern "C" fn init_errno_key() {
    let rc = unsafe { libc::pthread_key_create(ERRNO_KEY.0.get(), Some(free_errno_slot)) };
    if rc != 0 {
        abort_with_message(b"why2025-badge-emu-abi failed to initialize errno TLS\n")
    }
}

fn ensure_errno_key() -> libc::pthread_key_t {
    let rc = unsafe { libc::pthread_once(ERRNO_KEY_ONCE.0.get(), init_errno_key) };
    if rc != 0 {
        abort_with_message(b"why2025-badge-emu-abi failed to run errno TLS init\n")
    }

    unsafe { *ERRNO_KEY.0.get() }
}

fn errno_slot() -> *mut c_int {
    let key = ensure_errno_key();
    let existing = unsafe { libc::pthread_getspecific(key) }.cast::<c_int>();
    if !existing.is_null() {
        return existing;
    }

    let slot = unsafe { libc::malloc(mem::size_of::<c_int>().max(1)) }.cast::<c_int>();
    if slot.is_null() {
        abort_with_message(b"why2025-badge-emu-abi failed to allocate errno TLS\n")
    }

    unsafe {
        *slot = 0;
    }

    let rc = unsafe { libc::pthread_setspecific(key, slot.cast::<c_void>()) };
    if rc != 0 {
        unsafe {
            libc::free(slot.cast::<c_void>());
        }
        abort_with_message(b"why2025-badge-emu-abi failed to install errno TLS\n")
    }

    slot
}

pub fn __errno() -> *mut c_int {
    errno_slot()
}

pub fn set_errno(value: c_int) {
    unsafe {
        *__errno() = value;
    }
}

pub fn abort_unimplemented_symbol(symbol: &str, family: &str) -> ! {
    write_stderr(b"why2025-badge-emu-abi unsupported ");
    write_stderr(family.as_bytes());
    write_stderr(b" symbol: ");
    write_stderr(symbol.as_bytes());
    write_stderr(b"\n");
    abort_process()
}

pub fn abort_missing_host_symbol(symbol: &str) -> ! {
    write_stderr(b"why2025-badge-emu-abi could not resolve host symbol: ");
    write_stderr(symbol.as_bytes());
    write_stderr(b"\n");
    abort_process()
}

pub fn abort_with_message(message: &[u8]) -> ! {
    write_stderr(message);
    abort_process()
}

pub(crate) fn write_stderr(bytes: &[u8]) {
    if bytes.is_empty() {
        return;
    }

    unsafe {
        let _ = libc::syscall(
            libc::SYS_write as c_long,
            libc::STDERR_FILENO,
            bytes.as_ptr(),
            bytes.len(),
        );
    }
}

fn raw_syscall0(number: c_long) -> c_long {
    unsafe { libc::syscall(number) as c_long }
}

pub unsafe fn resolve_next_symbol(symbol: &'static [u8]) -> *mut c_void {
    unsafe {
        libc::dlerror();
        let resolved = libc::dlsym(libc::RTLD_NEXT, symbol.as_ptr().cast::<c_char>());
        let error = libc::dlerror();

        if error.is_null() && !resolved.is_null() {
            resolved
        } else {
            let name = core::str::from_utf8(&symbol[..symbol.len().saturating_sub(1)])
                .unwrap_or("<invalid>");
            abort_missing_host_symbol(name)
        }
    }
}

pub unsafe fn resolve_next_function<T: Copy>(symbol: &'static [u8]) -> T {
    let resolved = unsafe { resolve_next_symbol(symbol) };
    unsafe { core::mem::transmute_copy::<*mut c_void, T>(&resolved) }
}

pub unsafe fn resolve_next_object_value<T: Copy>(symbol: &'static [u8]) -> T {
    unsafe { *resolve_next_symbol(symbol).cast::<T>() }
}

fn raw_syscall1(number: c_long, arg0: c_long) -> c_long {
    unsafe { libc::syscall(number, arg0) as c_long }
}

fn raw_syscall3(number: c_long, arg0: c_long, arg1: c_long, arg2: c_long) -> c_long {
    unsafe { libc::syscall(number, arg0, arg1, arg2) as c_long }
}

pub fn abort_process() -> ! {
    let pid = raw_syscall0(libc::SYS_getpid as c_long);
    let tid = raw_syscall0(libc::SYS_gettid as c_long);
    let _ = raw_syscall3(
        libc::SYS_tgkill as c_long,
        pid,
        tid,
        libc::SIGABRT as c_long,
    );
    let _ = raw_syscall1(libc::SYS_exit_group as c_long, 134);

    loop {
        core::hint::spin_loop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errno_can_be_set() {
        set_errno(42);
        unsafe {
            assert_eq!(*__errno(), 42);
        }
        set_errno(0);
    }

    #[test]
    fn errno_is_thread_local() {
        set_errno(41);
        let main_slot = __errno() as usize;

        let (thread_slot, thread_errno) = std::thread::spawn(|| {
            set_errno(7);
            (__errno() as usize, unsafe { *__errno() })
        })
        .join()
        .expect("thread result");

        assert_eq!(unsafe { *__errno() }, 41);
        assert_eq!(thread_errno, 7);
        assert_ne!(main_slot, thread_slot);

        set_errno(0);
    }
}
