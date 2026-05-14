use core::cell::UnsafeCell;
use core::ffi::{c_int, c_long};

struct ErrnoCell(UnsafeCell<c_int>);

unsafe impl Sync for ErrnoCell {}

static ERRNO: ErrnoCell = ErrnoCell(UnsafeCell::new(0));

pub fn __errno() -> *mut c_int {
    ERRNO.0.get()
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

pub fn abort_with_message(message: &[u8]) -> ! {
    write_stderr(message);
    abort_process()
}

fn write_stderr(bytes: &[u8]) {
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
}
