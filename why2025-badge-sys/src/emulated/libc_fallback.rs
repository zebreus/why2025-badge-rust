//! Provides implementations for some libc functions that are available on the badge, but not on linux
//!
//! Most implementations are just stubs that return an error or unimplemented, but some are implemented.
//! If you want to use these functions, please implement them and submit a PR.
//!
//! The rustdoc on the remaining unimplemented stubs describes the checked-in upstream firmware tree
//! that this crate vendors under `why2025-badge-sys/firmware/` at commit
//! `a548d825a3295432d374939607feb552eb505210` (`Update espressif/eppp_link`).
//!
//! A few symbols are backed by small `why_stdio` implementations in that tree, some are only
//! header macros, and many are only declared or listed in `badgevms/symbols.yml` without any
//! project-local implementation. Where the repository does not expose the actual badge-side code,
//! the docs say so explicitly instead of inferring behavior from the function name alone.

use crate::types::*;

mod runtime;

type size_t = usize;

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __errno() -> *mut ::core::ffi::c_int {
    runtime::__errno()
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Differences from upstream BadgeVMS:
/// - Upstream `why_atexit` only logs and returns `0`; it does not register the callback.
/// - The host fallback registers a real host exit handler through `__cxa_atexit`, so callbacks do run at host process exit.
pub extern "C" fn atexit(
    __func: ::core::option::Option<unsafe extern "C" fn()>,
) -> ::core::ffi::c_int {
    runtime::atexit(__func)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub static _ctype_b: [::core::ffi::c_char; 0usize] = [];

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn atoff(__nptr: *const ::core::ffi::c_char) -> f32 {
    runtime::atoff(__nptr)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn fls(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int {
    runtime::fls(arg1)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn flsl(arg1: ::core::ffi::c_long) -> ::core::ffi::c_int {
    runtime::flsl(arg1)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn flsll(arg1: ::core::ffi::c_longlong) -> ::core::ffi::c_int {
    runtime::flsll(arg1)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __assert_func(
    arg1: *const ::core::ffi::c_char,
    arg2: ::core::ffi::c_int,
    arg3: *const ::core::ffi::c_char,
    arg4: *const ::core::ffi::c_char,
) -> ! {
    runtime::__assert_func(arg1, arg2, arg3, arg4)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __adddf3(a: f64, b: f64) -> f64 {
    runtime::__adddf3(a, b)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __subdf3(a: f64, b: f64) -> f64 {
    runtime::__subdf3(a, b)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __muldf3(a: f64, b: f64) -> f64 {
    runtime::__muldf3(a, b)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __divdf3(a: f64, b: f64) -> f64 {
    runtime::__divdf3(a, b)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __eqdf2(a: f64, b: f64) -> ::core::ffi::c_int {
    runtime::__eqdf2(a, b)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __gedf2(a: f64, b: f64) -> ::core::ffi::c_int {
    runtime::__gedf2(a, b)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __gtdf2(a: f64, b: f64) -> ::core::ffi::c_int {
    runtime::__gtdf2(a, b)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __ledf2(a: f64, b: f64) -> ::core::ffi::c_int {
    runtime::__ledf2(a, b)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __ltdf2(a: f64, b: f64) -> ::core::ffi::c_int {
    runtime::__ltdf2(a, b)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __extendsfdf2(a: f32) -> f64 {
    runtime::__extendsfdf2(a)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __truncdfsf2(a: f64) -> f32 {
    runtime::__truncdfsf2(a)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __extendhfsf2(a: __BindgenFloat16) -> f32 {
    runtime::__extendhfsf2(a)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __truncsfhf2(a: f32) -> __BindgenFloat16 {
    runtime::__truncsfhf2(a)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __fixdfsi(a: f64) -> i32 {
    runtime::__fixdfsi(a)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __fixdfdi(a: f64) -> i64 {
    runtime::__fixdfdi(a)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __fixunsdfsi(a: f64) -> u32 {
    runtime::__fixunsdfsi(a)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __floatdisf(a: i64) -> f32 {
    runtime::__floatdisf(a)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __floatsidf(a: i32) -> f64 {
    runtime::__floatsidf(a)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __floatundidf(a: u64) -> f64 {
    runtime::__floatundidf(a)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __floatundisf(a: u64) -> f32 {
    runtime::__floatundisf(a)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __floatunsidf(a: u32) -> f64 {
    runtime::__floatunsidf(a)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __issignalingf(f: f32) -> ::core::ffi::c_int {
    runtime::__issignalingf(f)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __nedf2(a: f64, b: f64) -> ::core::ffi::c_int {
    runtime::__nedf2(a, b)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __divdi3(a: i64, b: i64) -> i64 {
    runtime::__divdi3(a, b)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __udivdi3(a: u64, b: u64) -> u64 {
    runtime::__udivdi3(a, b)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __umoddi3(a: u64, b: u64) -> u64 {
    runtime::__umoddi3(a, b)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __clzsi2(a: u32) -> ::core::ffi::c_int {
    runtime::__clzsi2(a)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn __popcountsi2(a: u32) -> ::core::ffi::c_int {
    runtime::__popcountsi2(a)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Query the floating-point rounding mode.
///
/// # Upstream status
///
/// In the checked-in firmware tree this symbol is only declared in
/// `firmware/components/why_stdio/include/ieeefp.h` behind `__BSD_VISIBLE`.
/// That header defines `fp_rnd` as `int` with these constants:
///
/// - `FP_RN = 0` for round-to-nearest
/// - `FP_RM = 1` for round-down
/// - `FP_RP = 2` for round-up
/// - `FP_RZ = 3` for round-to-zero
///
/// No project-local implementation or caller was found under `firmware/`.
/// If the badge image resolves `fpgetround()` at link time, that definition comes from the
/// external libc or toolchain rather than project code in this repository.
///
/// Repository evidence therefore stops at the ABI surface: the tree does not reveal whether the
/// runtime touches hardware FP control state, whether it is thread-local, or whether failures are
/// representable at all.
pub extern "C" fn fpgetround() -> fp_rnd {
    runtime::fpgetround()
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Set the floating-point rounding mode.
///
/// # Upstream status
///
/// The checked-in firmware tree declares this setter next to `fpgetround()` in
/// `firmware/components/why_stdio/include/ieeefp.h`, again behind `__BSD_VISIBLE`, but ships no
/// project-local definition.
///
/// The accepted `fp_rnd` values are the same header constants `FP_RN`, `FP_RM`, `FP_RP`, and
/// `FP_RZ`. Beyond that, the repository does not expose real runtime behavior: it does not show
/// whether invalid values are ignored, masked, trapped, or normalized, nor whether the return value
/// is the previous mode, the new mode, or an error sentinel.
///
/// Any functioning badge-side implementation would have to come from external libc or libm rather
/// than the project-local firmware sources vendored here.
pub extern "C" fn fpsetround(arg1: fp_rnd) -> fp_rnd {
    runtime::fpsetround(arg1)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Query the floating-point exception mask.
///
/// # Upstream status
///
/// The checked-in tree only declares this symbol in
/// `firmware/components/why_stdio/include/ieeefp.h` behind `__BSD_VISIBLE`.
/// `fp_except` is an `int`, and the same header exposes these bit values:
///
/// - `FP_X_INV = 0x10`
/// - `FP_X_DX = 0x80`
/// - `FP_X_OFL = 0x04`
/// - `FP_X_UFL = 0x02`
/// - `FP_X_IMP = 0x01`
///
/// No implementation was found in project-local firmware code, so the repository does not reveal
/// whether these bits map to real hardware exception masks, a software shadow register, or a stub
/// implementation supplied by external libc.
pub extern "C" fn fpgetmask() -> fp_except {
    runtime::fpgetmask()
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Set the floating-point exception mask.
///
/// # Upstream status
///
/// This symbol is only declared in `firmware/components/why_stdio/include/ieeefp.h` and has no
/// project-local definition in the checked-in firmware tree.
///
/// The exposed mask bits are `FP_X_INV`, `FP_X_DX`, `FP_X_OFL`, `FP_X_UFL`, and `FP_X_IMP`, but
/// the tree does not say whether setting unsupported bits is ignored or rejected, whether the
/// function returns the old mask or the effective new mask, or whether the implementation is a real
/// hardware control path at all.
///
/// If the symbol exists on the badge, it is resolved outside this repository's firmware sources.
pub extern "C" fn fpsetmask(arg1: fp_except) -> fp_except {
    runtime::fpsetmask(arg1)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Query floating-point sticky exception flags.
///
/// # Upstream status
///
/// The checked-in firmware tree declares this symbol in
/// `firmware/components/why_stdio/include/ieeefp.h` behind `__BSD_VISIBLE` and does not provide a
/// corresponding project-local implementation.
///
/// The visible bit layout uses the same `fp_except` constants as the exception-mask APIs, but the
/// repository does not reveal whether these flags are latched in hardware, synthesized in software,
/// or never updated at all.
pub extern "C" fn fpgetsticky() -> fp_except {
    runtime::fpgetsticky()
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Set floating-point sticky exception flags.
///
/// # Upstream status
///
/// This is declared in `firmware/components/why_stdio/include/ieeefp.h` but has no project-local
/// definition in the checked-in firmware tree.
///
/// The repository therefore does not expose whether writing sticky bits clears them, ORs them in,
/// replaces them wholesale, or is unsupported on the actual badge runtime. Any real implementation
/// would have to come from external libc rather than from vendored project code.
pub extern "C" fn fpsetsticky(arg1: fp_except) -> fp_except {
    runtime::fpsetsticky(arg1)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Exact upstream behavior
///
/// `firmware/sdk_include/ctype.h` defines this as an inline wrapper that discards `locale_t` and
/// delegates to `isalnum(c)`. The vendored tree ships no out-of-line definition, so this fallback
/// only matters for symbol-level ABI consumers that bypass the inline.
pub extern "C" fn isalnum_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    runtime::isalnum_l(c, l)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Exact upstream behavior
///
/// `firmware/sdk_include/ctype.h` defines this as an inline wrapper that discards `locale_t` and
/// delegates to `isalpha(c)`. The vendored tree ships no out-of-line definition, so this fallback
/// only matters for symbol-level ABI consumers that bypass the inline.
pub extern "C" fn isalpha_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    runtime::isalpha_l(c, l)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Exact upstream behavior
///
/// `firmware/sdk_include/ctype.h` defines this as an inline wrapper that discards `locale_t` and
/// delegates to `isblank(c)`. The vendored tree ships no out-of-line definition, so this fallback
/// only matters for symbol-level ABI consumers that bypass the inline.
pub extern "C" fn isblank_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    runtime::isblank_l(c, l)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Exact upstream behavior
///
/// `firmware/sdk_include/ctype.h` defines this as an inline wrapper that discards `locale_t` and
/// delegates to `iscntrl(c)`. The vendored tree ships no out-of-line definition, so this fallback
/// only matters for symbol-level ABI consumers that bypass the inline.
pub extern "C" fn iscntrl_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    runtime::iscntrl_l(c, l)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Exact upstream behavior
///
/// `firmware/sdk_include/ctype.h` defines this as an inline wrapper that discards `locale_t` and
/// delegates to `isdigit(c)`. The vendored tree ships no out-of-line definition, so this fallback
/// only matters for symbol-level ABI consumers that bypass the inline.
pub extern "C" fn isdigit_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    runtime::isdigit_l(c, l)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Exact upstream behavior
///
/// `firmware/sdk_include/ctype.h` defines this as an inline wrapper that discards `locale_t` and
/// delegates to `isgraph(c)`. The vendored tree ships no out-of-line definition, so this fallback
/// only matters for symbol-level ABI consumers that bypass the inline.
pub extern "C" fn isgraph_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    runtime::isgraph_l(c, l)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Exact upstream behavior
///
/// `firmware/sdk_include/ctype.h` defines this as an inline wrapper that discards `locale_t` and
/// delegates to `islower(c)`. The vendored tree ships no out-of-line definition, so this fallback
/// only matters for symbol-level ABI consumers that bypass the inline.
pub extern "C" fn islower_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    runtime::islower_l(c, l)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Exact upstream behavior
///
/// `firmware/sdk_include/ctype.h` defines this as an inline wrapper that discards `locale_t` and
/// delegates to `isprint(c)`. The vendored tree ships no out-of-line definition, so this fallback
/// only matters for symbol-level ABI consumers that bypass the inline.
pub extern "C" fn isprint_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    runtime::isprint_l(c, l)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Exact upstream behavior
///
/// `firmware/sdk_include/ctype.h` defines this as an inline wrapper that discards `locale_t` and
/// delegates to `ispunct(c)`. The vendored tree ships no out-of-line definition, so this fallback
/// only matters for symbol-level ABI consumers that bypass the inline.
pub extern "C" fn ispunct_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    runtime::ispunct_l(c, l)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Exact upstream behavior
///
/// `firmware/sdk_include/ctype.h` defines this as an inline wrapper that discards `locale_t` and
/// delegates to `isspace(c)`. The vendored tree ships no out-of-line definition, so this fallback
/// only matters for symbol-level ABI consumers that bypass the inline.
pub extern "C" fn isspace_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    runtime::isspace_l(c, l)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Exact upstream behavior
///
/// `firmware/sdk_include/ctype.h` defines this as an inline wrapper that discards `locale_t` and
/// delegates to `isupper(c)`. The vendored tree ships no out-of-line definition, so this fallback
/// only matters for symbol-level ABI consumers that bypass the inline.
pub extern "C" fn isupper_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    runtime::isupper_l(c, l)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Exact upstream behavior
///
/// `firmware/sdk_include/ctype.h` defines this as an inline wrapper that discards `locale_t` and
/// delegates to `isxdigit(c)`. The vendored tree ships no out-of-line definition, so this fallback
/// only matters for symbol-level ABI consumers that bypass the inline.
pub extern "C" fn isxdigit_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    runtime::isxdigit_l(c, l)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Exact upstream behavior
///
/// `firmware/sdk_include/ctype.h` defines this as an inline wrapper that discards `locale_t` and
/// delegates to `tolower(c)`. The vendored tree ships no out-of-line definition, so this fallback
/// only matters for symbol-level ABI consumers that bypass the inline.
pub extern "C" fn tolower_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    runtime::tolower_l(c, l)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Exact upstream behavior
///
/// `firmware/sdk_include/ctype.h` defines this as an inline wrapper that discards `locale_t` and
/// delegates to `toupper(c)`. The vendored tree ships no out-of-line definition, so this fallback
/// only matters for symbol-level ABI consumers that bypass the inline.
pub extern "C" fn toupper_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    runtime::toupper_l(c, l)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Exact upstream behavior
///
/// `firmware/sdk_include/ctype.h` declares `isascii_l` and defines it as a macro that discards
/// `locale_t` and checks whether the input fits in 7-bit ASCII. The vendored tree ships no
/// out-of-line definition, so this fallback only matters for symbol-level ABI consumers that bypass
/// the macro.
pub extern "C" fn isascii_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    runtime::isascii_l(c, l)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Convert an `int` to text using a caller-provided radix and output buffer.
///
/// # Upstream status
///
/// The checked-in firmware tree only declares `itoa(int, char *, int)` in
/// `firmware/components/why_stdio/include/stdlib.h` behind `__MISC_VISIBLE`.
/// No project-local implementation was found under `firmware/`.
///
/// That declaration still matters because SDL's compatibility layer forwards `SDL_itoa()` to
/// `itoa()` when this symbol is available. Any exact formatting quirks therefore propagate into SDL
/// consumers, but those quirks are not inspectable in this repository.
///
/// The tree does not reveal supported radix ranges, how negative numbers are formatted outside base
/// 10, whether invalid bases return `NULL` or scribble anyway, or whether the implementation writes
/// through a null buffer pointer. Those details are external to the vendored firmware sources.
pub extern "C" fn itoa(
    arg1: ::core::ffi::c_int,
    arg2: *mut ::core::ffi::c_char,
    arg3: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    runtime::itoa(arg1, arg2, arg3)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Convert a signal number to text.
///
/// # Upstream status
///
/// The checked-in firmware tree declares this function in
/// `firmware/components/why_stdio/include/signal.h` behind `__MISC_VISIBLE` and defines
/// `SIG2STR_MAX` in the same header as either `17` or `12` depending on `sizeof(int)`.
/// No project-local implementation was found.
///
/// That means the repository does not expose the actual name table, case conventions, realtime
/// signal spelling, return codes, or NUL-termination guarantees. The only project-local facts are
/// the ABI shape and the advertised output-buffer size macro.
pub extern "C" fn sig2str(
    signum: ::core::ffi::c_int,
    str_: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    runtime::sig2str(signum, str_)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Parse a signal name back into a signal number.
///
/// # Upstream status
///
/// This symbol is only declared in `firmware/components/why_stdio/include/signal.h` behind
/// `__MISC_VISIBLE`; the checked-in firmware tree does not provide project-local code for it.
///
/// As a result, the repository does not reveal which spellings are accepted, whether names are
/// case-sensitive, whether numeric strings are allowed, or how realtime signal forms are parsed.
/// Any real behavior would come from an external libc implementation rather than from project code
/// shipped in this workspace.
pub extern "C" fn str2sig(
    str_: *const ::core::ffi::c_char,
    pnum: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    runtime::str2sig(str_, pnum)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Lowercase a mutable C string in place.
///
/// # Upstream status
///
/// The checked-in firmware tree declares `strlwr(char *)` in
/// `firmware/components/why_stdio/include/string.h` behind `__MISC_VISIBLE`, but no project-local
/// implementation exists under `firmware/`.
///
/// SDL ships its own `SDL_strlwr()` wrapper that forwards to `strlwr()` when available, so any
/// libc behavior here leaks into SDL consumers as well. The repository still does not expose the
/// exact badge-side rules: ASCII-only versus locale-aware conversion, UTF-8 blindness, null
/// handling, and write-in-place edge cases are all outside the checked-in project code.
pub extern "C" fn strlwr(arg1: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    runtime::strlwr(arg1)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Search for a substring inside a byte limit.
///
/// # Upstream status
///
/// The checked-in firmware tree declares `strnstr(const char *, const char *, size_t)` in
/// `firmware/components/why_stdio/include/string.h` behind `__BSD_VISIBLE` and marks it `__pure`,
/// but does not ship a project-local implementation.
///
/// SDL3's `SDL_strnstr()` wrapper forwards to this symbol when present, so any exact libc behavior
/// becomes observable through SDL as well. From repository evidence alone, the semantics of an
/// empty needle, truncated haystacks, null pointers, and out-of-range `maxlen` values remain
/// uninspectable.
pub extern "C" fn strnstr(
    arg1: *const ::core::ffi::c_char,
    arg2: *const ::core::ffi::c_char,
    arg3: size_t,
) -> *mut ::core::ffi::c_char {
    runtime::strnstr(arg1, arg2, arg3)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Parse a narrow string into `intmax_t` with an explicit locale argument.
///
/// # Upstream status
///
/// The checked-in firmware tree declares this symbol in
/// `firmware/components/why_stdio/include/inttypes.h` behind `__BSD_VISIBLE`.
/// It also ships a dormant implementation in `firmware/components/why_stdio/strtoimax_l.c`, but
/// `firmware/components/why_stdio/CMakeLists.txt` comments that file out, so the vendored
/// `why_stdio` component does not compile it into the badge firmware build.
///
/// The dormant source is tiny and exact:
///
/// - it casts the `locale_t` argument to void
/// - it ignores locale completely
/// - it returns `strtoimax(s, ptr, base)` unchanged
///
/// In other words, the only in-tree implementation path is a pure wrapper around the non-`_l`
/// variant. Because that wrapper is not built, the actual badge-side symbol, if one exists at all,
/// must come from external libc rather than from project-local `why_stdio` code.
pub extern "C" fn strtoimax_l(
    arg1: *const ::core::ffi::c_char,
    _restrict: *mut *mut ::core::ffi::c_char,
    arg2: ::core::ffi::c_int,
    arg3: locale_t,
) -> intmax_t {
    runtime::strtoimax_l(arg1, _restrict, arg2, arg3)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Parse a narrow string into `uintmax_t` with an explicit locale argument.
///
/// # Upstream status
///
/// This follows the same pattern as `strtoimax_l()`: the symbol is declared in
/// `firmware/components/why_stdio/include/inttypes.h`, an implementation exists in
/// `firmware/components/why_stdio/strtoumax_l.c`, and `why_stdio/CMakeLists.txt` comments that
/// source file out so it is not compiled into the checked-in firmware component.
///
/// The dormant source does exactly two things beyond forwarding:
///
/// - `(void) loc` to discard the locale argument
/// - `return strtoumax(s, ptr, base)`
///
/// So the only project-local behavior visible in-tree is "ignore locale and delegate". Whether the
/// actual badge runtime exposes a working symbol depends on external libc, not on vendored project
/// code.
pub extern "C" fn strtoumax_l(
    arg1: *const ::core::ffi::c_char,
    _restrict: *mut *mut ::core::ffi::c_char,
    arg2: ::core::ffi::c_int,
    arg3: locale_t,
) -> uintmax_t {
    runtime::strtoumax_l(arg1, _restrict, arg2, arg3)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Uppercase a mutable C string in place.
///
/// # Upstream status
///
/// The checked-in firmware tree declares `strupr(char *)` in
/// `firmware/components/why_stdio/include/string.h` behind `__MISC_VISIBLE`, but does not provide a
/// project-local implementation.
///
/// SDL's compatibility layer forwards `SDL_strupr()` to `strupr()` when available, so any libc
/// quirks become visible through SDL callers as well. The repository itself does not reveal whether
/// conversion is ASCII-only, locale-sensitive, UTF-8 unaware, or tolerant of null pointers.
pub extern "C" fn strupr(arg1: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    runtime::strupr(arg1)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Compare two byte sequences in a timing-safe manner and return whether they differ.
///
/// # Upstream status
///
/// The checked-in firmware tree only declares this BSD extension in
/// `firmware/components/why_stdio/include/string.h`; no project-local implementation was found.
///
/// The repository therefore cannot confirm the exact return convention, whether the implementation
/// is actually constant-time on the badge, or how it behaves on null pointers and zero-length
/// inputs. Only the ABI surface is visible here.
pub extern "C" fn timingsafe_bcmp(
    arg1: *const ::core::ffi::c_void,
    arg2: *const ::core::ffi::c_void,
    arg3: size_t,
) -> ::core::ffi::c_int {
    runtime::timingsafe_bcmp(arg1, arg2, arg3)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Compare two byte sequences in a timing-safe manner and return an ordering-like `int`.
///
/// # Upstream status
///
/// This symbol is only declared in `firmware/components/why_stdio/include/string.h` under
/// `__BSD_VISIBLE`; the checked-in firmware tree ships no project-local definition.
///
/// Because the implementation is not present in-repo, the exact relation between its return value
/// and lexicographic order, equality, or constant-time guarantees cannot be derived here.
pub extern "C" fn timingsafe_memcmp(
    arg1: *const ::core::ffi::c_void,
    arg2: *const ::core::ffi::c_void,
    arg3: size_t,
) -> ::core::ffi::c_int {
    runtime::timingsafe_memcmp(arg1, arg2, arg3)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Return the low 7 bits of a character code while ignoring the locale argument.
///
/// # Exact upstream behavior
///
/// In the checked-in firmware tree this is not really a function. `firmware/sdk_include/ctype.h`
/// declares `toascii_l(int, locale_t)` behind `__MISC_VISIBLE` and then immediately defines it as
/// this macro:
///
/// - `#define toascii_l(__c,__l) ((void) (__l),(__c)&0177)`
///
/// Consequences of that exact macro expansion:
///
/// - the locale argument is evaluated once and discarded
/// - the character expression is evaluated once
/// - the result is simply `c & 0x7f`
/// - no function call occurs in normal C code that includes the header
/// - no locale data is consulted
///
/// The checked-in firmware tree does not provide a project-local out-of-line definition, so this
/// host fallback only matters for symbol-level ABI consumers that bypass the macro.
pub extern "C" fn toascii_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    runtime::toascii_l(c, l)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Convert an `unsigned int` to text using a caller-provided radix and output buffer.
///
/// # Upstream status
///
/// The checked-in firmware tree declares `utoa(unsigned, char *, int)` in
/// `firmware/components/why_stdio/include/stdlib.h` behind `__MISC_VISIBLE`, alongside `__utoa()`,
/// but does not provide a project-local implementation.
///
/// No in-tree callers were found under `firmware/badgevms`, and SDL's wrapper code routes through
/// `_uitoa` rather than `utoa`, so the repository exposes even less behavioral evidence here than
/// for `itoa()`. Supported bases, formatting rules, null handling, and buffer write requirements are
/// all external to the vendored project sources.
pub extern "C" fn utoa(
    arg1: ::core::ffi::c_uint,
    arg2: *mut ::core::ffi::c_char,
    arg3: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    runtime::utoa(arg1, arg2, arg3)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Parse a wide string into `intmax_t` with an explicit locale argument.
///
/// # Upstream status
///
/// The checked-in firmware tree declares this symbol in
/// `firmware/components/why_stdio/include/inttypes.h` and ships a dormant source file,
/// `firmware/components/why_stdio/wcstoimax_l.c`, that is generated by defining `WIDE_CHARS` and
/// then including `strtoimax_l.c`.
///
/// That dormant implementation therefore behaves exactly like the narrow version:
///
/// - it ignores the locale with `(void) loc`
/// - it delegates directly to `wcstoimax(s, ptr, base)`
///
/// The file is commented out in `why_stdio/CMakeLists.txt`, so this wrapper is not compiled into
/// the vendored badge firmware component. Any real runtime definition would have to come from
/// external libc instead.
pub extern "C" fn wcstoimax_l(
    arg1: *const _wchar_t,
    _restrict: *mut *mut _wchar_t,
    arg2: ::core::ffi::c_int,
    arg3: locale_t,
) -> intmax_t {
    runtime::wcstoimax_l(arg1, _restrict, arg2, arg3)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Parse a wide string into `uintmax_t` with an explicit locale argument.
///
/// # Upstream status
///
/// This matches `wcstoimax_l()` structurally. The checked-in tree declares the symbol in
/// `firmware/components/why_stdio/include/inttypes.h` and ships a dormant
/// `firmware/components/why_stdio/wcstoumax_l.c` that defines `WIDE_CHARS` and includes
/// `strtoumax_l.c`.
///
/// The dormant code therefore has one observable quirk: locale is ignored entirely and the call is
/// forwarded unchanged to `wcstoumax(s, ptr, base)`. The file is not compiled into the vendored
/// `why_stdio` component, so this wrapper is a source-level hint, not active project-local badge
/// behavior.
pub extern "C" fn wcstoumax_l(
    arg1: *const _wchar_t,
    _restrict: *mut *mut _wchar_t,
    arg2: ::core::ffi::c_int,
    arg3: locale_t,
) -> uintmax_t {
    runtime::wcstoumax_l(arg1, _restrict, arg2, arg3)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Compute the reentrant single-precision gamma helper.
///
/// # Upstream status
///
/// The checked-in firmware tree does not provide a public declaration or project-local definition
/// for `gammaf_r()`. The symbol only appears in `firmware/badgevms/symbols.yml`, which is the
/// export-manifest input used for the generated BadgeVMS bindings.
///
/// Related symbols do exist in-tree:
///
/// - `lgamma_r()` and `lgammaf_r()` are declared in `firmware/sdk_include/math.h`
/// - SDL's bundled libm sources declare hidden internal helpers like `__ieee754_gamma_r`
///
/// But none of that yields an inspectable public `gammaf_r()` implementation for the badge. The
/// repository therefore cannot confirm sign handling through `arg2`, `errno` behavior, pole/NaN
/// handling, or whether the export manifest is stale.
pub extern "C" fn gammaf_r(arg1: f32, arg2: *mut ::core::ffi::c_int) -> f32 {
    runtime::gammaf_r(arg1, arg2)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Compute the reentrant double-precision gamma helper.
///
/// # Upstream status
///
/// As with `gammaf_r()`, the checked-in firmware tree only mentions this symbol in
/// `firmware/badgevms/symbols.yml`. No project-local header declaration or public implementation was
/// found under `firmware/components/why_stdio` or `firmware/sdk_include`.
///
/// The nearest in-tree relatives are `lgamma_r()` declarations and hidden SDL libm internals such
/// as `__ieee754_gamma_r`, which are not enough to reconstruct the exact badge-side ABI contract.
/// The repository therefore does not reveal how `arg2` is written, what happens on overflow or
/// poles, or whether the exported symbol is actually backed by the toolchain at runtime.
pub extern "C" fn gamma_r(arg1: f64, arg2: *mut ::core::ffi::c_int) -> f64 {
    runtime::gamma_r(arg1, arg2)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Return positive infinity as a `double`.
///
/// # Upstream status
///
/// The checked-in firmware tree declares `infinity(void)` in `firmware/sdk_include/math.h` and the
/// mirrored `components/why_stdio/include/math.h` as an older non-ANSI extension, but no
/// project-local implementation file exists.
///
/// The same headers also expose the compile-time macro `INFINITY`, so most callers can obtain an
/// infinite value without ever calling this function. The repository does not reveal whether an
/// actual badge-side symbol returns a canonical IEEE-754 `+inf`, forwards to a toolchain helper, or
/// is absent at link time.
pub extern "C" fn infinity() -> f64 {
    runtime::infinity()
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Return positive infinity as a `float`.
///
/// # Upstream status
///
/// The checked-in firmware tree declares `infinityf(void)` in `firmware/sdk_include/math.h` and
/// mirrors that declaration under `components/why_stdio/include/math.h`. No project-local source was
/// found.
///
/// `badgevms/symbols.yml` does list `infinityf`, so bindings expect the symbol to exist, but the
/// repository does not show where it comes from. As with `infinity()`, most code can instead use
/// the `INFINITY` macro, which avoids any symbol call entirely.
pub extern "C" fn infinityf() -> f32 {
    runtime::infinityf()
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Compute $10^x$ in double precision.
///
/// # Upstream status
///
/// The checked-in firmware tree declares this GNU extension in `firmware/sdk_include/math.h` and
/// lists it in `firmware/badgevms/symbols.yml`, but does not ship a project-local implementation.
///
/// There is one relevant interaction in `firmware/sdk_include/machine/fastmath.h`: on the unrelated
/// `__sysvnecv70_target` only, `exp10(x)` is macro-rewritten to `fast_exp10(x)`. That target guard
/// does not apply to the badge firmware here, so the vendored tree offers no active in-repo body.
///
/// The repository therefore cannot confirm rounding behavior, errno handling, overflow thresholds,
/// or whether this symbol is provided by the external libm toolchain on the badge.
pub extern "C" fn exp10(arg1: f64) -> f64 {
    runtime::exp10(arg1)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Compute $10^x$ in double precision through the historical `pow10` name.
///
/// # Upstream status
///
/// The checked-in firmware tree declares this GNU extension in `firmware/sdk_include/math.h` and
/// lists it in `firmware/badgevms/symbols.yml`, but does not provide a project-local implementation
/// or macro alias for the badge target.
///
/// Unlike `exp10`, no in-tree target-specific fastmath macro redirects `pow10()` here. Exact badge
/// behavior therefore remains external to the repository: this could be a toolchain libm symbol, an
/// alias to `exp10()`, or an unresolved export manifest entry.
pub extern "C" fn pow10(arg1: f64) -> f64 {
    runtime::pow10(arg1)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Compute $10^x$ in single precision.
///
/// # Upstream status
///
/// The checked-in firmware tree declares this GNU extension in `firmware/sdk_include/math.h` and
/// lists it in `firmware/badgevms/symbols.yml`, but does not ship a project-local implementation.
///
/// `firmware/sdk_include/machine/fastmath.h` does contain a macro form of `exp10f(x)` for the
/// unrelated `__sysvnecv70_target`, which is not the badge target in this repository. For the badge
/// tree actually vendored here, the implementation remains external and uninspectable.
pub extern "C" fn exp10f(arg1: f32) -> f32 {
    runtime::exp10f(arg1)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Compute $10^x$ in single precision through the historical `pow10f` name.
///
/// # Upstream status
///
/// The checked-in firmware tree declares this symbol in `firmware/sdk_include/math.h` and lists it
/// in `firmware/badgevms/symbols.yml`, but provides no project-local implementation.
///
/// The repository therefore cannot reveal whether the badge treats this as an alias of `exp10f()`,
/// an independent libm entry point, or merely a stale export declaration.
pub extern "C" fn pow10f(arg1: f32) -> f32 {
    runtime::pow10f(arg1)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub unsafe extern "C" fn diprintf(
    a: ::core::ffi::c_int,
    b: *const ::core::ffi::c_char,
    mut args: ...
) -> ::core::ffi::c_int {
    unsafe { runtime::diprintf_with_args(a, b, args.as_va_list()) }
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Format into a possibly reallocated C string buffer and report the written length.
///
/// # Exact upstream behavior
///
/// This is one of the few remaining stubs with real project-local code. The checked-in firmware
/// tree compiles `firmware/components/why_stdio/asnprintf.c`, whose entire body is:
///
/// - start a varargs list
/// - call `why_vasnprintf(str, lenp, fmt, ap)`
/// - end the varargs list
/// - return that pointer unchanged
///
/// The real behavior therefore lives in `why_vasnprintf()` and its output helper
/// `__why_file_str_put_alloc()`:
///
/// - the input buffer and capacity come from `FDEV_SETUP_STRING_ALLOC_BUF(str, *lenp)`
/// - `why_vfprintf()` renders into that string-backed `FILE`
/// - on success, it appends a terminating NUL with `why_fputc('\0', &f.file)`
/// - if the buffer fills, `__why_file_str_put_alloc()` grows it in 32-byte chunks
/// - the first growth from a caller-owned buffer allocates a fresh heap buffer with `why_malloc()`
///   and copies the old contents; later growth uses `why_realloc()`
/// - after formatting, `*lenp` is overwritten with the character count excluding the trailing NUL
/// - if heap allocation was used, upstream attempts a best-effort shrink to `i + 1` bytes, but a
///   failed shrink keeps the larger allocation and still returns success
/// - on formatting or allocation failure, upstream frees only buffers it allocated itself and
///   returns `NULL`
///
/// # Bugs and edge cases
///
/// - `lenp` is dereferenced immediately and is never null-checked
/// - if growth occurs, the returned pointer may differ from the incoming `str`, so callers must use
///   the return value rather than assuming in-place expansion
/// - when failure happens before any heap allocation, the caller's original buffer is left in place
///   and `*lenp` is not updated
/// - both `why_vasnprintf()` and `__why_file_str_put_alloc()` are marked `__disable_sanitizer`
pub unsafe extern "C" fn asnprintf(
    str_: *mut ::core::ffi::c_char,
    lenp: *mut size_t,
    fmt: *const ::core::ffi::c_char,
    mut args: ...
) -> *mut ::core::ffi::c_char {
    unsafe { runtime::asnprintf_with_args(str_, lenp, fmt, args.as_va_list()) }
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Format a `float` into a caller-provided buffer using `%g` semantics.
///
/// # Exact upstream behavior
///
/// The checked-in firmware tree compiles `firmware/components/why_stdio/gcvtf.c`, which is exactly:
///
/// - `__f_why_sprintf(buf, "%.*g", ndigit, __why_printf_float(invalue));`
/// - `return buf;`
///
/// Consequences of that exact implementation:
///
/// - formatting is delegated to the project's `sprintf` path with the `%g` conversion
/// - `ndigit` is forwarded directly as the precision argument
/// - the function always returns the original `buf` pointer
/// - the `sprintf` return value is ignored completely
/// - there is no buffer-length parameter and no overflow check
///
/// The helper `__why_printf_float(invalue)` exists only to satisfy the varargs calling convention;
/// it does not add any policy beyond whatever the underlying printf engine already does.
pub extern "C" fn gcvtf(
    arg1: f32,
    arg2: ::core::ffi::c_int,
    arg3: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    runtime::gcvtf(arg1, arg2, arg3)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Format a `long double` into a caller-provided buffer using `%Lg` semantics.
///
/// # Exact upstream behavior
///
/// The checked-in firmware tree compiles `firmware/components/why_stdio/gcvtl.c`, whose entire
/// implementation is:
///
/// - `__d_why_sprintf(buf, "%.*Lg", ndigit, invalue);`
/// - `return buf;`
///
/// That means upstream behavior is intentionally minimal:
///
/// - `ndigit` is passed through as the precision for `%Lg`
/// - the function ignores the formatter's return value
/// - it always returns the incoming `buf`
/// - there is no length argument, no null check, and no protection against buffer overflow
pub extern "C" fn gcvtl(
    _arg1: u128,
    _arg2: ::core::ffi::c_int,
    _arg3: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    runtime::gcvtl(_arg1, _arg2, _arg3)
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
/// Construct a buffered `FILE *` around caller-supplied cookie callbacks.
///
/// # Exact upstream behavior
///
/// The checked-in firmware tree compiles `firmware/components/why_stdio/funopen.c`.
/// Upstream does exactly this:
///
/// - start with `open_flags = 0`
/// - if `readfn != NULL`, OR in `__SRD`
/// - if `writefn != NULL`, OR in `__SWR`
/// - allocate one zeroed block of `sizeof(struct __file_bufio) + BUFSIZ` with `why_calloc(1, ...)`
/// - on allocation failure, return `NULL`
/// - treat the trailing `BUFSIZ` bytes of that allocation as the stdio buffer
/// - initialize the struct with
///   `FDEV_SETUP_BUFIO_PTR(cookie, buf, BUFSIZ, readfn, writefn, seekfn, closefn, open_flags, __BFALL)`
/// - return the resulting pointer cast to `FILE *`
///
/// `FDEV_SETUP_BUFIO_PTR` has several important side effects:
///
/// - it marks the stream as buffered (`__SBUF`)
/// - it stores the cookie pointer in `ptr`
/// - it sets `bflags = __BFALL | __BFPTR`, which means the `FILE` itself was allocated by stdio and
///   callback dispatch uses pointer-style hooks rather than integer file descriptors
/// - it wires stdio operations through `__why_bufio_put`, `__why_bufio_get`, `__why_bufio_flush`,
///   `__why_bufio_close`, `__why_bufio_seek`, and `__why_bufio_setvbuf`
///
/// Cleanup behavior is equally exact. `__why_bufio_close()` will:
///
/// - flush buffered output first
/// - call the `closefn` callback if one was supplied
/// - free the `FILE` allocation because `__BFALL` is set
/// - not separately free the buffer with `why_free(bf->buf)`, because `__BALL` is not set and the
///   buffer lives inside the same allocation as the `FILE`
///
/// # Edge cases and interactions
///
/// - `seekfn` and `closefn` do not affect `open_flags`; only `readfn` and `writefn` decide whether
///   the returned stream is readable and/or writable
/// - if both `readfn` and `writefn` are `NULL`, upstream still returns a `FILE *`, but later I/O
///   checks against `__SRD` and `__SWR` will reject normal reads and writes
/// - there is no validation of the cookie pointer or callback combinations
/// - on architectures where pointer and int callback ABIs differ, the `__BFPTR` flag forces the
///   pointer-callback path; on `x86_64`, `arm`, and `riscv`, the implementation also enables the
///   fast `BUFIO_ABI_MATCHES` path
pub extern "C" fn funopen(
    _cookie: *const ::core::ffi::c_void,
    _readfn: ::core::option::Option<
        unsafe extern "C" fn(
            cookie: *mut ::core::ffi::c_void,
            buf: *mut ::core::ffi::c_void,
            n: size_t,
        ) -> _ssize_t,
    >,
    _writefn: ::core::option::Option<
        unsafe extern "C" fn(
            cookie: *mut ::core::ffi::c_void,
            buf: *const ::core::ffi::c_void,
            n: size_t,
        ) -> _ssize_t,
    >,
    _seekfn: ::core::option::Option<
        unsafe extern "C" fn(
            cookie: *mut ::core::ffi::c_void,
            off: __off_t,
            whence: ::core::ffi::c_int,
        ) -> __off_t,
    >,
    _closefn: ::core::option::Option<
        unsafe extern "C" fn(cookie: *mut ::core::ffi::c_void) -> ::core::ffi::c_int,
    >,
) -> *mut FILE {
    runtime::funopen(_cookie, _readfn, _writefn, _seekfn, _closefn)
}
