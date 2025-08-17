//! Provides implementations for some libc functions that are available on the badge, but not on linux
//!
//! Most implementations are just stubs that return an error or unimplemented, but some are implemented.
//! If you want to use these functions, please implement them and submit a PR.

use crate::types::*;
use crate::{atof, isascii};

type size_t = usize;

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
