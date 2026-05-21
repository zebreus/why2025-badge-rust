use crate::runtime;
use crate::types::*;
use alloc::{boxed::Box, vec, vec::Vec};
use core::cell::UnsafeCell;
use core::ffi::{c_char, c_int, c_long, c_longlong, c_uint, c_ulong, c_ulonglong, c_void};
use core::sync::atomic::{AtomicUsize, Ordering};

struct PthreadOnce(UnsafeCell<libc::pthread_once_t>);
struct LibmHandle(UnsafeCell<*mut c_void>);
struct TmBuffer(UnsafeCell<tm>);

unsafe impl Sync for PthreadOnce {}
unsafe impl Sync for LibmHandle {}
unsafe impl Sync for TmBuffer {}

static LIBM_HANDLE: LibmHandle = LibmHandle(UnsafeCell::new(core::ptr::null_mut()));
static LIBM_HANDLE_ONCE: PthreadOnce = PthreadOnce(UnsafeCell::new(libc::PTHREAD_ONCE_INIT));
static GMTIME_BUFFER: TmBuffer = TmBuffer(UnsafeCell::new(tm {
    tm_sec: 0,
    tm_min: 0,
    tm_hour: 0,
    tm_mday: 0,
    tm_mon: 0,
    tm_year: 0,
    tm_wday: 0,
    tm_yday: 0,
    tm_isdst: 0,
}));
static LOCALTIME_BUFFER: TmBuffer = TmBuffer(UnsafeCell::new(tm {
    tm_sec: 0,
    tm_min: 0,
    tm_hour: 0,
    tm_mday: 0,
    tm_mon: 0,
    tm_year: 0,
    tm_wday: 0,
    tm_yday: 0,
    tm_isdst: 0,
}));

const MAX_WIDE_DESCRIPTORS: usize = 32;
const HOST_REGEX_MAGIC: c_uint = 0x5752_4558;

static WCTRANS_DESCRIPTORS: [AtomicUsize; MAX_WIDE_DESCRIPTORS] =
    [const { AtomicUsize::new(0) }; MAX_WIDE_DESCRIPTORS];
static WCTYPE_DESCRIPTORS: [AtomicUsize; MAX_WIDE_DESCRIPTORS] =
    [const { AtomicUsize::new(0) }; MAX_WIDE_DESCRIPTORS];

#[repr(C)]
struct HostRegex {
    __buffer: *mut c_void,
    __allocated: usize,
    __used: usize,
    __syntax: libc::c_ulong,
    __fastmap: *mut c_char,
    __translate: *mut c_char,
    __re_nsub: usize,
    __bitfield: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct HostRegmatch {
    rm_so: c_int,
    rm_eo: c_int,
}

struct RegexBridge {
    compiled: HostRegex,
}

type CosFn = unsafe extern "C" fn(f64) -> f64;
type CosfFn = unsafe extern "C" fn(f32) -> f32;
type SinFn = unsafe extern "C" fn(f64) -> f64;
type SinfFn = unsafe extern "C" fn(f32) -> f32;
type SincosFn = unsafe extern "C" fn(f64, *mut f64, *mut f64);
type SincosfFn = unsafe extern "C" fn(f32, *mut f32, *mut f32);
type HostWctransFn = unsafe extern "C" fn(*const c_char) -> usize;
type HostWctypeFn = unsafe extern "C" fn(*const c_char) -> usize;
type HostIswctypeFn = unsafe extern "C" fn(wint_t, usize) -> c_int;

extern "C" fn init_libm_handle() {
    let handle = unsafe { libc::dlopen(c"libm.so.6".as_ptr(), libc::RTLD_NOW | libc::RTLD_LOCAL) };
    if handle.is_null() {
        runtime::abort_with_message(b"why2025-badge-emu-abi failed to open libm.so.6\n")
    }

    unsafe {
        *LIBM_HANDLE.0.get() = handle;
    }
}

fn libm_handle() -> *mut c_void {
    let rc = unsafe { libc::pthread_once(LIBM_HANDLE_ONCE.0.get(), init_libm_handle) };
    if rc != 0 {
        runtime::abort_with_message(b"why2025-badge-emu-abi failed to initialize libm handle\n")
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
            runtime::abort_missing_host_symbol(name)
        }
    }
}

fn intern_descriptor(slots: &[AtomicUsize], value: usize) -> c_int {
    if value == 0 {
        return 0;
    }

    for (index, slot) in slots.iter().enumerate() {
        if slot.load(Ordering::Acquire) == value {
            return (index + 1) as c_int;
        }
    }

    for (index, slot) in slots.iter().enumerate() {
        if slot
            .compare_exchange(0, value, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
            || slot.load(Ordering::Acquire) == value
        {
            return (index + 1) as c_int;
        }
    }

    runtime::set_errno(libc::ENOMEM);
    0
}

fn resolve_descriptor(slots: &[AtomicUsize], handle: c_int) -> Option<usize> {
    if handle <= 0 {
        return None;
    }

    let index = handle as usize - 1;
    slots
        .get(index)
        .map(|slot| slot.load(Ordering::Acquire))
        .filter(|value| *value != 0)
}

fn badge_tm_to_host(value: &tm) -> libc::tm {
    let mut host: libc::tm = unsafe { core::mem::zeroed() };
    host.tm_sec = value.tm_sec;
    host.tm_min = value.tm_min;
    host.tm_hour = value.tm_hour;
    host.tm_mday = value.tm_mday;
    host.tm_mon = value.tm_mon;
    host.tm_year = value.tm_year;
    host.tm_wday = value.tm_wday;
    host.tm_yday = value.tm_yday;
    host.tm_isdst = value.tm_isdst;
    host
}

fn copy_host_tm_into_badge(dst: &mut tm, src: &libc::tm) {
    dst.tm_sec = src.tm_sec;
    dst.tm_min = src.tm_min;
    dst.tm_hour = src.tm_hour;
    dst.tm_mday = src.tm_mday;
    dst.tm_mon = src.tm_mon;
    dst.tm_year = src.tm_year;
    dst.tm_wday = src.tm_wday;
    dst.tm_yday = src.tm_yday;
    dst.tm_isdst = src.tm_isdst;
}

fn host_timeval_to_badge(value: &libc::timeval) -> timeval {
    timeval {
        tv_sec: value.tv_sec as time_t,
        tv_usec: value.tv_usec as _,
        __bindgen_padding_0: [0; 4],
    }
}

fn resolve_host_regcomp() -> unsafe extern "C" fn(*mut HostRegex, *const c_char, c_int) -> c_int {
    unsafe { runtime::resolve_next_function(b"regcomp\0") }
}

fn resolve_host_regerror()
-> unsafe extern "C" fn(c_int, *const HostRegex, *mut c_char, usize) -> usize {
    unsafe { runtime::resolve_next_function(b"regerror\0") }
}

fn resolve_host_regexec()
-> unsafe extern "C" fn(*const HostRegex, *const c_char, usize, *mut HostRegmatch, c_int) -> c_int {
    unsafe { runtime::resolve_next_function(b"regexec\0") }
}

fn resolve_host_regfree() -> unsafe extern "C" fn(*mut HostRegex) {
    unsafe { runtime::resolve_next_function(b"regfree\0") }
}

unsafe fn clear_badge_regex(preg: *mut regex_t) {
    if preg.is_null() {
        return;
    }

    unsafe {
        (*preg).re_magic = 0;
        (*preg).re_nsub = 0;
        (*preg).re_endp = core::ptr::null();
        (*preg).re_g = core::ptr::null_mut();
    }
}

unsafe fn regex_bridge_ptr(preg: *const regex_t) -> Option<*mut RegexBridge> {
    if preg.is_null() {
        return None;
    }

    let preg = unsafe { &*preg };
    if preg.re_magic != HOST_REGEX_MAGIC || preg.re_g.is_null() {
        return None;
    }

    Some(preg.re_g.cast::<RegexBridge>())
}

unsafe fn take_regex_bridge(preg: *mut regex_t) -> Option<Box<RegexBridge>> {
    let bridge = unsafe { regex_bridge_ptr(preg.cast_const()) }?;
    unsafe { clear_badge_regex(preg) };
    Some(unsafe { Box::from_raw(bridge) })
}

unsafe fn install_regex_bridge(preg: *mut regex_t, bridge: *mut RegexBridge) {
    unsafe {
        (*preg).re_magic = HOST_REGEX_MAGIC;
        (*preg).re_nsub = (*bridge).compiled.__re_nsub;
        (*preg).re_endp = core::ptr::null();
        (*preg).re_g = bridge.cast::<re_guts>();
    }
}

unsafe fn require_regex_bridge<'a>(preg: *const regex_t) -> &'a RegexBridge {
    let bridge = unsafe { regex_bridge_ptr(preg) }.unwrap_or_else(|| {
        runtime::abort_with_message(
            b"why2025-badge-emu-abi regex_t was not initialized by regcomp\n",
        )
    });
    unsafe { &*bridge }
}

fn host_timespec_to_badge(value: &libc::timespec) -> timespec {
    timespec {
        tv_sec: value.tv_sec as time_t,
        tv_nsec: value.tv_nsec as c_long,
        __bindgen_padding_0: [0; 4],
    }
}

fn badge_timeval_to_host(value: &timeval) -> libc::timeval {
    libc::timeval {
        tv_sec: value.tv_sec as libc::time_t,
        tv_usec: value.tv_usec as libc::suseconds_t,
    }
}

unsafe fn badge_fd_set_to_host(value: &fd_set) -> libc::fd_set {
    let mut host: libc::fd_set = unsafe { core::mem::zeroed() };
    let host_words = (&mut host as *mut libc::fd_set).cast::<libc::c_ulong>();

    for (index, word) in value.__fds_bits.iter().copied().enumerate() {
        unsafe { *host_words.add(index) = word as libc::c_ulong };
    }

    host
}

unsafe fn host_fd_set_to_badge(value: &libc::fd_set) -> fd_set {
    let mut badge: fd_set = unsafe { core::mem::zeroed() };
    let host_words = (value as *const libc::fd_set).cast::<libc::c_ulong>();

    for (index, slot) in badge.__fds_bits.iter_mut().enumerate() {
        *slot = unsafe { *host_words.add(index) } as c_ulong;
    }

    badge
}

macro_rules! forward_libm_fn {
    ($(fn $name:ident($($arg:ident : $arg_ty:ty),* $(,)?) -> $ret:ty = $symbol:literal;)+) => {
        $(
            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn $name($($arg: $arg_ty),*) -> $ret {
                let function: unsafe extern "C" fn($($arg_ty),*) -> $ret =
                    unsafe { resolve_libm_function($symbol) };
                unsafe { function($($arg),*) }
            }
        )+
    };
}

macro_rules! forward_next_fn {
    ($(fn $name:ident($($arg:ident : $arg_ty:ty),* $(,)?) -> $ret:ty = $symbol:literal;)+) => {
        $(
            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn $name($($arg: $arg_ty),*) -> $ret {
                let function: unsafe extern "C" fn($($arg_ty),*) -> $ret =
                    unsafe { runtime::resolve_next_function($symbol) };
                unsafe { function($($arg),*) }
            }
        )+
    };
}

macro_rules! forward_next_ignore_locale_fn {
    ($(fn $name:ident($($arg:ident : $arg_ty:ty),* ; $locale:ident : locale_t) -> $ret:ty = $symbol:literal;)+) => {
        $(
            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn $name($($arg: $arg_ty,)* $locale: locale_t) -> $ret {
                let _ = $locale;
                let function: unsafe extern "C" fn($($arg_ty),*) -> $ret =
                    unsafe { runtime::resolve_next_function($symbol) };
                unsafe { function($($arg),*) }
            }
        )+
    };
}

forward_libm_fn! {
    fn acos(value: f64) -> f64 = b"acos\0";
    fn acosf(value: f32) -> f32 = b"acosf\0";
    fn acosh(value: f64) -> f64 = b"acosh\0";
    fn acoshf(value: f32) -> f32 = b"acoshf\0";
    fn asin(value: f64) -> f64 = b"asin\0";
    fn asinf(value: f32) -> f32 = b"asinf\0";
    fn asinh(value: f64) -> f64 = b"asinh\0";
    fn asinhf(value: f32) -> f32 = b"asinhf\0";
    fn atan(value: f64) -> f64 = b"atan\0";
    fn atan2(left: f64, right: f64) -> f64 = b"atan2\0";
    fn atan2f(left: f32, right: f32) -> f32 = b"atan2f\0";
    fn atanf(value: f32) -> f32 = b"atanf\0";
    fn atanh(value: f64) -> f64 = b"atanh\0";
    fn atanhf(value: f32) -> f32 = b"atanhf\0";
    fn cbrt(value: f64) -> f64 = b"cbrt\0";
    fn cbrtf(value: f32) -> f32 = b"cbrtf\0";
    fn ceil(value: f64) -> f64 = b"ceil\0";
    fn ceilf(value: f32) -> f32 = b"ceilf\0";
    fn copysign(left: f64, right: f64) -> f64 = b"copysign\0";
    fn copysignf(left: f32, right: f32) -> f32 = b"copysignf\0";
    fn cosh(value: f64) -> f64 = b"cosh\0";
    fn coshf(value: f32) -> f32 = b"coshf\0";
    fn drem(left: f64, right: f64) -> f64 = b"drem\0";
    fn dremf(left: f32, right: f32) -> f32 = b"dremf\0";
    fn erf(value: f64) -> f64 = b"erf\0";
    fn erfc(value: f64) -> f64 = b"erfc\0";
    fn erfcf(value: f32) -> f32 = b"erfcf\0";
    fn erff(value: f32) -> f32 = b"erff\0";
    fn exp(value: f64) -> f64 = b"exp\0";
    fn exp2(value: f64) -> f64 = b"exp2\0";
    fn exp2f(value: f32) -> f32 = b"exp2f\0";
    fn expf(value: f32) -> f32 = b"expf\0";
    fn expm1(value: f64) -> f64 = b"expm1\0";
    fn expm1f(value: f32) -> f32 = b"expm1f\0";
    fn fabs(value: f64) -> f64 = b"fabs\0";
    fn fabsf(value: f32) -> f32 = b"fabsf\0";
    fn fdim(left: f64, right: f64) -> f64 = b"fdim\0";
    fn fdimf(left: f32, right: f32) -> f32 = b"fdimf\0";
    fn finite(value: f64) -> c_int = b"finite\0";
    fn finitef(value: f32) -> c_int = b"finitef\0";
    fn floor(value: f64) -> f64 = b"floor\0";
    fn floorf(value: f32) -> f32 = b"floorf\0";
    fn fma(left: f64, right: f64, value: f64) -> f64 = b"fma\0";
    fn fmaf(left: f32, right: f32, value: f32) -> f32 = b"fmaf\0";
    fn fmax(left: f64, right: f64) -> f64 = b"fmax\0";
    fn fmaxf(left: f32, right: f32) -> f32 = b"fmaxf\0";
    fn fmin(left: f64, right: f64) -> f64 = b"fmin\0";
    fn fminf(left: f32, right: f32) -> f32 = b"fminf\0";
    fn fmod(left: f64, right: f64) -> f64 = b"fmod\0";
    fn fmodf(left: f32, right: f32) -> f32 = b"fmodf\0";
    fn frexp(value: f64, exponent: *mut c_int) -> f64 = b"frexp\0";
    fn frexpf(value: f32, exponent: *mut c_int) -> f32 = b"frexpf\0";
    fn gamma(value: f64) -> f64 = b"gamma\0";
    fn gammaf(value: f32) -> f32 = b"gammaf\0";
    fn hypot(left: f64, right: f64) -> f64 = b"hypot\0";
    fn hypotf(left: f32, right: f32) -> f32 = b"hypotf\0";
    fn ilogb(value: f64) -> c_int = b"ilogb\0";
    fn ilogbf(value: f32) -> c_int = b"ilogbf\0";
    fn isinf(value: f64) -> c_int = b"isinf\0";
    fn isinff(value: f32) -> c_int = b"isinff\0";
    fn isnan(value: f64) -> c_int = b"isnan\0";
    fn isnanf(value: f32) -> c_int = b"isnanf\0";
    fn j0(value: f64) -> f64 = b"j0\0";
    fn j0f(value: f32) -> f32 = b"j0f\0";
    fn j1(value: f64) -> f64 = b"j1\0";
    fn j1f(value: f32) -> f32 = b"j1f\0";
    fn jn(order: c_int, value: f64) -> f64 = b"jn\0";
    fn jnf(order: c_int, value: f32) -> f32 = b"jnf\0";
    fn ldexp(value: f64, exp: c_int) -> f64 = b"ldexp\0";
    fn ldexpf(value: f32, exp: c_int) -> f32 = b"ldexpf\0";
    fn lgamma(value: f64) -> f64 = b"lgamma\0";
    fn lgamma_r(value: f64, signgamp: *mut c_int) -> f64 = b"lgamma_r\0";
    fn lgammaf(value: f32) -> f32 = b"lgammaf\0";
    fn lgammaf_r(value: f32, signgamp: *mut c_int) -> f32 = b"lgammaf_r\0";
    fn llrint(value: f64) -> c_longlong = b"llrint\0";
    fn llrintf(value: f32) -> c_longlong = b"llrintf\0";
    fn llround(value: f64) -> c_longlong = b"llround\0";
    fn llroundf(value: f32) -> c_longlong = b"llroundf\0";
    fn log(value: f64) -> f64 = b"log\0";
    fn log10(value: f64) -> f64 = b"log10\0";
    fn log10f(value: f32) -> f32 = b"log10f\0";
    fn log1p(value: f64) -> f64 = b"log1p\0";
    fn log1pf(value: f32) -> f32 = b"log1pf\0";
    fn log2(value: f64) -> f64 = b"log2\0";
    fn log2f(value: f32) -> f32 = b"log2f\0";
    fn logb(value: f64) -> f64 = b"logb\0";
    fn logbf(value: f32) -> f32 = b"logbf\0";
    fn logf(value: f32) -> f32 = b"logf\0";
    fn lrint(value: f64) -> c_long = b"lrint\0";
    fn lrintf(value: f32) -> c_long = b"lrintf\0";
    fn lround(value: f64) -> c_long = b"lround\0";
    fn lroundf(value: f32) -> c_long = b"lroundf\0";
    fn modf(value: f64, iptr: *mut f64) -> f64 = b"modf\0";
    fn modff(value: f32, iptr: *mut f32) -> f32 = b"modff\0";
    fn nan(tagp: *const c_char) -> f64 = b"nan\0";
    fn nanf(tagp: *const c_char) -> f32 = b"nanf\0";
    fn nearbyint(value: f64) -> f64 = b"nearbyint\0";
    fn nearbyintf(value: f32) -> f32 = b"nearbyintf\0";
    fn nextafter(value: f64, direction: f64) -> f64 = b"nextafter\0";
    fn nextafterf(value: f32, direction: f32) -> f32 = b"nextafterf\0";
    fn pow(left: f64, right: f64) -> f64 = b"pow\0";
    fn powf(left: f32, right: f32) -> f32 = b"powf\0";
    fn remainder(left: f64, right: f64) -> f64 = b"remainder\0";
    fn remainderf(left: f32, right: f32) -> f32 = b"remainderf\0";
    fn remquo(left: f64, right: f64, quo: *mut c_int) -> f64 = b"remquo\0";
    fn remquof(left: f32, right: f32, quo: *mut c_int) -> f32 = b"remquof\0";
    fn rint(value: f64) -> f64 = b"rint\0";
    fn rintf(value: f32) -> f32 = b"rintf\0";
    fn round(value: f64) -> f64 = b"round\0";
    fn roundf(value: f32) -> f32 = b"roundf\0";
    fn scalbln(value: f64, exp: c_long) -> f64 = b"scalbln\0";
    fn scalblnf(value: f32, exp: c_long) -> f32 = b"scalblnf\0";
    fn scalbn(value: f64, exp: c_int) -> f64 = b"scalbn\0";
    fn scalbnf(value: f32, exp: c_int) -> f32 = b"scalbnf\0";
    fn sinh(value: f64) -> f64 = b"sinh\0";
    fn sinhf(value: f32) -> f32 = b"sinhf\0";
    fn sqrt(value: f64) -> f64 = b"sqrt\0";
    fn sqrtf(value: f32) -> f32 = b"sqrtf\0";
    fn tan(value: f64) -> f64 = b"tan\0";
    fn tanf(value: f32) -> f32 = b"tanf\0";
    fn tanh(value: f64) -> f64 = b"tanh\0";
    fn tanhf(value: f32) -> f32 = b"tanhf\0";
    fn tgamma(value: f64) -> f64 = b"tgamma\0";
    fn tgammaf(value: f32) -> f32 = b"tgammaf\0";
    fn trunc(value: f64) -> f64 = b"trunc\0";
    fn truncf(value: f32) -> f32 = b"truncf\0";
    fn y0(value: f64) -> f64 = b"y0\0";
    fn y0f(value: f32) -> f32 = b"y0f\0";
    fn y1(value: f64) -> f64 = b"y1\0";
    fn y1f(value: f32) -> f32 = b"y1f\0";
    fn yn(order: c_int, value: f64) -> f64 = b"yn\0";
    fn ynf(order: c_int, value: f32) -> f32 = b"ynf\0";
}

forward_next_fn! {
    fn accept(sockfd: c_int, addr: *mut sockaddr, addrlen: *mut socklen_t) -> c_int = b"accept\0";
    fn a64l(input: *const c_char) -> c_long = b"a64l\0";
    fn abs(value: c_int) -> c_int = b"abs\0";
    fn atof(value: *const c_char) -> f64 = b"atof\0";
    fn atoi(value: *const c_char) -> c_int = b"atoi\0";
    fn atol(value: *const c_char) -> c_long = b"atol\0";
    fn atoll(value: *const c_char) -> c_longlong = b"atoll\0";
    fn bcmp(left: *const c_void, right: *const c_void, count: c_uint) -> c_int = b"bcmp\0";
    fn bcopy(src: *const c_void, dst: *mut c_void, count: c_uint) -> () = b"bcopy\0";
    fn bsearch(key: *const c_void, base: *const c_void, nmemb: usize, size: usize, compar: __compar_fn_t) -> *mut c_void = b"bsearch\0";
    fn bind(sockfd: c_int, addr: *const sockaddr, addrlen: socklen_t) -> c_int = b"bind\0";
    fn btowc(value: c_int) -> wint_t = b"btowc\0";
    fn clearerr(stream: *mut FILE) -> () = b"clearerr\0";
    fn clearerr_unlocked(stream: *mut FILE) -> () = b"clearerr_unlocked\0";
    fn clock() -> clock_t = b"clock\0";
    fn close(fd: c_int) -> c_int = b"close\0";
    fn closedir(dir: *mut DIR) -> c_int = b"closedir\0";
    fn connect(sockfd: c_int, addr: *const sockaddr, addrlen: socklen_t) -> c_int = b"connect\0";
    fn ctime(timer: *const time_t) -> *mut c_char = b"ctime\0";
    fn difftime(time2: time_t, time1: time_t) -> f64 = b"difftime\0";
    fn div(numer: c_int, denom: c_int) -> div_t = b"div\0";
    fn ffs(value: c_int) -> c_int = b"ffs\0";
    fn ffsl(value: c_long) -> c_int = b"ffsl\0";
    fn ffsll(value: c_longlong) -> c_int = b"ffsll\0";
    fn fclose(stream: *mut FILE) -> c_int = b"fclose\0";
    fn fdopen(fd: c_int, mode: *const c_char) -> *mut FILE = b"fdopen\0";
    fn feof(stream: *mut FILE) -> c_int = b"feof\0";
    fn ferror(stream: *mut FILE) -> c_int = b"ferror\0";
    fn fflush(stream: *mut FILE) -> c_int = b"fflush\0";
    fn fgetc(stream: *mut FILE) -> c_int = b"fgetc\0";
    fn fgetpos(stream: *mut FILE, pos: *mut fpos_t) -> c_int = b"fgetpos\0";
    fn fgets(buf: *mut c_char, size: c_int, stream: *mut FILE) -> *mut c_char = b"fgets\0";
    fn fileno(stream: *mut FILE) -> c_int = b"fileno\0";
    fn fmemopen(buf: *mut c_void, size: usize, mode: *const c_char) -> *mut FILE = b"fmemopen\0";
    fn fopen(path: *const c_char, mode: *const c_char) -> *mut FILE = b"fopen\0";
    fn fputc(value: c_int, stream: *mut FILE) -> c_int = b"fputc\0";
    fn fputs(value: *const c_char, stream: *mut FILE) -> c_int = b"fputs\0";
    fn fread(ptr: *mut c_void, size: c_uint, nmemb: c_uint, stream: *mut FILE) -> c_uint = b"fread\0";
    fn freopen(path: *const c_char, mode: *const c_char, stream: *mut FILE) -> *mut FILE = b"freopen\0";
    fn freeaddrinfo(ai: *mut addrinfo) -> () = b"freeaddrinfo\0";
    fn fseek(stream: *mut FILE, offset: c_long, whence: c_int) -> c_int = b"fseek\0";
    fn fseeko(stream: *mut FILE, offset: off_t, whence: c_int) -> c_int = b"fseeko\0";
    fn ftell(stream: *mut FILE) -> c_long = b"ftell\0";
    fn ftello(stream: *mut FILE) -> off_t = b"ftello\0";
    fn fwrite(ptr: *const c_void, size: c_uint, nmemb: c_uint, stream: *mut FILE) -> c_uint = b"fwrite\0";
    fn fnmatch(pattern: *const c_char, value: *const c_char, flags: c_int) -> c_int = b"fnmatch\0";
    fn fwide(stream: *mut FILE, mode: c_int) -> c_int = b"fwide\0";
    fn getentropy(buffer: *mut c_void, len: usize) -> c_int = b"getentropy\0";
    fn getc(stream: *mut FILE) -> c_int = b"getc\0";
    fn getdelim(lineptr: *mut *mut c_char, size: *mut usize, delim: c_int, stream: *mut FILE) -> isize = b"getdelim\0";
    fn getchar() -> c_int = b"getchar\0";
    fn getchar_unlocked() -> c_int = b"getchar_unlocked\0";
    fn getaddrinfo(nodename: *const c_char, servname: *const c_char, hints: *const addrinfo, res: *mut *mut addrinfo) -> c_int = b"getaddrinfo\0";
    fn getopt(argc: c_int, argv: *const [*mut c_char; 0usize], optstring: *const c_char) -> c_int = b"getopt\0";
    fn getopt_long(argc: c_int, argv: *const [*mut c_char; 0usize], shortopts: *const c_char, longopts: *const option, longind: *mut c_int) -> c_int = b"getopt_long\0";
    fn getopt_long_only(argc: c_int, argv: *const [*mut c_char; 0usize], shortopts: *const c_char, longopts: *const option, longind: *mut c_int) -> c_int = b"getopt_long_only\0";
    fn getsubopt(optionp: *mut *mut c_char, tokens: *const *mut c_char, valuep: *mut *mut c_char) -> c_int = b"getsubopt\0";
    fn gcvt(value: f64, ndigit: c_int, buf: *mut c_char) -> *mut c_char = b"gcvt\0";
    fn getline(lineptr: *mut *mut c_char, size: *mut usize, stream: *mut FILE) -> isize = b"getline\0";
    fn imaxabs(value: intmax_t) -> intmax_t = b"imaxabs\0";
    fn imaxdiv(numer: intmax_t, denomer: intmax_t) -> imaxdiv_t = b"imaxdiv\0";
    fn index(value: *const c_char, needle: c_int) -> *mut c_char = b"index\0";
    fn iconv(cd: iconv_t, inbuf: *mut *mut c_char, inbytesleft: *mut usize, outbuf: *mut *mut c_char, outbytesleft: *mut usize) -> usize = b"iconv\0";
    fn iconv_close(cd: iconv_t) -> c_int = b"iconv_close\0";
    fn iconv_open(tocode: *const c_char, fromcode: *const c_char) -> iconv_t = b"iconv_open\0";
    fn inet_aton(value: *const c_char, out: *mut in_addr) -> c_int = b"inet_aton\0";
    fn inet_ntoa(value: in_addr) -> *mut c_char = b"inet_ntoa\0";
    fn isalnum(value: c_int) -> c_int = b"isalnum\0";
    fn isalpha(value: c_int) -> c_int = b"isalpha\0";
    fn isascii(value: c_int) -> c_int = b"isascii\0";
    fn isatty(fd: c_int) -> c_int = b"isatty\0";
    fn isblank(value: c_int) -> c_int = b"isblank\0";
    fn iscntrl(value: c_int) -> c_int = b"iscntrl\0";
    fn isdigit(value: c_int) -> c_int = b"isdigit\0";
    fn isgraph(value: c_int) -> c_int = b"isgraph\0";
    fn islower(value: c_int) -> c_int = b"islower\0";
    fn isprint(value: c_int) -> c_int = b"isprint\0";
    fn ispunct(value: c_int) -> c_int = b"ispunct\0";
    fn isspace(value: c_int) -> c_int = b"isspace\0";
    fn isupper(value: c_int) -> c_int = b"isupper\0";
    fn iswalnum(value: wint_t) -> c_int = b"iswalnum\0";
    fn iswalpha(value: wint_t) -> c_int = b"iswalpha\0";
    fn iswblank(value: wint_t) -> c_int = b"iswblank\0";
    fn iswcntrl(value: wint_t) -> c_int = b"iswcntrl\0";
    fn iswdigit(value: wint_t) -> c_int = b"iswdigit\0";
    fn iswgraph(value: wint_t) -> c_int = b"iswgraph\0";
    fn iswlower(value: wint_t) -> c_int = b"iswlower\0";
    fn iswprint(value: wint_t) -> c_int = b"iswprint\0";
    fn iswpunct(value: wint_t) -> c_int = b"iswpunct\0";
    fn iswspace(value: wint_t) -> c_int = b"iswspace\0";
    fn iswupper(value: wint_t) -> c_int = b"iswupper\0";
    fn iswxdigit(value: wint_t) -> c_int = b"iswxdigit\0";
    fn isxdigit(value: c_int) -> c_int = b"isxdigit\0";
    fn l64a(input: c_long) -> *mut c_char = b"l64a\0";
    fn labs(value: c_long) -> c_long = b"labs\0";
    fn ldiv(numer: c_long, denom: c_long) -> ldiv_t = b"ldiv\0";
    fn llabs(value: c_longlong) -> c_longlong = b"llabs\0";
    fn lldiv(numer: c_longlong, denom: c_longlong) -> lldiv_t = b"lldiv\0";
    fn link(path1: *const c_char, path2: *const c_char) -> c_int = b"link\0";
    fn localeconv() -> *mut lconv = b"localeconv\0";
    fn lseek(fd: c_int, offset: off_t, whence: c_int) -> off_t = b"lseek\0";
    fn listen(sockfd: c_int, backlog: c_int) -> c_int = b"listen\0";
    fn mblen(value: *const c_char, count: usize) -> c_int = b"mblen\0";
    fn mbrlen(value: *const c_char, count: usize, state: *mut mbstate_t) -> usize = b"mbrlen\0";
    fn mbrtowc(pwc: *mut wchar_t, value: *const c_char, count: usize, state: *mut mbstate_t) -> usize = b"mbrtowc\0";
    fn mbsinit(state: *const mbstate_t) -> c_int = b"mbsinit\0";
    fn mbsnrtowcs(dst: *mut wchar_t, src: *mut *const c_char, nwc: usize, len: usize, state: *mut mbstate_t) -> usize = b"mbsnrtowcs\0";
    fn mbsrtowcs(dst: *mut wchar_t, src: *mut *const c_char, len: usize, state: *mut mbstate_t) -> usize = b"mbsrtowcs\0";
    fn mbstowcs(dst: *mut wchar_t, src: *const c_char, len: usize) -> usize = b"mbstowcs\0";
    fn mbtowc(pwc: *mut wchar_t, value: *const c_char, count: usize) -> c_int = b"mbtowc\0";
    fn memccpy(dst: *mut c_void, src: *const c_void, value: c_int, count: c_uint) -> *mut c_void = b"memccpy\0";
    fn memmem(haystack: *const c_void, haystack_len: usize, needle: *const c_void, needle_len: usize) -> *mut c_void = b"memmem\0";
    fn mempcpy(dst: *mut c_void, src: *const c_void, count: c_uint) -> *mut c_void = b"mempcpy\0";
    fn memrchr(value: *const c_void, needle: c_int, count: usize) -> *mut c_void = b"memrchr\0";
    fn mkdir(path: *const c_char, mode: mode_t) -> c_int = b"mkdir\0";
    fn nl_langinfo(item: nl_item) -> *mut c_char = b"nl_langinfo\0";
    fn opendir(name: *const c_char) -> *mut DIR = b"opendir\0";
    fn putchar(value: c_int) -> c_int = b"putchar\0";
    fn puts(value: *const c_char) -> c_int = b"puts\0";
    fn qsort(base: *mut c_void, nmemb: usize, size: usize, compar: __compar_fn_t) -> () = b"qsort\0";
    fn qsort_r(base: *mut c_void, nmemb: usize, size: usize, compar: ::core::option::Option<unsafe extern "C" fn(*const c_void, *const c_void, *mut c_void) -> c_int>, thunk: *mut c_void) -> () = b"qsort_r\0";
    fn rand() -> c_int = b"rand\0";
    fn rand_r(seed: *mut c_uint) -> c_int = b"rand_r\0";
    fn random() -> c_long = b"random\0";
    fn rawmemchr(value: *const c_void, needle: c_int) -> *mut c_void = b"rawmemchr\0";
    fn readdir(dir: *mut DIR) -> *mut dirent = b"readdir\0";
    fn remove(path: *const c_char) -> c_int = b"remove\0";
    fn rename(old: *const c_char, new: *const c_char) -> c_int = b"rename\0";
    fn rewind(stream: *mut FILE) -> () = b"rewind\0";
    fn rewinddir(dir: *mut DIR) -> () = b"rewinddir\0";
    fn rindex(value: *const c_char, needle: c_int) -> *mut c_char = b"rindex\0";
    fn rmdir(path: *const c_char) -> c_int = b"rmdir\0";
    fn rpmatch(response: *const c_char) -> c_int = b"rpmatch\0";
    fn setbuf(stream: *mut FILE, buf: *mut c_char) -> () = b"setbuf\0";
    fn setbuffer(stream: *mut FILE, buf: *mut c_char, size: usize) -> () = b"setbuffer\0";
    fn setlinebuf(stream: *mut FILE) -> () = b"setlinebuf\0";
    fn setvbuf(stream: *mut FILE, buf: *mut c_char, mode: c_int, size: usize) -> c_int = b"setvbuf\0";
    fn sleep(seconds: c_uint) -> c_uint = b"sleep\0";
    fn srand(seed: c_uint) -> () = b"srand\0";
    fn srandom(seed: c_uint) -> () = b"srandom\0";
    fn socket(domain: c_int, ty: c_int, protocol: c_int) -> c_int = b"socket\0";
    fn system(value: *const c_char) -> c_int = b"system\0";
    fn gets(buf: *mut c_char) -> *mut c_char = b"gets\0";
    fn stpcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char = b"stpcpy\0";
    fn stpncpy(dst: *mut c_char, src: *const c_char, count: usize) -> *mut c_char = b"stpncpy\0";
    fn strcasecmp(left: *const c_char, right: *const c_char) -> c_int = b"strcasecmp\0";
    fn strcasestr(haystack: *const c_char, needle: *const c_char) -> *mut c_char = b"strcasestr\0";
    fn strcat(dst: *mut c_char, src: *const c_char) -> *mut c_char = b"strcat\0";
    fn strchr(value: *const c_char, needle: c_int) -> *mut c_char = b"strchr\0";
    fn strchrnul(value: *const c_char, needle: c_int) -> *mut c_char = b"strchrnul\0";
    fn strcoll(left: *const c_char, right: *const c_char) -> c_int = b"strcoll\0";
    fn strcmp(left: *const c_char, right: *const c_char) -> c_int = b"strcmp\0";
    fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char = b"strcpy\0";
    fn strcspn(value: *const c_char, reject: *const c_char) -> c_uint = b"strcspn\0";
    fn strdup(value: *const c_char) -> *mut c_char = b"strdup\0";
    fn strerror(error: c_int) -> *mut c_char = b"strerror\0";
    fn strerror_r(error: c_int, buffer: *mut c_char, len: usize) -> *mut c_char = b"strerror_r\0";
    fn strlcat(dst: *mut c_char, src: *const c_char, size: c_uint) -> c_uint = b"strlcat\0";
    fn strlcpy(dst: *mut c_char, src: *const c_char, size: c_uint) -> c_uint = b"strlcpy\0";
    fn strncasecmp(left: *const c_char, right: *const c_char, count: c_uint) -> c_int = b"strncasecmp\0";
    fn strncat(dst: *mut c_char, src: *const c_char, count: c_uint) -> *mut c_char = b"strncat\0";
    fn strncmp(left: *const c_char, right: *const c_char, count: c_uint) -> c_int = b"strncmp\0";
    fn strncpy(dst: *mut c_char, src: *const c_char, count: c_uint) -> *mut c_char = b"strncpy\0";
    fn strndup(value: *const c_char, count: c_uint) -> *mut c_char = b"strndup\0";
    fn strnlen(value: *const c_char, count: usize) -> usize = b"strnlen\0";
    fn strpbrk(value: *const c_char, accept: *const c_char) -> *mut c_char = b"strpbrk\0";
    fn strrchr(value: *const c_char, needle: c_int) -> *mut c_char = b"strrchr\0";
    fn strsep(value: *mut *mut c_char, delim: *const c_char) -> *mut c_char = b"strsep\0";
    fn strspn(value: *const c_char, accept: *const c_char) -> c_uint = b"strspn\0";
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char = b"strstr\0";
    fn strtod(value: *const c_char, end_ptr: *mut *mut c_char) -> f64 = b"strtod\0";
    fn strtof(value: *const c_char, end_ptr: *mut *mut c_char) -> f32 = b"strtof\0";
    fn strtoimax(value: *const c_char, end_ptr: *mut *mut c_char, base: c_int) -> intmax_t = b"strtoimax\0";
    fn strtol(value: *const c_char, end_ptr: *mut *mut c_char, base: c_int) -> c_long = b"strtol\0";
    fn strtoll(value: *const c_char, end_ptr: *mut *mut c_char, base: c_int) -> c_longlong = b"strtoll\0";
    fn strtok(value: *mut c_char, delim: *const c_char) -> *mut c_char = b"strtok\0";
    fn strtok_r(value: *mut c_char, delim: *const c_char, save_ptr: *mut *mut c_char) -> *mut c_char = b"strtok_r\0";
    fn strtoul(value: *const c_char, end_ptr: *mut *mut c_char, base: c_int) -> c_ulong = b"strtoul\0";
    fn strtoull(value: *const c_char, end_ptr: *mut *mut c_char, base: c_int) -> c_ulonglong = b"strtoull\0";
    fn strtoumax(value: *const c_char, end_ptr: *mut *mut c_char, base: c_int) -> uintmax_t = b"strtoumax\0";
    fn strverscmp(left: *const c_char, right: *const c_char) -> c_int = b"strverscmp\0";
    fn strxfrm(dst: *mut c_char, src: *const c_char, size: c_uint) -> c_uint = b"strxfrm\0";
    fn swab(src: *const c_void, dst: *mut c_void, count: isize) -> () = b"swab\0";
    fn time(timer: *mut time_t) -> time_t = b"time\0";
    fn times(buf: *mut tms) -> clock_t = b"times\0";
    fn tdelete(key: *const c_void, rootp: *mut *mut c_void, compar: __compar_fn_t) -> *mut c_void = b"tdelete\0";
    fn tdestroy(root: *mut c_void, freefct: ::core::option::Option<unsafe extern "C" fn(*mut c_void)>) -> () = b"tdestroy\0";
    fn tfind(key: *const c_void, rootp: *mut *mut c_void, compar: __compar_fn_t) -> *mut c_void = b"tfind\0";
    fn toascii(value: c_int) -> c_int = b"toascii\0";
    fn tolower(value: c_int) -> c_int = b"tolower\0";
    fn toupper(value: c_int) -> c_int = b"toupper\0";
    fn towlower(value: wint_t) -> wint_t = b"towlower\0";
    fn towupper(value: wint_t) -> wint_t = b"towupper\0";
    fn tsearch(key: *const c_void, rootp: *mut *mut c_void, compar: __compar_fn_t) -> *mut c_void = b"tsearch\0";
    fn twalk(root: *const c_void, action: ::core::option::Option<unsafe extern "C" fn(*const c_void, VISIT, c_int)>) -> () = b"twalk\0";
    fn ungetc(value: c_int, stream: *mut FILE) -> c_int = b"ungetc\0";
    fn unlink(path: *const c_char) -> c_int = b"unlink\0";
    fn usleep(useconds: useconds_t) -> c_int = b"usleep\0";
    fn wcpcpy(dst: *mut wchar_t, src: *const wchar_t) -> *mut wchar_t = b"wcpcpy\0";
    fn wcpncpy(dst: *mut wchar_t, src: *const wchar_t, count: usize) -> *mut wchar_t = b"wcpncpy\0";
    fn wcscasecmp(left: *const wchar_t, right: *const wchar_t) -> c_int = b"wcscasecmp\0";
    fn wcscat(dst: *mut wchar_t, src: *const wchar_t) -> *mut wchar_t = b"wcscat\0";
    fn wcschr(value: *const wchar_t, needle: wchar_t) -> *mut wchar_t = b"wcschr\0";
    fn wcscmp(left: *const wchar_t, right: *const wchar_t) -> c_int = b"wcscmp\0";
    fn wcscoll(left: *const wchar_t, right: *const wchar_t) -> c_int = b"wcscoll\0";
    fn wcscpy(dst: *mut wchar_t, src: *const wchar_t) -> *mut wchar_t = b"wcscpy\0";
    fn wcscspn(value: *const wchar_t, reject: *const wchar_t) -> usize = b"wcscspn\0";
    fn wcsdup(value: *const wchar_t) -> *mut wchar_t = b"wcsdup\0";
    fn wcslen(value: *const wchar_t) -> usize = b"wcslen\0";
    fn wcsncasecmp(left: *const wchar_t, right: *const wchar_t, count: usize) -> c_int = b"wcsncasecmp\0";
    fn wcsncat(dst: *mut wchar_t, src: *const wchar_t, count: usize) -> *mut wchar_t = b"wcsncat\0";
    fn wcsncmp(left: *const wchar_t, right: *const wchar_t, count: usize) -> c_int = b"wcsncmp\0";
    fn wcsncpy(dst: *mut wchar_t, src: *const wchar_t, count: usize) -> *mut wchar_t = b"wcsncpy\0";
    fn wcsnlen(value: *const wchar_t, count: usize) -> usize = b"wcsnlen\0";
    fn wcspbrk(value: *const wchar_t, accept: *const wchar_t) -> *mut wchar_t = b"wcspbrk\0";
    fn wcsrchr(value: *const wchar_t, needle: wchar_t) -> *mut wchar_t = b"wcsrchr\0";
    fn wcsnrtombs(dst: *mut c_char, src: *mut *const wchar_t, nwc: usize, len: usize, state: *mut mbstate_t) -> usize = b"wcsnrtombs\0";
    fn wcsrtombs(dst: *mut c_char, src: *mut *const wchar_t, len: usize, state: *mut mbstate_t) -> usize = b"wcsrtombs\0";
    fn wcsspn(value: *const wchar_t, accept: *const wchar_t) -> usize = b"wcsspn\0";
    fn wcsstr(haystack: *const wchar_t, needle: *const wchar_t) -> *mut wchar_t = b"wcsstr\0";
    fn wcstod(value: *const wchar_t, end_ptr: *mut *mut wchar_t) -> f64 = b"wcstod\0";
    fn wcstof(value: *const wchar_t, end_ptr: *mut *mut wchar_t) -> f32 = b"wcstof\0";
    fn wcstoimax(value: *const wchar_t, end_ptr: *mut *mut wchar_t, base: c_int) -> intmax_t = b"wcstoimax\0";
    fn wcstok(value: *mut wchar_t, delim: *const wchar_t, save_ptr: *mut *mut wchar_t) -> *mut wchar_t = b"wcstok\0";
    fn wcstol(value: *const wchar_t, end_ptr: *mut *mut wchar_t, base: c_int) -> c_long = b"wcstol\0";
    fn wcstoll(value: *const wchar_t, end_ptr: *mut *mut wchar_t, base: c_int) -> c_longlong = b"wcstoll\0";
    fn wcstombs(dst: *mut c_char, src: *const wchar_t, len: usize) -> usize = b"wcstombs\0";
    fn wcstoul(value: *const wchar_t, end_ptr: *mut *mut wchar_t, base: c_int) -> c_ulong = b"wcstoul\0";
    fn wcstoull(value: *const wchar_t, end_ptr: *mut *mut wchar_t, base: c_int) -> c_ulonglong = b"wcstoull\0";
    fn wcstoumax(value: *const wchar_t, end_ptr: *mut *mut wchar_t, base: c_int) -> uintmax_t = b"wcstoumax\0";
    fn wcswidth(value: *const wchar_t, count: usize) -> c_int = b"wcswidth\0";
    fn wcsxfrm(dst: *mut wchar_t, src: *const wchar_t, size: usize) -> usize = b"wcsxfrm\0";
    fn wctob(value: wint_t) -> c_int = b"wctob\0";
    fn wctomb(dst: *mut c_char, value: wchar_t) -> c_int = b"wctomb\0";
    fn wcrtomb(dst: *mut c_char, value: wchar_t, state: *mut mbstate_t) -> usize = b"wcrtomb\0";
    fn wcwidth(value: wchar_t) -> c_int = b"wcwidth\0";
    fn wmemchr(value: *const wchar_t, needle: wchar_t, count: usize) -> *mut wchar_t = b"wmemchr\0";
    fn wmemcmp(left: *const wchar_t, right: *const wchar_t, count: usize) -> c_int = b"wmemcmp\0";
    fn wmemcpy(dst: *mut wchar_t, src: *const wchar_t, count: usize) -> *mut wchar_t = b"wmemcpy\0";
    fn wmemmove(dst: *mut wchar_t, src: *const wchar_t, count: usize) -> *mut wchar_t = b"wmemmove\0";
    fn wmempcpy(dst: *mut wchar_t, src: *const wchar_t, count: usize) -> *mut wchar_t = b"wmempcpy\0";
    fn wmemset(value: *mut wchar_t, fill: wchar_t, count: usize) -> *mut wchar_t = b"wmemset\0";
}

forward_next_ignore_locale_fn! {
    fn iswalnum_l(value: wint_t; locale: locale_t) -> c_int = b"iswalnum\0";
    fn iswalpha_l(value: wint_t; locale: locale_t) -> c_int = b"iswalpha\0";
    fn iswblank_l(value: wint_t; locale: locale_t) -> c_int = b"iswblank\0";
    fn iswcntrl_l(value: wint_t; locale: locale_t) -> c_int = b"iswcntrl\0";
    fn iswdigit_l(value: wint_t; locale: locale_t) -> c_int = b"iswdigit\0";
    fn iswgraph_l(value: wint_t; locale: locale_t) -> c_int = b"iswgraph\0";
    fn iswlower_l(value: wint_t; locale: locale_t) -> c_int = b"iswlower\0";
    fn iswprint_l(value: wint_t; locale: locale_t) -> c_int = b"iswprint\0";
    fn iswpunct_l(value: wint_t; locale: locale_t) -> c_int = b"iswpunct\0";
    fn iswspace_l(value: wint_t; locale: locale_t) -> c_int = b"iswspace\0";
    fn iswupper_l(value: wint_t; locale: locale_t) -> c_int = b"iswupper\0";
    fn iswxdigit_l(value: wint_t; locale: locale_t) -> c_int = b"iswxdigit\0";
    fn nl_langinfo_l(item: nl_item; locale: locale_t) -> *mut c_char = b"nl_langinfo\0";
    fn strcasecmp_l(left: *const c_char, right: *const c_char; locale: locale_t) -> c_int = b"strcasecmp\0";
    fn strcoll_l(left: *const c_char, right: *const c_char; locale: locale_t) -> c_int = b"strcoll\0";
    fn strncasecmp_l(left: *const c_char, right: *const c_char, count: c_uint; locale: locale_t) -> c_int = b"strncasecmp\0";
    fn strtod_l(value: *const c_char, end_ptr: *mut *mut c_char; locale: locale_t) -> f64 = b"strtod\0";
    fn strtof_l(value: *const c_char, end_ptr: *mut *mut c_char; locale: locale_t) -> f32 = b"strtof\0";
    fn strtol_l(value: *const c_char, end_ptr: *mut *mut c_char, base: c_int; locale: locale_t) -> c_long = b"strtol\0";
    fn strtoll_l(value: *const c_char, end_ptr: *mut *mut c_char, base: c_int; locale: locale_t) -> c_longlong = b"strtoll\0";
    fn strtoul_l(value: *const c_char, end_ptr: *mut *mut c_char, base: c_int; locale: locale_t) -> c_ulong = b"strtoul\0";
    fn strtoull_l(value: *const c_char, end_ptr: *mut *mut c_char, base: c_int; locale: locale_t) -> c_ulonglong = b"strtoull\0";
    fn strxfrm_l(dst: *mut c_char, src: *const c_char, size: c_uint; locale: locale_t) -> c_uint = b"strxfrm\0";
    fn towlower_l(value: wint_t; locale: locale_t) -> wint_t = b"towlower\0";
    fn towupper_l(value: wint_t; locale: locale_t) -> wint_t = b"towupper\0";
    fn wcscasecmp_l(left: *const wchar_t, right: *const wchar_t; locale: locale_t) -> c_int = b"wcscasecmp\0";
    fn wcscoll_l(left: *const wchar_t, right: *const wchar_t; locale: locale_t) -> c_int = b"wcscoll\0";
    fn wcsncasecmp_l(left: *const wchar_t, right: *const wchar_t, count: usize; locale: locale_t) -> c_int = b"wcsncasecmp\0";
    fn wcstod_l(value: *const wchar_t, end_ptr: *mut *mut wchar_t; locale: locale_t) -> f64 = b"wcstod\0";
    fn wcstof_l(value: *const wchar_t, end_ptr: *mut *mut wchar_t; locale: locale_t) -> f32 = b"wcstof\0";
    fn wcstol_l(value: *const wchar_t, end_ptr: *mut *mut wchar_t, base: c_int; locale: locale_t) -> c_long = b"wcstol\0";
    fn wcstoll_l(value: *const wchar_t, end_ptr: *mut *mut wchar_t, base: c_int; locale: locale_t) -> c_longlong = b"wcstoll\0";
    fn wcstoul_l(value: *const wchar_t, end_ptr: *mut *mut wchar_t, base: c_int; locale: locale_t) -> c_ulong = b"wcstoul\0";
    fn wcstoull_l(value: *const wchar_t, end_ptr: *mut *mut wchar_t, base: c_int; locale: locale_t) -> c_ulonglong = b"wcstoull\0";
    fn wcsxfrm_l(dst: *mut wchar_t, src: *const wchar_t, size: usize; locale: locale_t) -> usize = b"wcsxfrm\0";
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn iswctype(value: wint_t, desc: wctype_t) -> c_int {
    let Some(host_desc) = resolve_descriptor(&WCTYPE_DESCRIPTORS, desc) else {
        runtime::set_errno(libc::EINVAL);
        return 0;
    };

    let function: HostIswctypeFn = unsafe { runtime::resolve_next_function(b"iswctype\0") };
    unsafe { function(value, host_desc) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn iswctype_l(value: wint_t, desc: wctype_t, locale: locale_t) -> c_int {
    let _ = locale;
    unsafe { iswctype(value, desc) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wctrans(name: *const c_char) -> wctrans_t {
    let function: HostWctransFn = unsafe { runtime::resolve_next_function(b"wctrans\0") };
    let desc = unsafe { function(name) };
    intern_descriptor(&WCTRANS_DESCRIPTORS, desc)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wctrans_l(name: *const c_char, locale: locale_t) -> wctrans_t {
    let _ = locale;
    unsafe { wctrans(name) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wctype(name: *const c_char) -> wctype_t {
    let function: HostWctypeFn = unsafe { runtime::resolve_next_function(b"wctype\0") };
    let desc = unsafe { function(name) };
    intern_descriptor(&WCTYPE_DESCRIPTORS, desc)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wctype_l(name: *const c_char, locale: locale_t) -> wctype_t {
    let _ = locale;
    unsafe { wctype(name) }
}

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
pub unsafe extern "C" fn _Exit(status: c_int) -> ! {
    let function: unsafe extern "C" fn(c_int) -> ! =
        unsafe { runtime::resolve_next_function(b"_Exit\0") };
    unsafe { function(status) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn _exit(status: c_int) -> ! {
    let function: unsafe extern "C" fn(c_int) -> ! =
        unsafe { runtime::resolve_next_function(b"_exit\0") };
    unsafe { function(status) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn abort() -> ! {
    let function: unsafe extern "C" fn() -> ! =
        unsafe { runtime::resolve_next_function(b"abort\0") };
    unsafe { function() }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exit(status: c_int) -> ! {
    let function: unsafe extern "C" fn(c_int) -> ! =
        unsafe { runtime::resolve_next_function(b"exit\0") };
    unsafe { function(status) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cos(value: f64) -> f64 {
    let function: CosFn = unsafe { resolve_libm_function(b"cos\0") };
    unsafe { function(value) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cosf(value: f32) -> f32 {
    let function: CosfFn = unsafe { resolve_libm_function(b"cosf\0") };
    unsafe { function(value) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn free(ptr: *mut c_void) {
    let function: unsafe extern "C" fn(*mut c_void) =
        unsafe { runtime::resolve_next_function(b"free\0") };
    unsafe { function(ptr) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn gettimeofday(value: *mut timeval, tz: *mut c_void) -> c_int {
    let function: unsafe extern "C" fn(*mut libc::timeval, *mut c_void) -> c_int =
        unsafe { runtime::resolve_next_function(b"gettimeofday\0") };

    if value.is_null() {
        return unsafe { function(core::ptr::null_mut(), tz) };
    }

    let mut host_value = libc::timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let result = unsafe { function(&mut host_value, tz) };

    if result == 0 {
        unsafe { *value = host_timeval_to_badge(&host_value) };
    }

    result
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn clock_gettime(clock_id: clockid_t, tp: *mut timespec) -> c_int {
    let function: unsafe extern "C" fn(clockid_t, *mut libc::timespec) -> c_int =
        unsafe { runtime::resolve_next_function(b"clock_gettime\0") };

    if tp.is_null() {
        return unsafe { function(clock_id, core::ptr::null_mut()) };
    }

    let mut host_value = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    let result = unsafe { function(clock_id, &mut host_value) };

    if result == 0 {
        unsafe { *tp = host_timespec_to_badge(&host_value) };
    }

    result
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
pub unsafe extern "C" fn gmtime_r(timer: *const time_t, result: *mut tm) -> *mut tm {
    if result.is_null() {
        return core::ptr::null_mut();
    }

    let function: unsafe extern "C" fn(*const time_t, *mut libc::tm) -> *mut libc::tm =
        unsafe { runtime::resolve_next_function(b"gmtime_r\0") };
    let mut host_result = unsafe { core::mem::zeroed::<libc::tm>() };
    let resolved = unsafe { function(timer, &mut host_result) };

    if resolved.is_null() {
        core::ptr::null_mut()
    } else {
        unsafe { copy_host_tm_into_badge(&mut *result, &host_result) };
        result
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn gmtime(timer: *const time_t) -> *mut tm {
    let buffer = GMTIME_BUFFER.0.get();
    if unsafe { gmtime_r(timer, buffer) }.is_null() {
        core::ptr::null_mut()
    } else {
        buffer
    }
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
pub unsafe extern "C" fn localtime_r(timer: *const time_t, result: *mut tm) -> *mut tm {
    if result.is_null() {
        return core::ptr::null_mut();
    }

    let function: unsafe extern "C" fn(*const time_t, *mut libc::tm) -> *mut libc::tm =
        unsafe { runtime::resolve_next_function(b"localtime_r\0") };
    let mut host_result = unsafe { core::mem::zeroed::<libc::tm>() };
    let resolved = unsafe { function(timer, &mut host_result) };

    if resolved.is_null() {
        core::ptr::null_mut()
    } else {
        unsafe { copy_host_tm_into_badge(&mut *result, &host_result) };
        result
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn localtime(timer: *const time_t) -> *mut tm {
    let buffer = LOCALTIME_BUFFER.0.get();
    if unsafe { localtime_r(timer, buffer) }.is_null() {
        core::ptr::null_mut()
    } else {
        buffer
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mktime(timeptr: *mut tm) -> time_t {
    let function: unsafe extern "C" fn(*mut libc::tm) -> time_t =
        unsafe { runtime::resolve_next_function(b"mktime\0") };

    if timeptr.is_null() {
        return unsafe { function(core::ptr::null_mut()) };
    }

    let mut host_value = badge_tm_to_host(unsafe { &*timeptr });
    let result = unsafe { function(&mut host_value) };
    unsafe { copy_host_tm_into_badge(&mut *timeptr, &host_value) };
    result
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
pub unsafe extern "C" fn select(
    n: c_int,
    readfds: *mut fd_set,
    writefds: *mut fd_set,
    exceptfds: *mut fd_set,
    timeout: *mut timeval,
) -> c_int {
    let function: unsafe extern "C" fn(
        c_int,
        *mut libc::fd_set,
        *mut libc::fd_set,
        *mut libc::fd_set,
        *mut libc::timeval,
    ) -> c_int = unsafe { runtime::resolve_next_function(b"select\0") };

    let mut host_readfds = if readfds.is_null() {
        None
    } else {
        Some(unsafe { badge_fd_set_to_host(&*readfds) })
    };
    let mut host_writefds = if writefds.is_null() {
        None
    } else {
        Some(unsafe { badge_fd_set_to_host(&*writefds) })
    };
    let mut host_exceptfds = if exceptfds.is_null() {
        None
    } else {
        Some(unsafe { badge_fd_set_to_host(&*exceptfds) })
    };
    let mut host_timeout = if timeout.is_null() {
        None
    } else {
        Some(badge_timeval_to_host(unsafe { &*timeout }))
    };

    let result = unsafe {
        function(
            n,
            host_readfds
                .as_mut()
                .map_or(core::ptr::null_mut(), |value| value as *mut libc::fd_set),
            host_writefds
                .as_mut()
                .map_or(core::ptr::null_mut(), |value| value as *mut libc::fd_set),
            host_exceptfds
                .as_mut()
                .map_or(core::ptr::null_mut(), |value| value as *mut libc::fd_set),
            host_timeout
                .as_mut()
                .map_or(core::ptr::null_mut(), |value| value as *mut libc::timeval),
        )
    };

    if result >= 0 {
        if let Some(host_value) = host_readfds.as_ref() {
            unsafe { *readfds = host_fd_set_to_badge(host_value) };
        }
        if let Some(host_value) = host_writefds.as_ref() {
            unsafe { *writefds = host_fd_set_to_badge(host_value) };
        }
        if let Some(host_value) = host_exceptfds.as_ref() {
            unsafe { *exceptfds = host_fd_set_to_badge(host_value) };
        }
        if let Some(host_value) = host_timeout.as_ref() {
            unsafe { *timeout = host_timeval_to_badge(host_value) };
        }
    }

    result
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn regcomp(
    preg: *mut regex_t,
    pattern: *const c_char,
    cflags: c_int,
) -> c_int {
    if preg.is_null() {
        runtime::abort_with_message(b"why2025-badge-emu-abi regcomp received null regex_t\n")
    }

    if let Some(mut existing) = unsafe { take_regex_bridge(preg) } {
        unsafe { resolve_host_regfree()(&mut existing.compiled) };
    }

    let mut bridge = Box::new(RegexBridge {
        compiled: unsafe { core::mem::zeroed::<HostRegex>() },
    });
    let status = unsafe { resolve_host_regcomp()(&mut bridge.compiled, pattern, cflags) };
    let raw_bridge = Box::into_raw(bridge);
    unsafe { install_regex_bridge(preg, raw_bridge) };

    status
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn regerror(
    errcode: c_int,
    preg: *const regex_t,
    errbuf: *mut c_char,
    errbuf_size: usize,
) -> usize {
    let host_preg = unsafe { regex_bridge_ptr(preg) }
        .map(|bridge| unsafe { &(*bridge).compiled as *const HostRegex })
        .unwrap_or(core::ptr::null());
    unsafe { resolve_host_regerror()(errcode, host_preg, errbuf, errbuf_size) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn regexec(
    preg: *const regex_t,
    text: *const c_char,
    nmatch: usize,
    pmatch: *mut [regmatch_t; 0usize],
    eflags: c_int,
) -> c_int {
    let bridge = unsafe { require_regex_bridge(preg) };
    let copy_matches = nmatch > 0 && !pmatch.is_null();
    let mut host_matches = if copy_matches {
        vec![
            HostRegmatch {
                rm_so: -1,
                rm_eo: -1,
            };
            nmatch
        ]
    } else {
        Vec::new()
    };

    let status = unsafe {
        resolve_host_regexec()(
            &bridge.compiled,
            text,
            if copy_matches { nmatch } else { 0 },
            if copy_matches {
                host_matches.as_mut_ptr()
            } else {
                core::ptr::null_mut()
            },
            eflags,
        )
    };

    if copy_matches && status == 0 {
        let badge_matches = pmatch.cast::<regmatch_t>();
        for (index, matched) in host_matches.iter().copied().enumerate() {
            unsafe {
                badge_matches.add(index).write(regmatch_t {
                    rm_so: matched.rm_so as isize,
                    rm_eo: matched.rm_eo as isize,
                });
            }
        }
    }

    status
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn regfree(preg: *mut regex_t) {
    if let Some(mut bridge) = unsafe { take_regex_bridge(preg) } {
        unsafe { resolve_host_regfree()(&mut bridge.compiled) };
    } else {
        unsafe { clear_badge_regex(preg) };
    }
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
pub unsafe extern "C" fn asctime_r(value: *const tm, buf: *mut [c_char; 26usize]) -> *mut c_char {
    if value.is_null() {
        let function: unsafe extern "C" fn(*const libc::tm, *mut [c_char; 26usize]) -> *mut c_char =
            unsafe { runtime::resolve_next_function(b"asctime_r\0") };
        return unsafe { function(core::ptr::null(), buf) };
    }

    let function: unsafe extern "C" fn(*const libc::tm, *mut [c_char; 26usize]) -> *mut c_char =
        unsafe { runtime::resolve_next_function(b"asctime_r\0") };
    let host_value = badge_tm_to_host(unsafe { &*value });
    unsafe { function(&host_value, buf) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn asctime(value: *const tm) -> *mut c_char {
    let function: unsafe extern "C" fn(*const libc::tm) -> *mut c_char =
        unsafe { runtime::resolve_next_function(b"asctime\0") };

    if value.is_null() {
        return unsafe { function(core::ptr::null()) };
    }

    let host_value = badge_tm_to_host(unsafe { &*value });
    unsafe { function(&host_value) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ctime_r(timer: *const time_t, buf: *mut [c_char; 26usize]) -> *mut c_char {
    let function: unsafe extern "C" fn(*const time_t, *mut [c_char; 26usize]) -> *mut c_char =
        unsafe { runtime::resolve_next_function(b"ctime_r\0") };
    unsafe { function(timer, buf) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strftime(
    value: *mut c_char,
    maxsize: usize,
    fmt: *const c_char,
    tblock: *const tm,
) -> usize {
    let function: unsafe extern "C" fn(
        *mut c_char,
        usize,
        *const c_char,
        *const libc::tm,
    ) -> usize = unsafe { runtime::resolve_next_function(b"strftime\0") };

    if tblock.is_null() {
        return unsafe { function(value, maxsize, fmt, core::ptr::null()) };
    }

    let host_value = badge_tm_to_host(unsafe { &*tblock });
    unsafe { function(value, maxsize, fmt, &host_value) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strftime_l(
    value: *mut c_char,
    maxsize: usize,
    fmt: *const c_char,
    tblock: *const tm,
    locale: locale_t,
) -> usize {
    let _ = locale;
    unsafe { strftime(value, maxsize, fmt, tblock) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcsftime(
    value: *mut wchar_t,
    maxsize: usize,
    fmt: *const wchar_t,
    tblock: *const tm,
) -> usize {
    let function: unsafe extern "C" fn(
        *mut wchar_t,
        usize,
        *const wchar_t,
        *const libc::tm,
    ) -> usize = unsafe { runtime::resolve_next_function(b"wcsftime\0") };

    let host_tblock = unsafe { badge_tm_to_host(&*tblock) };
    unsafe { function(value, maxsize, fmt, &host_tblock) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcsftime_l(
    value: *mut wchar_t,
    maxsize: usize,
    fmt: *const wchar_t,
    tblock: *const tm,
    locale: locale_t,
) -> usize {
    let _ = locale;
    unsafe { wcsftime(value, maxsize, fmt, tblock) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strptime(
    input: *const c_char,
    fmt: *const c_char,
    result: *mut tm,
) -> *mut c_char {
    let function: unsafe extern "C" fn(*const c_char, *const c_char, *mut libc::tm) -> *mut c_char =
        unsafe { runtime::resolve_next_function(b"strptime\0") };

    if result.is_null() {
        return unsafe { function(input, fmt, core::ptr::null_mut()) };
    }

    let mut host_value = badge_tm_to_host(unsafe { &*result });
    let parsed = unsafe { function(input, fmt, &mut host_value) };
    unsafe { copy_host_tm_into_badge(&mut *result, &host_value) };
    parsed
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strptime_l(
    input: *const c_char,
    fmt: *const c_char,
    result: *mut tm,
    locale: locale_t,
) -> *mut c_char {
    let _ = locale;
    unsafe { strptime(input, fmt, result) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn stat(path: *const c_char, buf: *mut stat) -> c_int {
    let function: unsafe extern "C" fn(*const c_char, *mut stat) -> c_int =
        unsafe { runtime::resolve_next_function(b"stat\0") };
    unsafe { function(path, buf) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sin(value: f64) -> f64 {
    let function: SinFn = unsafe { resolve_libm_function(b"sin\0") };
    unsafe { function(value) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sincos(value: f64, sin_out: *mut f64, cos_out: *mut f64) {
    let function: SincosFn = unsafe { resolve_libm_function(b"sincos\0") };
    unsafe { function(value, sin_out, cos_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sincosf(value: f32, sin_out: *mut f32, cos_out: *mut f32) {
    let function: SincosfFn = unsafe { resolve_libm_function(b"sincosf\0") };
    unsafe { function(value, sin_out, cos_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sinf(value: f32) -> f32 {
    let function: SinfFn = unsafe { resolve_libm_function(b"sinf\0") };
    unsafe { function(value) }
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
    use std::ffi::{CStr, CString};
    use std::format;

    static TWALK_VISITS: AtomicUsize = AtomicUsize::new(0);

    unsafe extern "C" {
        static mut optarg: *mut c_char;
        static mut opterr: c_int;
        static mut optind: c_int;
    }

    fn assert_tm_matches_host(badge: &tm, host: &libc::tm) {
        assert_eq!(badge.tm_sec, host.tm_sec);
        assert_eq!(badge.tm_min, host.tm_min);
        assert_eq!(badge.tm_hour, host.tm_hour);
        assert_eq!(badge.tm_mday, host.tm_mday);
        assert_eq!(badge.tm_mon, host.tm_mon);
        assert_eq!(badge.tm_year, host.tm_year);
        assert_eq!(badge.tm_wday, host.tm_wday);
        assert_eq!(badge.tm_yday, host.tm_yday);
        assert_eq!(badge.tm_isdst, host.tm_isdst);
    }

    fn badge_fd_word_and_bit(fd: c_int) -> (usize, usize) {
        let bits_per_word = core::mem::size_of::<c_ulong>() * 8;
        let index = fd as usize / bits_per_word;
        let bit = fd as usize % bits_per_word;
        (index, bit)
    }

    fn set_badge_fd(set: &mut fd_set, fd: c_int) {
        let (index, bit) = badge_fd_word_and_bit(fd);
        set.__fds_bits[index] |= (1 as c_ulong) << bit;
    }

    fn badge_fd_is_set(set: &fd_set, fd: c_int) -> bool {
        let (index, bit) = badge_fd_word_and_bit(fd);
        (set.__fds_bits[index] & ((1 as c_ulong) << bit)) != 0
    }

    unsafe extern "C" fn compare_i32(left: *const c_void, right: *const c_void) -> c_int {
        let left = unsafe { *(left.cast::<i32>()) };
        let right = unsafe { *(right.cast::<i32>()) };
        left.cmp(&right) as c_int
    }

    unsafe extern "C" fn compare_i32_with_direction(
        left: *const c_void,
        right: *const c_void,
        thunk: *mut c_void,
    ) -> c_int {
        let left = unsafe { *(left.cast::<i32>()) };
        let right = unsafe { *(right.cast::<i32>()) };
        let direction = if thunk.is_null() {
            1
        } else {
            unsafe { *(thunk.cast::<c_int>()) }
        };

        if left < right {
            -direction
        } else if left > right {
            direction
        } else {
            0
        }
    }

    unsafe extern "C" fn free_i32(value: *mut c_void) {
        drop(unsafe { Box::from_raw(value.cast::<i32>()) });
    }

    unsafe extern "C" fn record_twalk_visit(_node: *const c_void, _visit: VISIT, _depth: c_int) {
        TWALK_VISITS.fetch_add(1, Ordering::Relaxed);
    }

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

    #[test]
    fn trig_forwards_match_expected_values() {
        unsafe {
            assert!((cos(0.0) - 1.0).abs() < 1e-12);
            assert!(sin(0.0).abs() < 1e-12);
            assert!((cosf(0.0) - 1.0).abs() < 1e-6);
            assert!(sinf(0.0).abs() < 1e-6);

            let mut sin_out = 0.0;
            let mut cos_out = 0.0;
            sincos(0.5, &mut sin_out, &mut cos_out);
            assert!((sin_out - 0.479_425_538_604_203).abs() < 1e-12);
            assert!((cos_out - 0.877_582_561_890_372_8).abs() < 1e-12);

            let mut sin_out_f = 0.0_f32;
            let mut cos_out_f = 0.0_f32;
            sincosf(0.5, &mut sin_out_f, &mut cos_out_f);
            assert!((sin_out_f - 0.479_425_55).abs() < 1e-6);
            assert!((cos_out_f - 0.877_582_55).abs() < 1e-6);
        }
    }

    #[test]
    fn scalar_libm_forwards_match_expected_values() {
        unsafe {
            assert!((acos(1.0) - 0.0).abs() < 1e-12);
            assert!((acosf(1.0) - 0.0).abs() < 1e-6);
            assert!((asin(0.0) - 0.0).abs() < 1e-12);
            assert!((asinf(0.0) - 0.0).abs() < 1e-6);
            assert!((atan2(1.0, 1.0) - 0.785_398_163_397_448_3).abs() < 1e-12);
            assert!((atan2f(1.0, 1.0) - 0.785_398_2).abs() < 1e-6);
            assert_eq!(ceil(-1.2), -1.0);
            assert_eq!(ceilf(-1.2), -1.0);
            assert_eq!(floor(-1.2), -2.0);
            assert_eq!(floorf(-1.2), -2.0);
            assert_eq!(copysign(1.0, -2.0), -1.0);
            assert_eq!(copysignf(1.0, -2.0), -1.0);
            assert!(erf(0.0).abs() < 1e-12);
            assert!(erff(0.0).abs() < 1e-6);
            assert_eq!(exp2(3.0), 8.0);
            assert_eq!(exp2f(3.0), 8.0);
            assert!(expm1(0.0).abs() < 1e-12);
            assert!(expm1f(0.0).abs() < 1e-6);
            assert_eq!(fabs(-3.0), 3.0);
            assert_eq!(fabsf(-3.0), 3.0);
            assert_eq!(fdim(5.0, 2.0), 3.0);
            assert_eq!(fdimf(5.0, 2.0), 3.0);
            assert_ne!(finite(1.0), 0);
            assert_eq!(finite(f64::INFINITY), 0);
            assert_ne!(finitef(1.0), 0);
            assert_eq!(finitef(f32::INFINITY), 0);
            assert_eq!(fma(2.0, 3.0, 4.0), 10.0);
            assert_eq!(fmaf(2.0, 3.0, 4.0), 10.0);
            assert_eq!(fmax(2.0, 3.0), 3.0);
            assert_eq!(fmaxf(2.0, 3.0), 3.0);
            assert_eq!(fmin(2.0, 3.0), 2.0);
            assert_eq!(fminf(2.0, 3.0), 2.0);
            assert_eq!(fmod(7.0, 4.0), 3.0);
            assert_eq!(fmodf(7.0, 4.0), 3.0);
            let mut exponent = 0;
            assert_eq!(frexp(8.0, &mut exponent), 0.5);
            assert_eq!(exponent, 4);
            let mut exponent_f = 0;
            assert_eq!(frexpf(8.0, &mut exponent_f), 0.5);
            assert_eq!(exponent_f, 4);
            assert_eq!(hypot(3.0, 4.0), 5.0);
            assert_eq!(hypotf(3.0, 4.0), 5.0);
            assert_eq!(ilogb(8.0), 3);
            assert_eq!(ilogbf(8.0), 3);
            assert_ne!(isinf(f64::INFINITY), 0);
            assert_ne!(isinff(f32::INFINITY), 0);
            assert_ne!(isnan(f64::NAN), 0);
            assert_ne!(isnanf(f32::NAN), 0);
        }
    }

    #[test]
    fn special_libm_forwards_match_expected_values() {
        unsafe {
            assert!((gamma(5.0) - 3.178_053_830_347_945_8).abs() < 1e-12);
            assert!((gammaf(5.0) - 3.178_053_9).abs() < 1e-6);
            assert!((j0(0.0) - 1.0).abs() < 1e-12);
            assert!((j0f(0.0) - 1.0).abs() < 1e-6);
            assert!(j1(0.0).abs() < 1e-12);
            assert!(j1f(0.0).abs() < 1e-6);
            assert!((jn(0, 0.25) - j0(0.25)).abs() < 1e-12);
            assert!((jnf(0, 0.25) - j0f(0.25)).abs() < 1e-6);
            assert!((jn(1, 0.25) - j1(0.25)).abs() < 1e-12);
            assert!((jnf(1, 0.25) - j1f(0.25)).abs() < 1e-6);
        }
    }

    #[test]
    fn extended_libm_forwards_match_expected_values() {
        unsafe {
            assert_eq!(ldexp(0.5, 4), 8.0);
            assert_eq!(ldexpf(0.5, 4), 8.0);
            assert!((lgamma(5.0) - 3.178_053_830_347_945_8).abs() < 1e-12);
            let mut sign = 0;
            assert!((lgamma_r(5.0, &mut sign) - 3.178_053_830_347_945_8).abs() < 1e-12);
            assert_eq!(sign, 1);
            let mut sign_f = 0;
            assert!((lgammaf(5.0) - 3.178_053_9).abs() < 1e-6);
            assert!((lgammaf_r(5.0, &mut sign_f) - 3.178_053_9).abs() < 1e-6);
            assert_eq!(sign_f, 1);
            assert_eq!(llrint(2.0), 2);
            assert_eq!(llrintf(2.0), 2);
            assert_eq!(llround(2.4), 2);
            assert_eq!(llroundf(2.4), 2);
            assert!(log(1.0).abs() < 1e-12);
            assert!(logf(1.0).abs() < 1e-6);
            assert_eq!(log10(100.0), 2.0);
            assert_eq!(log10f(100.0), 2.0);
            assert!((log1p(1.0) - core::f64::consts::LN_2).abs() < 1e-12);
            assert!((log1pf(1.0) - core::f32::consts::LN_2).abs() < 1e-6);
            assert_eq!(log2(8.0), 3.0);
            assert_eq!(log2f(8.0), 3.0);
            assert_eq!(logb(8.0), 3.0);
            assert_eq!(logbf(8.0), 3.0);
            assert_eq!(lrint(2.0), 2);
            assert_eq!(lrintf(2.0), 2);
            assert_eq!(lround(2.4), 2);
            assert_eq!(lroundf(2.4), 2);
            assert_eq!(nearbyint(2.0), 2.0);
            assert_eq!(nearbyintf(2.0), 2.0);
            assert!(nextafter(1.0, 2.0) > 1.0);
            assert!(nextafterf(1.0, 2.0) > 1.0);
            assert_eq!(pow(2.0, 3.0), 8.0);
            assert_eq!(powf(2.0, 3.0), 8.0);
            assert_eq!(remainder(5.0, 2.0), 1.0);
            assert_eq!(remainderf(5.0, 2.0), 1.0);
            let mut quo = 0;
            assert_eq!(remquo(5.0, 2.0, &mut quo), 1.0);
            assert_eq!(quo & 0x7, 2);
            let mut quo_f = 0;
            assert_eq!(remquof(5.0, 2.0, &mut quo_f), 1.0);
            assert_eq!(quo_f & 0x7, 2);
            assert_eq!(rint(2.0), 2.0);
            assert_eq!(rintf(2.0), 2.0);
            assert_eq!(round(1.5), 2.0);
            assert_eq!(roundf(1.5), 2.0);
            assert_eq!(scalbln(0.5, 4), 8.0);
            assert_eq!(scalblnf(0.5, 4), 8.0);
            assert_eq!(scalbn(0.5, 4), 8.0);
            assert_eq!(scalbnf(0.5, 4), 8.0);
            assert!(sinh(0.0).abs() < 1e-12);
            assert!(sinhf(0.0).abs() < 1e-6);
            assert_eq!(sqrt(9.0), 3.0);
            assert_eq!(sqrtf(9.0), 3.0);
            assert!(tan(0.0).abs() < 1e-12);
            assert!(tanf(0.0).abs() < 1e-6);
            assert!(tanh(0.0).abs() < 1e-12);
            assert!(tanhf(0.0).abs() < 1e-6);
            assert_eq!(tgamma(5.0), 24.0);
            assert_eq!(tgammaf(5.0), 24.0);
            assert_eq!(trunc(-1.75), -1.0);
            assert_eq!(truncf(-1.75), -1.0);
            assert!((yn(0, 1.0) - y0(1.0)).abs() < 1e-12);
            assert!((ynf(0, 1.0) - y0f(1.0)).abs() < 1e-6);
            assert!((yn(1, 1.0) - y1(1.0)).abs() < 1e-12);
            assert!((ynf(1, 1.0) - y1f(1.0)).abs() < 1e-6);
        }
    }

    #[test]
    fn scalar_libc_forwards_match_expected_values() {
        unsafe {
            assert_eq!(abs(-7), 7);
            assert_eq!(atoi(c"42".as_ptr()), 42);
            assert_eq!(atol(c"42".as_ptr()), 42);
            assert_eq!(atoll(c"42".as_ptr()), 42);

            let left = [1_u8, 2, 3, 4];
            let right = [1_u8, 2, 3, 5];
            let mut copied = [0_u8; 4];
            assert_eq!(
                bcmp(
                    left.as_ptr().cast(),
                    left.as_ptr().cast(),
                    left.len() as c_uint
                ),
                0
            );
            assert_ne!(
                bcmp(
                    left.as_ptr().cast(),
                    right.as_ptr().cast(),
                    left.len() as c_uint
                ),
                0
            );
            bcopy(
                left.as_ptr().cast(),
                copied.as_mut_ptr().cast(),
                left.len() as c_uint,
            );
            assert_eq!(copied, left);

            let sorted = [1_i32, 3, 7, 9];
            let key = 7_i32;
            let found = bsearch(
                (&key as *const i32).cast(),
                sorted.as_ptr().cast(),
                sorted.len(),
                core::mem::size_of::<i32>(),
                Some(compare_i32),
            )
            .cast::<i32>();
            assert!(!found.is_null());
            assert_eq!(*found, 7);

            assert_eq!(btowc(b'A' as c_int), b'A' as wint_t);
            let _ = clock();
            assert_eq!(difftime(10, 7), 3.0);

            let quotient = div(7, 3);
            assert_eq!(quotient.quot, 2);
            assert_eq!(quotient.rem, 1);
            assert_eq!(ffs(0b0100_0000), 7);
            assert_eq!(ffsl(1 << 12), 13);
            assert_eq!(ffsll(1 << 20), 21);
            assert_eq!(fnmatch(c"a*".as_ptr(), c"abc".as_ptr(), 0), 0);

            let mut random = [0_u8; 16];
            assert_eq!(getentropy(random.as_mut_ptr().cast(), random.len()), 0);
            assert_eq!(imaxabs(-9), 9);
            let intmax_div = imaxdiv(9, 4);
            assert_eq!(intmax_div.quot, 2);
            assert_eq!(intmax_div.rem, 1);

            let indexed = index(c"badge".as_ptr(), b'd' as c_int);
            assert_eq!(CStr::from_ptr(indexed).to_bytes(), b"dge");
            assert_eq!(a64l(l64a(12_345)), 12_345);
        }
    }

    #[test]
    fn locale_ignoring_host_forwards_match_expected_values() {
        unsafe {
            assert_ne!(iswalnum_l('A' as wint_t, 77), 0);
            assert_ne!(iswspace_l(' ' as wint_t, 77), 0);
            assert_eq!(towlower_l('Q' as wint_t, 77), 'q' as wint_t);
            assert_eq!(towupper_l('q' as wint_t, 77), 'Q' as wint_t);

            let alpha = wctype_l(c"alpha".as_ptr(), 77);
            assert_ne!(alpha, 0);
            assert_ne!(iswctype_l('A' as wint_t, alpha, 77), 0);
            assert_eq!(strcasecmp_l(c"Badge".as_ptr(), c"badge".as_ptr(), 77), 0);
            assert_eq!(strncasecmp_l(c"Badge".as_ptr(), c"ba".as_ptr(), 2, 77), 0);
            assert_eq!(strcoll_l(c"badge".as_ptr(), c"badge".as_ptr(), 77), 0);

            let mut end = core::ptr::null_mut();
            assert_eq!(strtod_l(c"3.5".as_ptr(), &mut end, 77), 3.5);
            assert_eq!(strtof_l(c"3.5".as_ptr(), &mut end, 77), 3.5);
            assert_eq!(strtol_l(c"42".as_ptr(), &mut end, 10, 77), 42);
            assert_eq!(strtoll_l(c"42".as_ptr(), &mut end, 10, 77), 42);
            assert_eq!(strtoul_l(c"42".as_ptr(), &mut end, 10, 77), 42);
            assert_eq!(strtoull_l(c"42".as_ptr(), &mut end, 10, 77), 42);

            let mut transformed = [0 as c_char; 16];
            assert!(
                strxfrm_l(
                    transformed.as_mut_ptr(),
                    c"badge".as_ptr(),
                    transformed.len() as c_uint,
                    77
                ) > 0
            );
            let item = nl_langinfo_l(libc::CODESET as nl_item, 77);
            assert!(!item.is_null());

            let wide_badge = [
                'B' as wchar_t,
                'a' as wchar_t,
                'd' as wchar_t,
                'g' as wchar_t,
                'e' as wchar_t,
                0,
            ];
            let wide_badge_lower = [
                'b' as wchar_t,
                'a' as wchar_t,
                'd' as wchar_t,
                'g' as wchar_t,
                'e' as wchar_t,
                0,
            ];
            let wide_digits = ['4' as wchar_t, '2' as wchar_t, 0];
            let mut wide_end = core::ptr::null_mut();
            assert_eq!(
                wcscasecmp_l(wide_badge.as_ptr(), wide_badge_lower.as_ptr(), 77),
                0
            );
            assert_eq!(wcscoll_l(wide_badge.as_ptr(), wide_badge.as_ptr(), 77), 0);
            assert_eq!(
                wcsncasecmp_l(wide_badge.as_ptr(), wide_badge_lower.as_ptr(), 5, 77),
                0
            );
            assert_eq!(wcstod_l(wide_digits.as_ptr(), &mut wide_end, 77), 42.0);
            assert_eq!(wcstof_l(wide_digits.as_ptr(), &mut wide_end, 77), 42.0);
            assert_eq!(wcstol_l(wide_digits.as_ptr(), &mut wide_end, 10, 77), 42);
            assert_eq!(wcstoll_l(wide_digits.as_ptr(), &mut wide_end, 10, 77), 42);
            assert_eq!(wcstoul_l(wide_digits.as_ptr(), &mut wide_end, 10, 77), 42);
            assert_eq!(wcstoull_l(wide_digits.as_ptr(), &mut wide_end, 10, 77), 42);

            let mut wide_transformed = [0 as wchar_t; 16];
            assert!(
                wcsxfrm_l(
                    wide_transformed.as_mut_ptr(),
                    wide_badge.as_ptr(),
                    wide_transformed.len(),
                    77
                ) > 0
            );
            assert_ne!(wctrans_l(c"tolower".as_ptr(), 77), 0);
        }
    }

    #[test]
    fn ctype_host_forwards_match_expected_values() {
        unsafe {
            assert_ne!(isalnum('A' as c_int), 0);
            assert_ne!(isalpha('A' as c_int), 0);
            assert_ne!(isascii(0x7f), 0);
            assert_ne!(isblank(' ' as c_int), 0);
            assert_ne!(iscntrl('\n' as c_int), 0);
            assert_ne!(isdigit('7' as c_int), 0);
            assert_ne!(isgraph('!' as c_int), 0);
            assert_ne!(islower('q' as c_int), 0);
            assert_ne!(isprint('~' as c_int), 0);
            assert_ne!(ispunct('!' as c_int), 0);
            assert_ne!(isspace(' ' as c_int), 0);
            assert_ne!(isupper('Q' as c_int), 0);
            assert_ne!(isxdigit('f' as c_int), 0);

            assert_ne!(iswalnum('A' as wint_t), 0);
            assert_ne!(iswalpha('A' as wint_t), 0);
            assert_ne!(iswblank(' ' as wint_t), 0);
            assert_ne!(iswcntrl('\n' as wint_t), 0);
            assert_ne!(iswdigit('7' as wint_t), 0);
            assert_ne!(iswgraph('!' as wint_t), 0);
            assert_ne!(iswlower('q' as wint_t), 0);
            assert_ne!(iswprint('~' as wint_t), 0);
            assert_ne!(iswpunct('!' as wint_t), 0);
            assert_ne!(iswspace(' ' as wint_t), 0);
            assert_ne!(iswupper('Q' as wint_t), 0);
            assert_ne!(iswxdigit('f' as wint_t), 0);
        }
    }

    #[test]
    fn scalar_string_and_time_host_forwards_match_expected_values() {
        unsafe {
            let host_rand_r: unsafe extern "C" fn(*mut c_uint) -> c_int =
                runtime::resolve_next_function(b"rand_r\0");
            let host_time: unsafe extern "C" fn(*mut time_t) -> time_t =
                runtime::resolve_next_function(b"time\0");

            assert_eq!(atof(c"3.5".as_ptr()), 3.5);
            assert_eq!(labs(-42), 42);
            let long_div = ldiv(17, 5);
            assert_eq!(long_div.quot, 3);
            assert_eq!(long_div.rem, 2);
            assert_eq!(llabs(-99), 99);
            let longlong_div = lldiv(19, 4);
            assert_eq!(longlong_div.quot, 4);
            assert_eq!(longlong_div.rem, 3);

            let mut ascending = [4_i32, 1, 3, 2];
            qsort(
                ascending.as_mut_ptr().cast(),
                ascending.len(),
                core::mem::size_of::<i32>(),
                Some(compare_i32),
            );
            assert_eq!(ascending, [1, 2, 3, 4]);

            let mut descending = [4_i32, 1, 3, 2];
            let direction = -1;
            qsort_r(
                descending.as_mut_ptr().cast(),
                descending.len(),
                core::mem::size_of::<i32>(),
                Some(compare_i32_with_direction),
                (&direction as *const c_int).cast_mut().cast(),
            );
            assert_eq!(descending, [4, 3, 2, 1]);

            let mut seed = 123_u32;
            let mut host_seed = 123_u32;
            assert_eq!(rand_r(&mut seed), host_rand_r(&mut host_seed));
            assert_eq!(seed, host_seed);

            let suffix = rindex(c"badge".as_ptr(), b'd' as c_int);
            assert_eq!(CStr::from_ptr(suffix).to_bytes(), b"dge");

            let mut copied = [0 as c_char; 16];
            let copied_end = stpcpy(copied.as_mut_ptr(), c"badge".as_ptr());
            assert_eq!(copied_end, copied.as_mut_ptr().add(5));
            assert_eq!(CStr::from_ptr(copied.as_ptr()).to_bytes(), b"badge");

            let mut copied_n = [1 as c_char; 8];
            let copied_n_end = stpncpy(copied_n.as_mut_ptr(), c"bad".as_ptr(), 6);
            assert_eq!(copied_n_end, copied_n.as_mut_ptr().add(3));
            assert_eq!(
                &copied_n[..6],
                &[b'b' as c_char, b'a' as c_char, b'd' as c_char, 0, 0, 0]
            );

            assert_eq!(strcasecmp(c"Badge".as_ptr(), c"badge".as_ptr()), 0);
            assert_eq!(strncasecmp(c"Badge".as_ptr(), c"ba".as_ptr(), 2), 0);
            assert_eq!(strcoll(c"badge".as_ptr(), c"badge".as_ptr()), 0);

            let mut end = core::ptr::null_mut();
            assert_eq!(strtod(c"12.5tail".as_ptr(), &mut end), 12.5);
            assert_eq!(CStr::from_ptr(end).to_bytes(), b"tail");
            assert_eq!(strtof(c"7.25".as_ptr(), &mut end), 7.25);
            assert_eq!(strtoimax(c"-123".as_ptr(), &mut end, 10), -123);
            assert_eq!(strtol(c"2a".as_ptr(), &mut end, 16), 42);
            assert_eq!(strtoll(c"-42".as_ptr(), &mut end, 10), -42);
            assert_eq!(strtoul(c"52".as_ptr(), &mut end, 10), 52);
            assert_eq!(strtoull(c"64".as_ptr(), &mut end, 10), 64);
            assert_eq!(strtoumax(c"255".as_ptr(), &mut end, 10), 255);

            let mut transformed = [0 as c_char; 16];
            assert!(
                strxfrm(
                    transformed.as_mut_ptr(),
                    c"badge".as_ptr(),
                    transformed.len() as c_uint
                ) > 0
            );

            let input = [0x01_u8, 0x02, 0x03, 0x04];
            let mut output = [0_u8; 4];
            swab(
                input.as_ptr().cast(),
                output.as_mut_ptr().cast(),
                input.len() as isize,
            );
            assert_eq!(output, [0x02, 0x01, 0x04, 0x03]);

            let mut badge_time = 0;
            let badge_now = time(&mut badge_time);
            let mut host_time_value = 0;
            let host_now = host_time(&mut host_time_value);
            assert_eq!(badge_now, badge_time);
            assert_eq!(host_now, host_time_value);
            assert!(badge_now >= host_now - 1);
            assert!(badge_now <= host_now + 1);

            assert_eq!(toascii(0xff), 0x7f);
            assert_eq!(tolower('Q' as c_int), 'q' as c_int);
            assert_eq!(toupper('q' as c_int), 'Q' as c_int);
            assert_eq!(towlower('Q' as wint_t), 'q' as wint_t);
            assert_eq!(towupper('q' as wint_t), 'Q' as wint_t);
            assert_eq!(sleep(0), 0);
            assert_eq!(usleep(0), 0);

            let codeset = nl_langinfo(libc::CODESET as nl_item);
            assert!(!codeset.is_null());
            assert!(!CStr::from_ptr(codeset).to_bytes().is_empty());

            let wide = [
                'b' as wchar_t,
                'a' as wchar_t,
                'd' as wchar_t,
                'g' as wchar_t,
                'e' as wchar_t,
                0,
            ];
            let duplicated = wcsdup(wide.as_ptr());
            assert!(!duplicated.is_null());
            assert_eq!(core::slice::from_raw_parts(duplicated, wide.len()), wide);
            free(duplicated.cast());
        }
    }

    #[test]
    fn wide_char_host_forwards_match_expected_values() {
        unsafe {
            let wide_badge = [
                'B' as wchar_t,
                'a' as wchar_t,
                'd' as wchar_t,
                'g' as wchar_t,
                'e' as wchar_t,
                0,
            ];
            let wide_badge_lower = [
                'b' as wchar_t,
                'a' as wchar_t,
                'd' as wchar_t,
                'g' as wchar_t,
                'e' as wchar_t,
                0,
            ];
            let wide_digits = ['4' as wchar_t, '2' as wchar_t, 'x' as wchar_t, 0];

            let mut copied = [0 as wchar_t; 16];
            let wcpcpy_end = wcpcpy(copied.as_mut_ptr(), wide_badge.as_ptr());
            assert_eq!(wcpcpy_end, copied.as_mut_ptr().add(5));
            assert_eq!(&copied[..6], &wide_badge);

            let mut copied_n = [1 as wchar_t; 8];
            let wcpncpy_end = wcpncpy(copied_n.as_mut_ptr(), wide_badge.as_ptr(), 6);
            assert_eq!(wcpncpy_end, copied_n.as_mut_ptr().add(5));
            assert_eq!(&copied_n[..6], &wide_badge);

            assert_eq!(
                wcscasecmp(wide_badge.as_ptr(), wide_badge_lower.as_ptr()),
                0
            );
            assert_eq!(wcscmp(wide_badge.as_ptr(), wide_badge.as_ptr()), 0);
            assert_eq!(wcsncmp(wide_badge.as_ptr(), wide_badge.as_ptr(), 1), 0);
            assert_eq!(wcscoll(wide_badge.as_ptr(), wide_badge.as_ptr()), 0);
            assert_eq!(wcslen(wide_badge.as_ptr()), 5);
            assert_eq!(wcsnlen(wide_badge.as_ptr(), 3), 3);

            let mut concatenated = ['b' as wchar_t, 'a' as wchar_t, 0, 0, 0, 0, 0, 0];
            wcscat(
                concatenated.as_mut_ptr(),
                ['d' as wchar_t, 'g' as wchar_t, 'e' as wchar_t, 0].as_ptr(),
            );
            assert_eq!(
                &concatenated[..6],
                &[
                    'b' as wchar_t,
                    'a' as wchar_t,
                    'd' as wchar_t,
                    'g' as wchar_t,
                    'e' as wchar_t,
                    0
                ]
            );

            let mut copied_again = [0 as wchar_t; 8];
            wcscpy(copied_again.as_mut_ptr(), wide_badge.as_ptr());
            assert_eq!(&copied_again[..6], &wide_badge);

            let mut copied_limited = [0 as wchar_t; 8];
            wcsncpy(copied_limited.as_mut_ptr(), wide_badge.as_ptr(), 6);
            assert_eq!(&copied_limited[..6], &wide_badge);

            let mut ncat = ['b' as wchar_t, 'a' as wchar_t, 0, 0, 0, 0, 0, 0];
            wcsncat(
                ncat.as_mut_ptr(),
                ['d' as wchar_t, 'g' as wchar_t, 'e' as wchar_t, 0].as_ptr(),
                2,
            );
            assert_eq!(
                &ncat[..5],
                &[
                    'b' as wchar_t,
                    'a' as wchar_t,
                    'd' as wchar_t,
                    'g' as wchar_t,
                    0
                ]
            );

            assert_eq!(
                wcscspn(wide_badge.as_ptr(), ['g' as wchar_t, 0].as_ptr()),
                3
            );
            assert_eq!(
                wcsspn(
                    ['b' as wchar_t, 'a' as wchar_t, 'a' as wchar_t, 0].as_ptr(),
                    ['b' as wchar_t, 'a' as wchar_t, 0].as_ptr()
                ),
                3
            );
            assert_eq!(*wcschr(wide_badge.as_ptr(), 'd' as wchar_t), 'd' as wchar_t);
            assert_eq!(
                *wcsrchr(
                    [
                        'b' as wchar_t,
                        'a' as wchar_t,
                        'd' as wchar_t,
                        'g' as wchar_t,
                        'd' as wchar_t,
                        0
                    ]
                    .as_ptr(),
                    'd' as wchar_t
                ),
                'd' as wchar_t
            );
            assert_eq!(
                *wcspbrk(
                    wide_badge.as_ptr(),
                    ['x' as wchar_t, 'd' as wchar_t, 0].as_ptr()
                ),
                'd' as wchar_t
            );
            assert_eq!(
                *wcsstr(
                    wide_badge.as_ptr(),
                    ['d' as wchar_t, 'g' as wchar_t, 0].as_ptr()
                ),
                'd' as wchar_t
            );
            assert_eq!(
                wcsncasecmp(wide_badge.as_ptr(), wide_badge_lower.as_ptr(), 5),
                0
            );

            let mut wide_end = core::ptr::null_mut();
            assert_eq!(wcstod(wide_digits.as_ptr(), &mut wide_end), 42.0);
            assert_eq!(*wide_end, 'x' as wchar_t);
            assert_eq!(wcstof(wide_digits.as_ptr(), &mut wide_end), 42.0);
            assert_eq!(wcstoimax(wide_digits.as_ptr(), &mut wide_end, 10), 42);
            assert_eq!(wcstol(wide_digits.as_ptr(), &mut wide_end, 10), 42);
            assert_eq!(wcstoll(wide_digits.as_ptr(), &mut wide_end, 10), 42);
            assert_eq!(wcstoul(wide_digits.as_ptr(), &mut wide_end, 10), 42);
            assert_eq!(wcstoull(wide_digits.as_ptr(), &mut wide_end, 10), 42);
            assert_eq!(wcstoumax(wide_digits.as_ptr(), &mut wide_end, 10), 42);

            let mut transformed = [0 as wchar_t; 16];
            assert!(
                wcsxfrm(
                    transformed.as_mut_ptr(),
                    wide_badge.as_ptr(),
                    transformed.len()
                ) > 0
            );

            let mut tokenized = ['a' as wchar_t, ',' as wchar_t, 'b' as wchar_t, 0];
            let mut save_ptr = core::ptr::null_mut();
            let token1 = wcstok(
                tokenized.as_mut_ptr(),
                [',' as wchar_t, 0].as_ptr(),
                &mut save_ptr,
            );
            let token2 = wcstok(
                core::ptr::null_mut(),
                [',' as wchar_t, 0].as_ptr(),
                &mut save_ptr,
            );
            assert_eq!(*token1, 'a' as wchar_t);
            assert_eq!(*token2, 'b' as wchar_t);

            let mut multibyte = [0 as c_char; 16];
            assert_eq!(
                wcstombs(
                    multibyte.as_mut_ptr(),
                    wide_badge_lower.as_ptr(),
                    multibyte.len()
                ),
                5
            );
            assert_eq!(CStr::from_ptr(multibyte.as_ptr()).to_bytes(), b"badge");
            assert_eq!(wctob('A' as wint_t), 'A' as c_int);

            let mut one_char = [0 as c_char; 8];
            assert_eq!(wctomb(one_char.as_mut_ptr(), 'A' as wchar_t), 1);
            assert_eq!(one_char[0], 'A' as c_char);

            assert_eq!(wcwidth('A' as wchar_t), 1);
            assert_eq!(wcswidth(wide_badge_lower.as_ptr(), 5), 5);

            let source = [
                'b' as wchar_t,
                'a' as wchar_t,
                'd' as wchar_t,
                'g' as wchar_t,
                'e' as wchar_t,
            ];
            let mut wide_mem = [0 as wchar_t; 8];
            wmemcpy(wide_mem.as_mut_ptr(), source.as_ptr(), source.len());
            assert_eq!(&wide_mem[..5], &source);
            assert_eq!(wmemcmp(wide_mem.as_ptr(), source.as_ptr(), source.len()), 0);
            assert_eq!(
                *wmemchr(wide_mem.as_ptr(), 'd' as wchar_t, source.len()),
                'd' as wchar_t
            );
            assert_eq!(
                wmempcpy(wide_mem.as_mut_ptr(), source.as_ptr(), 2),
                wide_mem.as_mut_ptr().add(2)
            );
            wmemset(wide_mem.as_mut_ptr().add(5), '!' as wchar_t, 2);
            assert_eq!(wide_mem[5], '!' as wchar_t);
            assert_eq!(wide_mem[6], '!' as wchar_t);

            let mut overlapping = [
                'a' as wchar_t,
                'b' as wchar_t,
                'c' as wchar_t,
                'd' as wchar_t,
                0,
            ];
            wmemmove(overlapping.as_mut_ptr().add(1), overlapping.as_ptr(), 3);
            assert_eq!(
                &overlapping[..4],
                &[
                    'a' as wchar_t,
                    'a' as wchar_t,
                    'b' as wchar_t,
                    'c' as wchar_t
                ]
            );
        }
    }

    #[test]
    fn multibyte_host_forwards_match_expected_values() {
        unsafe {
            assert_eq!(mblen(c"A".as_ptr(), 1), 1);

            let mut state = core::mem::zeroed::<mbstate_t>();
            assert_ne!(mbsinit(&state), 0);
            assert_eq!(mbrlen(c"A".as_ptr(), 1, &mut state), 1);

            let mut wide = 0 as wchar_t;
            let mut state = core::mem::zeroed::<mbstate_t>();
            assert_eq!(mbrtowc(&mut wide, c"A".as_ptr(), 1, &mut state), 1);
            assert_eq!(wide, 'A' as wchar_t);
            assert_ne!(mbsinit(&state), 0);

            let mut wide_simple = 0 as wchar_t;
            assert_eq!(mbtowc(&mut wide_simple, c"A".as_ptr(), 1), 1);
            assert_eq!(wide_simple, 'A' as wchar_t);

            let mut wide_buf = [0 as wchar_t; 8];
            assert_eq!(
                mbstowcs(wide_buf.as_mut_ptr(), c"badge".as_ptr(), wide_buf.len()),
                5
            );
            assert_eq!(
                &wide_buf[..5],
                &[
                    'b' as wchar_t,
                    'a' as wchar_t,
                    'd' as wchar_t,
                    'g' as wchar_t,
                    'e' as wchar_t
                ]
            );

            let mut src = c"badge".as_ptr();
            let mut wide_from_src = [0 as wchar_t; 8];
            let mut state = core::mem::zeroed::<mbstate_t>();
            assert_eq!(
                mbsrtowcs(
                    wide_from_src.as_mut_ptr(),
                    &mut src,
                    wide_from_src.len(),
                    &mut state
                ),
                5
            );
            assert!(src.is_null());
            assert_eq!(
                &wide_from_src[..5],
                &[
                    'b' as wchar_t,
                    'a' as wchar_t,
                    'd' as wchar_t,
                    'g' as wchar_t,
                    'e' as wchar_t
                ]
            );

            let mut src = c"badge".as_ptr();
            let mut wide_limited = [0 as wchar_t; 8];
            let mut state = core::mem::zeroed::<mbstate_t>();
            assert_eq!(
                mbsnrtowcs(
                    wide_limited.as_mut_ptr(),
                    &mut src,
                    3,
                    wide_limited.len(),
                    &mut state
                ),
                3
            );
            assert_eq!(
                &wide_limited[..3],
                &['b' as wchar_t, 'a' as wchar_t, 'd' as wchar_t]
            );
            assert_eq!(*src, 'g' as c_char);

            let mut out = [0 as c_char; 8];
            let mut state = core::mem::zeroed::<mbstate_t>();
            assert_eq!(wcrtomb(out.as_mut_ptr(), 'A' as wchar_t, &mut state), 1);
            assert_eq!(out[0], 'A' as c_char);

            let wide_badge = [
                'b' as wchar_t,
                'a' as wchar_t,
                'd' as wchar_t,
                'g' as wchar_t,
                'e' as wchar_t,
                0,
            ];
            let mut src = wide_badge.as_ptr();
            let mut multibyte = [0 as c_char; 16];
            let mut state = core::mem::zeroed::<mbstate_t>();
            assert_eq!(
                wcsrtombs(
                    multibyte.as_mut_ptr(),
                    &mut src,
                    multibyte.len(),
                    &mut state
                ),
                5
            );
            assert!(src.is_null());
            assert_eq!(CStr::from_ptr(multibyte.as_ptr()).to_bytes(), b"badge");

            let mut src = wide_badge.as_ptr();
            let mut limited = [0 as c_char; 16];
            let mut state = core::mem::zeroed::<mbstate_t>();
            assert_eq!(
                wcsnrtombs(limited.as_mut_ptr(), &mut src, 3, limited.len(), &mut state),
                3
            );
            assert_eq!(
                &limited[..3],
                &[b'b' as c_char, b'a' as c_char, b'd' as c_char]
            );
            assert_eq!(*src, 'g' as wchar_t);
        }
    }

    #[test]
    fn wide_time_and_misc_host_forwards_match_expected_values() {
        unsafe {
            let host_rpmatch: unsafe extern "C" fn(*const c_char) -> c_int =
                runtime::resolve_next_function(b"rpmatch\0");
            let host_gcvt: unsafe extern "C" fn(f64, c_int, *mut c_char) -> *mut c_char =
                runtime::resolve_next_function(b"gcvt\0");
            let host_wcsftime: unsafe extern "C" fn(
                *mut wchar_t,
                usize,
                *const wchar_t,
                *const libc::tm,
            ) -> usize = runtime::resolve_next_function(b"wcsftime\0");

            let mut integer = 0.0;
            assert_eq!(modf(3.25, &mut integer), 0.25);
            assert_eq!(integer, 3.0);
            let mut integer_f = 0.0_f32;
            assert_eq!(modff(3.25, &mut integer_f), 0.25);
            assert_eq!(integer_f, 3.0);
            assert!(nan(c"".as_ptr()).is_nan());
            assert!(nanf(c"".as_ptr()).is_nan());

            assert_eq!(rpmatch(c"y".as_ptr()), host_rpmatch(c"y".as_ptr()));
            assert_eq!(rpmatch(c"n".as_ptr()), host_rpmatch(c"n".as_ptr()));

            let mut gcvt_buf = [0 as c_char; 32];
            let mut host_gcvt_buf = [0 as c_char; 32];
            assert_eq!(
                CStr::from_ptr(gcvt(3.25, 6, gcvt_buf.as_mut_ptr())).to_bytes(),
                CStr::from_ptr(host_gcvt(3.25, 6, host_gcvt_buf.as_mut_ptr())).to_bytes(),
            );

            let badge_tm = tm {
                tm_sec: 56,
                tm_min: 34,
                tm_hour: 12,
                tm_mday: 7,
                tm_mon: 5,
                tm_year: 124,
                tm_wday: 5,
                tm_yday: 158,
                tm_isdst: 0,
            };
            let host_tm = badge_tm_to_host(&badge_tm);
            let format = [
                '%' as wchar_t,
                'Y' as wchar_t,
                '-' as wchar_t,
                '%' as wchar_t,
                'm' as wchar_t,
                '-' as wchar_t,
                '%' as wchar_t,
                'd' as wchar_t,
                0,
            ];
            let mut badge_output = [0 as wchar_t; 32];
            let mut host_output = [0 as wchar_t; 32];

            assert_eq!(
                wcsftime(
                    badge_output.as_mut_ptr(),
                    badge_output.len(),
                    format.as_ptr(),
                    &badge_tm
                ),
                host_wcsftime(
                    host_output.as_mut_ptr(),
                    host_output.len(),
                    format.as_ptr(),
                    &host_tm
                ),
            );
            assert_eq!(badge_output, host_output);

            let mut locale_output = [0 as wchar_t; 32];
            assert_eq!(
                wcsftime_l(
                    locale_output.as_mut_ptr(),
                    locale_output.len(),
                    format.as_ptr(),
                    &badge_tm,
                    77
                ),
                host_wcsftime(
                    host_output.as_mut_ptr(),
                    host_output.len(),
                    format.as_ptr(),
                    &host_tm
                ),
            );
            assert_eq!(locale_output, host_output);
        }
    }

    #[test]
    fn stdio_and_file_control_host_forwards_match_expected_values() {
        unsafe {
            let orientation_stream = libc::tmpfile().cast::<FILE>();
            assert!(!orientation_stream.is_null());
            assert!(fwide(orientation_stream, -1) < 0);
            fclose(orientation_stream);

            let setvbuf_stream = libc::tmpfile().cast::<FILE>();
            assert!(!setvbuf_stream.is_null());
            let mut full_buffer = [0 as c_char; 64];
            assert_eq!(
                setvbuf(
                    setvbuf_stream,
                    full_buffer.as_mut_ptr(),
                    libc::_IOFBF,
                    full_buffer.len()
                ),
                0
            );
            fclose(setvbuf_stream);

            let setbuf_stream = libc::tmpfile().cast::<FILE>();
            assert!(!setbuf_stream.is_null());
            setbuf(setbuf_stream, core::ptr::null_mut());
            fclose(setbuf_stream);

            let setbuffer_stream = libc::tmpfile().cast::<FILE>();
            assert!(!setbuffer_stream.is_null());
            let mut line_buffer = [0 as c_char; 64];
            setbuffer(
                setbuffer_stream,
                line_buffer.as_mut_ptr(),
                line_buffer.len(),
            );
            fclose(setbuffer_stream);

            let setlinebuf_stream = libc::tmpfile().cast::<FILE>();
            assert!(!setlinebuf_stream.is_null());
            setlinebuf(setlinebuf_stream);
            fclose(setlinebuf_stream);

            let io_stream = libc::tmpfile().cast::<FILE>();
            assert!(!io_stream.is_null());
            assert!(fputs(c"bc".as_ptr(), io_stream) >= 0);
            fflush(io_stream);
            rewind(io_stream);
            assert_eq!(fgetc(io_stream), 'b' as c_int);
            assert_eq!(ungetc('a' as c_int, io_stream), 'a' as c_int);
            assert_eq!(fgetc(io_stream), 'a' as c_int);
            assert_eq!(fgetc(io_stream), 'c' as c_int);
            fclose(io_stream);

            let locale = localeconv();
            assert!(!locale.is_null());
            assert!(!(*locale).decimal_point.is_null());
            assert!(
                !CStr::from_ptr((*locale).decimal_point)
                    .to_bytes()
                    .is_empty()
            );

            assert_eq!(system(c"true".as_ptr()), 0);

            let temp_root = std::env::temp_dir();
            let stem = format!(
                "why2025-badge-emu-abi-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("unix time")
                    .as_nanos()
            );
            let source_path = temp_root.join(format!("{stem}-source"));
            let link_path = temp_root.join(format!("{stem}-link"));
            std::fs::write(&source_path, b"badge").expect("write source file");
            let source_c =
                CString::new(source_path.to_string_lossy().as_bytes()).expect("source c string");
            let link_c =
                CString::new(link_path.to_string_lossy().as_bytes()).expect("link c string");

            assert_eq!(link(source_c.as_ptr(), link_c.as_ptr()), 0);
            assert_eq!(std::fs::read(&link_path).expect("read link file"), b"badge");
            assert_eq!(unlink(link_c.as_ptr()), 0);
            assert_eq!(unlink(source_c.as_ptr()), 0);
        }
    }

    #[test]
    fn tree_and_process_time_host_forwards_match_expected_values() {
        unsafe {
            let host_times: unsafe extern "C" fn(*mut libc::tms) -> clock_t =
                runtime::resolve_next_function(b"times\0");

            let mut host = core::mem::zeroed::<libc::tms>();
            let host_ticks = host_times(&mut host);
            let mut badge = core::mem::zeroed::<tms>();
            let badge_ticks = times(&mut badge);
            assert_ne!(host_ticks as i64, -1);
            assert_ne!(badge_ticks, clock_t::MAX);
            let tick_slack = libc::sysconf(libc::_SC_CLK_TCK).max(1) as i128;
            let host_ticks = host_ticks as i128;
            let badge_ticks = badge_ticks as i128;
            assert!(badge_ticks >= host_ticks);
            assert!(badge_ticks - host_ticks <= tick_slack);
            let host_utime = host.tms_utime as i128;
            let badge_utime = badge.tms_utime as i128;
            assert!(badge_utime >= host_utime);
            assert!(badge_utime - host_utime <= tick_slack);

            let mut root = core::ptr::null_mut::<c_void>();
            let one = Box::into_raw(Box::new(1_i32));
            let three = Box::into_raw(Box::new(3_i32));
            let five = Box::into_raw(Box::new(5_i32));

            assert!(!tsearch(one.cast(), &mut root, Some(compare_i32)).is_null());
            assert!(!tsearch(three.cast(), &mut root, Some(compare_i32)).is_null());
            assert!(!tsearch(five.cast(), &mut root, Some(compare_i32)).is_null());

            let lookup = 3_i32;
            assert!(!tfind((&lookup as *const i32).cast(), &mut root, Some(compare_i32)).is_null());

            TWALK_VISITS.store(0, Ordering::Relaxed);
            twalk(root.cast_const(), Some(record_twalk_visit));
            assert!(TWALK_VISITS.load(Ordering::Relaxed) > 0);

            assert!(
                !tdelete((&lookup as *const i32).cast(), &mut root, Some(compare_i32)).is_null()
            );
            drop(Box::from_raw(three));
            assert!(tfind((&lookup as *const i32).cast(), &mut root, Some(compare_i32)).is_null());

            tdestroy(root, Some(free_i32));
        }
    }

    #[test]
    fn getopt_host_forwards_match_expected_values() {
        unsafe {
            optind = 1;
            opterr = 0;

            let program = CString::new("prog").expect("program arg");
            let short_arg = CString::new("-a").expect("short arg");
            let short_argv = [
                program.as_ptr().cast_mut(),
                short_arg.as_ptr().cast_mut(),
                core::ptr::null_mut(),
            ];
            assert_eq!(
                getopt(2, short_argv.as_ptr().cast(), c"a".as_ptr()),
                'a' as c_int,
            );
            assert_eq!(getopt(2, short_argv.as_ptr().cast(), c"a".as_ptr()), -1);

            optind = 1;
            opterr = 0;
            let long_arg = CString::new("--beta").expect("long arg");
            let long_value = CString::new("value").expect("long value");
            let long_argv = [
                program.as_ptr().cast_mut(),
                long_arg.as_ptr().cast_mut(),
                long_value.as_ptr().cast_mut(),
                core::ptr::null_mut(),
            ];
            let long_options = [
                option {
                    name: c"beta".as_ptr(),
                    has_arg: 1,
                    flag: core::ptr::null_mut(),
                    val: 'b' as c_int,
                },
                core::mem::zeroed(),
            ];
            let mut long_index = -1;
            assert_eq!(
                getopt_long(
                    3,
                    long_argv.as_ptr().cast(),
                    c"b:".as_ptr(),
                    long_options.as_ptr(),
                    &mut long_index,
                ),
                'b' as c_int,
            );
            assert_eq!(long_index, 0);
            assert_eq!(CStr::from_ptr(optarg).to_bytes(), b"value");

            optind = 1;
            opterr = 0;
            let long_only_arg = CString::new("-beta").expect("long-only arg");
            let long_only_argv = [
                program.as_ptr().cast_mut(),
                long_only_arg.as_ptr().cast_mut(),
                long_value.as_ptr().cast_mut(),
                core::ptr::null_mut(),
            ];
            let mut long_only_index = -1;
            assert_eq!(
                getopt_long_only(
                    3,
                    long_only_argv.as_ptr().cast(),
                    c"b:".as_ptr(),
                    long_options.as_ptr(),
                    &mut long_only_index,
                ),
                'b' as c_int,
            );
            assert_eq!(long_only_index, 0);
            assert_eq!(CStr::from_ptr(optarg).to_bytes(), b"value");
        }
    }

    #[test]
    fn translated_time_forwards_match_host_behavior() {
        unsafe {
            let host_gettimeofday: unsafe extern "C" fn(*mut libc::timeval, *mut c_void) -> c_int =
                runtime::resolve_next_function(b"gettimeofday\0");
            let host_clock_gettime: unsafe extern "C" fn(clockid_t, *mut libc::timespec) -> c_int =
                runtime::resolve_next_function(b"clock_gettime\0");
            let host_gmtime_r: unsafe extern "C" fn(*const time_t, *mut libc::tm) -> *mut libc::tm =
                runtime::resolve_next_function(b"gmtime_r\0");
            let host_localtime_r: unsafe extern "C" fn(
                *const time_t,
                *mut libc::tm,
            ) -> *mut libc::tm = runtime::resolve_next_function(b"localtime_r\0");
            let host_mktime: unsafe extern "C" fn(*mut libc::tm) -> time_t =
                runtime::resolve_next_function(b"mktime\0");
            let host_asctime_r: unsafe extern "C" fn(
                *const libc::tm,
                *mut [c_char; 26usize],
            ) -> *mut c_char = runtime::resolve_next_function(b"asctime_r\0");
            let host_ctime_r: unsafe extern "C" fn(
                *const time_t,
                *mut [c_char; 26usize],
            ) -> *mut c_char = runtime::resolve_next_function(b"ctime_r\0");
            let host_strftime: unsafe extern "C" fn(
                *mut c_char,
                usize,
                *const c_char,
                *const libc::tm,
            ) -> usize = runtime::resolve_next_function(b"strftime\0");
            let host_strptime: unsafe extern "C" fn(
                *const c_char,
                *const c_char,
                *mut libc::tm,
            ) -> *mut c_char = runtime::resolve_next_function(b"strptime\0");

            let mut badge_now = timeval {
                tv_sec: 0,
                tv_usec: 0,
                __bindgen_padding_0: [0; 4],
            };
            let mut host_now = libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            };
            assert_eq!(gettimeofday(&mut badge_now, core::ptr::null_mut()), 0);
            assert_eq!(host_gettimeofday(&mut host_now, core::ptr::null_mut()), 0);
            assert!(badge_now.tv_sec >= host_now.tv_sec as time_t - 1);
            assert!(badge_now.tv_sec <= host_now.tv_sec as time_t + 1);
            assert!(badge_now.tv_usec >= 0);
            assert!(badge_now.tv_usec < 1_000_000);

            let mut badge_clock = timespec {
                tv_sec: 0,
                tv_nsec: 0,
                __bindgen_padding_0: [0; 4],
            };
            let mut host_clock = libc::timespec {
                tv_sec: 0,
                tv_nsec: 0,
            };
            let realtime = libc::CLOCK_REALTIME as clockid_t;
            assert_eq!(clock_gettime(realtime, &mut badge_clock), 0);
            assert_eq!(host_clock_gettime(realtime, &mut host_clock), 0);
            assert!(badge_clock.tv_sec >= host_clock.tv_sec as time_t - 1);
            assert!(badge_clock.tv_sec <= host_clock.tv_sec as time_t + 1);
            assert!(badge_clock.tv_nsec >= 0);
            assert!(badge_clock.tv_nsec < 1_000_000_000);

            let timer: time_t = 1_717_161_234;
            let mut badge_gmt = core::mem::zeroed::<tm>();
            let mut host_gmt = core::mem::zeroed::<libc::tm>();
            assert!(core::ptr::eq(
                gmtime_r(&timer, &mut badge_gmt),
                &mut badge_gmt
            ));
            assert!(core::ptr::eq(
                host_gmtime_r(&timer, &mut host_gmt),
                &mut host_gmt
            ));
            assert_tm_matches_host(&badge_gmt, &host_gmt);

            let mut badge_local = core::mem::zeroed::<tm>();
            let mut host_local = core::mem::zeroed::<libc::tm>();
            assert!(core::ptr::eq(
                localtime_r(&timer, &mut badge_local),
                &mut badge_local
            ));
            assert!(core::ptr::eq(
                host_localtime_r(&timer, &mut host_local),
                &mut host_local
            ));
            assert_tm_matches_host(&badge_local, &host_local);

            let mut badge_asctime = [0 as c_char; 26];
            let mut host_asctime = [0 as c_char; 26];
            assert_eq!(
                asctime_r(&badge_gmt, &mut badge_asctime),
                badge_asctime.as_mut_ptr()
            );
            assert_eq!(
                host_asctime_r(&host_gmt, &mut host_asctime),
                host_asctime.as_mut_ptr()
            );
            assert_eq!(
                CStr::from_ptr(badge_asctime.as_ptr()).to_bytes(),
                CStr::from_ptr(host_asctime.as_ptr()).to_bytes()
            );

            let mut badge_ctime = [0 as c_char; 26];
            let mut host_ctime = [0 as c_char; 26];
            assert_eq!(ctime_r(&timer, &mut badge_ctime), badge_ctime.as_mut_ptr());
            assert_eq!(
                host_ctime_r(&timer, &mut host_ctime),
                host_ctime.as_mut_ptr()
            );
            assert_eq!(
                CStr::from_ptr(badge_ctime.as_ptr()).to_bytes(),
                CStr::from_ptr(host_ctime.as_ptr()).to_bytes()
            );

            let mut badge_mktime = tm {
                tm_sec: 5,
                tm_min: 4,
                tm_hour: 3,
                tm_mday: 2,
                tm_mon: 0,
                tm_year: 124,
                tm_wday: 0,
                tm_yday: 0,
                tm_isdst: -1,
            };
            let mut host_mktime_tm = badge_tm_to_host(&badge_mktime);
            assert_eq!(mktime(&mut badge_mktime), host_mktime(&mut host_mktime_tm));
            assert_tm_matches_host(&badge_mktime, &host_mktime_tm);

            let format = c"%Y-%m-%d %H:%M:%S";
            let mut badge_formatted = [0 as c_char; 64];
            let mut host_formatted = [0 as c_char; 64];
            assert_eq!(
                strftime(
                    badge_formatted.as_mut_ptr(),
                    badge_formatted.len(),
                    format.as_ptr(),
                    &badge_gmt
                ),
                host_strftime(
                    host_formatted.as_mut_ptr(),
                    host_formatted.len(),
                    format.as_ptr(),
                    &host_gmt
                )
            );
            assert_eq!(
                CStr::from_ptr(badge_formatted.as_ptr()).to_bytes(),
                CStr::from_ptr(host_formatted.as_ptr()).to_bytes()
            );
            assert_eq!(
                strftime_l(
                    badge_formatted.as_mut_ptr(),
                    badge_formatted.len(),
                    format.as_ptr(),
                    &badge_gmt,
                    77
                ),
                host_strftime(
                    host_formatted.as_mut_ptr(),
                    host_formatted.len(),
                    format.as_ptr(),
                    &host_gmt
                )
            );

            let mut badge_parsed = core::mem::zeroed::<tm>();
            let mut host_parsed = core::mem::zeroed::<libc::tm>();
            let parsed_input = c"2024-06-05 04:03:02";
            let badge_end = strptime(parsed_input.as_ptr(), format.as_ptr(), &mut badge_parsed);
            let host_end = host_strptime(parsed_input.as_ptr(), format.as_ptr(), &mut host_parsed);
            assert_eq!(
                badge_end.offset_from(parsed_input.as_ptr()),
                host_end.offset_from(parsed_input.as_ptr())
            );
            assert_tm_matches_host(&badge_parsed, &host_parsed);
        }
    }

    #[test]
    fn translated_select_forward_matches_host_behavior() {
        unsafe {
            let host_select: unsafe extern "C" fn(
                c_int,
                *mut libc::fd_set,
                *mut libc::fd_set,
                *mut libc::fd_set,
                *mut libc::timeval,
            ) -> c_int = runtime::resolve_next_function(b"select\0");

            let mut pipefds = [0; 2];
            assert_eq!(libc::pipe(pipefds.as_mut_ptr()), 0);

            let read_fd = pipefds[0];
            let write_fd = pipefds[1];
            let byte = [b'X'];
            assert_eq!(libc::write(write_fd, byte.as_ptr().cast(), 1), 1);

            let mut badge_readfds: fd_set = core::mem::zeroed();
            set_badge_fd(&mut badge_readfds, read_fd);
            let mut badge_timeout = timeval {
                tv_sec: 0,
                tv_usec: 0,
                __bindgen_padding_0: [0; 4],
            };

            let mut host_readfds: libc::fd_set = core::mem::zeroed();
            libc::FD_ZERO(&mut host_readfds);
            libc::FD_SET(read_fd, &mut host_readfds);
            let mut host_timeout = libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            };

            assert_eq!(
                select(
                    read_fd + 1,
                    &mut badge_readfds,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    &mut badge_timeout,
                ),
                host_select(
                    read_fd + 1,
                    &mut host_readfds,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    &mut host_timeout,
                )
            );

            assert!(badge_fd_is_set(&badge_readfds, read_fd));
            assert!(libc::FD_ISSET(read_fd, &host_readfds));
            assert_eq!(badge_timeout.tv_sec, host_timeout.tv_sec as time_t);
            assert_eq!(badge_timeout.tv_usec, host_timeout.tv_usec as suseconds_t);

            libc::close(read_fd);
            libc::close(write_fd);
        }
    }

    #[test]
    fn string_memory_host_forwards_match_expected_values() {
        unsafe {
            let source = *b"badge\0";
            let mut copied = [0_u8; 8];
            assert_eq!(
                mempcpy(copied.as_mut_ptr().cast(), source.as_ptr().cast(), 5).cast::<u8>(),
                copied.as_mut_ptr().add(5)
            );
            assert_eq!(&copied[..5], b"badge");

            let mut until_d = [0_u8; 8];
            assert_eq!(
                memccpy(
                    until_d.as_mut_ptr().cast(),
                    source.as_ptr().cast(),
                    b'd' as c_int,
                    5
                )
                .cast::<u8>(),
                until_d.as_mut_ptr().add(3)
            );
            assert_eq!(&until_d[..3], b"bad");

            assert_eq!(
                memmem(source.as_ptr().cast(), 5, b"dg".as_ptr().cast(), 2).cast::<u8>(),
                source.as_ptr().add(2).cast_mut()
            );
            assert_eq!(
                memrchr(source.as_ptr().cast(), b'a' as c_int, 5).cast::<u8>(),
                source.as_ptr().add(1).cast_mut()
            );
            assert_eq!(
                rawmemchr(source.as_ptr().cast(), b'g' as c_int).cast::<u8>(),
                source.as_ptr().add(3).cast_mut()
            );

            let mut built = [0 as c_char; 16];
            strcpy(built.as_mut_ptr(), c"bad".as_ptr());
            strcat(built.as_mut_ptr(), c"ge".as_ptr());
            assert_eq!(CStr::from_ptr(built.as_ptr()).to_bytes(), b"badge");
            assert_eq!(
                strlcpy(built.as_mut_ptr(), c"badge".as_ptr(), built.len() as c_uint),
                5
            );
            assert_eq!(
                strlcat(built.as_mut_ptr(), c"!!!".as_ptr(), built.len() as c_uint),
                8
            );
            assert_eq!(CStr::from_ptr(built.as_ptr()).to_bytes(), b"badge!!!");
            strncat(built.as_mut_ptr(), c"xyz".as_ptr(), 2);
            assert_eq!(CStr::from_ptr(built.as_ptr()).to_bytes(), b"badge!!!xy");

            assert_eq!(strcmp(c"badge".as_ptr(), c"badge".as_ptr()), 0);
            assert_eq!(strncmp(c"badge".as_ptr(), c"badger".as_ptr(), 5), 0);
            assert_eq!(strcspn(c"badge".as_ptr(), c"dg".as_ptr()), 2);
            assert_eq!(strspn(c"abc123".as_ptr(), c"abc".as_ptr()), 3);
            assert_eq!(strnlen(c"badge".as_ptr(), 32), 5);
            assert!(strverscmp(c"file9".as_ptr(), c"file10".as_ptr()) < 0);

            assert_eq!(
                CStr::from_ptr(strchr(c"badge".as_ptr(), b'd' as c_int)).to_bytes(),
                b"dge"
            );
            assert_eq!(
                CStr::from_ptr(strchrnul(c"badge".as_ptr(), b'z' as c_int)).to_bytes(),
                b""
            );
            assert_eq!(
                CStr::from_ptr(strrchr(c"bananas".as_ptr(), b'a' as c_int)).to_bytes(),
                b"as"
            );
            assert_eq!(
                CStr::from_ptr(strcasestr(c"BadgeLink".as_ptr(), c"link".as_ptr())).to_bytes(),
                b"Link"
            );
            assert_eq!(
                CStr::from_ptr(strstr(c"BadgeLink".as_ptr(), c"geL".as_ptr())).to_bytes(),
                b"geLink"
            );
            assert_eq!(
                CStr::from_ptr(strpbrk(c"badge".as_ptr(), c"xyde".as_ptr())).to_bytes(),
                b"dge"
            );

            let dup = strdup(c"badge".as_ptr());
            assert_eq!(CStr::from_ptr(dup).to_bytes(), b"badge");
            free(dup.cast());

            let nd = strndup(c"badger".as_ptr(), 4);
            assert_eq!(CStr::from_ptr(nd).to_bytes(), b"badg");
            free(nd.cast());

            let err = strerror(libc::EINVAL);
            assert!(!err.is_null());
            let mut errbuf = [0 as c_char; 64];
            assert!(!strerror_r(libc::EINVAL, errbuf.as_mut_ptr(), errbuf.len()).is_null());

            let mut token_buffer = [0 as c_char; 16];
            strcpy(token_buffer.as_mut_ptr(), c"one,two".as_ptr());
            let mut state = core::ptr::null_mut();
            let first = strtok_r(token_buffer.as_mut_ptr(), c",".as_ptr(), &mut state);
            let second = strtok_r(core::ptr::null_mut(), c",".as_ptr(), &mut state);
            assert_eq!(CStr::from_ptr(first).to_bytes(), b"one");
            assert_eq!(CStr::from_ptr(second).to_bytes(), b"two");

            let mut sep_buffer = [0 as c_char; 16];
            strcpy(sep_buffer.as_mut_ptr(), c"red:blue".as_ptr());
            let mut cursor = sep_buffer.as_mut_ptr();
            let first_sep = strsep(&mut cursor, c":".as_ptr());
            let second_sep = strsep(&mut cursor, c":".as_ptr());
            assert_eq!(CStr::from_ptr(first_sep).to_bytes(), b"red");
            assert_eq!(CStr::from_ptr(second_sep).to_bytes(), b"blue");

            let mut strtok_buffer = [0 as c_char; 16];
            strcpy(strtok_buffer.as_mut_ptr(), c"aa|bb".as_ptr());
            let first_tok = strtok(strtok_buffer.as_mut_ptr(), c"|".as_ptr());
            let second_tok = strtok(core::ptr::null_mut(), c"|".as_ptr());
            assert_eq!(CStr::from_ptr(first_tok).to_bytes(), b"aa");
            assert_eq!(CStr::from_ptr(second_tok).to_bytes(), b"bb");

            let format = c"%Y-%m-%d %H:%M:%S";
            let input = c"2024-06-05 04:03:02";
            let mut parsed = core::mem::zeroed::<tm>();
            let end = strptime_l(input.as_ptr(), format.as_ptr(), &mut parsed, 77);
            assert_eq!(end.offset_from(input.as_ptr()), 19);
            assert_eq!(parsed.tm_year, 124);
            assert_eq!(parsed.tm_mon, 5);
            assert_eq!(parsed.tm_mday, 5);
        }
    }

    #[test]
    fn file_dir_host_forwards_match_expected_values() {
        unsafe {
            let unique = format!(
                "/tmp/why2025-badge-emu-abi-{}-{}",
                libc::getpid(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            );
            let dir = CString::new(unique).unwrap();
            let file_a = CString::new(format!("{}/alpha.txt", dir.to_str().unwrap())).unwrap();
            let file_b = CString::new(format!("{}/beta.txt", dir.to_str().unwrap())).unwrap();

            assert_eq!(mkdir(dir.as_ptr(), 0o755), 0);

            let fd = open(
                file_a.as_ptr(),
                libc::O_CREAT | libc::O_RDWR | libc::O_TRUNC,
                0o644,
            );
            assert!(fd >= 0);
            assert_eq!(write(fd, b"badge".as_ptr().cast(), 5), 5);
            assert_eq!(lseek(fd, 0, libc::SEEK_SET), 0);

            let mut buffer = [0_u8; 5];
            assert_eq!(read(fd, buffer.as_mut_ptr().cast(), buffer.len()), 5);
            assert_eq!(&buffer, b"badge");
            assert_eq!(close(fd), 0);

            let dirp = opendir(dir.as_ptr());
            assert!(!dirp.is_null());
            assert!(!readdir(dirp).is_null());
            rewinddir(dirp);
            assert!(!readdir(dirp).is_null());
            assert_eq!(closedir(dirp), 0);

            assert_eq!(rename(file_a.as_ptr(), file_b.as_ptr()), 0);
            assert_eq!(remove(file_b.as_ptr()), 0);
            assert_eq!(rmdir(dir.as_ptr()), 0);
        }
    }

    #[test]
    fn stdio_host_forwards_match_expected_values() {
        unsafe {
            let unique = format!(
                "/tmp/why2025-badge-emu-abi-stdio-{}-{}",
                libc::getpid(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            );
            let path = CString::new(unique).unwrap();
            let path2 = CString::new(format!("{}.reopened", path.to_str().unwrap())).unwrap();

            let stream = fopen(path.as_ptr(), c"w+".as_ptr());
            assert!(!stream.is_null());
            assert!(fileno(stream) >= 0);
            assert!(fputs(c"badge\n".as_ptr(), stream) >= 0);
            assert_eq!(fputc('w' as c_int, stream), 'w' as c_int);
            assert_eq!(fwrite(b"ifi".as_ptr().cast(), 1, 3, stream), 3);
            assert_eq!(fflush(stream), 0);

            let mut pos = core::mem::zeroed::<fpos_t>();
            assert_eq!(fgetpos(stream, &mut pos), 0);
            assert!(ftell(stream) >= 0);
            assert_eq!(fseek(stream, 0, libc::SEEK_SET), 0);
            assert_eq!(ftello(stream), 0);

            let mut line = [0 as c_char; 32];
            assert_eq!(
                CStr::from_ptr(fgets(line.as_mut_ptr(), line.len() as c_int, stream)).to_bytes(),
                b"badge\n"
            );
            assert_eq!(getc(stream), 'w' as c_int);
            assert_eq!(fgetc(stream), 'i' as c_int);

            assert_eq!(fseeko(stream, 0, libc::SEEK_SET), 0);
            let mut all = [0_u8; 16];
            assert_eq!(fread(all.as_mut_ptr().cast(), 1, 10, stream), 10);
            assert_eq!(&all[..10], b"badge\nwifi");
            assert_eq!(fgetc(stream), libc::EOF);
            assert_ne!(feof(stream), 0);
            assert_eq!(ferror(stream), 0);
            clearerr(stream);
            assert_eq!(feof(stream), 0);
            clearerr_unlocked(stream);

            assert!(!freopen(path.as_ptr(), c"r".as_ptr(), stream).is_null());
            assert_eq!(fgetc(stream), 'b' as c_int);
            assert_eq!(fclose(stream), 0);

            let fd = open(path.as_ptr(), libc::O_RDONLY, 0);
            assert!(fd >= 0);
            let fd_stream = fdopen(fd, c"r".as_ptr());
            assert!(!fd_stream.is_null());
            let mut getline_buf = core::ptr::null_mut();
            let mut getline_cap = 0;
            assert_eq!(getline(&mut getline_buf, &mut getline_cap, fd_stream), 6);
            assert_eq!(CStr::from_ptr(getline_buf).to_bytes(), b"badge\n");
            free(getline_buf.cast());

            let mut getdelim_buf = core::ptr::null_mut();
            let mut getdelim_cap = 0;
            assert_eq!(
                getdelim(
                    &mut getdelim_buf,
                    &mut getdelim_cap,
                    'i' as c_int,
                    fd_stream
                ),
                2
            );
            assert_eq!(CStr::from_ptr(getdelim_buf).to_bytes(), b"wi");
            free(getdelim_buf.cast());
            assert_eq!(fclose(fd_stream), 0);

            let mut memory = [0 as c_char; 16];
            let mem_stream = fmemopen(memory.as_mut_ptr().cast(), memory.len(), c"w+".as_ptr());
            assert!(!mem_stream.is_null());
            assert_eq!(fputs(c"emu".as_ptr(), mem_stream), 1);
            assert_eq!(fflush(mem_stream), 0);
            assert_eq!(fseek(mem_stream, 0, libc::SEEK_SET), 0);
            let mut mem_buf = [0 as c_char; 8];
            assert_eq!(
                CStr::from_ptr(fgets(
                    mem_buf.as_mut_ptr(),
                    mem_buf.len() as c_int,
                    mem_stream
                ))
                .to_bytes(),
                b"emu"
            );
            assert_eq!(fclose(mem_stream), 0);

            assert_eq!(rename(path.as_ptr(), path2.as_ptr()), 0);
            assert_eq!(remove(path2.as_ptr()), 0);
        }
    }

    #[test]
    fn networking_host_forwards_match_expected_values() {
        unsafe {
            let mut addr = in_addr { s_addr: 0 };
            assert_ne!(inet_aton(c"127.0.0.1".as_ptr(), &mut addr), 0);
            assert_eq!(CStr::from_ptr(inet_ntoa(addr)).to_bytes(), b"127.0.0.1");

            let mut addrinfo_result = core::ptr::null_mut();
            assert_eq!(
                getaddrinfo(
                    c"127.0.0.1".as_ptr(),
                    c"0".as_ptr(),
                    core::ptr::null(),
                    &mut addrinfo_result,
                ),
                0
            );
            assert!(!addrinfo_result.is_null());
            freeaddrinfo(addrinfo_result);

            let server = socket(libc::AF_INET, libc::SOCK_STREAM, 0);
            assert!(server >= 0);

            let mut server_addr = libc::sockaddr_in {
                sin_family: libc::AF_INET as libc::sa_family_t,
                sin_port: 0,
                sin_addr: libc::in_addr {
                    s_addr: u32::from_ne_bytes([127, 0, 0, 1]),
                },
                sin_zero: [0; 8],
            };
            assert_eq!(
                bind(
                    server,
                    (&server_addr as *const libc::sockaddr_in).cast(),
                    core::mem::size_of::<libc::sockaddr_in>() as socklen_t,
                ),
                0
            );
            assert_eq!(listen(server, 1), 0);

            let mut server_len = core::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t;
            assert_eq!(
                libc::getsockname(
                    server,
                    (&mut server_addr as *mut libc::sockaddr_in).cast(),
                    &mut server_len,
                ),
                0
            );

            let client = socket(libc::AF_INET, libc::SOCK_STREAM, 0);
            assert!(client >= 0);
            assert_eq!(
                connect(
                    client,
                    (&server_addr as *const libc::sockaddr_in).cast(),
                    server_len as socklen_t,
                ),
                0
            );

            let accepted = accept(server, core::ptr::null_mut(), core::ptr::null_mut());
            assert!(accepted >= 0);
            assert_eq!(write(client, b"Z".as_ptr().cast(), 1), 1);
            let mut byte = [0_u8; 1];
            assert_eq!(read(accepted, byte.as_mut_ptr().cast(), 1), 1);
            assert_eq!(byte, [b'Z']);

            assert_eq!(close(accepted), 0);
            assert_eq!(close(client), 0);
            assert_eq!(close(server), 0);
        }
    }

    #[test]
    fn small_host_forwards_match_expected_values() {
        unsafe {
            let timer: time_t = 1_717_161_234;
            let mut badge_gmt = core::mem::zeroed::<tm>();
            assert!(core::ptr::eq(
                gmtime_r(&timer, &mut badge_gmt),
                &mut badge_gmt
            ));

            let mut asctime_buf = [0 as c_char; 26];
            assert_eq!(
                CStr::from_ptr(asctime(&badge_gmt)).to_bytes(),
                CStr::from_ptr(asctime_r(&badge_gmt, &mut asctime_buf)).to_bytes()
            );

            let mut ctime_buf = [0 as c_char; 26];
            assert_eq!(
                CStr::from_ptr(ctime(&timer)).to_bytes(),
                CStr::from_ptr(ctime_r(&timer, &mut ctime_buf)).to_bytes()
            );

            srand(1234);
            let first_rand = rand();
            srand(1234);
            let second_rand = rand();
            assert_eq!(first_rand, second_rand);

            srandom(5678);
            let first_random = random();
            srandom(5678);
            let second_random = random();
            assert_eq!(first_random, second_random);

            let mut options = b"mode=fast,flag\0".to_vec();
            let mut option_ptr = options.as_mut_ptr().cast::<c_char>();
            let tokens = [c"mode".as_ptr().cast_mut(), c"flag".as_ptr().cast_mut()];
            let mut value_ptr = core::ptr::null_mut();

            assert_eq!(
                getsubopt(&mut option_ptr, tokens.as_ptr(), &mut value_ptr),
                0
            );
            assert_eq!(CStr::from_ptr(value_ptr).to_bytes(), b"fast");
            assert_eq!(
                getsubopt(&mut option_ptr, tokens.as_ptr(), &mut value_ptr),
                1
            );
            assert!(value_ptr.is_null());
        }
    }

    #[test]
    fn regex_iconv_and_static_time_forwards_match_expected_values() {
        unsafe {
            let timer: time_t = 1_717_161_234;
            let mut gmt = core::mem::zeroed::<tm>();
            let mut local = core::mem::zeroed::<tm>();
            let gmt_ptr = gmtime(&timer);
            let local_ptr = localtime(&timer);
            assert!(!gmt_ptr.is_null());
            assert!(!local_ptr.is_null());
            assert!(core::ptr::eq(gmtime_r(&timer, &mut gmt), &mut gmt));
            assert!(core::ptr::eq(localtime_r(&timer, &mut local), &mut local));
            assert_eq!(gmt.tm_year, (*gmt_ptr).tm_year);
            assert_eq!(gmt.tm_yday, (*gmt_ptr).tm_yday);
            assert_eq!(local.tm_hour, (*local_ptr).tm_hour);
            assert_eq!(local.tm_mday, (*local_ptr).tm_mday);

            let cd = iconv_open(c"UTF-8".as_ptr(), c"UTF-8".as_ptr());
            assert_ne!(cd as isize, -1);
            let mut input = b"badge".to_vec();
            let mut input_ptr = input.as_mut_ptr().cast::<c_char>();
            let mut input_left = input.len();
            let mut output = [0_u8; 16];
            let mut output_ptr = output.as_mut_ptr().cast::<c_char>();
            let mut output_left = output.len();
            assert_eq!(
                iconv(
                    cd,
                    &mut input_ptr,
                    &mut input_left,
                    &mut output_ptr,
                    &mut output_left,
                ),
                0
            );
            assert_eq!(input_left, 0);
            assert_eq!(&output[..5], b"badge");
            assert_eq!(iconv_close(cd), 0);
        }
    }

    #[test]
    fn regex_bridge_compiles_execs_and_formats_errors() {
        unsafe {
            let mut regex = core::mem::zeroed::<regex_t>();
            assert_eq!(
                regcomp(&mut regex, c"ba(dg)e".as_ptr(), libc::REG_EXTENDED),
                0
            );
            assert_eq!(regex.re_magic, HOST_REGEX_MAGIC);
            assert_eq!(regex.re_nsub, 1);

            let mut matches = [regmatch_t {
                rm_so: -1,
                rm_eo: -1,
            }];
            assert_eq!(
                regexec(
                    &regex,
                    c"badge".as_ptr(),
                    matches.len(),
                    matches.as_mut_ptr().cast::<[regmatch_t; 0usize]>(),
                    0,
                ),
                0
            );
            assert_eq!(matches[0].rm_so, 0);
            assert_eq!(matches[0].rm_eo, 5);

            assert_eq!(
                regexec(
                    &regex,
                    c"hello".as_ptr(),
                    0,
                    core::ptr::null_mut::<[regmatch_t; 0usize]>(),
                    0,
                ),
                libc::REG_NOMATCH
            );

            regfree(&mut regex);
            assert_eq!(regex.re_magic, 0);
            assert!(regex.re_g.is_null());

            let mut bad_regex = core::mem::zeroed::<regex_t>();
            let err = regcomp(&mut bad_regex, c"[".as_ptr(), libc::REG_EXTENDED);
            assert_ne!(err, 0);
            let mut errbuf = [0 as c_char; 128];
            let size = regerror(err, &bad_regex, errbuf.as_mut_ptr(), errbuf.len());
            assert!(size > 0);
            assert!(!CStr::from_ptr(errbuf.as_ptr()).to_bytes().is_empty());
            regfree(&mut bad_regex);
        }
    }
}
