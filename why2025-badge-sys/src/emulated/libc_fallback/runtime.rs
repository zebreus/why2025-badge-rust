use crate::types::*;
use crate::{atof, isascii};
use core::cell::UnsafeCell;
use core::ffi::{CStr, VaList, c_char};
use core::{ptr, slice};
use std::{
    borrow::Cow,
    cell::Cell,
    io::{self, Write},
};

type size_t = usize;

struct HostErrnoCell(UnsafeCell<::core::ffi::c_int>);

unsafe impl Sync for HostErrnoCell {}

static HOST_ERRNO: HostErrnoCell = HostErrnoCell(UnsafeCell::new(0));

std::thread_local! {
    static HOST_FP_MASK: Cell<fp_except> = const { Cell::new(0) };
    static HOST_LOCALE: Cell<locale_t> = const { Cell::new(-1) };
}

const FP_RN_CONST: fp_rnd = 0;
const FP_RM_CONST: fp_rnd = 1;
const FP_RP_CONST: fp_rnd = 2;
const FP_RZ_CONST: fp_rnd = 3;

const FP_X_INV_CONST: fp_except = 0x10;
const FP_X_DX_CONST: fp_except = 0x80;
const FP_X_OFL_CONST: fp_except = 0x04;
const FP_X_UFL_CONST: fp_except = 0x02;
const FP_X_IMP_CONST: fp_except = 0x01;
const FP_SUPPORTED_MASK: fp_except =
    FP_X_INV_CONST | FP_X_DX_CONST | FP_X_OFL_CONST | FP_X_UFL_CONST | FP_X_IMP_CONST;

const HOST_FE_INVALID: ::core::ffi::c_int = 0x01;
const HOST_FE_DIVBYZERO: ::core::ffi::c_int = 0x04;
const HOST_FE_OVERFLOW: ::core::ffi::c_int = 0x08;
const HOST_FE_UNDERFLOW: ::core::ffi::c_int = 0x10;
const HOST_FE_INEXACT: ::core::ffi::c_int = 0x20;
const HOST_FE_ALL_EXCEPT: ::core::ffi::c_int =
    HOST_FE_INVALID | HOST_FE_DIVBYZERO | HOST_FE_OVERFLOW | HOST_FE_UNDERFLOW | HOST_FE_INEXACT;

const HOST_FE_TONEAREST: ::core::ffi::c_int = 0;
const HOST_FE_DOWNWARD: ::core::ffi::c_int = 0x400;
const HOST_FE_UPWARD: ::core::ffi::c_int = 0x800;
const HOST_FE_TOWARDZERO: ::core::ffi::c_int = 0xc00;

const BADGE_FE_INVALID: ::core::ffi::c_int = 0x10;
const BADGE_FE_DIVBYZERO: ::core::ffi::c_int = 0x08;
const BADGE_FE_OVERFLOW: ::core::ffi::c_int = 0x04;
const BADGE_FE_UNDERFLOW: ::core::ffi::c_int = 0x02;
const BADGE_FE_INEXACT: ::core::ffi::c_int = 0x01;
const BADGE_FE_ALL_EXCEPT: ::core::ffi::c_int = BADGE_FE_INVALID
    | BADGE_FE_DIVBYZERO
    | BADGE_FE_OVERFLOW
    | BADGE_FE_UNDERFLOW
    | BADGE_FE_INEXACT;

const BADGE_FE_TONEAREST: ::core::ffi::c_int = 0;
const BADGE_FE_TOWARDZERO: ::core::ffi::c_int = 0x01;
const BADGE_FE_DOWNWARD: ::core::ffi::c_int = 0x02;
const BADGE_FE_UPWARD: ::core::ffi::c_int = 0x03;
const BADGE_FE_TONEAREST_MM: ::core::ffi::c_int = 0x04;
const BADGE_FE_RMODE_MASK: usize = 0x7;
const BADGE_FENV_ROUND_SHIFT: usize = 8;

const SIGNAL_NAMES: &[(::core::ffi::c_int, &[u8])] = &[
    (1, b"HUP"),
    (2, b"INT"),
    (3, b"QUIT"),
    (4, b"ILL"),
    (5, b"TRAP"),
    (6, b"ABRT"),
    (7, b"EMT"),
    (8, b"FPE"),
    (9, b"KILL"),
    (10, b"BUS"),
    (11, b"SEGV"),
    (12, b"SYS"),
    (13, b"PIPE"),
    (14, b"ALRM"),
    (15, b"TERM"),
    (16, b"URG"),
    (17, b"STOP"),
    (18, b"TSTP"),
    (19, b"CONT"),
    (20, b"CHLD"),
    (21, b"TTIN"),
    (22, b"TTOU"),
    (23, b"IO"),
    (24, b"XCPU"),
    (25, b"XFSZ"),
    (26, b"VTALRM"),
    (27, b"PROF"),
    (28, b"WINCH"),
    (29, b"LOST"),
    (30, b"USR1"),
    (31, b"USR2"),
];

const SIGNAL_ALIASES: &[(&[u8], ::core::ffi::c_int)] = &[(b"IOT", 6), (b"CLD", 20), (b"POLL", 23)];

unsafe extern "C" {
    fn fegetround() -> ::core::ffi::c_int;
    fn fesetround(round: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn fetestexcept(excepts: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn feclearexcept(excepts: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn feraiseexcept(excepts: ::core::ffi::c_int) -> ::core::ffi::c_int;
    #[link_name = "__cxa_atexit"]
    fn host_cxa_atexit(
        func: ::core::option::Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
        arg: *mut ::core::ffi::c_void,
        dso_handle: *mut ::core::ffi::c_void,
    ) -> ::core::ffi::c_int;
    #[allow(clashing_extern_declarations)]
    #[link_name = "vsnprintf"]
    fn host_vsnprintf(
        buf: *mut ::core::ffi::c_char,
        size: size_t,
        fmt: *const ::core::ffi::c_char,
        ap: VaList<'_, '_>,
    ) -> ::core::ffi::c_int;
    #[allow(clashing_extern_declarations)]
    #[link_name = "vdprintf"]
    fn host_vdprintf(
        fd: ::core::ffi::c_int,
        fmt: *const ::core::ffi::c_char,
        ap: VaList<'_, '_>,
    ) -> ::core::ffi::c_int;
    #[allow(clashing_extern_declarations)]
    #[link_name = "vprintf"]
    fn host_vprintf(fmt: *const ::core::ffi::c_char, ap: VaList<'_, '_>) -> ::core::ffi::c_int;
    #[allow(clashing_extern_declarations)]
    #[link_name = "vprintf"]
    fn host_vprintf_raw(
        fmt: *const ::core::ffi::c_char,
        ap: __builtin_va_list,
    ) -> ::core::ffi::c_int;
    #[allow(clashing_extern_declarations)]
    #[link_name = "vfprintf"]
    fn host_vfprintf(
        stream: *mut FILE,
        fmt: *const ::core::ffi::c_char,
        ap: VaList<'_, '_>,
    ) -> ::core::ffi::c_int;
    #[allow(clashing_extern_declarations)]
    #[link_name = "vfprintf"]
    fn host_vfprintf_raw(
        stream: *mut FILE,
        fmt: *const ::core::ffi::c_char,
        ap: __builtin_va_list,
    ) -> ::core::ffi::c_int;
    #[allow(clashing_extern_declarations)]
    #[link_name = "vsprintf"]
    fn host_vsprintf(
        buf: *mut ::core::ffi::c_char,
        fmt: *const ::core::ffi::c_char,
        ap: VaList<'_, '_>,
    ) -> ::core::ffi::c_int;
    #[allow(clashing_extern_declarations)]
    #[link_name = "vsprintf"]
    fn host_vsprintf_raw(
        buf: *mut ::core::ffi::c_char,
        fmt: *const ::core::ffi::c_char,
        ap: __builtin_va_list,
    ) -> ::core::ffi::c_int;
    #[allow(clashing_extern_declarations)]
    #[link_name = "vsnprintf"]
    fn host_vsnprintf_raw(
        buf: *mut ::core::ffi::c_char,
        size: size_t,
        fmt: *const ::core::ffi::c_char,
        ap: __builtin_va_list,
    ) -> ::core::ffi::c_int;
    #[allow(clashing_extern_declarations)]
    #[link_name = "vasprintf"]
    fn host_vasprintf(
        strp: *mut *mut ::core::ffi::c_char,
        fmt: *const ::core::ffi::c_char,
        ap: VaList<'_, '_>,
    ) -> ::core::ffi::c_int;
    #[allow(clashing_extern_declarations)]
    #[link_name = "vasprintf"]
    fn host_vasprintf_raw(
        strp: *mut *mut ::core::ffi::c_char,
        fmt: *const ::core::ffi::c_char,
        ap: __gnuc_va_list,
    ) -> ::core::ffi::c_int;
    #[allow(clashing_extern_declarations)]
    #[link_name = "vscanf"]
    fn host_vscanf(fmt: *const ::core::ffi::c_char, ap: VaList<'_, '_>) -> ::core::ffi::c_int;
    #[allow(clashing_extern_declarations)]
    #[link_name = "vscanf"]
    fn host_vscanf_raw(
        fmt: *const ::core::ffi::c_char,
        ap: __builtin_va_list,
    ) -> ::core::ffi::c_int;
    #[allow(clashing_extern_declarations)]
    #[link_name = "vfscanf"]
    fn host_vfscanf(
        stream: *mut FILE,
        fmt: *const ::core::ffi::c_char,
        ap: VaList<'_, '_>,
    ) -> ::core::ffi::c_int;
    #[allow(clashing_extern_declarations)]
    #[link_name = "vfscanf"]
    fn host_vfscanf_raw(
        stream: *mut FILE,
        fmt: *const ::core::ffi::c_char,
        ap: __builtin_va_list,
    ) -> ::core::ffi::c_int;
    #[allow(clashing_extern_declarations)]
    #[link_name = "vsscanf"]
    fn host_vsscanf(
        buf: *const ::core::ffi::c_char,
        fmt: *const ::core::ffi::c_char,
        ap: VaList<'_, '_>,
    ) -> ::core::ffi::c_int;
    #[allow(clashing_extern_declarations)]
    #[link_name = "vsscanf"]
    fn host_vsscanf_raw(
        buf: *const ::core::ffi::c_char,
        fmt: *const ::core::ffi::c_char,
        ap: __builtin_va_list,
    ) -> ::core::ffi::c_int;
    fn sprintf(
        buf: *mut ::core::ffi::c_char,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn wcstoll(
        nptr: *const _wchar_t,
        endptr: *mut *mut _wchar_t,
        base: ::core::ffi::c_int,
    ) -> ::core::ffi::c_longlong;
    fn wcstoull(
        nptr: *const _wchar_t,
        endptr: *mut *mut _wchar_t,
        base: ::core::ffi::c_int,
    ) -> ::core::ffi::c_ulonglong;
}

fn set_host_errno(value: ::core::ffi::c_int) {
    unsafe {
        *HOST_ERRNO.0.get() = value;
    }
}

#[cfg(target_os = "linux")]
fn set_system_errno(value: ::core::ffi::c_int) {
    unsafe {
        *libc::__errno_location() = value;
    }
}

#[cfg(not(target_os = "linux"))]
fn set_system_errno(_value: ::core::ffi::c_int) {}

#[cfg(target_os = "linux")]
fn sync_host_errno_from_system() {
    unsafe {
        *HOST_ERRNO.0.get() = *libc::__errno_location();
    }
}

#[cfg(not(target_os = "linux"))]
fn sync_host_errno_from_system() {
    set_host_errno(0);
}

pub(crate) fn abort_unemulatable_symbol(symbol: &str, reason: &str) -> ! {
    let mut stderr = io::stderr().lock();
    let _ = writeln!(
        stderr,
        "{symbol} is only partially emulated on the host: {reason}"
    );
    std::process::abort()
}

fn clear_errno() {
    set_host_errno(0);
    set_system_errno(0);
}

fn ascii_lower(byte: u8) -> u8 {
    if byte.is_ascii_uppercase() {
        byte + (b'a' - b'A')
    } else {
        byte
    }
}

fn ascii_upper(byte: u8) -> u8 {
    if byte.is_ascii_lowercase() {
        byte - (b'a' - b'A')
    } else {
        byte
    }
}

unsafe fn transform_c_string_in_place(
    ptr_: *mut ::core::ffi::c_char,
    transform: fn(u8) -> u8,
) -> *mut ::core::ffi::c_char {
    if ptr_.is_null() {
        return ptr::null_mut();
    }

    let mut cursor = ptr_;
    while unsafe { *cursor } != 0 {
        unsafe {
            *cursor = transform(*cursor as u8) as ::core::ffi::c_char;
            cursor = cursor.add(1);
        }
    }

    ptr_
}

unsafe fn write_c_string_bytes(dst: *mut ::core::ffi::c_char, bytes: &[u8]) {
    for (index, byte) in bytes.iter().copied().enumerate() {
        unsafe {
            *dst.add(index) = byte as ::core::ffi::c_char;
        }
    }
    unsafe {
        *dst.add(bytes.len()) = 0;
    }
}

unsafe fn wide_strlen(ptr_: *const wchar_t) -> usize {
    if ptr_.is_null() {
        return 0;
    }

    let mut len = 0;
    while unsafe { *ptr_.add(len) } != 0 {
        len += 1;
    }
    len
}

unsafe fn wide_strnlen(ptr_: *const wchar_t, max_len: usize) -> usize {
    if ptr_.is_null() {
        return 0;
    }

    let mut len = 0;
    while len < max_len && unsafe { *ptr_.add(len) } != 0 {
        len += 1;
    }
    len
}

fn normalize_signal_name<'a>(bytes: &'a [u8]) -> &'a [u8] {
    if bytes.len() >= 3 && bytes[..3].eq_ignore_ascii_case(b"SIG") {
        &bytes[3..]
    } else {
        bytes
    }
}

fn signal_name(signum: ::core::ffi::c_int) -> Option<&'static [u8]> {
    SIGNAL_NAMES
        .iter()
        .find(|(candidate, _)| *candidate == signum)
        .map(|(_, name)| *name)
}

fn parse_signal_name(bytes: &[u8]) -> Option<::core::ffi::c_int> {
    let normalized = normalize_signal_name(bytes);

    if let Ok(text) = std::str::from_utf8(normalized) {
        if let Ok(value) = text.parse::<::core::ffi::c_int>() {
            if signal_name(value).is_some() {
                return Some(value);
            }
        }
    }

    if let Some((signum, _)) = SIGNAL_NAMES
        .iter()
        .find(|(_, name)| normalized.eq_ignore_ascii_case(name))
    {
        return Some(*signum);
    }

    SIGNAL_ALIASES
        .iter()
        .find(|(alias, _)| normalized.eq_ignore_ascii_case(alias))
        .map(|(_, signum)| *signum)
}

fn host_round_mode_to_badge(mode: ::core::ffi::c_int) -> fp_rnd {
    match mode {
        HOST_FE_TONEAREST => FP_RN_CONST,
        HOST_FE_DOWNWARD => FP_RM_CONST,
        HOST_FE_UPWARD => FP_RP_CONST,
        HOST_FE_TOWARDZERO => FP_RZ_CONST,
        _ => FP_RN_CONST,
    }
}

fn badge_round_mode_to_host(mode: fp_rnd) -> Option<::core::ffi::c_int> {
    match mode {
        FP_RN_CONST => Some(HOST_FE_TONEAREST),
        FP_RM_CONST => Some(HOST_FE_DOWNWARD),
        FP_RP_CONST => Some(HOST_FE_UPWARD),
        FP_RZ_CONST => Some(HOST_FE_TOWARDZERO),
        _ => None,
    }
}

fn host_excepts_to_badge(mask: ::core::ffi::c_int) -> fp_except {
    let mut result = 0;
    if mask & HOST_FE_INVALID != 0 {
        result |= FP_X_INV_CONST;
    }
    if mask & HOST_FE_DIVBYZERO != 0 {
        result |= FP_X_DX_CONST;
    }
    if mask & HOST_FE_OVERFLOW != 0 {
        result |= FP_X_OFL_CONST;
    }
    if mask & HOST_FE_UNDERFLOW != 0 {
        result |= FP_X_UFL_CONST;
    }
    if mask & HOST_FE_INEXACT != 0 {
        result |= FP_X_IMP_CONST;
    }
    result
}

fn host_excepts_to_badge_fe(mask: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut result = 0;
    if mask & HOST_FE_INVALID != 0 {
        result |= BADGE_FE_INVALID;
    }
    if mask & HOST_FE_DIVBYZERO != 0 {
        result |= BADGE_FE_DIVBYZERO;
    }
    if mask & HOST_FE_OVERFLOW != 0 {
        result |= BADGE_FE_OVERFLOW;
    }
    if mask & HOST_FE_UNDERFLOW != 0 {
        result |= BADGE_FE_UNDERFLOW;
    }
    if mask & HOST_FE_INEXACT != 0 {
        result |= BADGE_FE_INEXACT;
    }
    result
}

fn badge_excepts_to_host(mask: fp_except) -> ::core::ffi::c_int {
    let mut result = 0;
    if mask & FP_X_INV_CONST != 0 {
        result |= HOST_FE_INVALID;
    }
    if mask & FP_X_DX_CONST != 0 {
        result |= HOST_FE_DIVBYZERO;
    }
    if mask & FP_X_OFL_CONST != 0 {
        result |= HOST_FE_OVERFLOW;
    }
    if mask & FP_X_UFL_CONST != 0 {
        result |= HOST_FE_UNDERFLOW;
    }
    if mask & FP_X_IMP_CONST != 0 {
        result |= HOST_FE_INEXACT;
    }
    result
}

fn badge_fe_excepts_to_host(mask: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut result = 0;
    if mask & BADGE_FE_INVALID != 0 {
        result |= HOST_FE_INVALID;
    }
    if mask & BADGE_FE_DIVBYZERO != 0 {
        result |= HOST_FE_DIVBYZERO;
    }
    if mask & BADGE_FE_OVERFLOW != 0 {
        result |= HOST_FE_OVERFLOW;
    }
    if mask & BADGE_FE_UNDERFLOW != 0 {
        result |= HOST_FE_UNDERFLOW;
    }
    if mask & BADGE_FE_INEXACT != 0 {
        result |= HOST_FE_INEXACT;
    }
    result
}

fn host_round_mode_to_badge_fe(mode: ::core::ffi::c_int) -> ::core::ffi::c_int {
    match mode {
        HOST_FE_TONEAREST => BADGE_FE_TONEAREST,
        HOST_FE_DOWNWARD => BADGE_FE_DOWNWARD,
        HOST_FE_UPWARD => BADGE_FE_UPWARD,
        HOST_FE_TOWARDZERO => BADGE_FE_TOWARDZERO,
        _ => BADGE_FE_TONEAREST,
    }
}

fn badge_fe_round_to_host(mode: ::core::ffi::c_int) -> Option<::core::ffi::c_int> {
    match mode {
        BADGE_FE_TONEAREST | BADGE_FE_TONEAREST_MM => Some(HOST_FE_TONEAREST),
        BADGE_FE_DOWNWARD => Some(HOST_FE_DOWNWARD),
        BADGE_FE_UPWARD => Some(HOST_FE_UPWARD),
        BADGE_FE_TOWARDZERO => Some(HOST_FE_TOWARDZERO),
        _ => None,
    }
}

fn encode_badge_fenv(flags: ::core::ffi::c_int, round: ::core::ffi::c_int) -> fenv_t {
    (flags as usize) | (((round as usize) & BADGE_FE_RMODE_MASK) << BADGE_FENV_ROUND_SHIFT)
}

fn decode_badge_fenv(env: fenv_t) -> (::core::ffi::c_int, ::core::ffi::c_int) {
    let flags = (env as ::core::ffi::c_int) & BADGE_FE_ALL_EXCEPT;
    let round = ((env >> BADGE_FENV_ROUND_SHIFT) & BADGE_FE_RMODE_MASK) as ::core::ffi::c_int;
    (flags, round)
}

fn current_badge_fenv() -> fenv_t {
    let flags = host_excepts_to_badge_fe(unsafe { fetestexcept(HOST_FE_ALL_EXCEPT) });
    let round = host_round_mode_to_badge_fe(unsafe { fegetround() });
    encode_badge_fenv(flags, round)
}

fn store_fp_mask(mask: fp_except) {
    HOST_FP_MASK.with(|value| value.set(mask & FP_SUPPORTED_MASK));
}

fn load_fp_mask() -> fp_except {
    HOST_FP_MASK.with(Cell::get)
}

fn is_valid_radix(radix: ::core::ffi::c_int) -> bool {
    (2..=36).contains(&radix)
}

unsafe fn format_unsigned_to_buffer(
    value: u64,
    dst: *mut ::core::ffi::c_char,
    radix: u32,
    negative: bool,
) -> *mut ::core::ffi::c_char {
    const DIGITS: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";

    let mut scratch = [0u8; 66];
    let mut len = 0usize;
    let mut remaining = value;

    loop {
        scratch[len] = DIGITS[(remaining % radix as u64) as usize];
        len += 1;
        remaining /= radix as u64;
        if remaining == 0 {
            break;
        }
    }

    let mut out_index = 0usize;
    if negative {
        unsafe {
            *dst = b'-' as ::core::ffi::c_char;
        }
        out_index = 1;
    }

    for index in 0..len {
        unsafe {
            *dst.add(out_index + index) = scratch[len - 1 - index] as ::core::ffi::c_char;
        }
    }
    unsafe {
        *dst.add(out_index + len) = 0;
    }

    dst
}

fn c_string_or_placeholder<'a>(ptr: *const c_char, placeholder: &'a str) -> Cow<'a, str> {
    if ptr.is_null() {
        Cow::Borrowed(placeholder)
    } else {
        unsafe { CStr::from_ptr(ptr) }.to_string_lossy()
    }
}

fn compare_f64_nan_high(a: f64, b: f64) -> ::core::ffi::c_int {
    match a.partial_cmp(&b) {
        Some(core::cmp::Ordering::Less) => -1,
        Some(core::cmp::Ordering::Equal) => 0,
        Some(core::cmp::Ordering::Greater) => 1,
        None => 1,
    }
}

fn compare_f64_nan_low(a: f64, b: f64) -> ::core::ffi::c_int {
    match a.partial_cmp(&b) {
        Some(core::cmp::Ordering::Less) => -1,
        Some(core::cmp::Ordering::Equal) => 0,
        Some(core::cmp::Ordering::Greater) => 1,
        None => -1,
    }
}

fn half_bits_to_f32(bits: u16) -> f32 {
    let sign = (u32::from(bits & 0x8000)) << 16;
    let exponent = (bits >> 10) & 0x1f;
    let mantissa = bits & 0x03ff;

    let f32_bits = match exponent {
        0 if mantissa == 0 => sign,
        0 => {
            let mut normalized_mantissa = u32::from(mantissa);
            let mut exponent_adjust = 0_u32;
            while (normalized_mantissa & 0x0400) == 0 {
                normalized_mantissa <<= 1;
                exponent_adjust += 1;
            }
            normalized_mantissa &= 0x03ff;
            let f32_exponent = 127_u32 - 14 - exponent_adjust;
            sign | (f32_exponent << 23) | (normalized_mantissa << 13)
        }
        0x1f => sign | 0x7f80_0000 | (u32::from(mantissa) << 13),
        _ => {
            let f32_exponent = u32::from(exponent) + (127 - 15);
            sign | (f32_exponent << 23) | (u32::from(mantissa) << 13)
        }
    };

    f32::from_bits(f32_bits)
}

fn f32_to_half_bits(value: f32) -> u16 {
    let bits = value.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let exponent = ((bits >> 23) & 0xff) as i32;
    let mantissa = bits & 0x007f_ffff;

    if exponent == 0xff {
        if mantissa == 0 {
            return sign | 0x7c00;
        }

        let mut half_mantissa = (mantissa >> 13) as u16;
        if half_mantissa == 0 {
            half_mantissa = 1;
        }
        return sign | 0x7c00 | half_mantissa | 0x0200;
    }

    let half_exponent = exponent - 127 + 15;
    if half_exponent >= 0x1f {
        return sign | 0x7c00;
    }

    if half_exponent <= 0 {
        if half_exponent < -10 {
            return sign;
        }

        let mantissa_with_hidden_bit = mantissa | 0x0080_0000;
        let shift = (14 - half_exponent) as u32;
        let mut half_mantissa = (mantissa_with_hidden_bit >> shift) as u16;
        let round_bit = 1_u32 << (shift - 1);
        let round_remainder = mantissa_with_hidden_bit & (round_bit - 1);
        if (mantissa_with_hidden_bit & round_bit) != 0
            && (round_remainder != 0 || (half_mantissa & 1) != 0)
        {
            half_mantissa = half_mantissa.wrapping_add(1);
        }

        if half_mantissa == 0x0400 {
            return sign | 0x0400;
        }

        return sign | half_mantissa;
    }

    let mut half_exponent = half_exponent as u16;
    let mut half_mantissa = (mantissa >> 13) as u16;
    let round_remainder = mantissa & 0x1fff;
    if round_remainder > 0x1000 || (round_remainder == 0x1000 && (half_mantissa & 1) != 0) {
        half_mantissa = half_mantissa.wrapping_add(1);
        if half_mantissa == 0x0400 {
            half_mantissa = 0;
            half_exponent = half_exponent.wrapping_add(1);
            if half_exponent >= 0x1f {
                return sign | 0x7c00;
            }
        }
    }

    sign | (half_exponent << 10) | half_mantissa
}

unsafe extern "C" fn atexit_trampoline(ctx: *mut ::core::ffi::c_void) {
    let callback =
        unsafe { core::mem::transmute::<*mut ::core::ffi::c_void, unsafe extern "C" fn()>(ctx) };
    unsafe { callback() };
}

pub(super) fn __errno() -> *mut ::core::ffi::c_int {
    HOST_ERRNO.0.get()
}

pub(super) fn atexit(__func: ::core::option::Option<unsafe extern "C" fn()>) -> ::core::ffi::c_int {
    let Some(callback) = __func else {
        set_host_errno(libc::EINVAL);
        return -1;
    };

    clear_errno();
    let status = unsafe {
        host_cxa_atexit(
            Some(atexit_trampoline),
            callback as *const () as *mut ::core::ffi::c_void,
            ptr::null_mut(),
        )
    };
    sync_host_errno_from_system();
    status
}

pub static _ctype_b: [::core::ffi::c_char; 0usize] = [];

pub(super) fn atoff(__nptr: *const ::core::ffi::c_char) -> f32 {
    unsafe {
        let result = atof(__nptr);
        return result as f32;
    }
}

pub(super) fn fls(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int {
    for i in (0..32).rev() {
        if (arg1 & (1 << i)) != 0 {
            return i + 1;
        }
    }
    0
}

pub(super) fn flsl(arg1: ::core::ffi::c_long) -> ::core::ffi::c_int {
    for i in (0..64).rev() {
        if (arg1 & (1 << i)) != 0 {
            return i + 1;
        }
    }
    0
}

pub(super) fn flsll(arg1: ::core::ffi::c_longlong) -> ::core::ffi::c_int {
    for i in (0..64).rev() {
        if (arg1 & (1 << i)) != 0 {
            return i + 1;
        }
    }
    0
}

pub(super) fn __assert_func(
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

pub(super) fn __adddf3(a: f64, b: f64) -> f64 {
    a + b
}

pub(super) fn __subdf3(a: f64, b: f64) -> f64 {
    a - b
}

pub(super) fn __muldf3(a: f64, b: f64) -> f64 {
    a * b
}

pub(super) fn __divdf3(a: f64, b: f64) -> f64 {
    a / b
}

pub(super) fn __eqdf2(a: f64, b: f64) -> ::core::ffi::c_int {
    compare_f64_nan_high(a, b)
}

pub(super) fn __gedf2(a: f64, b: f64) -> ::core::ffi::c_int {
    compare_f64_nan_low(a, b)
}

pub(super) fn __gtdf2(a: f64, b: f64) -> ::core::ffi::c_int {
    compare_f64_nan_low(a, b)
}

pub(super) fn __ledf2(a: f64, b: f64) -> ::core::ffi::c_int {
    compare_f64_nan_high(a, b)
}

pub(super) fn __ltdf2(a: f64, b: f64) -> ::core::ffi::c_int {
    compare_f64_nan_high(a, b)
}

pub(super) fn __extendsfdf2(a: f32) -> f64 {
    a as f64
}

pub(super) fn __truncdfsf2(a: f64) -> f32 {
    a as f32
}

pub(super) fn __extendhfsf2(a: __BindgenFloat16) -> f32 {
    half_bits_to_f32(a.0)
}

pub(super) fn __truncsfhf2(a: f32) -> __BindgenFloat16 {
    __BindgenFloat16(f32_to_half_bits(a))
}

pub(super) fn __fixdfsi(a: f64) -> i32 {
    a as i32
}

pub(super) fn __fixdfdi(a: f64) -> i64 {
    a as i64
}

pub(super) fn __fixunsdfsi(a: f64) -> u32 {
    a as u32
}

pub(super) fn __floatdisf(a: i64) -> f32 {
    a as f32
}

pub(super) fn __floatsidf(a: i32) -> f64 {
    a as f64
}

pub(super) fn __floatundidf(a: u64) -> f64 {
    a as f64
}

pub(super) fn __floatundisf(a: u64) -> f32 {
    a as f32
}

pub(super) fn __floatunsidf(a: u32) -> f64 {
    a as f64
}

pub(super) fn __issignalingf(f: f32) -> ::core::ffi::c_int {
    let bits = f.to_bits();
    let exponent = bits & 0x7f80_0000;
    let mantissa = bits & 0x007f_ffff;
    let quiet_bit = 0x0040_0000;

    (exponent == 0x7f80_0000 && mantissa != 0 && (mantissa & quiet_bit) == 0) as ::core::ffi::c_int
}

pub(super) fn __nedf2(a: f64, b: f64) -> ::core::ffi::c_int {
    compare_f64_nan_high(a, b)
}

pub(super) fn __divdi3(a: i64, b: i64) -> i64 {
    a / b
}

pub(super) fn __udivdi3(a: u64, b: u64) -> u64 {
    a / b
}

pub(super) fn __umoddi3(a: u64, b: u64) -> u64 {
    a % b
}

pub(super) fn __clzsi2(a: u32) -> ::core::ffi::c_int {
    a.leading_zeros() as ::core::ffi::c_int
}

pub(super) fn __popcountsi2(a: u32) -> ::core::ffi::c_int {
    a.count_ones() as ::core::ffi::c_int
}

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
pub(super) fn fpgetround() -> fp_rnd {
    host_round_mode_to_badge(unsafe { fegetround() })
}

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
pub(super) fn fpsetround(arg1: fp_rnd) -> fp_rnd {
    let previous = fpgetround();
    if let Some(host_mode) = badge_round_mode_to_host(arg1) {
        unsafe {
            fesetround(host_mode);
        }
    }
    previous
}

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
pub(super) fn fpgetmask() -> fp_except {
    load_fp_mask()
}

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
pub(super) fn fpsetmask(arg1: fp_except) -> fp_except {
    let previous = fpgetmask();
    store_fp_mask(arg1);
    previous
}

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
pub(super) fn fpgetsticky() -> fp_except {
    host_excepts_to_badge(unsafe { fetestexcept(HOST_FE_ALL_EXCEPT) })
}

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
pub(super) fn fpsetsticky(arg1: fp_except) -> fp_except {
    let previous = fpgetsticky();
    unsafe {
        feclearexcept(HOST_FE_ALL_EXCEPT);
        let requested = badge_excepts_to_host(arg1 & FP_SUPPORTED_MASK);
        if requested != 0 {
            feraiseexcept(requested);
        }
    }
    previous
}

pub(super) fn feclearexcept_emulated(excepts: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe { feclearexcept(badge_fe_excepts_to_host(excepts & BADGE_FE_ALL_EXCEPT)) }
}

pub(super) fn fegetexceptflag_emulated(
    flagp: *mut fexcept_t,
    excepts: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if flagp.is_null() {
        set_host_errno(libc::EINVAL);
        return -1;
    }

    let masked = excepts & BADGE_FE_ALL_EXCEPT;
    let flags = host_excepts_to_badge_fe(unsafe { fetestexcept(HOST_FE_ALL_EXCEPT) }) & masked;
    unsafe {
        *flagp = flags as fexcept_t;
    }
    0
}

pub(super) fn fesetexceptflag_emulated(
    flagp: *const fexcept_t,
    excepts: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if flagp.is_null() {
        set_host_errno(libc::EINVAL);
        return -1;
    }

    let masked = excepts & BADGE_FE_ALL_EXCEPT;
    let new_flags = badge_fe_excepts_to_host((unsafe { *flagp } as ::core::ffi::c_int) & masked);
    let host_mask = badge_fe_excepts_to_host(masked);

    unsafe {
        feclearexcept(host_mask);
        if new_flags != 0 {
            feraiseexcept(new_flags);
        }
    }

    0
}

pub(super) fn feraiseexcept_emulated(excepts: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe { feraiseexcept(badge_fe_excepts_to_host(excepts & BADGE_FE_ALL_EXCEPT)) }
}

pub(super) fn fetestexcept_emulated(excepts: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let masked = excepts & BADGE_FE_ALL_EXCEPT;
    let host_mask = badge_fe_excepts_to_host(masked);
    let active = unsafe { fetestexcept(host_mask) };
    host_excepts_to_badge_fe(active) & masked
}

pub(super) fn fegetround_emulated() -> ::core::ffi::c_int {
    host_round_mode_to_badge_fe(unsafe { fegetround() })
}

pub(super) fn fesetround_emulated(rounding_mode: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let Some(host_mode) = badge_fe_round_to_host(rounding_mode) else {
        return 1;
    };
    unsafe { fesetround(host_mode) }
}

pub(super) fn fegetenv_emulated(envp: *mut fenv_t) -> ::core::ffi::c_int {
    if envp.is_null() {
        set_host_errno(libc::EINVAL);
        return -1;
    }

    unsafe {
        *envp = current_badge_fenv();
    }
    0
}

pub(super) fn feholdexcept_emulated(envp: *mut fenv_t) -> ::core::ffi::c_int {
    let status = fegetenv_emulated(envp);
    if status != 0 {
        return status;
    }

    unsafe {
        feclearexcept(HOST_FE_ALL_EXCEPT);
    }
    0
}

pub(super) fn fesetenv_emulated(envp: *const fenv_t) -> ::core::ffi::c_int {
    if envp.is_null() {
        set_host_errno(libc::EINVAL);
        return -1;
    }

    let (flags, round) = decode_badge_fenv(unsafe { *envp });
    let Some(host_round) = badge_fe_round_to_host(round) else {
        return 1;
    };

    unsafe {
        fesetround(host_round);
        feclearexcept(HOST_FE_ALL_EXCEPT);
        let host_flags = badge_fe_excepts_to_host(flags);
        if host_flags != 0 {
            feraiseexcept(host_flags);
        }
    }

    0
}

pub(super) fn feupdateenv_emulated(envp: *const fenv_t) -> ::core::ffi::c_int {
    let saved = unsafe { fetestexcept(HOST_FE_ALL_EXCEPT) };
    let status = fesetenv_emulated(envp);
    if status != 0 {
        return status;
    }

    unsafe {
        if saved != 0 {
            feraiseexcept(saved);
        }
    }
    0
}

pub(super) fn isalnum_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    let _ = l;
    unsafe { crate::isalnum(c) }
}

pub(super) fn isalpha_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    let _ = l;
    unsafe { crate::isalpha(c) }
}

pub(super) fn isblank_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    let _ = l;
    unsafe { crate::isblank(c) }
}

pub(super) fn iscntrl_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    let _ = l;
    unsafe { crate::iscntrl(c) }
}

pub(super) fn isdigit_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    let _ = l;
    unsafe { crate::isdigit(c) }
}

pub(super) fn isgraph_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    let _ = l;
    unsafe { crate::isgraph(c) }
}

pub(super) fn islower_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    let _ = l;
    unsafe { crate::islower(c) }
}

pub(super) fn isprint_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    let _ = l;
    unsafe { crate::isprint(c) }
}

pub(super) fn ispunct_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    let _ = l;
    unsafe { crate::ispunct(c) }
}

pub(super) fn isspace_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    let _ = l;
    unsafe { crate::isspace(c) }
}

pub(super) fn isupper_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    let _ = l;
    unsafe { crate::isupper(c) }
}

pub(super) fn isxdigit_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    let _ = l;
    unsafe { crate::isxdigit(c) }
}

pub(super) fn tolower_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    let _ = l;
    unsafe { crate::tolower(c) }
}

pub(super) fn toupper_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    let _ = l;
    unsafe { crate::toupper(c) }
}

pub(super) fn isascii_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    let _ = l;
    unsafe { isascii(c) }
}

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
pub(super) fn itoa(
    arg1: ::core::ffi::c_int,
    arg2: *mut ::core::ffi::c_char,
    arg3: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if arg2.is_null() || !is_valid_radix(arg3) {
        set_host_errno(libc::EINVAL);
        return ptr::null_mut();
    }

    set_host_errno(0);

    let negative = arg3 == 10 && arg1 < 0;
    let magnitude = if negative {
        (arg1 as i64).unsigned_abs()
    } else {
        (arg1 as ::core::ffi::c_uint) as u64
    };

    unsafe { format_unsigned_to_buffer(magnitude, arg2, arg3 as u32, negative) }
}

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
pub(super) fn sig2str(
    signum: ::core::ffi::c_int,
    str_: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let Some(name) = signal_name(signum) else {
        set_host_errno(libc::EINVAL);
        return -1;
    };

    if str_.is_null() {
        set_host_errno(libc::EINVAL);
        return -1;
    }

    set_host_errno(0);
    unsafe {
        write_c_string_bytes(str_, name);
    }
    0
}

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
pub(super) fn str2sig(
    str_: *const ::core::ffi::c_char,
    pnum: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if str_.is_null() || pnum.is_null() {
        set_host_errno(libc::EINVAL);
        return -1;
    }

    let bytes = unsafe { CStr::from_ptr(str_) }.to_bytes();
    let Some(signum) = parse_signal_name(bytes) else {
        set_host_errno(libc::EINVAL);
        return -1;
    };

    set_host_errno(0);
    unsafe {
        *pnum = signum;
    }
    0
}

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
pub(super) fn strlwr(arg1: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    unsafe { transform_c_string_in_place(arg1, ascii_lower) }
}

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
pub(super) fn strnstr(
    arg1: *const ::core::ffi::c_char,
    arg2: *const ::core::ffi::c_char,
    arg3: size_t,
) -> *mut ::core::ffi::c_char {
    if arg1.is_null() || arg2.is_null() {
        return ptr::null_mut();
    }

    let needle_len = unsafe { CStr::from_ptr(arg2) }.to_bytes().len();
    if needle_len == 0 {
        return arg1 as *mut ::core::ffi::c_char;
    }

    let haystack = arg1.cast::<u8>();
    let needle = arg2.cast::<u8>();

    for offset in 0..arg3 {
        let current = unsafe { *haystack.add(offset) };
        if current == 0 {
            break;
        }
        if offset + needle_len > arg3 {
            break;
        }

        let mut matched = true;
        for inner in 0..needle_len {
            let hay = unsafe { *haystack.add(offset + inner) };
            let nee = unsafe { *needle.add(inner) };
            if hay == 0 || hay != nee {
                matched = false;
                break;
            }
        }

        if matched {
            return unsafe { haystack.add(offset) } as *mut ::core::ffi::c_char;
        }
    }

    ptr::null_mut()
}

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
pub(super) fn strtoimax_l(
    arg1: *const ::core::ffi::c_char,
    _restrict: *mut *mut ::core::ffi::c_char,
    arg2: ::core::ffi::c_int,
    arg3: locale_t,
) -> intmax_t {
    let _ = arg3;
    clear_errno();
    let result = unsafe { libc::strtoll(arg1.cast(), _restrict.cast(), arg2) } as intmax_t;
    sync_host_errno_from_system();
    result
}

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
pub(super) fn strtoumax_l(
    arg1: *const ::core::ffi::c_char,
    _restrict: *mut *mut ::core::ffi::c_char,
    arg2: ::core::ffi::c_int,
    arg3: locale_t,
) -> uintmax_t {
    let _ = arg3;
    clear_errno();
    let result = unsafe { libc::strtoull(arg1.cast(), _restrict.cast(), arg2) } as uintmax_t;
    sync_host_errno_from_system();
    result
}

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
pub(super) fn strupr(arg1: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    unsafe { transform_c_string_in_place(arg1, ascii_upper) }
}

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
pub(super) fn timingsafe_bcmp(
    arg1: *const ::core::ffi::c_void,
    arg2: *const ::core::ffi::c_void,
    arg3: size_t,
) -> ::core::ffi::c_int {
    if arg3 == 0 {
        return 0;
    }

    let left = unsafe { slice::from_raw_parts(arg1.cast::<u8>(), arg3) };
    let right = unsafe { slice::from_raw_parts(arg2.cast::<u8>(), arg3) };
    let mut diff = 0u8;
    for index in 0..arg3 {
        diff |= left[index] ^ right[index];
    }
    diff as ::core::ffi::c_int
}

/// Compare two byte sequences in a timing-safe manner and return an ordering-like `int`.
///
/// # Upstream status
///
/// This symbol is only declared in `firmware/components/why_stdio/include/string.h` under
/// `__BSD_VISIBLE`; the checked-in firmware tree ships no project-local definition.
///
/// Because the implementation is not present in-repo, the exact relation between its return value
/// and lexicographic order, equality, or constant-time guarantees cannot be derived here.
pub(super) fn timingsafe_memcmp(
    arg1: *const ::core::ffi::c_void,
    arg2: *const ::core::ffi::c_void,
    arg3: size_t,
) -> ::core::ffi::c_int {
    if arg3 == 0 {
        return 0;
    }

    let left = unsafe { slice::from_raw_parts(arg1.cast::<u8>(), arg3) };
    let right = unsafe { slice::from_raw_parts(arg2.cast::<u8>(), arg3) };
    let mut left_less = 0u32;
    let mut right_less = 0u32;

    for index in 0..arg3 {
        let left_byte = left[index] as u32;
        let right_byte = right[index] as u32;
        let undecided = 1 ^ (left_less | right_less);

        left_less |= ((left_byte.wrapping_sub(right_byte) >> 31) & 1) & undecided;
        right_less |= ((right_byte.wrapping_sub(left_byte) >> 31) & 1) & undecided;
    }

    match (left_less != 0, right_less != 0) {
        (true, false) => -1,
        (false, true) => 1,
        _ => 0,
    }
}

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
pub(super) fn toascii_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    let _ = l;
    c & 0x7f
}

pub(super) fn uselocale(arg1: locale_t) -> locale_t {
    HOST_LOCALE.with(|current| {
        let previous = current.get();
        if arg1 != 0 {
            current.set(arg1);
        }
        previous
    })
}

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
pub(super) fn utoa(
    arg1: ::core::ffi::c_uint,
    arg2: *mut ::core::ffi::c_char,
    arg3: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if arg2.is_null() || !is_valid_radix(arg3) {
        set_host_errno(libc::EINVAL);
        return ptr::null_mut();
    }

    set_host_errno(0);
    unsafe { format_unsigned_to_buffer(arg1 as u64, arg2, arg3 as u32, false) }
}

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
pub(super) fn wcstoimax_l(
    arg1: *const _wchar_t,
    _restrict: *mut *mut _wchar_t,
    arg2: ::core::ffi::c_int,
    arg3: locale_t,
) -> intmax_t {
    let _ = arg3;
    clear_errno();
    let result = unsafe { wcstoll(arg1, _restrict, arg2) } as intmax_t;
    sync_host_errno_from_system();
    result
}

pub(super) fn wcslcpy(dst: *mut wchar_t, src: *const wchar_t, size: usize) -> usize {
    if src.is_null() || (dst.is_null() && size != 0) {
        set_host_errno(libc::EINVAL);
        return 0;
    }

    let src_len = unsafe { wide_strlen(src) };

    if size != 0 {
        let copy_len = core::cmp::min(src_len, size - 1);
        for index in 0..copy_len {
            unsafe {
                *dst.add(index) = *src.add(index);
            }
        }
        unsafe {
            *dst.add(copy_len) = 0;
        }
    }

    src_len
}

pub(super) fn wcslcat(dst: *mut wchar_t, src: *const wchar_t, size: usize) -> usize {
    if src.is_null() || (dst.is_null() && size != 0) {
        set_host_errno(libc::EINVAL);
        return 0;
    }

    let src_len = unsafe { wide_strlen(src) };
    if size == 0 {
        return src_len;
    }

    let dst_len = unsafe { wide_strnlen(dst, size) };
    if dst_len == size {
        return size + src_len;
    }

    let copy_len = core::cmp::min(src_len, size - dst_len - 1);
    for index in 0..copy_len {
        unsafe {
            *dst.add(dst_len + index) = *src.add(index);
        }
    }
    unsafe {
        *dst.add(dst_len + copy_len) = 0;
    }

    dst_len + src_len
}

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
pub(super) fn wcstoumax_l(
    arg1: *const _wchar_t,
    _restrict: *mut *mut _wchar_t,
    arg2: ::core::ffi::c_int,
    arg3: locale_t,
) -> uintmax_t {
    let _ = arg3;
    clear_errno();
    let result = unsafe { wcstoull(arg1, _restrict, arg2) } as uintmax_t;
    sync_host_errno_from_system();
    result
}

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
pub(super) fn gammaf_r(arg1: f32, arg2: *mut ::core::ffi::c_int) -> f32 {
    unsafe { crate::lgammaf_r(arg1, arg2) }
}

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
pub(super) fn gamma_r(arg1: f64, arg2: *mut ::core::ffi::c_int) -> f64 {
    unsafe { crate::lgamma_r(arg1, arg2) }
}

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
pub(super) fn infinity() -> f64 {
    f64::INFINITY
}

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
pub(super) fn infinityf() -> f32 {
    f32::INFINITY
}

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
pub(super) fn exp10(arg1: f64) -> f64 {
    10.0_f64.powf(arg1)
}

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
pub(super) fn pow10(arg1: f64) -> f64 {
    exp10(arg1)
}

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
pub(super) fn exp10f(arg1: f32) -> f32 {
    10.0_f32.powf(arg1)
}

/// Compute $10^x$ in single precision through the historical `pow10f` name.
///
/// # Upstream status
///
/// The checked-in firmware tree declares this symbol in `firmware/sdk_include/math.h` and lists it
/// in `firmware/badgevms/symbols.yml`, but provides no project-local implementation.
///
/// The repository therefore cannot reveal whether the badge treats this as an alias of `exp10f()`,
/// an independent libm entry point, or merely a stale export declaration.
pub(super) fn pow10f(arg1: f32) -> f32 {
    exp10f(arg1)
}

pub(super) unsafe fn diprintf_with_args(
    a: ::core::ffi::c_int,
    b: *const ::core::ffi::c_char,
    args: VaList<'_, '_>,
) -> ::core::ffi::c_int {
    unsafe { args.with_copy(|mut copy| host_vdprintf(a, b, copy.as_va_list())) }
}

pub(super) unsafe fn printf_with_args(
    fmt: *const ::core::ffi::c_char,
    args: VaList<'_, '_>,
) -> ::core::ffi::c_int {
    let result = unsafe { args.with_copy(|mut copy| host_vprintf(fmt, copy.as_va_list())) };
    sync_host_errno_from_system();
    result
}

pub(super) unsafe fn fprintf_with_args(
    stream: *mut FILE,
    fmt: *const ::core::ffi::c_char,
    args: VaList<'_, '_>,
) -> ::core::ffi::c_int {
    let result =
        unsafe { args.with_copy(|mut copy| host_vfprintf(stream, fmt, copy.as_va_list())) };
    sync_host_errno_from_system();
    result
}

pub(super) unsafe fn sprintf_with_args(
    buf: *mut ::core::ffi::c_char,
    fmt: *const ::core::ffi::c_char,
    args: VaList<'_, '_>,
) -> ::core::ffi::c_int {
    let result = unsafe { args.with_copy(|mut copy| host_vsprintf(buf, fmt, copy.as_va_list())) };
    sync_host_errno_from_system();
    result
}

pub(super) unsafe fn snprintf_with_args(
    buf: *mut ::core::ffi::c_char,
    size: ::core::ffi::c_uint,
    fmt: *const ::core::ffi::c_char,
    args: VaList<'_, '_>,
) -> ::core::ffi::c_int {
    let result = unsafe {
        args.with_copy(|mut copy| host_vsnprintf(buf, size as size_t, fmt, copy.as_va_list()))
    };
    sync_host_errno_from_system();
    result
}

pub(super) unsafe fn asprintf_with_args(
    strp: *mut *mut ::core::ffi::c_char,
    fmt: *const ::core::ffi::c_char,
    args: VaList<'_, '_>,
) -> ::core::ffi::c_int {
    let result = unsafe { args.with_copy(|mut copy| host_vasprintf(strp, fmt, copy.as_va_list())) };
    sync_host_errno_from_system();
    result
}

pub(super) unsafe fn scanf_with_args(
    fmt: *const ::core::ffi::c_char,
    args: VaList<'_, '_>,
) -> ::core::ffi::c_int {
    let result = unsafe { args.with_copy(|mut copy| host_vscanf(fmt, copy.as_va_list())) };
    sync_host_errno_from_system();
    result
}

pub(super) unsafe fn fscanf_with_args(
    stream: *mut FILE,
    fmt: *const ::core::ffi::c_char,
    args: VaList<'_, '_>,
) -> ::core::ffi::c_int {
    let result = unsafe { args.with_copy(|mut copy| host_vfscanf(stream, fmt, copy.as_va_list())) };
    sync_host_errno_from_system();
    result
}

pub(super) unsafe fn sscanf_with_args(
    buf: *const ::core::ffi::c_char,
    fmt: *const ::core::ffi::c_char,
    args: VaList<'_, '_>,
) -> ::core::ffi::c_int {
    let result = unsafe { args.with_copy(|mut copy| host_vsscanf(buf, fmt, copy.as_va_list())) };
    sync_host_errno_from_system();
    result
}

pub(super) unsafe fn vprintf(
    fmt: *const ::core::ffi::c_char,
    ap: __builtin_va_list,
) -> ::core::ffi::c_int {
    let result = unsafe { host_vprintf_raw(fmt, ap) };
    sync_host_errno_from_system();
    result
}

pub(super) unsafe fn vfprintf(
    stream: *mut FILE,
    fmt: *const ::core::ffi::c_char,
    ap: __builtin_va_list,
) -> ::core::ffi::c_int {
    let result = unsafe { host_vfprintf_raw(stream, fmt, ap) };
    sync_host_errno_from_system();
    result
}

pub(super) unsafe fn vsprintf(
    buf: *mut ::core::ffi::c_char,
    fmt: *const ::core::ffi::c_char,
    ap: __builtin_va_list,
) -> ::core::ffi::c_int {
    let result = unsafe { host_vsprintf_raw(buf, fmt, ap) };
    sync_host_errno_from_system();
    result
}

pub(super) unsafe fn vsnprintf(
    buf: *mut ::core::ffi::c_char,
    size: ::core::ffi::c_uint,
    fmt: *const ::core::ffi::c_char,
    ap: __builtin_va_list,
) -> ::core::ffi::c_int {
    let result = unsafe { host_vsnprintf_raw(buf, size as size_t, fmt, ap) };
    sync_host_errno_from_system();
    result
}

pub(super) unsafe fn vasprintf(
    strp: *mut *mut ::core::ffi::c_char,
    fmt: *const ::core::ffi::c_char,
    ap: __gnuc_va_list,
) -> ::core::ffi::c_int {
    let result = unsafe { host_vasprintf_raw(strp, fmt, ap) };
    sync_host_errno_from_system();
    result
}

pub(super) unsafe fn vscanf(
    fmt: *const ::core::ffi::c_char,
    ap: __builtin_va_list,
) -> ::core::ffi::c_int {
    let result = unsafe { host_vscanf_raw(fmt, ap) };
    sync_host_errno_from_system();
    result
}

pub(super) unsafe fn vfscanf(
    stream: *mut FILE,
    fmt: *const ::core::ffi::c_char,
    ap: __builtin_va_list,
) -> ::core::ffi::c_int {
    let result = unsafe { host_vfscanf_raw(stream, fmt, ap) };
    sync_host_errno_from_system();
    result
}

pub(super) unsafe fn vsscanf(
    buf: *const ::core::ffi::c_char,
    fmt: *const ::core::ffi::c_char,
    ap: __builtin_va_list,
) -> ::core::ffi::c_int {
    let result = unsafe { host_vsscanf_raw(buf, fmt, ap) };
    sync_host_errno_from_system();
    result
}

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
pub(super) unsafe fn asnprintf_with_args(
    str_: *mut ::core::ffi::c_char,
    lenp: *mut size_t,
    fmt: *const ::core::ffi::c_char,
    args: VaList<'_, '_>,
) -> *mut ::core::ffi::c_char {
    if lenp.is_null() || fmt.is_null() {
        set_host_errno(libc::EINVAL);
        return ptr::null_mut();
    }

    let mut capacity = unsafe { *lenp };
    let mut buffer = str_;
    let mut allocated = false;

    let required = unsafe {
        args.with_copy(|mut copy| host_vsnprintf(ptr::null_mut(), 0, fmt, copy.as_va_list()))
    };
    if required < 0 {
        sync_host_errno_from_system();
        return ptr::null_mut();
    }

    let required_len = required as usize;
    let required_capacity = required_len.saturating_add(1);

    if buffer.is_null() || capacity < required_capacity {
        buffer = unsafe { libc::malloc(required_capacity) }.cast::<::core::ffi::c_char>();
        if buffer.is_null() {
            set_host_errno(libc::ENOMEM);
            return ptr::null_mut();
        }
        capacity = required_capacity;
        allocated = true;
    }

    let written = unsafe {
        args.with_copy(|mut copy| host_vsnprintf(buffer, capacity, fmt, copy.as_va_list()))
    };
    if written < 0 {
        if allocated {
            unsafe {
                libc::free(buffer.cast());
            }
        }
        sync_host_errno_from_system();
        return ptr::null_mut();
    }

    if allocated {
        let shrunk = unsafe { libc::realloc(buffer.cast(), required_capacity) }
            .cast::<::core::ffi::c_char>();
        if !shrunk.is_null() {
            buffer = shrunk;
        }
    }

    unsafe {
        *lenp = written as usize;
    }
    sync_host_errno_from_system();
    buffer
}

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
pub(super) fn gcvtf(
    arg1: f32,
    arg2: ::core::ffi::c_int,
    arg3: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if arg3.is_null() {
        set_host_errno(libc::EINVAL);
        return ptr::null_mut();
    }

    unsafe {
        sprintf(arg3, c"%.*g".as_ptr(), arg2, arg1 as f64);
    }
    sync_host_errno_from_system();
    arg3
}

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
pub(super) fn gcvtl(
    arg1: u128,
    arg2: ::core::ffi::c_int,
    arg3: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let _ = (arg1, arg2, arg3);
    abort_unemulatable_symbol(
        "gcvtl",
        "badge long double formatting uses a guest ABI that is not host-compatible today",
    )
}

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
pub(super) fn funopen(
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
    let _ = (cookie, readfn, writefn, seekfn, closefn);
    abort_unemulatable_symbol(
        "funopen",
        "cookie-backed FILE allocation and callback dispatch semantics are not implemented in the host emulator yet",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use std::{ffi::CStr, os::unix::process::ExitStatusExt, process::Command};

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
    fn compiler_rt_arithmetic_and_conversion_helpers_match_host_ops() {
        assert_eq!(__adddf3(1.25, 2.5), 3.75);
        assert_eq!(__subdf3(5.5, 2.0), 3.5);
        assert_eq!(__muldf3(1.5, 4.0), 6.0);
        assert_eq!(__divdf3(9.0, 4.0), 2.25);

        assert_eq!(__extendsfdf2(1.5), 1.5_f64);
        assert_eq!(__truncdfsf2(1.5), 1.5_f32);
        assert_eq!(__floatsidf(-42), -42.0);
        assert_eq!(__floatdisf(-123_456_789), -123_456_789_f32);
        assert_eq!(__floatunsidf(u32::MAX), u32::MAX as f64);
        assert_eq!(__fixdfsi(42.9), 42);
        assert_eq!(__fixdfdi(-42.9), -42);
        assert_eq!(__fixunsdfsi(42.9), 42);
    }

    #[test]
    fn compiler_rt_half_conversion_helpers_match_ieee754_binary16() {
        assert_eq!(__extendhfsf2(__BindgenFloat16(0x3c00)), 1.0);
        assert_eq!(__extendhfsf2(__BindgenFloat16(0xc000)), -2.0);
        assert_eq!(__extendhfsf2(__BindgenFloat16(0x0001)), 2.0_f32.powi(-24));
        assert!(__extendhfsf2(__BindgenFloat16(0x7c00)).is_infinite());

        assert_eq!(__truncsfhf2(1.0).0, 0x3c00);
        assert_eq!(__truncsfhf2(-2.0).0, 0xc000);
        assert_eq!(__truncsfhf2(2.0_f32.powi(-24)).0, 0x0001);
        assert_eq!(__truncsfhf2(f32::INFINITY).0, 0x7c00);

        let halfway = 1.0 + 2.0_f32.powi(-11);
        let above_halfway = 1.0 + (2.0_f32.powi(-11) + 2.0_f32.powi(-12));
        assert_eq!(__truncsfhf2(halfway).0, 0x3c00);
        assert_eq!(__truncsfhf2(above_halfway).0, 0x3c01);
    }

    #[test]
    fn compiler_rt_comparison_helpers_follow_expected_ordering() {
        assert_eq!(__eqdf2(2.0, 2.0), 0);
        assert_ne!(__eqdf2(1.0, 2.0), 0);

        assert!(__ledf2(1.0, 2.0) < 0);
        assert_eq!(__ledf2(2.0, 2.0), 0);
        assert!(__ledf2(f64::NAN, 2.0) > 0);

        assert!(__ltdf2(1.0, 2.0) < 0);
        assert!(__ltdf2(f64::NAN, 2.0) > 0);

        assert!(__gtdf2(2.0, 1.0) > 0);
        assert!(__gtdf2(f64::NAN, 2.0) < 0);

        assert!(__gedf2(2.0, 1.0) > 0);
        assert_eq!(__gedf2(2.0, 2.0), 0);
        assert!(__gedf2(f64::NAN, 2.0) < 0);
    }

    #[test]
    fn compiler_rt_integer_helpers_match_host_ops() {
        assert_eq!(__divdi3(81, -9), -9);
        assert_eq!(__udivdi3(81, 9), 9);
        assert_eq!(__umoddi3(82, 9), 1);
        assert_eq!(__clzsi2(0), 32);
        assert_eq!(__clzsi2(1), 31);
        assert_eq!(__popcountsi2(0b1011_0001), 4);
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
        const TEST_NAME: &str = "emulated::libc_fallback::runtime::tests::assert_func_aborts";

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

    #[test]
    fn toascii_l_masks_to_low_seven_bits() {
        assert_eq!(toascii_l(0x141, 123), 0x41);
    }

    #[test]
    fn locale_ctype_wrappers_ignore_locale_and_match_plain_helpers() {
        for locale in [0, 77] {
            assert_eq!(isalnum_l('A' as c_char as i32, locale), unsafe {
                crate::isalnum('A' as c_char as i32)
            });
            assert_eq!(isalpha_l('Q' as c_char as i32, locale), unsafe {
                crate::isalpha('Q' as c_char as i32)
            });
            assert_eq!(isblank_l(' ' as c_char as i32, locale), unsafe {
                crate::isblank(' ' as c_char as i32)
            });
            assert_eq!(iscntrl_l('\n' as c_char as i32, locale), unsafe {
                crate::iscntrl('\n' as c_char as i32)
            });
            assert_eq!(isdigit_l('7' as c_char as i32, locale), unsafe {
                crate::isdigit('7' as c_char as i32)
            });
            assert_eq!(isgraph_l('!' as c_char as i32, locale), unsafe {
                crate::isgraph('!' as c_char as i32)
            });
            assert_eq!(islower_l('q' as c_char as i32, locale), unsafe {
                crate::islower('q' as c_char as i32)
            });
            assert_eq!(isprint_l(' ' as c_char as i32, locale), unsafe {
                crate::isprint(' ' as c_char as i32)
            });
            assert_eq!(ispunct_l('!' as c_char as i32, locale), unsafe {
                crate::ispunct('!' as c_char as i32)
            });
            assert_eq!(isspace_l('\t' as c_char as i32, locale), unsafe {
                crate::isspace('\t' as c_char as i32)
            });
            assert_eq!(isupper_l('Q' as c_char as i32, locale), unsafe {
                crate::isupper('Q' as c_char as i32)
            });
            assert_eq!(isxdigit_l('f' as c_char as i32, locale), unsafe {
                crate::isxdigit('f' as c_char as i32)
            });
            assert_eq!(tolower_l('Q' as c_char as i32, locale), unsafe {
                crate::tolower('Q' as c_char as i32)
            });
            assert_eq!(toupper_l('q' as c_char as i32, locale), unsafe {
                crate::toupper('q' as c_char as i32)
            });
            assert_eq!(isascii_l(0x41, locale), unsafe { crate::isascii(0x41) });
        }
    }

    #[test]
    fn itoa_and_utoa_format_expected_radices() {
        let mut signed = [0 as c_char; 32];
        let mut unsigned = [0 as c_char; 32];

        let signed_ptr = itoa(-42, signed.as_mut_ptr(), 10);
        let unsigned_ptr = utoa(255, unsigned.as_mut_ptr(), 16);

        assert_eq!(signed_ptr, signed.as_mut_ptr());
        assert_eq!(unsigned_ptr, unsigned.as_mut_ptr());
        assert_eq!(
            unsafe { CStr::from_ptr(signed.as_ptr()) }.to_bytes(),
            b"-42"
        );
        assert_eq!(
            unsafe { CStr::from_ptr(unsigned.as_ptr()) }.to_bytes(),
            b"ff"
        );
    }

    #[test]
    fn case_conversion_helpers_are_ascii_only() {
        let mut lower = *b"AbC123!\0";
        let mut upper = *b"aBc123!\0";

        strlwr(lower.as_mut_ptr().cast());
        strupr(upper.as_mut_ptr().cast());

        assert_eq!(&lower[..7], b"abc123!");
        assert_eq!(&upper[..7], b"ABC123!");
    }

    #[test]
    fn strnstr_respects_length_limit() {
        let haystack = b"abcdef\0";
        let needle = b"bcd\0";

        let found = strnstr(haystack.as_ptr().cast(), needle.as_ptr().cast(), 6);
        let missing = strnstr(haystack.as_ptr().cast(), needle.as_ptr().cast(), 3);

        assert_eq!(found, unsafe { haystack.as_ptr().add(1) }.cast_mut().cast());
        assert!(missing.is_null());
    }

    #[test]
    fn locale_wrappers_delegate_to_host_parsers() {
        let input = b"123xyz\0";
        let mut end = ptr::null_mut();

        let value = strtoimax_l(input.as_ptr().cast(), &mut end, 10, 0);

        assert_eq!(value, 123);
        assert_eq!(end, unsafe { input.as_ptr().add(3) }.cast_mut().cast());
        assert_eq!(unsafe { *__errno() }, 0);
    }

    #[test]
    fn wide_locale_wrappers_delegate_to_host_parsers() {
        let input: [_wchar_t; 5] = [b'4' as _wchar_t, b'2' as _wchar_t, b'x' as _wchar_t, 0, 0];
        let mut end = ptr::null_mut();

        let value = wcstoimax_l(input.as_ptr(), &mut end, 10, 0);

        assert_eq!(value, 42);
        assert_eq!(end, unsafe { input.as_ptr().add(2) }.cast_mut());
        assert_eq!(unsafe { *__errno() }, 0);
    }

    #[test]
    fn timing_safe_compares_match_expected_results() {
        let left = b"abc";
        let equal = b"abc";
        let greater = b"abd";

        assert_eq!(
            timingsafe_bcmp(left.as_ptr().cast(), equal.as_ptr().cast(), left.len()),
            0
        );
        assert_ne!(
            timingsafe_bcmp(left.as_ptr().cast(), greater.as_ptr().cast(), left.len()),
            0
        );
        assert_eq!(
            timingsafe_memcmp(left.as_ptr().cast(), equal.as_ptr().cast(), left.len()),
            0
        );
        assert_eq!(
            timingsafe_memcmp(left.as_ptr().cast(), greater.as_ptr().cast(), left.len()),
            -1
        );
        assert_eq!(
            timingsafe_memcmp(greater.as_ptr().cast(), left.as_ptr().cast(), left.len()),
            1
        );
    }

    #[test]
    fn signal_conversion_uses_badge_names() {
        let mut buffer = [0 as c_char; 17];
        let mut signum = 0;

        assert_eq!(sig2str(30, buffer.as_mut_ptr()), 0);
        assert_eq!(
            unsafe { CStr::from_ptr(buffer.as_ptr()) }.to_bytes(),
            b"USR1"
        );

        assert_eq!(str2sig(c"sigpoll".as_ptr(), &mut signum), 0);
        assert_eq!(signum, 23);
    }

    #[test]
    fn floating_point_helpers_roundtrip_host_state() {
        let previous_round = fpgetround();
        let previous_mask = fpgetmask();

        assert_eq!(fpsetround(FP_RZ_CONST), previous_round);
        assert_eq!(fpgetround(), FP_RZ_CONST);

        assert_eq!(fpsetmask(FP_X_INV_CONST | FP_X_IMP_CONST), previous_mask);
        assert_eq!(fpgetmask(), FP_X_INV_CONST | FP_X_IMP_CONST);

        fpsetsticky(0);
        assert_eq!(fpgetsticky(), 0);

        fpsetround(previous_round);
        fpsetmask(previous_mask);
    }

    #[test]
    fn infinity_and_pow10_helpers_match_host_math() {
        assert!(infinity().is_infinite() && infinity().is_sign_positive());
        assert!(infinityf().is_infinite() && infinityf().is_sign_positive());
        assert_eq!(exp10(3.0), 1000.0);
        assert_eq!(pow10(2.0), 100.0);
        assert_eq!(exp10f(3.0), 1000.0);
        assert_eq!(pow10f(2.0), 100.0);
    }

    #[test]
    fn asnprintf_formats_and_reports_length() {
        let mut len = 4usize;
        let buffer = unsafe { libc::malloc(len) }.cast::<c_char>();
        assert!(!buffer.is_null());

        let result = unsafe {
            crate::emulated::libc_fallback::asnprintf(
                buffer,
                &mut len,
                c"%s %d".as_ptr(),
                c"hi".as_ptr(),
                42,
            )
        };

        assert!(!result.is_null());
        assert_eq!(len, 5);
        assert_eq!(unsafe { CStr::from_ptr(result) }.to_bytes(), b"hi 42");

        unsafe {
            libc::free(result.cast());
        }
    }

    #[test]
    fn gcvtf_uses_general_formatting() {
        let mut buffer = [0 as c_char; 32];
        let result = gcvtf(12.5, 4, buffer.as_mut_ptr());

        assert_eq!(result, buffer.as_mut_ptr());
        assert_eq!(
            unsafe { CStr::from_ptr(buffer.as_ptr()) }.to_bytes(),
            b"12.5"
        );
    }

    #[test]
    fn diprintf_formats_to_file_descriptor() {
        let mut fds = [0; 2];
        assert_eq!(unsafe { libc::pipe(fds.as_mut_ptr()) }, 0);

        let written = unsafe {
            crate::emulated::libc_fallback::diprintf(fds[1], c"%s %d".as_ptr(), c"hi".as_ptr(), 42)
        };
        assert_eq!(written, 5);

        assert_eq!(unsafe { libc::close(fds[1]) }, 0);

        let mut buffer = [0_u8; 32];
        let read = unsafe { libc::read(fds[0], buffer.as_mut_ptr().cast(), buffer.len()) };
        assert_eq!(read, 5);
        assert_eq!(&buffer[..read as usize], b"hi 42");

        assert_eq!(unsafe { libc::close(fds[0]) }, 0);
    }

    #[test]
    fn gamma_helpers_delegate_to_lgamma_helpers() {
        let mut expected_sign_f32 = 0;
        let mut actual_sign_f32 = 0;
        let expected_f32 = unsafe { crate::lgammaf_r(0.5, &mut expected_sign_f32) };
        let actual_f32 = gammaf_r(0.5, &mut actual_sign_f32);

        assert_eq!(actual_sign_f32, expected_sign_f32);
        assert_eq!(actual_f32, expected_f32);

        let mut expected_sign_f64 = 0;
        let mut actual_sign_f64 = 0;
        let expected_f64 = unsafe { crate::lgamma_r(0.5, &mut expected_sign_f64) };
        let actual_f64 = gamma_r(0.5, &mut actual_sign_f64);

        assert_eq!(actual_sign_f64, expected_sign_f64);
        assert_eq!(actual_f64, expected_f64);
    }
}
