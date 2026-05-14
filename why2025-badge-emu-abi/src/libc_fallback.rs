use crate::runtime;
use crate::types::*;
use core::cell::UnsafeCell;
use core::ffi::{CStr, VaList, c_char, c_int, c_long, c_longlong, c_uint, c_void};
use core::{mem, ptr, slice};

struct PthreadKey(UnsafeCell<libc::pthread_key_t>);
struct PthreadOnce(UnsafeCell<libc::pthread_once_t>);
struct LibmHandle(UnsafeCell<*mut c_void>);

unsafe impl Sync for PthreadKey {}
unsafe impl Sync for PthreadOnce {}
unsafe impl Sync for LibmHandle {}

static FP_MASK_KEY: PthreadKey = PthreadKey(UnsafeCell::new(0));
static FP_MASK_KEY_ONCE: PthreadOnce = PthreadOnce(UnsafeCell::new(libc::PTHREAD_ONCE_INIT));
static LOCALE_KEY: PthreadKey = PthreadKey(UnsafeCell::new(0));
static LOCALE_KEY_ONCE: PthreadOnce = PthreadOnce(UnsafeCell::new(libc::PTHREAD_ONCE_INIT));
static LIBM_HANDLE: LibmHandle = LibmHandle(UnsafeCell::new(core::ptr::null_mut()));
static LIBM_HANDLE_ONCE: PthreadOnce = PthreadOnce(UnsafeCell::new(libc::PTHREAD_ONCE_INIT));

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

const HOST_FE_INVALID: c_int = 0x01;
const HOST_FE_DIVBYZERO: c_int = 0x04;
const HOST_FE_OVERFLOW: c_int = 0x08;
const HOST_FE_UNDERFLOW: c_int = 0x10;
const HOST_FE_INEXACT: c_int = 0x20;
const HOST_FE_ALL_EXCEPT: c_int =
    HOST_FE_INVALID | HOST_FE_DIVBYZERO | HOST_FE_OVERFLOW | HOST_FE_UNDERFLOW | HOST_FE_INEXACT;

const HOST_FE_TONEAREST: c_int = 0;
const HOST_FE_DOWNWARD: c_int = 0x400;
const HOST_FE_UPWARD: c_int = 0x800;
const HOST_FE_TOWARDZERO: c_int = 0xc00;

const BADGE_FE_INVALID: c_int = 0x10;
const BADGE_FE_DIVBYZERO: c_int = 0x08;
const BADGE_FE_OVERFLOW: c_int = 0x04;
const BADGE_FE_UNDERFLOW: c_int = 0x02;
const BADGE_FE_INEXACT: c_int = 0x01;
const BADGE_FE_ALL_EXCEPT: c_int = BADGE_FE_INVALID
    | BADGE_FE_DIVBYZERO
    | BADGE_FE_OVERFLOW
    | BADGE_FE_UNDERFLOW
    | BADGE_FE_INEXACT;

const BADGE_FE_TONEAREST: c_int = 0;
const BADGE_FE_TOWARDZERO: c_int = 0x01;
const BADGE_FE_DOWNWARD: c_int = 0x02;
const BADGE_FE_UPWARD: c_int = 0x03;
const BADGE_FE_TONEAREST_MM: c_int = 0x04;
const BADGE_FE_RMODE_MASK: usize = 0x7;
const BADGE_FENV_ROUND_SHIFT: usize = 8;

const SIGNAL_NAMES: &[(c_int, &[u8])] = &[
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

const SIGNAL_ALIASES: &[(&[u8], c_int)] = &[(b"IOT", 6), (b"CLD", 20), (b"POLL", 23)];

type FeGetRoundFn = unsafe extern "C" fn() -> c_int;
type FeSetRoundFn = unsafe extern "C" fn(c_int) -> c_int;
type FeTestExceptFn = unsafe extern "C" fn(c_int) -> c_int;
type FeClearExceptFn = unsafe extern "C" fn(c_int) -> c_int;
type FeRaiseExceptFn = unsafe extern "C" fn(c_int) -> c_int;
type LgammaRFn = unsafe extern "C" fn(f64, *mut c_int) -> f64;
type LgammafRFn = unsafe extern "C" fn(f32, *mut c_int) -> f32;
type PowFn = unsafe extern "C" fn(f64, f64) -> f64;
type PowfFn = unsafe extern "C" fn(f32, f32) -> f32;
type SprintfFn = unsafe extern "C" fn(*mut c_char, *const c_char, ...) -> c_int;

unsafe extern "C" {
    #[link_name = "__cxa_atexit"]
    fn host_cxa_atexit(
        func: Option<unsafe extern "C" fn(*mut c_void)>,
        arg: *mut c_void,
        dso_handle: *mut c_void,
    ) -> c_int;
    #[link_name = "__errno_location"]
    fn host_errno_location() -> *mut c_int;
    fn wcstoll(value: *const _wchar_t, endptr: *mut *mut _wchar_t, base: c_int) -> i64;
    fn wcstoull(value: *const _wchar_t, endptr: *mut *mut _wchar_t, base: c_int) -> u64;
}

extern "C" fn init_libm_handle() {
    let handle = unsafe { libc::dlopen(c"libm.so.6".as_ptr(), libc::RTLD_NOW | libc::RTLD_LOCAL) };
    if handle.is_null() {
        crate::runtime::abort_with_message(b"why2025-badge-emu-abi failed to open libm.so.6\n")
    }

    unsafe {
        *LIBM_HANDLE.0.get() = handle;
    }
}

fn libm_handle() -> *mut c_void {
    let rc = unsafe { libc::pthread_once(LIBM_HANDLE_ONCE.0.get(), init_libm_handle) };
    if rc != 0 {
        crate::runtime::abort_with_message(
            b"why2025-badge-emu-abi failed to initialize libm handle\n",
        )
    }

    unsafe { *LIBM_HANDLE.0.get() }
}

unsafe fn resolve_libm_function<T: Copy>(symbol: &'static [u8]) -> T {
    let handle = libm_handle();
    unsafe {
        libc::dlerror();
        let resolved = libc::dlsym(handle, symbol.as_ptr().cast::<c_char>());
        let error = libc::dlerror();
        if error.is_null() && !resolved.is_null() {
            core::mem::transmute_copy::<*mut c_void, T>(&resolved)
        } else {
            let name = core::str::from_utf8(&symbol[..symbol.len().saturating_sub(1)])
                .unwrap_or("<invalid>");
            crate::runtime::abort_missing_host_symbol(name)
        }
    }
}

fn host_fegetround() -> c_int {
    let function: FeGetRoundFn = unsafe { resolve_libm_function(b"fegetround\0") };
    unsafe { function() }
}

fn host_fesetround(round: c_int) -> c_int {
    let function: FeSetRoundFn = unsafe { resolve_libm_function(b"fesetround\0") };
    unsafe { function(round) }
}

fn host_fetestexcept(excepts: c_int) -> c_int {
    let function: FeTestExceptFn = unsafe { resolve_libm_function(b"fetestexcept\0") };
    unsafe { function(excepts) }
}

fn host_feclearexcept(excepts: c_int) -> c_int {
    let function: FeClearExceptFn = unsafe { resolve_libm_function(b"feclearexcept\0") };
    unsafe { function(excepts) }
}

fn host_feraiseexcept(excepts: c_int) -> c_int {
    let function: FeRaiseExceptFn = unsafe { resolve_libm_function(b"feraiseexcept\0") };
    unsafe { function(excepts) }
}

fn host_pow(base: f64, exponent: f64) -> f64 {
    let function: PowFn = unsafe { resolve_libm_function(b"pow\0") };
    unsafe { function(base, exponent) }
}

fn host_powf(base: f32, exponent: f32) -> f32 {
    let function: PowfFn = unsafe { resolve_libm_function(b"powf\0") };
    unsafe { function(base, exponent) }
}

fn host_lgamma_r(value: f64, sign: *mut c_int) -> f64 {
    let function: LgammaRFn = unsafe { resolve_libm_function(b"lgamma_r\0") };
    unsafe { function(value, sign) }
}

fn host_lgammaf_r(value: f32, sign: *mut c_int) -> f32 {
    let function: LgammafRFn = unsafe { resolve_libm_function(b"lgammaf_r\0") };
    unsafe { function(value, sign) }
}

fn host_sprintf(buffer: *mut c_char, format: *const c_char, precision: c_int, value: f64) -> c_int {
    let function: SprintfFn = unsafe { runtime::resolve_next_function(b"sprintf\0") };
    unsafe { function(buffer, format, precision, value) }
}

unsafe extern "C" fn free_fp_mask_slot(value: *mut c_void) {
    if !value.is_null() {
        unsafe {
            libc::free(value);
        }
    }
}

unsafe extern "C" fn free_locale_slot(value: *mut c_void) {
    if !value.is_null() {
        unsafe {
            libc::free(value);
        }
    }
}

extern "C" fn init_fp_mask_key() {
    let rc = unsafe { libc::pthread_key_create(FP_MASK_KEY.0.get(), Some(free_fp_mask_slot)) };
    if rc != 0 {
        crate::runtime::abort_with_message(
            b"why2025-badge-emu-abi failed to initialize fp-mask TLS\n",
        )
    }
}

fn ensure_fp_mask_key() -> libc::pthread_key_t {
    let rc = unsafe { libc::pthread_once(FP_MASK_KEY_ONCE.0.get(), init_fp_mask_key) };
    if rc != 0 {
        crate::runtime::abort_with_message(
            b"why2025-badge-emu-abi failed to run fp-mask TLS init\n",
        )
    }

    unsafe { *FP_MASK_KEY.0.get() }
}

extern "C" fn init_locale_key() {
    let rc = unsafe { libc::pthread_key_create(LOCALE_KEY.0.get(), Some(free_locale_slot)) };
    if rc != 0 {
        crate::runtime::abort_with_message(
            b"why2025-badge-emu-abi failed to initialize locale TLS\n",
        )
    }
}

fn ensure_locale_key() -> libc::pthread_key_t {
    let rc = unsafe { libc::pthread_once(LOCALE_KEY_ONCE.0.get(), init_locale_key) };
    if rc != 0 {
        crate::runtime::abort_with_message(b"why2025-badge-emu-abi failed to run locale TLS init\n")
    }

    unsafe { *LOCALE_KEY.0.get() }
}

fn fp_mask_slot() -> *mut fp_except {
    let key = ensure_fp_mask_key();
    let existing = unsafe { libc::pthread_getspecific(key) }.cast::<fp_except>();
    if !existing.is_null() {
        return existing;
    }

    let slot = unsafe { libc::malloc(mem::size_of::<fp_except>().max(1)) }.cast::<fp_except>();
    if slot.is_null() {
        crate::runtime::abort_with_message(
            b"why2025-badge-emu-abi failed to allocate fp-mask TLS\n",
        )
    }

    unsafe {
        *slot = 0;
    }

    let rc = unsafe { libc::pthread_setspecific(key, slot.cast::<c_void>()) };
    if rc != 0 {
        unsafe {
            libc::free(slot.cast::<c_void>());
        }
        crate::runtime::abort_with_message(b"why2025-badge-emu-abi failed to install fp-mask TLS\n")
    }

    slot
}

fn locale_slot() -> *mut locale_t {
    let key = ensure_locale_key();
    let existing = unsafe { libc::pthread_getspecific(key) }.cast::<locale_t>();
    if !existing.is_null() {
        return existing;
    }

    let slot = unsafe { libc::malloc(mem::size_of::<locale_t>().max(1)) }.cast::<locale_t>();
    if slot.is_null() {
        crate::runtime::abort_with_message(b"why2025-badge-emu-abi failed to allocate locale TLS\n")
    }

    unsafe {
        *slot = -1;
    }

    let rc = unsafe { libc::pthread_setspecific(key, slot.cast::<c_void>()) };
    if rc != 0 {
        unsafe {
            libc::free(slot.cast::<c_void>());
        }
        crate::runtime::abort_with_message(b"why2025-badge-emu-abi failed to install locale TLS\n")
    }

    slot
}

fn host_round_mode_to_badge(mode: c_int) -> fp_rnd {
    match mode {
        HOST_FE_TONEAREST => FP_RN_CONST,
        HOST_FE_DOWNWARD => FP_RM_CONST,
        HOST_FE_UPWARD => FP_RP_CONST,
        HOST_FE_TOWARDZERO => FP_RZ_CONST,
        _ => FP_RN_CONST,
    }
}

fn badge_round_mode_to_host(mode: fp_rnd) -> Option<c_int> {
    match mode {
        FP_RN_CONST => Some(HOST_FE_TONEAREST),
        FP_RM_CONST => Some(HOST_FE_DOWNWARD),
        FP_RP_CONST => Some(HOST_FE_UPWARD),
        FP_RZ_CONST => Some(HOST_FE_TOWARDZERO),
        _ => None,
    }
}

fn host_excepts_to_badge(mask: c_int) -> fp_except {
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

fn host_excepts_to_badge_fe(mask: c_int) -> c_int {
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

fn badge_excepts_to_host(mask: fp_except) -> c_int {
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

fn badge_fe_excepts_to_host(mask: c_int) -> c_int {
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

fn host_round_mode_to_badge_fe(mode: c_int) -> c_int {
    match mode {
        HOST_FE_TONEAREST => BADGE_FE_TONEAREST,
        HOST_FE_DOWNWARD => BADGE_FE_DOWNWARD,
        HOST_FE_UPWARD => BADGE_FE_UPWARD,
        HOST_FE_TOWARDZERO => BADGE_FE_TOWARDZERO,
        _ => BADGE_FE_TONEAREST,
    }
}

fn badge_fe_round_to_host(mode: c_int) -> Option<c_int> {
    match mode {
        BADGE_FE_TONEAREST | BADGE_FE_TONEAREST_MM => Some(HOST_FE_TONEAREST),
        BADGE_FE_DOWNWARD => Some(HOST_FE_DOWNWARD),
        BADGE_FE_UPWARD => Some(HOST_FE_UPWARD),
        BADGE_FE_TOWARDZERO => Some(HOST_FE_TOWARDZERO),
        _ => None,
    }
}

fn encode_badge_fenv(flags: c_int, round: c_int) -> fenv_t {
    (flags as usize) | (((round as usize) & BADGE_FE_RMODE_MASK) << BADGE_FENV_ROUND_SHIFT)
}

fn decode_badge_fenv(env: fenv_t) -> (c_int, c_int) {
    let flags = (env as c_int) & BADGE_FE_ALL_EXCEPT;
    let round = ((env >> BADGE_FENV_ROUND_SHIFT) & BADGE_FE_RMODE_MASK) as c_int;
    (flags, round)
}

fn current_badge_fenv() -> fenv_t {
    let flags = host_excepts_to_badge_fe(host_fetestexcept(HOST_FE_ALL_EXCEPT));
    let round = host_round_mode_to_badge_fe(host_fegetround());
    encode_badge_fenv(flags, round)
}

fn store_fp_mask(mask: fp_except) {
    unsafe {
        *fp_mask_slot() = mask & FP_SUPPORTED_MASK;
    }
}

fn load_fp_mask() -> fp_except {
    unsafe { *fp_mask_slot() }
}

unsafe extern "C" fn atexit_trampoline(ctx: *mut c_void) {
    let callback = unsafe { core::mem::transmute::<*mut c_void, unsafe extern "C" fn()>(ctx) };
    unsafe { callback() };
}

fn clear_host_errno() {
    unsafe {
        let host_errno = host_errno_location();
        if !host_errno.is_null() {
            *host_errno = 0;
        }
    }
    runtime::set_errno(0);
}

fn sync_errno_from_host() {
    let value = unsafe {
        let host_errno = host_errno_location();
        if host_errno.is_null() { 0 } else { *host_errno }
    };
    runtime::set_errno(value);
}

fn raw_va_list(ap: VaList<'_, '_>) -> __builtin_va_list {
    unsafe { core::mem::transmute_copy::<VaList<'_, '_>, __builtin_va_list>(&ap) }
}

fn resolve_vdprintf_raw() -> unsafe extern "C" fn(c_int, *const c_char, __builtin_va_list) -> c_int
{
    unsafe { runtime::resolve_next_function(b"vdprintf\0") }
}

fn resolve_vprintf_raw() -> unsafe extern "C" fn(*const c_char, __builtin_va_list) -> c_int {
    unsafe { runtime::resolve_next_function(b"vprintf\0") }
}

fn resolve_vfprintf_raw()
-> unsafe extern "C" fn(*mut FILE, *const c_char, __builtin_va_list) -> c_int {
    unsafe { runtime::resolve_next_function(b"vfprintf\0") }
}

fn resolve_vsprintf_raw()
-> unsafe extern "C" fn(*mut c_char, *const c_char, __builtin_va_list) -> c_int {
    unsafe { runtime::resolve_next_function(b"vsprintf\0") }
}

fn resolve_vsnprintf_raw()
-> unsafe extern "C" fn(*mut c_char, usize, *const c_char, __builtin_va_list) -> c_int {
    unsafe { runtime::resolve_next_function(b"vsnprintf\0") }
}

fn resolve_vasprintf_raw()
-> unsafe extern "C" fn(*mut *mut c_char, *const c_char, __gnuc_va_list) -> c_int {
    unsafe { runtime::resolve_next_function(b"vasprintf\0") }
}

fn resolve_vscanf_raw() -> unsafe extern "C" fn(*const c_char, __builtin_va_list) -> c_int {
    unsafe { runtime::resolve_next_function(b"vscanf\0") }
}

fn resolve_vfscanf_raw()
-> unsafe extern "C" fn(*mut FILE, *const c_char, __builtin_va_list) -> c_int {
    unsafe { runtime::resolve_next_function(b"vfscanf\0") }
}

fn resolve_vsscanf_raw()
-> unsafe extern "C" fn(*const c_char, *const c_char, __builtin_va_list) -> c_int {
    unsafe { runtime::resolve_next_function(b"vsscanf\0") }
}

unsafe fn diprintf_with_args(fd: c_int, fmt: *const c_char, args: VaList<'_, '_>) -> c_int {
    clear_host_errno();
    let function = resolve_vdprintf_raw();
    let result =
        unsafe { args.with_copy(|mut copy| function(fd, fmt, raw_va_list(copy.as_va_list()))) };
    sync_errno_from_host();
    result
}

unsafe fn printf_with_args(fmt: *const c_char, args: VaList<'_, '_>) -> c_int {
    clear_host_errno();
    let function = resolve_vprintf_raw();
    let result =
        unsafe { args.with_copy(|mut copy| function(fmt, raw_va_list(copy.as_va_list()))) };
    sync_errno_from_host();
    result
}

unsafe fn fprintf_with_args(stream: *mut FILE, fmt: *const c_char, args: VaList<'_, '_>) -> c_int {
    clear_host_errno();
    let function = resolve_vfprintf_raw();
    let result =
        unsafe { args.with_copy(|mut copy| function(stream, fmt, raw_va_list(copy.as_va_list()))) };
    sync_errno_from_host();
    result
}

unsafe fn sprintf_with_args(buf: *mut c_char, fmt: *const c_char, args: VaList<'_, '_>) -> c_int {
    clear_host_errno();
    let function = resolve_vsprintf_raw();
    let result =
        unsafe { args.with_copy(|mut copy| function(buf, fmt, raw_va_list(copy.as_va_list()))) };
    sync_errno_from_host();
    result
}

unsafe fn snprintf_with_args(
    buf: *mut c_char,
    size: c_uint,
    fmt: *const c_char,
    args: VaList<'_, '_>,
) -> c_int {
    clear_host_errno();
    let function = resolve_vsnprintf_raw();
    let result = unsafe {
        args.with_copy(|mut copy| function(buf, size as usize, fmt, raw_va_list(copy.as_va_list())))
    };
    sync_errno_from_host();
    result
}

unsafe fn asprintf_with_args(
    strp: *mut *mut c_char,
    fmt: *const c_char,
    args: VaList<'_, '_>,
) -> c_int {
    clear_host_errno();
    let function = resolve_vasprintf_raw();
    let result =
        unsafe { args.with_copy(|mut copy| function(strp, fmt, raw_va_list(copy.as_va_list()))) };
    sync_errno_from_host();
    result
}

unsafe fn asnprintf_with_args(
    str_: *mut c_char,
    lenp: *mut usize,
    fmt: *const c_char,
    args: VaList<'_, '_>,
) -> *mut c_char {
    if lenp.is_null() || fmt.is_null() {
        runtime::set_errno(libc::EINVAL);
        return ptr::null_mut();
    }

    clear_host_errno();

    let mut capacity = unsafe { *lenp };
    let mut buffer = str_;
    let mut allocated = false;
    let function = resolve_vsnprintf_raw();

    let required = unsafe {
        args.with_copy(|mut copy| function(ptr::null_mut(), 0, fmt, raw_va_list(copy.as_va_list())))
    };
    if required < 0 {
        sync_errno_from_host();
        return ptr::null_mut();
    }

    let required_len = required as usize;
    let required_capacity = required_len.saturating_add(1);

    if buffer.is_null() || capacity < required_capacity {
        buffer = unsafe { libc::malloc(required_capacity) }.cast::<c_char>();
        if buffer.is_null() {
            runtime::set_errno(libc::ENOMEM);
            return ptr::null_mut();
        }

        capacity = required_capacity;
        allocated = true;
    }

    let written = unsafe {
        args.with_copy(|mut copy| function(buffer, capacity, fmt, raw_va_list(copy.as_va_list())))
    };
    if written < 0 {
        if allocated {
            unsafe { libc::free(buffer.cast()) };
        }
        sync_errno_from_host();
        return ptr::null_mut();
    }

    if allocated {
        let shrunk = unsafe { libc::realloc(buffer.cast(), required_capacity) }.cast::<c_char>();
        if !shrunk.is_null() {
            buffer = shrunk;
        }
    }

    unsafe {
        *lenp = written as usize;
    }
    sync_errno_from_host();
    buffer
}

unsafe fn scanf_with_args(fmt: *const c_char, args: VaList<'_, '_>) -> c_int {
    clear_host_errno();
    let function = resolve_vscanf_raw();
    let result =
        unsafe { args.with_copy(|mut copy| function(fmt, raw_va_list(copy.as_va_list()))) };
    sync_errno_from_host();
    result
}

unsafe fn fscanf_with_args(stream: *mut FILE, fmt: *const c_char, args: VaList<'_, '_>) -> c_int {
    clear_host_errno();
    let function = resolve_vfscanf_raw();
    let result =
        unsafe { args.with_copy(|mut copy| function(stream, fmt, raw_va_list(copy.as_va_list()))) };
    sync_errno_from_host();
    result
}

unsafe fn sscanf_with_args(buf: *const c_char, fmt: *const c_char, args: VaList<'_, '_>) -> c_int {
    clear_host_errno();
    let function = resolve_vsscanf_raw();
    let result =
        unsafe { args.with_copy(|mut copy| function(buf, fmt, raw_va_list(copy.as_va_list()))) };
    sync_errno_from_host();
    result
}

unsafe fn vprintf_raw(fmt: *const c_char, ap: __builtin_va_list) -> c_int {
    clear_host_errno();
    let result = unsafe { resolve_vprintf_raw()(fmt, ap) };
    sync_errno_from_host();
    result
}

unsafe fn vfprintf_raw(stream: *mut FILE, fmt: *const c_char, ap: __builtin_va_list) -> c_int {
    clear_host_errno();
    let result = unsafe { resolve_vfprintf_raw()(stream, fmt, ap) };
    sync_errno_from_host();
    result
}

unsafe fn vsprintf_raw(buf: *mut c_char, fmt: *const c_char, ap: __builtin_va_list) -> c_int {
    clear_host_errno();
    let result = unsafe { resolve_vsprintf_raw()(buf, fmt, ap) };
    sync_errno_from_host();
    result
}

unsafe fn vsnprintf_raw(
    buf: *mut c_char,
    size: c_uint,
    fmt: *const c_char,
    ap: __builtin_va_list,
) -> c_int {
    clear_host_errno();
    let result = unsafe { resolve_vsnprintf_raw()(buf, size as usize, fmt, ap) };
    sync_errno_from_host();
    result
}

unsafe fn vasprintf_raw(strp: *mut *mut c_char, fmt: *const c_char, ap: __gnuc_va_list) -> c_int {
    clear_host_errno();
    let result = unsafe { resolve_vasprintf_raw()(strp, fmt, ap) };
    sync_errno_from_host();
    result
}

unsafe fn vscanf_raw(fmt: *const c_char, ap: __builtin_va_list) -> c_int {
    clear_host_errno();
    let result = unsafe { resolve_vscanf_raw()(fmt, ap) };
    sync_errno_from_host();
    result
}

unsafe fn vfscanf_raw(stream: *mut FILE, fmt: *const c_char, ap: __builtin_va_list) -> c_int {
    clear_host_errno();
    let result = unsafe { resolve_vfscanf_raw()(stream, fmt, ap) };
    sync_errno_from_host();
    result
}

unsafe fn vsscanf_raw(buf: *const c_char, fmt: *const c_char, ap: __builtin_va_list) -> c_int {
    clear_host_errno();
    let result = unsafe { resolve_vsscanf_raw()(buf, fmt, ap) };
    sync_errno_from_host();
    result
}

fn write_c_string_or_placeholder(value: *const c_char, placeholder: &[u8]) {
    if value.is_null() {
        runtime::write_stderr(placeholder);
        return;
    }

    let bytes = unsafe { CStr::from_ptr(value).to_bytes() };
    runtime::write_stderr(bytes);
}

fn write_decimal(value: c_int) {
    let mut remaining = value as i64;
    let mut buffer = [0u8; 32];
    let mut len = 0usize;

    if remaining == 0 {
        runtime::write_stderr(b"0");
        return;
    }

    if remaining < 0 {
        runtime::write_stderr(b"-");
        remaining = -remaining;
    }

    while remaining != 0 {
        buffer[len] = b'0' + (remaining % 10) as u8;
        len += 1;
        remaining /= 10;
    }

    while len != 0 {
        len -= 1;
        runtime::write_stderr(&buffer[len..len + 1]);
    }
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

fn is_ascii_alnum(value: c_int) -> c_int {
    let byte = value as u8;
    (byte.is_ascii_alphanumeric()) as c_int
}

fn is_ascii_alpha(value: c_int) -> c_int {
    let byte = value as u8;
    (byte.is_ascii_alphabetic()) as c_int
}

fn is_ascii_blank(value: c_int) -> c_int {
    ((value as u8) == b' ' || (value as u8) == b'\t') as c_int
}

fn is_ascii_cntrl(value: c_int) -> c_int {
    let byte = value as u8;
    (byte.is_ascii_control()) as c_int
}

fn is_ascii_digit(value: c_int) -> c_int {
    let byte = value as u8;
    (byte.is_ascii_digit()) as c_int
}

fn is_ascii_graph(value: c_int) -> c_int {
    let byte = value as u8;
    (byte.is_ascii_graphic()) as c_int
}

fn is_ascii_lower(value: c_int) -> c_int {
    let byte = value as u8;
    (byte.is_ascii_lowercase()) as c_int
}

fn is_ascii_print(value: c_int) -> c_int {
    let byte = value as u8;
    (byte.is_ascii_graphic() || byte == b' ') as c_int
}

fn is_ascii_punct(value: c_int) -> c_int {
    let byte = value as u8;
    (byte.is_ascii_punctuation()) as c_int
}

fn is_ascii_space(value: c_int) -> c_int {
    matches!(value as u8, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r') as c_int
}

fn is_ascii_upper(value: c_int) -> c_int {
    let byte = value as u8;
    (byte.is_ascii_uppercase()) as c_int
}

fn is_ascii_xdigit(value: c_int) -> c_int {
    let byte = value as u8;
    (byte.is_ascii_hexdigit()) as c_int
}

fn is_ascii(value: c_int) -> c_int {
    ((value & !0x7f) == 0) as c_int
}

unsafe fn transform_c_string_in_place(ptr_: *mut c_char, transform: fn(u8) -> u8) -> *mut c_char {
    if ptr_.is_null() {
        return ptr::null_mut();
    }

    let mut cursor = ptr_;
    while unsafe { *cursor } != 0 {
        unsafe {
            *cursor = transform(*cursor as u8) as c_char;
            cursor = cursor.add(1);
        }
    }

    ptr_
}

unsafe fn write_c_string_bytes(dst: *mut c_char, bytes: &[u8]) {
    for (index, byte) in bytes.iter().copied().enumerate() {
        unsafe {
            *dst.add(index) = byte as c_char;
        }
    }

    unsafe {
        *dst.add(bytes.len()) = 0;
    }
}

fn normalize_signal_name(bytes: &[u8]) -> &[u8] {
    if bytes.len() >= 3 && bytes[..3].eq_ignore_ascii_case(b"SIG") {
        &bytes[3..]
    } else {
        bytes
    }
}

fn signal_name(signum: c_int) -> Option<&'static [u8]> {
    SIGNAL_NAMES
        .iter()
        .find(|(candidate, _)| *candidate == signum)
        .map(|(_, name)| *name)
}

fn parse_signal_name(bytes: &[u8]) -> Option<c_int> {
    let normalized = normalize_signal_name(bytes);

    if let Ok(text) = core::str::from_utf8(normalized) {
        if let Ok(value) = text.parse::<c_int>() {
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

unsafe fn format_unsigned_to_buffer(
    value: u64,
    dst: *mut c_char,
    radix: u32,
    negative: bool,
) -> *mut c_char {
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
            *dst = b'-' as c_char;
        }
        out_index = 1;
    }

    for index in 0..len {
        unsafe {
            *dst.add(out_index + index) = scratch[len - 1 - index] as c_char;
        }
    }

    unsafe {
        *dst.add(out_index + len) = 0;
    }

    dst
}

unsafe fn wide_strlen(ptr_: *const wchar_t) -> usize {
    if ptr_.is_null() {
        return 0;
    }

    let mut len = 0usize;
    while unsafe { *ptr_.add(len) } != 0 {
        len += 1;
    }
    len
}

unsafe fn wide_strnlen(ptr_: *const wchar_t, max_len: usize) -> usize {
    if ptr_.is_null() {
        return 0;
    }

    let mut len = 0usize;
    while len < max_len && unsafe { *ptr_.add(len) } != 0 {
        len += 1;
    }
    len
}

fn compare_f64_nan_high(left: f64, right: f64) -> c_int {
    match left.partial_cmp(&right) {
        Some(core::cmp::Ordering::Less) => -1,
        Some(core::cmp::Ordering::Equal) => 0,
        Some(core::cmp::Ordering::Greater) => 1,
        None => 1,
    }
}

fn compare_f64_nan_low(left: f64, right: f64) -> c_int {
    match left.partial_cmp(&right) {
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

#[unsafe(no_mangle)]
pub extern "C" fn atexit(callback: Option<unsafe extern "C" fn()>) -> c_int {
    let Some(callback) = callback else {
        runtime::set_errno(libc::EINVAL);
        return -1;
    };

    clear_host_errno();
    let status = unsafe {
        host_cxa_atexit(
            Some(atexit_trampoline),
            callback as *const () as *mut c_void,
            core::ptr::null_mut(),
        )
    };
    sync_errno_from_host();
    status
}

#[unsafe(no_mangle)]
pub extern "C" fn __assert_func(
    file: *const c_char,
    line: c_int,
    function: *const c_char,
    expression: *const c_char,
) -> ! {
    runtime::write_stderr(b"assertion \"");
    write_c_string_or_placeholder(expression, b"<unknown expression>");
    runtime::write_stderr(b"\" failed: file \"");
    write_c_string_or_placeholder(file, b"<unknown file>");
    runtime::write_stderr(b"\", line ");
    write_decimal(line);
    if !function.is_null() {
        runtime::write_stderr(b", function: ");
        write_c_string_or_placeholder(function, b"<unknown function>");
    }
    runtime::write_stderr(b"\n");
    runtime::abort_process()
}

#[unsafe(no_mangle)]
pub extern "C" fn atoff(value: *const c_char) -> f32 {
    unsafe { libc::atof(value) as f32 }
}

#[unsafe(no_mangle)]
pub extern "C" fn fls(value: c_int) -> c_int {
    for bit in (0..32).rev() {
        if (value & (1 << bit)) != 0 {
            return bit + 1;
        }
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn flsl(value: c_long) -> c_int {
    for bit in (0..64).rev() {
        if (value & (1 << bit)) != 0 {
            return bit + 1;
        }
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn flsll(value: c_longlong) -> c_int {
    for bit in (0..64).rev() {
        if (value & (1 << bit)) != 0 {
            return bit + 1;
        }
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn __adddf3(left: f64, right: f64) -> f64 {
    left + right
}

#[unsafe(no_mangle)]
pub extern "C" fn __subdf3(left: f64, right: f64) -> f64 {
    left - right
}

#[unsafe(no_mangle)]
pub extern "C" fn __muldf3(left: f64, right: f64) -> f64 {
    left * right
}

#[unsafe(no_mangle)]
pub extern "C" fn __divdf3(left: f64, right: f64) -> f64 {
    left / right
}

#[unsafe(no_mangle)]
pub extern "C" fn __eqdf2(left: f64, right: f64) -> c_int {
    compare_f64_nan_high(left, right)
}

#[unsafe(no_mangle)]
pub extern "C" fn __gedf2(left: f64, right: f64) -> c_int {
    compare_f64_nan_low(left, right)
}

#[unsafe(no_mangle)]
pub extern "C" fn __gtdf2(left: f64, right: f64) -> c_int {
    compare_f64_nan_low(left, right)
}

#[unsafe(no_mangle)]
pub extern "C" fn __ledf2(left: f64, right: f64) -> c_int {
    compare_f64_nan_high(left, right)
}

#[unsafe(no_mangle)]
pub extern "C" fn __ltdf2(left: f64, right: f64) -> c_int {
    compare_f64_nan_high(left, right)
}

#[unsafe(no_mangle)]
pub extern "C" fn __extendsfdf2(value: f32) -> f64 {
    value as f64
}

#[unsafe(no_mangle)]
pub extern "C" fn __truncdfsf2(value: f64) -> f32 {
    value as f32
}

#[unsafe(no_mangle)]
pub extern "C" fn __extendhfsf2(value: __BindgenFloat16) -> f32 {
    half_bits_to_f32(value.0)
}

#[unsafe(no_mangle)]
pub extern "C" fn __truncsfhf2(value: f32) -> __BindgenFloat16 {
    __BindgenFloat16(f32_to_half_bits(value))
}

#[unsafe(no_mangle)]
pub extern "C" fn __fixdfsi(value: f64) -> i32 {
    value as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn __fixdfdi(value: f64) -> i64 {
    value as i64
}

#[unsafe(no_mangle)]
pub extern "C" fn __fixunsdfsi(value: f64) -> u32 {
    value as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn __floatdisf(value: i64) -> f32 {
    value as f32
}

#[unsafe(no_mangle)]
pub extern "C" fn __floatsidf(value: i32) -> f64 {
    value as f64
}

#[unsafe(no_mangle)]
pub extern "C" fn __floatundidf(value: u64) -> f64 {
    value as f64
}

#[unsafe(no_mangle)]
pub extern "C" fn __floatundisf(value: u64) -> f32 {
    value as f32
}

#[unsafe(no_mangle)]
pub extern "C" fn __floatunsidf(value: u32) -> f64 {
    value as f64
}

#[unsafe(no_mangle)]
pub extern "C" fn __issignalingf(value: f32) -> c_int {
    let bits = value.to_bits();
    let exponent = bits & 0x7f80_0000;
    let mantissa = bits & 0x007f_ffff;
    let quiet_bit = 0x0040_0000;

    (exponent == 0x7f80_0000 && mantissa != 0 && (mantissa & quiet_bit) == 0) as c_int
}

#[unsafe(no_mangle)]
pub extern "C" fn __nedf2(left: f64, right: f64) -> c_int {
    compare_f64_nan_high(left, right)
}

#[unsafe(no_mangle)]
pub extern "C" fn __divdi3(left: i64, right: i64) -> i64 {
    left / right
}

#[unsafe(no_mangle)]
pub extern "C" fn __udivdi3(left: u64, right: u64) -> u64 {
    left / right
}

#[unsafe(no_mangle)]
pub extern "C" fn __umoddi3(left: u64, right: u64) -> u64 {
    left % right
}

#[unsafe(no_mangle)]
pub extern "C" fn __clzsi2(value: u32) -> c_int {
    value.leading_zeros() as c_int
}

#[unsafe(no_mangle)]
pub extern "C" fn __popcountsi2(value: u32) -> c_int {
    value.count_ones() as c_int
}

#[unsafe(no_mangle)]
pub extern "C" fn fpgetround() -> fp_rnd {
    host_round_mode_to_badge(host_fegetround())
}

#[unsafe(no_mangle)]
pub extern "C" fn fpsetround(value: fp_rnd) -> fp_rnd {
    let previous = fpgetround();
    if let Some(host_mode) = badge_round_mode_to_host(value) {
        host_fesetround(host_mode);
    }
    previous
}

#[unsafe(no_mangle)]
pub extern "C" fn fpgetmask() -> fp_except {
    load_fp_mask()
}

#[unsafe(no_mangle)]
pub extern "C" fn fpsetmask(value: fp_except) -> fp_except {
    let previous = fpgetmask();
    store_fp_mask(value);
    previous
}

#[unsafe(no_mangle)]
pub extern "C" fn fpgetsticky() -> fp_except {
    host_excepts_to_badge(host_fetestexcept(HOST_FE_ALL_EXCEPT))
}

#[unsafe(no_mangle)]
pub extern "C" fn fpsetsticky(value: fp_except) -> fp_except {
    let previous = fpgetsticky();
    host_feclearexcept(HOST_FE_ALL_EXCEPT);
    let requested = badge_excepts_to_host(value & FP_SUPPORTED_MASK);
    if requested != 0 {
        host_feraiseexcept(requested);
    }
    previous
}

#[unsafe(no_mangle)]
pub extern "C" fn feclearexcept(excepts: c_int) -> c_int {
    host_feclearexcept(badge_fe_excepts_to_host(excepts & BADGE_FE_ALL_EXCEPT))
}

#[unsafe(no_mangle)]
pub extern "C" fn fegetexceptflag(flagp: *mut fexcept_t, excepts: c_int) -> c_int {
    if flagp.is_null() {
        runtime::set_errno(libc::EINVAL);
        return -1;
    }

    let masked = excepts & BADGE_FE_ALL_EXCEPT;
    let flags = host_excepts_to_badge_fe(host_fetestexcept(HOST_FE_ALL_EXCEPT)) & masked;
    unsafe {
        *flagp = flags as fexcept_t;
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn fesetexceptflag(flagp: *const fexcept_t, excepts: c_int) -> c_int {
    if flagp.is_null() {
        runtime::set_errno(libc::EINVAL);
        return -1;
    }

    let masked = excepts & BADGE_FE_ALL_EXCEPT;
    let new_flags = badge_fe_excepts_to_host((unsafe { *flagp } as c_int) & masked);
    let host_mask = badge_fe_excepts_to_host(masked);

    host_feclearexcept(host_mask);
    if new_flags != 0 {
        host_feraiseexcept(new_flags);
    }

    0
}

#[unsafe(no_mangle)]
pub extern "C" fn feraiseexcept(excepts: c_int) -> c_int {
    host_feraiseexcept(badge_fe_excepts_to_host(excepts & BADGE_FE_ALL_EXCEPT))
}

#[unsafe(no_mangle)]
pub extern "C" fn fetestexcept(excepts: c_int) -> c_int {
    let masked = excepts & BADGE_FE_ALL_EXCEPT;
    let host_mask = badge_fe_excepts_to_host(masked);
    let active = host_fetestexcept(host_mask);
    host_excepts_to_badge_fe(active) & masked
}

#[unsafe(no_mangle)]
pub extern "C" fn fegetround() -> c_int {
    host_round_mode_to_badge_fe(host_fegetround())
}

#[unsafe(no_mangle)]
pub extern "C" fn fesetround(rounding_mode: c_int) -> c_int {
    let Some(host_mode) = badge_fe_round_to_host(rounding_mode) else {
        return 1;
    };
    host_fesetround(host_mode)
}

#[unsafe(no_mangle)]
pub extern "C" fn fegetenv(envp: *mut fenv_t) -> c_int {
    if envp.is_null() {
        runtime::set_errno(libc::EINVAL);
        return -1;
    }

    unsafe {
        *envp = current_badge_fenv();
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn feholdexcept(envp: *mut fenv_t) -> c_int {
    let status = fegetenv(envp);
    if status != 0 {
        return status;
    }

    host_feclearexcept(HOST_FE_ALL_EXCEPT);
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn fesetenv(envp: *const fenv_t) -> c_int {
    if envp.is_null() {
        runtime::set_errno(libc::EINVAL);
        return -1;
    }

    let (flags, round) = decode_badge_fenv(unsafe { *envp });
    let Some(host_round) = badge_fe_round_to_host(round) else {
        return 1;
    };

    host_fesetround(host_round);
    host_feclearexcept(HOST_FE_ALL_EXCEPT);
    let host_flags = badge_fe_excepts_to_host(flags);
    if host_flags != 0 {
        host_feraiseexcept(host_flags);
    }

    0
}

#[unsafe(no_mangle)]
pub extern "C" fn feupdateenv(envp: *const fenv_t) -> c_int {
    let saved = host_fetestexcept(HOST_FE_ALL_EXCEPT);
    let status = fesetenv(envp);
    if status != 0 {
        return status;
    }

    if saved != 0 {
        host_feraiseexcept(saved);
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn isalnum_l(value: c_int, locale: locale_t) -> c_int {
    let _ = locale;
    is_ascii_alnum(value)
}

#[unsafe(no_mangle)]
pub extern "C" fn isalpha_l(value: c_int, locale: locale_t) -> c_int {
    let _ = locale;
    is_ascii_alpha(value)
}

#[unsafe(no_mangle)]
pub extern "C" fn isblank_l(value: c_int, locale: locale_t) -> c_int {
    let _ = locale;
    is_ascii_blank(value)
}

#[unsafe(no_mangle)]
pub extern "C" fn iscntrl_l(value: c_int, locale: locale_t) -> c_int {
    let _ = locale;
    is_ascii_cntrl(value)
}

#[unsafe(no_mangle)]
pub extern "C" fn isdigit_l(value: c_int, locale: locale_t) -> c_int {
    let _ = locale;
    is_ascii_digit(value)
}

#[unsafe(no_mangle)]
pub extern "C" fn isgraph_l(value: c_int, locale: locale_t) -> c_int {
    let _ = locale;
    is_ascii_graph(value)
}

#[unsafe(no_mangle)]
pub extern "C" fn islower_l(value: c_int, locale: locale_t) -> c_int {
    let _ = locale;
    is_ascii_lower(value)
}

#[unsafe(no_mangle)]
pub extern "C" fn isprint_l(value: c_int, locale: locale_t) -> c_int {
    let _ = locale;
    is_ascii_print(value)
}

#[unsafe(no_mangle)]
pub extern "C" fn ispunct_l(value: c_int, locale: locale_t) -> c_int {
    let _ = locale;
    is_ascii_punct(value)
}

#[unsafe(no_mangle)]
pub extern "C" fn isspace_l(value: c_int, locale: locale_t) -> c_int {
    let _ = locale;
    is_ascii_space(value)
}

#[unsafe(no_mangle)]
pub extern "C" fn isupper_l(value: c_int, locale: locale_t) -> c_int {
    let _ = locale;
    is_ascii_upper(value)
}

#[unsafe(no_mangle)]
pub extern "C" fn isxdigit_l(value: c_int, locale: locale_t) -> c_int {
    let _ = locale;
    is_ascii_xdigit(value)
}

#[unsafe(no_mangle)]
pub extern "C" fn tolower_l(value: c_int, locale: locale_t) -> c_int {
    let _ = locale;
    ascii_lower(value as u8) as c_int
}

#[unsafe(no_mangle)]
pub extern "C" fn toupper_l(value: c_int, locale: locale_t) -> c_int {
    let _ = locale;
    ascii_upper(value as u8) as c_int
}

#[unsafe(no_mangle)]
pub extern "C" fn isascii_l(value: c_int, locale: locale_t) -> c_int {
    let _ = locale;
    is_ascii(value)
}

#[unsafe(no_mangle)]
pub extern "C" fn itoa(value: c_int, buffer: *mut c_char, radix: c_int) -> *mut c_char {
    if buffer.is_null() || !(2..=36).contains(&radix) {
        runtime::set_errno(libc::EINVAL);
        return ptr::null_mut();
    }

    runtime::set_errno(0);
    let negative = radix == 10 && value < 0;
    let magnitude = if negative {
        (value as i64).unsigned_abs()
    } else {
        (value as c_uint) as u64
    };

    unsafe { format_unsigned_to_buffer(magnitude, buffer, radix as u32, negative) }
}

#[unsafe(no_mangle)]
pub extern "C" fn sig2str(signum: c_int, buffer: *mut c_char) -> c_int {
    let Some(name) = signal_name(signum) else {
        runtime::set_errno(libc::EINVAL);
        return -1;
    };
    if buffer.is_null() {
        runtime::set_errno(libc::EINVAL);
        return -1;
    }

    runtime::set_errno(0);
    unsafe {
        write_c_string_bytes(buffer, name);
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn str2sig(name: *const c_char, signum_out: *mut c_int) -> c_int {
    if name.is_null() || signum_out.is_null() {
        runtime::set_errno(libc::EINVAL);
        return -1;
    }

    let bytes = unsafe { CStr::from_ptr(name) }.to_bytes();
    let Some(signum) = parse_signal_name(bytes) else {
        runtime::set_errno(libc::EINVAL);
        return -1;
    };

    runtime::set_errno(0);
    unsafe {
        *signum_out = signum;
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn strlwr(value: *mut c_char) -> *mut c_char {
    unsafe { transform_c_string_in_place(value, ascii_lower) }
}

#[unsafe(no_mangle)]
pub extern "C" fn strnstr(
    haystack: *const c_char,
    needle: *const c_char,
    max_len: usize,
) -> *mut c_char {
    if haystack.is_null() || needle.is_null() {
        return ptr::null_mut();
    }

    let needle_len = unsafe { CStr::from_ptr(needle) }.to_bytes().len();
    if needle_len == 0 {
        return haystack as *mut c_char;
    }

    let haystack = haystack.cast::<u8>();
    let needle = needle.cast::<u8>();

    for offset in 0..max_len {
        let current = unsafe { *haystack.add(offset) };
        if current == 0 || offset + needle_len > max_len {
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
            return unsafe { haystack.add(offset) } as *mut c_char;
        }
    }

    ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "C" fn strupr(value: *mut c_char) -> *mut c_char {
    unsafe { transform_c_string_in_place(value, ascii_upper) }
}

#[unsafe(no_mangle)]
pub extern "C" fn timingsafe_bcmp(left: *const c_void, right: *const c_void, len: usize) -> c_int {
    if len == 0 {
        return 0;
    }

    let left = unsafe { slice::from_raw_parts(left.cast::<u8>(), len) };
    let right = unsafe { slice::from_raw_parts(right.cast::<u8>(), len) };
    let mut diff = 0u8;
    for index in 0..len {
        diff |= left[index] ^ right[index];
    }
    diff as c_int
}

#[unsafe(no_mangle)]
pub extern "C" fn timingsafe_memcmp(
    left: *const c_void,
    right: *const c_void,
    len: usize,
) -> c_int {
    if len == 0 {
        return 0;
    }

    let left = unsafe { slice::from_raw_parts(left.cast::<u8>(), len) };
    let right = unsafe { slice::from_raw_parts(right.cast::<u8>(), len) };
    let mut left_less = 0u32;
    let mut right_less = 0u32;

    for index in 0..len {
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

#[unsafe(no_mangle)]
pub extern "C" fn toascii_l(value: c_int, locale: locale_t) -> c_int {
    let _ = locale;
    value & 0x7f
}

#[unsafe(no_mangle)]
pub extern "C" fn uselocale(locale: locale_t) -> locale_t {
    let slot = locale_slot();
    let previous = unsafe { *slot };
    if locale != 0 {
        unsafe {
            *slot = locale;
        }
    }
    previous
}

#[unsafe(no_mangle)]
pub extern "C" fn utoa(value: c_uint, buffer: *mut c_char, radix: c_int) -> *mut c_char {
    if buffer.is_null() || !(2..=36).contains(&radix) {
        runtime::set_errno(libc::EINVAL);
        return ptr::null_mut();
    }

    runtime::set_errno(0);
    unsafe { format_unsigned_to_buffer(value as u64, buffer, radix as u32, false) }
}

#[unsafe(no_mangle)]
pub extern "C" fn strtoimax_l(
    value: *const c_char,
    endptr: *mut *mut c_char,
    base: c_int,
    locale: locale_t,
) -> intmax_t {
    let _ = locale;
    clear_host_errno();
    let result = unsafe { libc::strtoll(value.cast(), endptr.cast(), base) } as intmax_t;
    sync_errno_from_host();
    result
}

#[unsafe(no_mangle)]
pub extern "C" fn strtoumax_l(
    value: *const c_char,
    endptr: *mut *mut c_char,
    base: c_int,
    locale: locale_t,
) -> uintmax_t {
    let _ = locale;
    clear_host_errno();
    let result = unsafe { libc::strtoull(value.cast(), endptr.cast(), base) } as uintmax_t;
    sync_errno_from_host();
    result
}

#[unsafe(no_mangle)]
pub extern "C" fn wcstoimax_l(
    value: *const _wchar_t,
    endptr: *mut *mut _wchar_t,
    base: c_int,
    locale: locale_t,
) -> intmax_t {
    let _ = locale;
    clear_host_errno();
    let result = unsafe { wcstoll(value, endptr, base) } as intmax_t;
    sync_errno_from_host();
    result
}

#[unsafe(no_mangle)]
pub extern "C" fn wcslcpy(dst: *mut wchar_t, src: *const wchar_t, size: usize) -> usize {
    if src.is_null() || (dst.is_null() && size != 0) {
        runtime::set_errno(libc::EINVAL);
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

#[unsafe(no_mangle)]
pub extern "C" fn wcslcat(dst: *mut wchar_t, src: *const wchar_t, size: usize) -> usize {
    if src.is_null() || (dst.is_null() && size != 0) {
        runtime::set_errno(libc::EINVAL);
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

#[unsafe(no_mangle)]
pub extern "C" fn wcstoumax_l(
    value: *const _wchar_t,
    endptr: *mut *mut _wchar_t,
    base: c_int,
    locale: locale_t,
) -> uintmax_t {
    let _ = locale;
    clear_host_errno();
    let result = unsafe { wcstoull(value, endptr, base) } as uintmax_t;
    sync_errno_from_host();
    result
}

#[unsafe(no_mangle)]
pub extern "C" fn gammaf_r(value: f32, sign: *mut c_int) -> f32 {
    host_lgammaf_r(value, sign)
}

#[unsafe(no_mangle)]
pub extern "C" fn gamma_r(value: f64, sign: *mut c_int) -> f64 {
    host_lgamma_r(value, sign)
}

#[unsafe(no_mangle)]
pub extern "C" fn infinity() -> f64 {
    f64::INFINITY
}

#[unsafe(no_mangle)]
pub extern "C" fn infinityf() -> f32 {
    f32::INFINITY
}

#[unsafe(no_mangle)]
pub extern "C" fn exp10(value: f64) -> f64 {
    host_pow(10.0, value)
}

#[unsafe(no_mangle)]
pub extern "C" fn pow10(value: f64) -> f64 {
    exp10(value)
}

#[unsafe(no_mangle)]
pub extern "C" fn exp10f(value: f32) -> f32 {
    host_powf(10.0, value)
}

#[unsafe(no_mangle)]
pub extern "C" fn pow10f(value: f32) -> f32 {
    exp10f(value)
}

#[unsafe(no_mangle)]
pub extern "C" fn gcvtf(value: f32, digits: c_int, buffer: *mut c_char) -> *mut c_char {
    if buffer.is_null() {
        runtime::set_errno(libc::EINVAL);
        return ptr::null_mut();
    }

    clear_host_errno();
    let _ = host_sprintf(buffer, c"%.*g".as_ptr(), digits, value as f64);
    sync_errno_from_host();
    buffer
}

#[unsafe(no_mangle)]
pub extern "C" fn gcvtl(value: u128, digits: c_int, buffer: *mut c_char) -> *mut c_char {
    let _ = (value, digits, buffer);
    runtime::abort_unimplemented_symbol("gcvtl", "libc_fallback")
}

#[unsafe(no_mangle)]
pub extern "C" fn funopen(
    cookie: *const c_void,
    readfn: Option<
        unsafe extern "C" fn(cookie: *mut c_void, buf: *mut c_void, n: usize) -> _ssize_t,
    >,
    writefn: Option<
        unsafe extern "C" fn(cookie: *mut c_void, buf: *const c_void, n: usize) -> _ssize_t,
    >,
    seekfn: Option<
        unsafe extern "C" fn(cookie: *mut c_void, off: __off_t, whence: c_int) -> __off_t,
    >,
    closefn: Option<unsafe extern "C" fn(cookie: *mut c_void) -> c_int>,
) -> *mut FILE {
    let _ = (cookie, readfn, writefn, seekfn, closefn);
    runtime::abort_unimplemented_symbol("funopen", "libc_fallback")
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn diprintf(fd: c_int, fmt: *const c_char, mut args: ...) -> c_int {
    unsafe { diprintf_with_args(fd, fmt, args.as_va_list()) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn printf(fmt: *const c_char, mut args: ...) -> c_int {
    unsafe { printf_with_args(fmt, args.as_va_list()) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fprintf(stream: *mut FILE, fmt: *const c_char, mut args: ...) -> c_int {
    unsafe { fprintf_with_args(stream, fmt, args.as_va_list()) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vprintf(fmt: *const c_char, ap: __builtin_va_list) -> c_int {
    unsafe { vprintf_raw(fmt, ap) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vfprintf(
    stream: *mut FILE,
    fmt: *const c_char,
    ap: __builtin_va_list,
) -> c_int {
    unsafe { vfprintf_raw(stream, fmt, ap) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sprintf(buf: *mut c_char, fmt: *const c_char, mut args: ...) -> c_int {
    unsafe { sprintf_with_args(buf, fmt, args.as_va_list()) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn snprintf(
    buf: *mut c_char,
    size: c_uint,
    fmt: *const c_char,
    mut args: ...
) -> c_int {
    unsafe { snprintf_with_args(buf, size, fmt, args.as_va_list()) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vsprintf(
    buf: *mut c_char,
    fmt: *const c_char,
    ap: __builtin_va_list,
) -> c_int {
    unsafe { vsprintf_raw(buf, fmt, ap) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vsnprintf(
    buf: *mut c_char,
    size: c_uint,
    fmt: *const c_char,
    ap: __builtin_va_list,
) -> c_int {
    unsafe { vsnprintf_raw(buf, size, fmt, ap) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn asprintf(
    strp: *mut *mut c_char,
    fmt: *const c_char,
    mut args: ...
) -> c_int {
    unsafe { asprintf_with_args(strp, fmt, args.as_va_list()) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn asnprintf(
    str_: *mut c_char,
    lenp: *mut usize,
    fmt: *const c_char,
    mut args: ...
) -> *mut c_char {
    unsafe { asnprintf_with_args(str_, lenp, fmt, args.as_va_list()) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vasprintf(
    strp: *mut *mut c_char,
    fmt: *const c_char,
    ap: __gnuc_va_list,
) -> c_int {
    unsafe { vasprintf_raw(strp, fmt, ap) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn scanf(fmt: *const c_char, mut args: ...) -> c_int {
    unsafe { scanf_with_args(fmt, args.as_va_list()) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fscanf(stream: *mut FILE, fmt: *const c_char, mut args: ...) -> c_int {
    unsafe { fscanf_with_args(stream, fmt, args.as_va_list()) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vscanf(fmt: *const c_char, ap: __builtin_va_list) -> c_int {
    unsafe { vscanf_raw(fmt, ap) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vfscanf(
    stream: *mut FILE,
    fmt: *const c_char,
    ap: __builtin_va_list,
) -> c_int {
    unsafe { vfscanf_raw(stream, fmt, ap) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sscanf(buf: *const c_char, fmt: *const c_char, mut args: ...) -> c_int {
    unsafe { sscanf_with_args(buf, fmt, args.as_va_list()) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vsscanf(
    buf: *const c_char,
    fmt: *const c_char,
    ap: __builtin_va_list,
) -> c_int {
    unsafe { vsscanf_raw(buf, fmt, ap) }
}

#[cfg(test)]
mod tests {
    use super::*;

    unsafe extern "C" fn noop_atexit() {}

    #[test]
    fn atexit_rejects_null_callback() {
        runtime::set_errno(0);
        assert_eq!(atexit(None), -1);
        assert_eq!(unsafe { *crate::__errno() }, libc::EINVAL);
    }

    #[test]
    fn atexit_registers_callback() {
        runtime::set_errno(17);
        assert_eq!(atexit(Some(noop_atexit)), 0);
        assert_eq!(unsafe { *crate::__errno() }, 0);
    }

    #[test]
    fn signal_helpers_round_trip() {
        let mut name = [0 as c_char; 16];
        let mut signum = 0;

        assert_eq!(sig2str(libc::SIGABRT, name.as_mut_ptr()), 0);
        assert_eq!(unsafe { CStr::from_ptr(name.as_ptr()) }.to_bytes(), b"ABRT");
        assert_eq!(str2sig(c"SIGABRT".as_ptr(), &mut signum), 0);
        assert_eq!(signum, libc::SIGABRT);
        assert_eq!(str2sig(c"IOT".as_ptr(), &mut signum), 0);
        assert_eq!(signum, libc::SIGABRT);
    }

    #[test]
    fn locale_wrappers_ignore_locale_and_locale_slot_is_thread_local() {
        assert_eq!(isalnum_l(b'A' as c_int, 77), 1);
        assert_eq!(tolower_l(b'Q' as c_int, 77), b'q' as c_int);
        assert_eq!(toupper_l(b'q' as c_int, 77), b'Q' as c_int);
        assert_eq!(isascii_l(0x7f, 77), 1);

        assert_eq!(uselocale(42), -1);
        assert_eq!(uselocale(0), 42);

        let thread_previous = std::thread::spawn(|| uselocale(0))
            .join()
            .expect("thread locale");
        assert_eq!(thread_previous, -1);
    }

    #[test]
    fn string_helpers_transform_and_search() {
        let mut lower = *b"Rust\0";
        let mut upper = *b"Rust\0";
        let haystack = c"Hello badge";
        let needle = c"badge";

        assert_eq!(strlwr(lower.as_mut_ptr().cast()), lower.as_mut_ptr().cast());
        assert_eq!(&lower[..4], b"rust");
        assert_eq!(strupr(upper.as_mut_ptr().cast()), upper.as_mut_ptr().cast());
        assert_eq!(&upper[..4], b"RUST");
        assert_eq!(
            unsafe { CStr::from_ptr(strnstr(haystack.as_ptr(), needle.as_ptr(), 11)) }.to_bytes(),
            b"badge"
        );
        assert!(strnstr(haystack.as_ptr(), needle.as_ptr(), 5).is_null());
    }

    #[test]
    fn integer_format_and_compare_helpers_work() {
        let mut signed = [0 as c_char; 16];
        let mut unsigned = [0 as c_char; 16];
        let left = [1_u8, 2, 3];
        let right = [1_u8, 2, 4];

        assert_eq!(
            unsafe { CStr::from_ptr(itoa(-42, signed.as_mut_ptr(), 10)) }.to_bytes(),
            b"-42"
        );
        assert_eq!(
            unsafe { CStr::from_ptr(utoa(255, unsigned.as_mut_ptr(), 16)) }.to_bytes(),
            b"ff"
        );
        assert_eq!(
            timingsafe_bcmp(left.as_ptr().cast(), left.as_ptr().cast(), left.len()),
            0
        );
        assert_ne!(
            timingsafe_bcmp(left.as_ptr().cast(), right.as_ptr().cast(), left.len()),
            0
        );
        assert_eq!(
            timingsafe_memcmp(left.as_ptr().cast(), right.as_ptr().cast(), left.len()),
            -1
        );
        assert_eq!(
            timingsafe_memcmp(right.as_ptr().cast(), left.as_ptr().cast(), left.len()),
            1
        );
    }

    #[test]
    fn math_alias_helpers_match_expected_values() {
        assert!(infinity().is_infinite());
        assert!(infinityf().is_infinite());
        assert_eq!(exp10(3.0), 1000.0);
        assert_eq!(pow10(2.0), 100.0);
        assert_eq!(exp10f(2.0), 100.0);
        assert_eq!(pow10f(1.0), 10.0);
        assert_eq!(toascii_l(0xff, 0), 0x7f);
    }

    #[test]
    fn locale_numeric_and_wide_helpers_work() {
        let mut endptr = core::ptr::null_mut();
        let mut wide_endptr = core::ptr::null_mut();
        let mut dst = [0 as wchar_t; 8];
        let src = [b'a' as wchar_t, b'b' as wchar_t, 0];
        let digits = [b'4' as _wchar_t, b'2' as _wchar_t, 0];

        assert_eq!(strtoimax_l(c"-42".as_ptr(), &mut endptr, 10, 12), -42);
        assert_eq!(strtoumax_l(c"ff".as_ptr(), &mut endptr, 16, 12), 255);
        assert_eq!(wcstoimax_l(digits.as_ptr(), &mut wide_endptr, 10, 12), 42);
        assert_eq!(wcstoumax_l(digits.as_ptr(), &mut wide_endptr, 10, 12), 42);
        assert_eq!(wcslcpy(dst.as_mut_ptr(), src.as_ptr(), dst.len()), 2);
        assert_eq!(dst[0], b'a' as wchar_t);
        assert_eq!(dst[1], b'b' as wchar_t);
        assert_eq!(wcslcat(dst.as_mut_ptr(), src.as_ptr(), dst.len()), 4);
        assert_eq!(dst[2], b'a' as wchar_t);
        assert_eq!(dst[3], b'b' as wchar_t);
    }

    #[test]
    fn gamma_and_gcvtf_helpers_match_host_behavior() {
        let mut sign = 0;
        let mut expected_sign = 0;
        let mut actual = [0 as c_char; 32];
        let mut expected = [0 as c_char; 32];

        assert_eq!(
            gamma_r(3.5, &mut sign),
            host_lgamma_r(3.5, &mut expected_sign)
        );
        assert_eq!(sign, expected_sign);
        assert_eq!(
            gammaf_r(3.5, &mut sign),
            host_lgammaf_r(3.5, &mut expected_sign)
        );
        assert_eq!(sign, expected_sign);

        clear_host_errno();
        let _ = host_sprintf(expected.as_mut_ptr(), c"%.*g".as_ptr(), 4, 12.5);
        sync_errno_from_host();
        assert_eq!(gcvtf(12.5, 4, actual.as_mut_ptr()), actual.as_mut_ptr());
        assert_eq!(
            unsafe { CStr::from_ptr(actual.as_ptr()) }.to_bytes(),
            unsafe { CStr::from_ptr(expected.as_ptr()) }.to_bytes()
        );
    }

    unsafe extern "C" fn call_vsnprintf(
        buf: *mut c_char,
        size: c_uint,
        fmt: *const c_char,
        mut args: ...
    ) -> c_int {
        unsafe { snprintf_with_args(buf, size, fmt, args.as_va_list()) }
    }

    unsafe extern "C" fn call_vsscanf(
        buf: *const c_char,
        fmt: *const c_char,
        mut args: ...
    ) -> c_int {
        unsafe { sscanf_with_args(buf, fmt, args.as_va_list()) }
    }

    #[test]
    fn varargs_stdio_helpers_match_expected_values() {
        unsafe {
            let mut buffer = [0 as c_char; 64];
            assert_eq!(
                sprintf(
                    buffer.as_mut_ptr(),
                    c"%s %d".as_ptr(),
                    c"badge".as_ptr(),
                    42
                ),
                8
            );
            assert_eq!(CStr::from_ptr(buffer.as_ptr()).to_bytes(), b"badge 42");

            let mut truncated = [0 as c_char; 8];
            assert_eq!(
                snprintf(
                    truncated.as_mut_ptr(),
                    truncated.len() as c_uint,
                    c"%s-%d".as_ptr(),
                    c"badge".as_ptr(),
                    42,
                ),
                8
            );
            assert_eq!(CStr::from_ptr(truncated.as_ptr()).to_bytes(), b"badge-4");

            let mut vbuffer = [0 as c_char; 64];
            assert_eq!(
                call_vsnprintf(
                    vbuffer.as_mut_ptr(),
                    vbuffer.len() as c_uint,
                    c"%s %d".as_ptr(),
                    c"badge".as_ptr(),
                    42,
                ),
                8
            );
            assert_eq!(CStr::from_ptr(vbuffer.as_ptr()).to_bytes(), b"badge 42");

            let mut allocated = core::ptr::null_mut();
            assert_eq!(
                asprintf(&mut allocated, c"%s %d".as_ptr(), c"badge".as_ptr(), 42),
                8
            );
            assert_eq!(CStr::from_ptr(allocated).to_bytes(), b"badge 42");
            libc::free(allocated.cast());

            let mut inline_len = 16_usize;
            let mut inline_buffer = [0 as c_char; 16];
            let inline_ptr = asnprintf(
                inline_buffer.as_mut_ptr(),
                &mut inline_len,
                c"%s %d".as_ptr(),
                c"badge".as_ptr(),
                42,
            );
            assert_eq!(inline_ptr, inline_buffer.as_mut_ptr());
            assert_eq!(inline_len, 8);
            assert_eq!(CStr::from_ptr(inline_ptr).to_bytes(), b"badge 42");

            let mut grown_len = 4_usize;
            let mut grown_buffer = [0 as c_char; 4];
            let grown_ptr = asnprintf(
                grown_buffer.as_mut_ptr(),
                &mut grown_len,
                c"%s %d".as_ptr(),
                c"badge".as_ptr(),
                42,
            );
            assert!(!grown_ptr.is_null());
            assert_ne!(grown_ptr, grown_buffer.as_mut_ptr());
            assert_eq!(grown_len, 8);
            assert_eq!(CStr::from_ptr(grown_ptr).to_bytes(), b"badge 42");
            libc::free(grown_ptr.cast());

            let stream = crate::host_forward::fmemopen(
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                c"w+".as_ptr(),
            );
            assert!(!stream.is_null());
            assert_eq!(fprintf(stream, c"%s %d".as_ptr(), c"wifi".as_ptr(), 7), 6);
            assert_eq!(crate::host_forward::fflush(stream), 0);
            assert_eq!(crate::host_forward::fseek(stream, 0, libc::SEEK_SET), 0);
            let mut word = [0 as c_char; 16];
            let mut number = 0;
            assert_eq!(
                fscanf(stream, c"%15s %d".as_ptr(), word.as_mut_ptr(), &mut number),
                2
            );
            assert_eq!(CStr::from_ptr(word.as_ptr()).to_bytes(), b"wifi");
            assert_eq!(number, 7);
            assert_eq!(crate::host_forward::fclose(stream), 0);

            let mut parsed_word = [0 as c_char; 16];
            let mut parsed_number = 0;
            assert_eq!(
                sscanf(
                    c"emu 9".as_ptr(),
                    c"%15s %d".as_ptr(),
                    parsed_word.as_mut_ptr(),
                    &mut parsed_number
                ),
                2
            );
            assert_eq!(CStr::from_ptr(parsed_word.as_ptr()).to_bytes(), b"emu");
            assert_eq!(parsed_number, 9);

            let mut vparsed_word = [0 as c_char; 16];
            let mut vparsed_number = 0;
            assert_eq!(
                call_vsscanf(
                    c"emu 11".as_ptr(),
                    c"%15s %d".as_ptr(),
                    vparsed_word.as_mut_ptr(),
                    &mut vparsed_number
                ),
                2
            );
            assert_eq!(CStr::from_ptr(vparsed_word.as_ptr()).to_bytes(), b"emu");
            assert_eq!(vparsed_number, 11);
        }
    }

    #[test]
    fn atoff_converts_to_f32() {
        assert_eq!(atoff(c"1.5".as_ptr()), 1.5);
    }

    #[test]
    fn fls_family_matches_expected_bits() {
        assert_eq!(fls(0), 0);
        assert_eq!(fls(0b101000), 6);
        assert_eq!(flsl(1 << 40), 41);
        assert_eq!(flsll(1 << 55), 56);
    }

    #[test]
    fn float_helpers_match_basic_operations() {
        assert_eq!(__adddf3(1.0, 2.5), 3.5);
        assert_eq!(__subdf3(5.0, 1.25), 3.75);
        assert_eq!(__muldf3(2.0, 4.0), 8.0);
        assert_eq!(__divdf3(9.0, 3.0), 3.0);
        assert_eq!(__eqdf2(2.0, 2.0), 0);
        assert_eq!(__gedf2(2.0, 1.0), 1);
        assert_eq!(__ltdf2(f64::NAN, 1.0), 1);
    }

    #[test]
    fn half_conversion_round_trips_common_values() {
        let half = __truncsfhf2(1.5);
        assert_eq!(__extendhfsf2(half), 1.5);
    }

    #[test]
    fn integer_helpers_match_builtin_ops() {
        assert_eq!(__fixdfsi(7.9), 7);
        assert_eq!(__fixdfdi(9.2), 9);
        assert_eq!(__fixunsdfsi(12.4), 12);
        assert_eq!(__floatdisf(-3), -3.0);
        assert_eq!(__floatsidf(-3), -3.0);
        assert_eq!(__floatundidf(7), 7.0);
        assert_eq!(__floatundisf(7), 7.0);
        assert_eq!(__floatunsidf(7), 7.0);
        assert_eq!(__divdi3(9, 3), 3);
        assert_eq!(__udivdi3(9, 3), 3);
        assert_eq!(__umoddi3(10, 3), 1);
        assert_eq!(__clzsi2(1), 31);
        assert_eq!(__popcountsi2(0b1011), 3);
    }

    #[test]
    fn issignalingf_detects_signaling_nan() {
        let signaling_nan = f32::from_bits(0x7f80_0001);
        let quiet_nan = f32::from_bits(0x7fc0_0001);

        assert_eq!(__issignalingf(signaling_nan), 1);
        assert_eq!(__issignalingf(quiet_nan), 0);
    }

    #[test]
    fn fp_mask_is_thread_local() {
        let previous = fpsetmask(FP_X_INV_CONST);
        assert_eq!(fpgetmask(), FP_X_INV_CONST);

        let thread_mask = std::thread::spawn(|| {
            assert_eq!(fpgetmask(), 0);
            let prior = fpsetmask(FP_X_DX_CONST);
            assert_eq!(prior, 0);
            fpgetmask()
        })
        .join()
        .expect("thread fp mask");

        assert_eq!(thread_mask, FP_X_DX_CONST);
        assert_eq!(fpgetmask(), FP_X_INV_CONST);

        fpsetmask(previous);
    }

    #[test]
    fn fe_env_roundtrip_works() {
        let original_round = fpgetround();
        let original_sticky = fpgetsticky();
        let original_mask = fpgetmask();
        let mut env: fenv_t = 0;

        assert_eq!(fegetenv(&mut env), 0);
        assert_eq!(fpsetround(FP_RM_CONST), original_round);
        assert_eq!(fpgetround(), FP_RM_CONST);
        assert_eq!(fpsetsticky(FP_X_INV_CONST), original_sticky);
        assert_eq!(fpgetsticky() & FP_X_INV_CONST, FP_X_INV_CONST);
        assert_eq!(fesetenv(&env), 0);
        assert_eq!(fpgetround(), original_round);

        fpsetsticky(original_sticky);
        fpsetmask(original_mask);
    }

    #[test]
    fn fe_null_pointers_set_errno() {
        runtime::set_errno(0);
        assert_eq!(
            fegetexceptflag(core::ptr::null_mut(), BADGE_FE_ALL_EXCEPT),
            -1
        );
        assert_eq!(unsafe { *crate::__errno() }, libc::EINVAL);

        runtime::set_errno(0);
        assert_eq!(fegetenv(core::ptr::null_mut()), -1);
        assert_eq!(unsafe { *crate::__errno() }, libc::EINVAL);
    }
}
