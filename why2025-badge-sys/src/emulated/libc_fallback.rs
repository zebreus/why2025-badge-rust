//! Provides implementations for some libc functions that are available on the badge, but not on linux
//!
//! Most implementations are just stubs that return an error or unimplemented, but some are implemented.
//! If you want to use these functions, please implement them and submit a PR.

use crate::types::*;
use crate::{atof, isascii};
use core::ffi::{CStr, c_char};
use std::{borrow::Cow, io::{self, Write}};

type size_t = usize;

fn c_string_or_placeholder<'a>(ptr: *const c_char, placeholder: &'a str) -> Cow<'a, str> {
    if ptr.is_null() {
        Cow::Borrowed(placeholder)
    } else {
        unsafe { CStr::from_ptr(ptr) }.to_string_lossy()
    }
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn atoff(__nptr: *const ::core::ffi::c_char) -> f32 {
    unsafe {
        let result = atof(__nptr);
        return result as f32;
    }
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn fls(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int {
    for i in (0..32).rev() {
        if (arg1 & (1 << i)) != 0 {
            return i + 1;
        }
    }
    0
}
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn flsl(arg1: ::core::ffi::c_long) -> ::core::ffi::c_int {
    for i in (0..64).rev() {
        if (arg1 & (1 << i)) != 0 {
            return i + 1;
        }
    }
    0
}
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn flsll(arg1: ::core::ffi::c_longlong) -> ::core::ffi::c_int {
    for i in (0..64).rev() {
        if (arg1 & (1 << i)) != 0 {
            return i + 1;
        }
    }
    0
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __assert_func(
    arg1: *const ::core::ffi::c_char,
    arg2: ::core::ffi::c_int,
    arg3: *const ::core::ffi::c_char,
    arg4: *const ::core::ffi::c_char,
) -> ! {
    let file = c_string_or_placeholder(arg1, "<unknown file>");
    let function = if arg3.is_null() {
        None
    } else {
        Some(c_string_or_placeholder(arg3, "<unknown function>"))
    };
    let expression = c_string_or_placeholder(arg4, "<unknown expression>");

    let mut stderr = io::stderr().lock();
    let _ = if let Some(function) = function {
        writeln!(
            stderr,
            "assertion \"{}\" failed: file \"{}\", line {}, function: {}",
            expression, file, arg2, function
        )
    } else {
        writeln!(
            stderr,
            "assertion \"{}\" failed: file \"{}\", line {}",
            expression, file, arg2
        )
    };

    std::process::abort()
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __floatundidf(a: u64) -> f64 {
    a as f64
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __floatundisf(a: u64) -> f32 {
    a as f32
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __issignalingf(f: f32) -> ::core::ffi::c_int {
    let bits = f.to_bits();
    let exponent = bits & 0x7f80_0000;
    let mantissa = bits & 0x007f_ffff;
    let quiet_bit = 0x0040_0000;

    (exponent == 0x7f80_0000 && mantissa != 0 && (mantissa & quiet_bit) == 0)
        as ::core::ffi::c_int
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __nedf2(a: f64, b: f64) -> ::core::ffi::c_int {
    match a.partial_cmp(&b) {
        Some(core::cmp::Ordering::Less) => -1,
        Some(core::cmp::Ordering::Equal) => 0,
        Some(core::cmp::Ordering::Greater) => 1,
        None => 1,
    }
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn fpgetround() -> fp_rnd {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn fpsetround(arg1: fp_rnd) -> fp_rnd {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn fpgetmask() -> fp_except {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn fpsetmask(arg1: fp_except) -> fp_except {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn fpgetsticky() -> fp_except {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn fpsetsticky(arg1: fp_except) -> fp_except {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn isascii_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    unsafe { isascii(c) }
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn itoa(
    arg1: ::core::ffi::c_int,
    arg2: *mut ::core::ffi::c_char,
    arg3: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn sig2str(
    signum: ::core::ffi::c_int,
    str_: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn str2sig(
    str_: *const ::core::ffi::c_char,
    pnum: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn strlwr(arg1: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn strnstr(
    arg1: *const ::core::ffi::c_char,
    arg2: *const ::core::ffi::c_char,
    arg3: size_t,
) -> *mut ::core::ffi::c_char {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn strtoimax_l(
    arg1: *const ::core::ffi::c_char,
    _restrict: *mut *mut ::core::ffi::c_char,
    arg2: ::core::ffi::c_int,
    arg3: locale_t,
) -> intmax_t {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn strtoumax_l(
    arg1: *const ::core::ffi::c_char,
    _restrict: *mut *mut ::core::ffi::c_char,
    arg2: ::core::ffi::c_int,
    arg3: locale_t,
) -> uintmax_t {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn strupr(arg1: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn timingsafe_bcmp(
    arg1: *const ::core::ffi::c_void,
    arg2: *const ::core::ffi::c_void,
    arg3: size_t,
) -> ::core::ffi::c_int {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn timingsafe_memcmp(
    arg1: *const ::core::ffi::c_void,
    arg2: *const ::core::ffi::c_void,
    arg3: size_t,
) -> ::core::ffi::c_int {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn toascii_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn utoa(
    arg1: ::core::ffi::c_uint,
    arg2: *mut ::core::ffi::c_char,
    arg3: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn wcstoimax_l(
    arg1: *const _wchar_t,
    _restrict: *mut *mut _wchar_t,
    arg2: ::core::ffi::c_int,
    arg3: locale_t,
) -> intmax_t {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn wcstoumax_l(
    arg1: *const _wchar_t,
    _restrict: *mut *mut _wchar_t,
    arg2: ::core::ffi::c_int,
    arg3: locale_t,
) -> uintmax_t {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn gammaf_r(arg1: f32, arg2: *mut ::core::ffi::c_int) -> f32 {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn gamma_r(arg1: f64, arg2: *mut ::core::ffi::c_int) -> f64 {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn infinity() -> f64 {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn infinityf() -> f32 {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn exp10(arg1: f64) -> f64 {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn pow10(arg1: f64) -> f64 {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn exp10f(arg1: f32) -> f32 {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn pow10f(arg1: f32) -> f32 {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}

unsafe extern "C" {
    #[link_name = "dprintf"]
    pub unsafe fn diprintf(
        a: ::core::ffi::c_int,
        b: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub unsafe extern "C" fn asnprintf(
    str_: *mut ::core::ffi::c_char,
    lenp: *mut size_t,
    fmt: *const ::core::ffi::c_char,
    ...
) -> *mut ::core::ffi::c_char {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn gcvtf(
    arg1: f32,
    arg2: ::core::ffi::c_int,
    arg3: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn gcvtl(
    arg1: u128,
    arg2: ::core::ffi::c_int,
    arg3: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn funopen(
    cookie: *const ::core::ffi::c_void,
    readfn: ::core::option::Option<
        unsafe extern "C" fn(
            cookie: *mut ::core::ffi::c_void,
            buf: *mut ::core::ffi::c_void,
            n: size_t,
        ) -> _ssize_t,
    >,
    writefn: ::core::option::Option<
        unsafe extern "C" fn(
            cookie: *mut ::core::ffi::c_void,
            buf: *const ::core::ffi::c_void,
            n: size_t,
        ) -> _ssize_t,
    >,
    seekfn: ::core::option::Option<
        unsafe extern "C" fn(
            cookie: *mut ::core::ffi::c_void,
            off: __off_t,
            whence: ::core::ffi::c_int,
        ) -> __off_t,
    >,
    closefn: ::core::option::Option<
        unsafe extern "C" fn(cookie: *mut ::core::ffi::c_void) -> ::core::ffi::c_int,
    >,
) -> *mut FILE {
    unimplemented!("If you need this function please open an issue or PR to implement it");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use std::{os::unix::process::ExitStatusExt, process::Command};

    #[test]
    fn floatundidf_matches_host_conversion() {
        assert_eq!(__floatundidf(0), 0.0);
        assert_eq!(__floatundidf(1), 1.0);
        assert_eq!(__floatundidf(1u64 << 63), (1u64 << 63) as f64);
        assert_eq!(__floatundidf(u64::MAX), u64::MAX as f64);
    }

    #[test]
    fn floatundisf_matches_host_conversion() {
        assert_eq!(__floatundisf(0), 0.0);
        assert_eq!(__floatundisf(1), 1.0);
        assert_eq!(__floatundisf(1u64 << 24), (1u64 << 24) as f32);
        assert_eq!(__floatundisf(u64::MAX), u64::MAX as f32);
    }

    #[test]
    fn nedf2_uses_compiler_rt_ordering() {
        assert_eq!(__nedf2(1.0, 2.0), -1);
        assert_eq!(__nedf2(2.0, 2.0), 0);
        assert_eq!(__nedf2(3.0, 2.0), 1);
        assert_eq!(__nedf2(f64::NAN, 2.0), 1);
        assert_eq!(__nedf2(2.0, f64::NAN), 1);
    }

    #[test]
    fn issignalingf_distinguishes_signaling_nan() {
        let signaling_nan = f32::from_bits(0x7f80_0001);
        let quiet_nan = f32::from_bits(0x7fc0_0000);

        assert_eq!(__issignalingf(signaling_nan), 1);
        assert_eq!(__issignalingf(quiet_nan), 0);
        assert_eq!(__issignalingf(f32::INFINITY), 0);
        assert_eq!(__issignalingf(1.0), 0);
    }

    #[test]
    #[cfg(unix)]
    fn assert_func_aborts() {
        const ENV_NAME: &str = "WHY2025_ASSERT_FUNC_TEST";
        const TEST_NAME: &str = "emulated::libc_fallback::tests::assert_func_aborts";

        if std::env::var_os(ENV_NAME).is_some() {
            __assert_func(
                b"test.c\0".as_ptr().cast(),
                42,
                b"demo_function\0".as_ptr().cast(),
                b"x != 0\0".as_ptr().cast(),
            );
        }

        let output = Command::new(std::env::current_exe().expect("current test binary path"))
            .arg("--exact")
            .arg(TEST_NAME)
            .env(ENV_NAME, "1")
            .output()
            .expect("spawn child test process");

        assert!(!output.status.success());
        assert_eq!(output.status.signal(), Some(libc::SIGABRT));

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("assertion \"x != 0\" failed"));
        assert!(stderr.contains("file \"test.c\", line 42"));
        assert!(stderr.contains("function: demo_function"));
    }
}
