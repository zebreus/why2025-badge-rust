use crate::{
    __compar_fn_t, _ssize_t, DIR, FILE, VISIT, addrinfo, clock_t, clockid_t, div_t, fd_set, fpos_t,
    iconv_t, imaxdiv_t, in_addr, intmax_t, lconv, ldiv_t, lldiv_t, locale_t, mbstate_t, mode_t,
    nl_item, off_t, option, pid_t, regex_t, sockaddr, socklen_t, stat as stat_t, termios, time_t,
    timespec, timeval, tm, tms, uintmax_t, useconds_t, wchar_t, wctrans_t, wctype_t, wint_t,
};
use core::ffi::{c_char, c_int, c_long, c_longlong, c_uint, c_ulong, c_ulonglong, c_void};

mod runtime;

use runtime::call_resolved;

const BADGE_FD_SET_WORDS: usize = 2;

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

fn host_timeval_to_badge(value: &libc::timeval) -> timeval {
    timeval {
        tv_sec: value.tv_sec as time_t,
        tv_usec: value.tv_usec as _,
        __bindgen_padding_0: [0; 4],
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
    let mut badge = fd_set {
        __fds_bits: [0; BADGE_FD_SET_WORDS],
    };
    let host_words = (value as *const libc::fd_set).cast::<libc::c_ulong>();

    for (index, slot) in badge.__fds_bits.iter_mut().enumerate() {
        *slot = unsafe { *host_words.add(index) } as c_ulong;
    }

    badge
}

macro_rules! forward_resolved_fn {
    ($(fn $name:ident($($arg:ident : $arg_ty:ty),* $(,)?) -> $ret:ty = $resolver:path;)+) => {
        $(
            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn $name($($arg: $arg_ty),*) -> $ret {
                call_resolved!($resolver $(, $arg)*)
            }
        )+
    };
}

macro_rules! forward_ignore_locale_resolved_fn {
    ($(fn $name:ident($($arg:ident : $arg_ty:ty),* ; $locale:ident : locale_t) -> $ret:ty = $resolver:path;)+) => {
        $(
            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn $name($($arg: $arg_ty,)* $locale: locale_t) -> $ret {
                let _ = $locale;
                call_resolved!($resolver $(, $arg)*)
            }
        )+
    };
}

forward_resolved_fn! {
    fn a64l(input: *const c_char) -> c_long = runtime::real_a64l;
    fn abs(value: c_int) -> c_int = runtime::real_abs;
    fn acos(value: f64) -> f64 = runtime::real_acos;
    fn acosf(value: f32) -> f32 = runtime::real_acosf;
    fn acosh(value: f64) -> f64 = runtime::real_acosh;
    fn acoshf(value: f32) -> f32 = runtime::real_acoshf;
    fn asin(value: f64) -> f64 = runtime::real_asin;
    fn asinf(value: f32) -> f32 = runtime::real_asinf;
    fn asinh(value: f64) -> f64 = runtime::real_asinh;
    fn asinhf(value: f32) -> f32 = runtime::real_asinhf;
    fn asctime_r(timer: *const tm, buf: *mut [c_char; 26usize]) -> *mut c_char = runtime::real_asctime_r;
    fn atan(value: f64) -> f64 = runtime::real_atan;
    fn atan2(left: f64, right: f64) -> f64 = runtime::real_atan2;
    fn atan2f(left: f32, right: f32) -> f32 = runtime::real_atan2f;
    fn atanf(value: f32) -> f32 = runtime::real_atanf;
    fn atanh(value: f64) -> f64 = runtime::real_atanh;
    fn atanhf(value: f32) -> f32 = runtime::real_atanhf;
    fn atof(value: *const c_char) -> f64 = runtime::real_atof;
    fn atoi(value: *const c_char) -> c_int = runtime::real_atoi;
    fn atol(value: *const c_char) -> c_long = runtime::real_atol;
    fn atoll(value: *const c_char) -> c_longlong = runtime::real_atoll;
    fn bcmp(left: *const c_void, right: *const c_void, count: c_uint) -> c_int = runtime::real_bcmp;
    fn bcopy(src: *const c_void, dst: *mut c_void, count: c_uint) -> () = runtime::real_bcopy;
    fn bsearch(key: *const c_void, base: *const c_void, nmemb: usize, size: usize, compar: __compar_fn_t) -> *mut c_void = runtime::real_bsearch;
    fn btowc(value: c_int) -> wint_t = runtime::real_btowc;
    fn bzero(ptr: *mut c_void, count: c_uint) -> () = runtime::real_bzero;
    fn calloc(count: c_uint, size: c_uint) -> *mut c_void = runtime::real_calloc;
    fn cbrt(value: f64) -> f64 = runtime::real_cbrt;
    fn cbrtf(value: f32) -> f32 = runtime::real_cbrtf;
    fn ceil(value: f64) -> f64 = runtime::real_ceil;
    fn ceilf(value: f32) -> f32 = runtime::real_ceilf;
    fn clock() -> clock_t = runtime::real_clock;
    fn copysign(left: f64, right: f64) -> f64 = runtime::real_copysign;
    fn copysignf(left: f32, right: f32) -> f32 = runtime::real_copysignf;
    fn cos(value: f64) -> f64 = runtime::real_cos;
    fn cosf(value: f32) -> f32 = runtime::real_cosf;
    fn cosh(value: f64) -> f64 = runtime::real_cosh;
    fn coshf(value: f32) -> f32 = runtime::real_coshf;
    fn ctime_r(timer: *const time_t, buf: *mut [c_char; 26usize]) -> *mut c_char = runtime::real_ctime_r;
    fn difftime(time2: time_t, time1: time_t) -> f64 = runtime::real_difftime;
    fn div(numer: c_int, denom: c_int) -> div_t = runtime::real_div;
    fn drem(left: f64, right: f64) -> f64 = runtime::real_drem;
    fn dremf(left: f32, right: f32) -> f32 = runtime::real_dremf;
    fn erf(value: f64) -> f64 = runtime::real_erf;
    fn erfc(value: f64) -> f64 = runtime::real_erfc;
    fn erfcf(value: f32) -> f32 = runtime::real_erfcf;
    fn erff(value: f32) -> f32 = runtime::real_erff;
    fn exp(value: f64) -> f64 = runtime::real_exp;
    fn exp2(value: f64) -> f64 = runtime::real_exp2;
    fn exp2f(value: f32) -> f32 = runtime::real_exp2f;
    fn expf(value: f32) -> f32 = runtime::real_expf;
    fn explicit_bzero(ptr: *mut c_void, count: usize) -> () = runtime::real_explicit_bzero;
    fn expm1(value: f64) -> f64 = runtime::real_expm1;
    fn expm1f(value: f32) -> f32 = runtime::real_expm1f;
    fn fabs(value: f64) -> f64 = runtime::real_fabs;
    fn fabsf(value: f32) -> f32 = runtime::real_fabsf;
    fn fdim(left: f64, right: f64) -> f64 = runtime::real_fdim;
    fn fdimf(left: f32, right: f32) -> f32 = runtime::real_fdimf;
    fn finite(value: f64) -> c_int = runtime::real_finite;
    fn finitef(value: f32) -> c_int = runtime::real_finitef;
    fn ffs(value: c_int) -> c_int = runtime::real_ffs;
    fn ffsl(value: c_long) -> c_int = runtime::real_ffsl;
    fn ffsll(value: c_longlong) -> c_int = runtime::real_ffsll;
    fn floor(value: f64) -> f64 = runtime::real_floor;
    fn floorf(value: f32) -> f32 = runtime::real_floorf;
    fn fma(left: f64, right: f64, value: f64) -> f64 = runtime::real_fma;
    fn fmaf(left: f32, right: f32, value: f32) -> f32 = runtime::real_fmaf;
    fn fmax(left: f64, right: f64) -> f64 = runtime::real_fmax;
    fn fmaxf(left: f32, right: f32) -> f32 = runtime::real_fmaxf;
    fn fmin(left: f64, right: f64) -> f64 = runtime::real_fmin;
    fn fminf(left: f32, right: f32) -> f32 = runtime::real_fminf;
    fn fmod(left: f64, right: f64) -> f64 = runtime::real_fmod;
    fn fmodf(left: f32, right: f32) -> f32 = runtime::real_fmodf;
    fn fnmatch(pattern: *const c_char, value: *const c_char, flags: c_int) -> c_int = runtime::real_fnmatch;
    fn free(ptr: *mut c_void) -> () = runtime::real_free;
    fn frexp(value: f64, exp: *mut c_int) -> f64 = runtime::real_frexp;
    fn frexpf(value: f32, exp: *mut c_int) -> f32 = runtime::real_frexpf;
    fn fwide(stream: *mut FILE, mode: c_int) -> c_int = runtime::real_fwide;
    fn gamma(value: f64) -> f64 = runtime::real_gamma;
    fn gammaf(value: f32) -> f32 = runtime::real_gammaf;
    fn gcvt(value: f64, ndigit: c_int, buf: *mut c_char) -> *mut c_char = runtime::real_gcvt;
    fn getentropy(buf: *mut c_void, count: usize) -> c_int = runtime::real_getentropy;
    fn getopt(argc: c_int, argv: *const [*mut c_char; 0usize], optstring: *const c_char) -> c_int = runtime::real_getopt;
    fn getopt_long(argc: c_int, argv: *const [*mut c_char; 0usize], shortopts: *const c_char, longopts: *const option, longind: *mut c_int) -> c_int = runtime::real_getopt_long;
    fn getopt_long_only(argc: c_int, argv: *const [*mut c_char; 0usize], shortopts: *const c_char, longopts: *const option, longind: *mut c_int) -> c_int = runtime::real_getopt_long_only;
    fn hypot(left: f64, right: f64) -> f64 = runtime::real_hypot;
    fn hypotf(left: f32, right: f32) -> f32 = runtime::real_hypotf;
    fn iconv(cd: iconv_t, inbuf: *mut *mut c_char, inbytesleft: *mut usize, outbuf: *mut *mut c_char, outbytesleft: *mut usize) -> usize = runtime::real_iconv;
    fn ilogb(value: f64) -> c_int = runtime::real_ilogb;
    fn ilogbf(value: f32) -> c_int = runtime::real_ilogbf;
    fn imaxabs(value: intmax_t) -> intmax_t = runtime::real_imaxabs;
    fn imaxdiv(numer: intmax_t, denom: intmax_t) -> imaxdiv_t = runtime::real_imaxdiv;
    fn index(value: *const c_char, needle: c_int) -> *mut c_char = runtime::real_index;
    fn isinf(value: f64) -> c_int = runtime::real_isinf;
    fn isinff(value: f32) -> c_int = runtime::real_isinff;
    fn isnan(value: f64) -> c_int = runtime::real_isnan;
    fn isnanf(value: f32) -> c_int = runtime::real_isnanf;
    fn j0(value: f64) -> f64 = runtime::real_j0;
    fn j0f(value: f32) -> f32 = runtime::real_j0f;
    fn j1(value: f64) -> f64 = runtime::real_j1;
    fn j1f(value: f32) -> f32 = runtime::real_j1f;
    fn jn(order: c_int, value: f64) -> f64 = runtime::real_jn;
    fn jnf(order: c_int, value: f32) -> f32 = runtime::real_jnf;
    fn l64a(value: c_long) -> *mut c_char = runtime::real_l64a;
    fn labs(value: c_long) -> c_long = runtime::real_labs;
    fn ldexp(value: f64, exp: c_int) -> f64 = runtime::real_ldexp;
    fn ldexpf(value: f32, exp: c_int) -> f32 = runtime::real_ldexpf;
    fn ldiv(numer: c_long, denom: c_long) -> ldiv_t = runtime::real_ldiv;
    fn link(path1: *const c_char, path2: *const c_char) -> c_int = runtime::real_link;
    fn lgamma(value: f64) -> f64 = runtime::real_lgamma;
    fn lgamma_r(value: f64, signgamp: *mut c_int) -> f64 = runtime::real_lgamma_r;
    fn lgammaf(value: f32) -> f32 = runtime::real_lgammaf;
    fn lgammaf_r(value: f32, signgamp: *mut c_int) -> f32 = runtime::real_lgammaf_r;
    fn llabs(value: c_longlong) -> c_longlong = runtime::real_llabs;
    fn lldiv(numer: c_longlong, denom: c_longlong) -> lldiv_t = runtime::real_lldiv;
    fn llrint(value: f64) -> c_longlong = runtime::real_llrint;
    fn llrintf(value: f32) -> c_longlong = runtime::real_llrintf;
    fn llround(value: f64) -> c_longlong = runtime::real_llround;
    fn llroundf(value: f32) -> c_longlong = runtime::real_llroundf;
    fn localeconv() -> *mut lconv = runtime::real_localeconv;
    fn log(value: f64) -> f64 = runtime::real_log;
    fn log10(value: f64) -> f64 = runtime::real_log10;
    fn log10f(value: f32) -> f32 = runtime::real_log10f;
    fn log1p(value: f64) -> f64 = runtime::real_log1p;
    fn log1pf(value: f32) -> f32 = runtime::real_log1pf;
    fn log2(value: f64) -> f64 = runtime::real_log2;
    fn log2f(value: f32) -> f32 = runtime::real_log2f;
    fn logb(value: f64) -> f64 = runtime::real_logb;
    fn logbf(value: f32) -> f32 = runtime::real_logbf;
    fn logf(value: f32) -> f32 = runtime::real_logf;
    fn lrint(value: f64) -> c_long = runtime::real_lrint;
    fn lrintf(value: f32) -> c_long = runtime::real_lrintf;
    fn lround(value: f64) -> c_long = runtime::real_lround;
    fn lroundf(value: f32) -> c_long = runtime::real_lroundf;
    fn malloc(size: c_uint) -> *mut c_void = runtime::real_malloc;
    fn mblen(value: *const c_char, count: usize) -> c_int = runtime::real_mblen;
    fn mbrlen(value: *const c_char, count: usize, state: *mut mbstate_t) -> usize = runtime::real_mbrlen;
    fn mbrtowc(pwc: *mut wchar_t, value: *const c_char, count: usize, state: *mut mbstate_t) -> usize = runtime::real_mbrtowc;
    fn mbsinit(state: *const mbstate_t) -> c_int = runtime::real_mbsinit;
    fn mbsnrtowcs(dst: *mut wchar_t, src: *mut *const c_char, nwc: usize, len: usize, state: *mut mbstate_t) -> usize = runtime::real_mbsnrtowcs;
    fn mbsrtowcs(dst: *mut wchar_t, src: *mut *const c_char, len: usize, state: *mut mbstate_t) -> usize = runtime::real_mbsrtowcs;
    fn mbstowcs(dst: *mut wchar_t, src: *const c_char, len: usize) -> usize = runtime::real_mbstowcs;
    fn mbtowc(pwc: *mut wchar_t, value: *const c_char, count: usize) -> c_int = runtime::real_mbtowc;
    fn modf(value: f64, iptr: *mut f64) -> f64 = runtime::real_modf;
    fn modff(value: f32, iptr: *mut f32) -> f32 = runtime::real_modff;
    fn nan(tagp: *const c_char) -> f64 = runtime::real_nan;
    fn nanf(tagp: *const c_char) -> f32 = runtime::real_nanf;
    fn nearbyint(value: f64) -> f64 = runtime::real_nearbyint;
    fn nearbyintf(value: f32) -> f32 = runtime::real_nearbyintf;
    fn nextafter(left: f64, right: f64) -> f64 = runtime::real_nextafter;
    fn nextafterf(left: f32, right: f32) -> f32 = runtime::real_nextafterf;
    fn nl_langinfo(item: nl_item) -> *mut c_char = runtime::real_nl_langinfo;
    fn pow(left: f64, right: f64) -> f64 = runtime::real_pow;
    fn powf(left: f32, right: f32) -> f32 = runtime::real_powf;
    fn qsort(base: *mut c_void, nmemb: usize, size: usize, compar: __compar_fn_t) -> () = runtime::real_qsort;
    fn qsort_r(base: *mut c_void, nmemb: usize, size: usize, compar: ::core::option::Option<unsafe extern "C" fn(*const c_void, *const c_void, *mut c_void) -> c_int>, thunk: *mut c_void) -> () = runtime::real_qsort_r;
    fn rand_r(seed: *mut c_uint) -> c_int = runtime::real_rand_r;
    fn realloc(ptr: *mut c_void, size: c_uint) -> *mut c_void = runtime::real_realloc;
    fn reallocarray(ptr: *mut c_void, nmemb: usize, size: usize) -> *mut c_void = runtime::real_reallocarray;
    fn remainder(left: f64, right: f64) -> f64 = runtime::real_remainder;
    fn remainderf(left: f32, right: f32) -> f32 = runtime::real_remainderf;
    fn remquo(left: f64, right: f64, quo: *mut c_int) -> f64 = runtime::real_remquo;
    fn remquof(left: f32, right: f32, quo: *mut c_int) -> f32 = runtime::real_remquof;
    fn rindex(value: *const c_char, needle: c_int) -> *mut c_char = runtime::real_rindex;
    fn rint(value: f64) -> f64 = runtime::real_rint;
    fn rintf(value: f32) -> f32 = runtime::real_rintf;
    fn round(value: f64) -> f64 = runtime::real_round;
    fn roundf(value: f32) -> f32 = runtime::real_roundf;
    fn rpmatch(response: *const c_char) -> c_int = runtime::real_rpmatch;
    fn scalbln(value: f64, exp: c_long) -> f64 = runtime::real_scalbln;
    fn scalblnf(value: f32, exp: c_long) -> f32 = runtime::real_scalblnf;
    fn scalbn(value: f64, exp: c_int) -> f64 = runtime::real_scalbn;
    fn scalbnf(value: f32, exp: c_int) -> f32 = runtime::real_scalbnf;
    fn sin(value: f64) -> f64 = runtime::real_sin;
    fn sincos(value: f64, sinp: *mut f64, cosp: *mut f64) -> () = runtime::real_sincos;
    fn sincosf(value: f32, sinp: *mut f32, cosp: *mut f32) -> () = runtime::real_sincosf;
    fn sinf(value: f32) -> f32 = runtime::real_sinf;
    fn sinh(value: f64) -> f64 = runtime::real_sinh;
    fn sinhf(value: f32) -> f32 = runtime::real_sinhf;
    fn sleep(seconds: c_uint) -> c_uint = runtime::real_sleep;
    fn sqrt(value: f64) -> f64 = runtime::real_sqrt;
    fn sqrtf(value: f32) -> f32 = runtime::real_sqrtf;
    fn strcasecmp(left: *const c_char, right: *const c_char) -> c_int = runtime::real_strcasecmp;
    fn strcoll(left: *const c_char, right: *const c_char) -> c_int = runtime::real_strcoll;
    fn strncasecmp(left: *const c_char, right: *const c_char, count: c_uint) -> c_int = runtime::real_strncasecmp;
    fn strtod(value: *const c_char, end_ptr: *mut *mut c_char) -> f64 = runtime::real_strtod;
    fn strtof(value: *const c_char, end_ptr: *mut *mut c_char) -> f32 = runtime::real_strtof;
    fn strtoimax(value: *const c_char, end_ptr: *mut *mut c_char, base: c_int) -> intmax_t = runtime::real_strtoimax;
    fn strtol(value: *const c_char, end_ptr: *mut *mut c_char, base: c_int) -> c_long = runtime::real_strtol;
    fn strtoll(value: *const c_char, end_ptr: *mut *mut c_char, base: c_int) -> c_longlong = runtime::real_strtoll;
    fn strtoul(value: *const c_char, end_ptr: *mut *mut c_char, base: c_int) -> c_ulong = runtime::real_strtoul;
    fn strtoull(value: *const c_char, end_ptr: *mut *mut c_char, base: c_int) -> c_ulonglong = runtime::real_strtoull;
    fn strtoumax(value: *const c_char, end_ptr: *mut *mut c_char, base: c_int) -> uintmax_t = runtime::real_strtoumax;
    fn strxfrm(dst: *mut c_char, src: *const c_char, size: c_uint) -> c_uint = runtime::real_strxfrm;
    fn swab(src: *const c_void, dst: *mut c_void, count: isize) -> () = runtime::real_swab;
    fn tan(value: f64) -> f64 = runtime::real_tan;
    fn tanf(value: f32) -> f32 = runtime::real_tanf;
    fn tanh(value: f64) -> f64 = runtime::real_tanh;
    fn tanhf(value: f32) -> f32 = runtime::real_tanhf;
    fn tdelete(key: *const c_void, rootp: *mut *mut c_void, compar: __compar_fn_t) -> *mut c_void = runtime::real_tdelete;
    fn tdestroy(root: *mut c_void, freefct: ::core::option::Option<unsafe extern "C" fn(*mut c_void)>) -> () = runtime::real_tdestroy;
    fn tfind(key: *const c_void, rootp: *mut *mut c_void, compar: __compar_fn_t) -> *mut c_void = runtime::real_tfind;
    fn tgamma(value: f64) -> f64 = runtime::real_tgamma;
    fn tgammaf(value: f32) -> f32 = runtime::real_tgammaf;
    fn time(timer: *mut time_t) -> time_t = runtime::real_time;
    fn times(buf: *mut tms) -> clock_t = runtime::real_times;
    fn tsearch(key: *const c_void, rootp: *mut *mut c_void, compar: __compar_fn_t) -> *mut c_void = runtime::real_tsearch;
    fn trunc(value: f64) -> f64 = runtime::real_trunc;
    fn truncf(value: f32) -> f32 = runtime::real_truncf;
    fn twalk(root: *const c_void, action: ::core::option::Option<unsafe extern "C" fn(*const c_void, VISIT, c_int)>) -> () = runtime::real_twalk;
    fn usleep(useconds: useconds_t) -> c_int = runtime::real_usleep;
    fn wcpcpy(dst: *mut wchar_t, src: *const wchar_t) -> *mut wchar_t = runtime::real_wcpcpy;
    fn wcpncpy(dst: *mut wchar_t, src: *const wchar_t, count: usize) -> *mut wchar_t = runtime::real_wcpncpy;
    fn wcrtomb(dst: *mut c_char, wc: wchar_t, state: *mut mbstate_t) -> usize = runtime::real_wcrtomb;
    fn wcscoll(left: *const wchar_t, right: *const wchar_t) -> c_int = runtime::real_wcscoll;
    fn wcsnrtombs(dst: *mut c_char, src: *mut *const wchar_t, nwc: usize, len: usize, state: *mut mbstate_t) -> usize = runtime::real_wcsnrtombs;
    fn wcsrtombs(dst: *mut c_char, src: *mut *const wchar_t, len: usize, state: *mut mbstate_t) -> usize = runtime::real_wcsrtombs;
    fn wcstod(value: *const wchar_t, end_ptr: *mut *mut wchar_t) -> f64 = runtime::real_wcstod;
    fn wcstof(value: *const wchar_t, end_ptr: *mut *mut wchar_t) -> f32 = runtime::real_wcstof;
    fn wcstoimax(value: *const wchar_t, end_ptr: *mut *mut wchar_t, base: c_int) -> intmax_t = runtime::real_wcstoimax;
    fn wcstol(value: *const wchar_t, end_ptr: *mut *mut wchar_t, base: c_int) -> c_long = runtime::real_wcstol;
    fn wcstoll(value: *const wchar_t, end_ptr: *mut *mut wchar_t, base: c_int) -> c_longlong = runtime::real_wcstoll;
    fn wcstombs(dst: *mut c_char, src: *const wchar_t, len: usize) -> usize = runtime::real_wcstombs;
    fn wcstoul(value: *const wchar_t, end_ptr: *mut *mut wchar_t, base: c_int) -> c_ulong = runtime::real_wcstoul;
    fn wcstoull(value: *const wchar_t, end_ptr: *mut *mut wchar_t, base: c_int) -> c_ulonglong = runtime::real_wcstoull;
    fn wcstoumax(value: *const wchar_t, end_ptr: *mut *mut wchar_t, base: c_int) -> uintmax_t = runtime::real_wcstoumax;
    fn wcsxfrm(dst: *mut wchar_t, src: *const wchar_t, size: usize) -> usize = runtime::real_wcsxfrm;
    fn wctomb(dst: *mut c_char, wc: wchar_t) -> c_int = runtime::real_wctomb;
    fn y0(value: f64) -> f64 = runtime::real_y0;
    fn y0f(value: f32) -> f32 = runtime::real_y0f;
    fn y1(value: f64) -> f64 = runtime::real_y1;
    fn y1f(value: f32) -> f32 = runtime::real_y1f;
    fn yn(order: c_int, value: f64) -> f64 = runtime::real_yn;
    fn ynf(order: c_int, value: f32) -> f32 = runtime::real_ynf;
}

forward_ignore_locale_resolved_fn! {
    fn iswalnum_l(value: wint_t; locale: locale_t) -> c_int = runtime::real_iswalnum;
    fn iswalpha_l(value: wint_t; locale: locale_t) -> c_int = runtime::real_iswalpha;
    fn iswblank_l(value: wint_t; locale: locale_t) -> c_int = runtime::real_iswblank;
    fn iswcntrl_l(value: wint_t; locale: locale_t) -> c_int = runtime::real_iswcntrl;
    fn iswctype_l(value: wint_t, desc: wctype_t; locale: locale_t) -> c_int = runtime::real_iswctype;
    fn iswdigit_l(value: wint_t; locale: locale_t) -> c_int = runtime::real_iswdigit;
    fn iswgraph_l(value: wint_t; locale: locale_t) -> c_int = runtime::real_iswgraph;
    fn iswlower_l(value: wint_t; locale: locale_t) -> c_int = runtime::real_iswlower;
    fn iswprint_l(value: wint_t; locale: locale_t) -> c_int = runtime::real_iswprint;
    fn iswpunct_l(value: wint_t; locale: locale_t) -> c_int = runtime::real_iswpunct;
    fn iswspace_l(value: wint_t; locale: locale_t) -> c_int = runtime::real_iswspace;
    fn iswupper_l(value: wint_t; locale: locale_t) -> c_int = runtime::real_iswupper;
    fn iswxdigit_l(value: wint_t; locale: locale_t) -> c_int = runtime::real_iswxdigit;
    fn nl_langinfo_l(item: nl_item; locale: locale_t) -> *mut c_char = runtime::real_nl_langinfo;
    fn strcasecmp_l(left: *const c_char, right: *const c_char; locale: locale_t) -> c_int = runtime::real_strcasecmp;
    fn strcoll_l(left: *const c_char, right: *const c_char; locale: locale_t) -> c_int = runtime::real_strcoll;
    fn strncasecmp_l(left: *const c_char, right: *const c_char, count: c_uint; locale: locale_t) -> c_int = runtime::real_strncasecmp;
    fn strtod_l(value: *const c_char, end_ptr: *mut *mut c_char; locale: locale_t) -> f64 = runtime::real_strtod;
    fn strtof_l(value: *const c_char, end_ptr: *mut *mut c_char; locale: locale_t) -> f32 = runtime::real_strtof;
    fn strtol_l(value: *const c_char, end_ptr: *mut *mut c_char, base: c_int; locale: locale_t) -> c_long = runtime::real_strtol;
    fn strtoll_l(value: *const c_char, end_ptr: *mut *mut c_char, base: c_int; locale: locale_t) -> c_longlong = runtime::real_strtoll;
    fn strtoul_l(value: *const c_char, end_ptr: *mut *mut c_char, base: c_int; locale: locale_t) -> c_ulong = runtime::real_strtoul;
    fn strtoull_l(value: *const c_char, end_ptr: *mut *mut c_char, base: c_int; locale: locale_t) -> c_ulonglong = runtime::real_strtoull;
    fn strxfrm_l(dst: *mut c_char, src: *const c_char, size: c_uint; locale: locale_t) -> c_uint = runtime::real_strxfrm;
    fn towlower_l(value: wint_t; locale: locale_t) -> wint_t = runtime::real_towlower;
    fn towupper_l(value: wint_t; locale: locale_t) -> wint_t = runtime::real_towupper;
    fn wcscasecmp_l(left: *const wchar_t, right: *const wchar_t; locale: locale_t) -> c_int = runtime::real_wcscasecmp;
    fn wcscoll_l(left: *const wchar_t, right: *const wchar_t; locale: locale_t) -> c_int = runtime::real_wcscoll;
    fn wcsncasecmp_l(left: *const wchar_t, right: *const wchar_t, count: usize; locale: locale_t) -> c_int = runtime::real_wcsncasecmp;
    fn wcstod_l(value: *const wchar_t, end_ptr: *mut *mut wchar_t; locale: locale_t) -> f64 = runtime::real_wcstod;
    fn wcstof_l(value: *const wchar_t, end_ptr: *mut *mut wchar_t; locale: locale_t) -> f32 = runtime::real_wcstof;
    fn wcstol_l(value: *const wchar_t, end_ptr: *mut *mut wchar_t, base: c_int; locale: locale_t) -> c_long = runtime::real_wcstol;
    fn wcstoll_l(value: *const wchar_t, end_ptr: *mut *mut wchar_t, base: c_int; locale: locale_t) -> c_longlong = runtime::real_wcstoll;
    fn wcstoul_l(value: *const wchar_t, end_ptr: *mut *mut wchar_t, base: c_int; locale: locale_t) -> c_ulong = runtime::real_wcstoul;
    fn wcstoull_l(value: *const wchar_t, end_ptr: *mut *mut wchar_t, base: c_int; locale: locale_t) -> c_ulonglong = runtime::real_wcstoull;
    fn wcsxfrm_l(dst: *mut wchar_t, src: *const wchar_t, size: usize; locale: locale_t) -> usize = runtime::real_wcsxfrm;
    fn wctrans_l(name: *const c_char; locale: locale_t) -> wctrans_t = runtime::real_wctrans;
    fn wctype_l(name: *const c_char; locale: locale_t) -> wctype_t = runtime::real_wctype;
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
static INIT_WRAPPED_OBJECTS: extern "C" fn() = runtime::init_wrapped_objects;

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - Upstream `why__Exit` is just `why_exit(status)`, which logs and deletes only the current task.
/// - Host forwarding uses libc `_Exit()` and terminates the whole host process immediately.
pub unsafe extern "C" fn _Exit(status: c_int) -> ! {
    call_resolved!(runtime::real_exit_cap, status)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - Upstream `why__exit` is just `why_exit(status)`, which logs and deletes only the current task.
/// - Host forwarding uses libc `_exit()` and terminates the whole host process immediately.
pub unsafe extern "C" fn _exit(status: c_int) -> ! {
    call_resolved!(runtime::real_exit_underscore, status)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - Upstream `why_abort` logs a warning and deletes only the current task.
/// - Host forwarding uses libc `abort()` and raises a real process abort instead.
pub unsafe extern "C" fn abort() -> ! {
    call_resolved!(runtime::real_abort)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn accept(
    sockfd: c_int,
    addr: *mut sockaddr,
    addrlen: *mut socklen_t,
) -> c_int {
    call_resolved!(runtime::real_accept, sockfd, addr, addrlen)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - Upstream `why_asctime` calls `asctime_r` into a task-local buffer.
/// - Host forwarding keeps libc `asctime()` storage semantics instead of BadgeVMS's per-task buffer.
pub unsafe extern "C" fn asctime(tblock: *const tm) -> *mut c_char {
    call_resolved!(runtime::real_asctime, tblock)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bind(sockfd: c_int, addr: *const sockaddr, addrlen: socklen_t) -> c_int {
    call_resolved!(runtime::real_bind, sockfd, addr, addrlen)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn clearerr(stream: *mut FILE) {
    call_resolved!(runtime::real_clearerr, stream)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn clearerr_unlocked(stream: *mut FILE) {
    call_resolved!(runtime::real_clearerr_unlocked, stream)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn close(fd: c_int) -> c_int {
    call_resolved!(runtime::real_close, fd)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn closedir(dir: *mut DIR) -> c_int {
    call_resolved!(runtime::real_closedir, dir)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn connect(
    sockfd: c_int,
    addr: *const sockaddr,
    addrlen: socklen_t,
) -> c_int {
    call_resolved!(runtime::real_connect, sockfd, addr, addrlen)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - Upstream `why_ctime` calls `ctime_r` into a task-local buffer.
/// - Host forwarding keeps libc `ctime()` storage semantics instead of BadgeVMS's per-task buffer.
pub unsafe extern "C" fn ctime(timer: *const time_t) -> *mut c_char {
    call_resolved!(runtime::real_ctime, timer)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - Upstream `why_exit` logs the task PID, ignores the status, and deletes only the current task.
/// - Host forwarding uses libc `exit()`, which terminates the whole host process, flushes stdio, and runs `atexit` handlers.
pub unsafe extern "C" fn exit(status: c_int) -> ! {
    call_resolved!(runtime::real_exit, status)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fclose(stream: *mut FILE) -> c_int {
    call_resolved!(runtime::real_fclose, stream)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fdopen(fd: c_int, mode: *const c_char) -> *mut FILE {
    call_resolved!(runtime::real_fdopen, fd, mode)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn feof(stream: *mut FILE) -> c_int {
    call_resolved!(runtime::real_feof, stream)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ferror(stream: *mut FILE) -> c_int {
    call_resolved!(runtime::real_ferror, stream)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fflush(stream: *mut FILE) -> c_int {
    call_resolved!(runtime::real_fflush, stream)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fgetc(stream: *mut FILE) -> c_int {
    call_resolved!(runtime::real_fgetc, stream)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fgetpos(stream: *mut FILE, pos: *mut fpos_t) -> c_int {
    call_resolved!(runtime::real_fgetpos, stream, pos)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fgets(buf: *mut c_char, size: c_int, stream: *mut FILE) -> *mut c_char {
    call_resolved!(runtime::real_fgets, buf, size, stream)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fileno(stream: *mut FILE) -> c_int {
    call_resolved!(runtime::real_fileno, stream)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fmemopen(buf: *mut c_void, size: usize, mode: *const c_char) -> *mut FILE {
    call_resolved!(runtime::real_fmemopen, buf, size, mode)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fopen(path: *const c_char, mode: *const c_char) -> *mut FILE {
    call_resolved!(runtime::real_fopen, path, mode)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fputc(value: c_int, stream: *mut FILE) -> c_int {
    call_resolved!(runtime::real_fputc, value, stream)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fputs(value: *const c_char, stream: *mut FILE) -> c_int {
    call_resolved!(runtime::real_fputs, value, stream)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fread(
    ptr: *mut c_void,
    size: c_uint,
    nmemb: c_uint,
    stream: *mut FILE,
) -> c_uint {
    call_resolved!(runtime::real_fread, ptr, size, nmemb, stream)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn freeaddrinfo(ai: *mut addrinfo) {
    call_resolved!(runtime::real_freeaddrinfo, ai)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn freopen(
    path: *const c_char,
    mode: *const c_char,
    stream: *mut FILE,
) -> *mut FILE {
    call_resolved!(runtime::real_freopen, path, mode, stream)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fseek(stream: *mut FILE, offset: c_long, whence: c_int) -> c_int {
    call_resolved!(runtime::real_fseek, stream, offset, whence)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fseeko(stream: *mut FILE, offset: off_t, whence: c_int) -> c_int {
    call_resolved!(runtime::real_fseeko, stream, offset, whence)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fstat(fd: c_int, buf: *mut stat_t) -> c_int {
    call_resolved!(runtime::real_fstat, fd, buf)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ftell(stream: *mut FILE) -> c_long {
    call_resolved!(runtime::real_ftell, stream)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ftello(stream: *mut FILE) -> off_t {
    call_resolved!(runtime::real_ftello, stream)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fwrite(
    ptr: *const c_void,
    size: c_uint,
    nmemb: c_uint,
    stream: *mut FILE,
) -> c_uint {
    call_resolved!(runtime::real_fwrite, ptr, size, nmemb, stream)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn getaddrinfo(
    nodename: *const c_char,
    servname: *const c_char,
    hints: *const addrinfo,
    res: *mut *mut addrinfo,
) -> c_int {
    call_resolved!(runtime::real_getaddrinfo, nodename, servname, hints, res)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn getc(stream: *mut FILE) -> c_int {
    call_resolved!(runtime::real_getc, stream)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn getdelim(
    lineptr: *mut *mut c_char,
    n: *mut usize,
    delim: c_int,
    stream: *mut FILE,
) -> _ssize_t {
    call_resolved!(runtime::real_getdelim, lineptr, n, delim, stream)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn getchar() -> c_int {
    call_resolved!(runtime::real_getchar)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn getchar_unlocked() -> c_int {
    call_resolved!(runtime::real_getchar_unlocked)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn getenv(name: *const c_char) -> *mut c_char {
    unsafe { runtime::getenv_impl(name) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn getsubopt(
    optionp: *mut *mut c_char,
    tokens: *const *mut c_char,
    valuep: *mut *mut c_char,
) -> c_int {
    call_resolved!(runtime::real_getsubopt, optionp, tokens, valuep)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn getline(
    lineptr: *mut *mut c_char,
    n: *mut usize,
    stream: *mut FILE,
) -> _ssize_t {
    call_resolved!(runtime::real_getline, lineptr, n, stream)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn getpid() -> pid_t {
    unsafe { runtime::getpid_impl() }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn gets(buf: *mut c_char) -> *mut c_char {
    call_resolved!(runtime::real_gets, buf)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn gettimeofday(value: *mut timeval, tz: *mut c_void) -> c_int {
    if value.is_null() {
        return call_resolved!(runtime::real_gettimeofday, core::ptr::null_mut(), tz);
    }

    let mut host_value = libc::timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let result = call_resolved!(runtime::real_gettimeofday, &mut host_value, tz);

    if result == 0 {
        unsafe { *value = host_timeval_to_badge(&host_value) };
    }

    result
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - Upstream `why_gmtime` calls `gmtime_r` into a task-local `tm`.
/// - Host forwarding keeps libc `gmtime()` storage semantics instead of BadgeVMS's per-task struct.
pub unsafe extern "C" fn gmtime(timer: *const time_t) -> *mut tm {
    call_resolved!(runtime::real_gmtime, timer)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn gmtime_r(timer: *const time_t, result: *mut tm) -> *mut tm {
    if result.is_null() {
        return core::ptr::null_mut();
    }

    let mut host_result = unsafe { core::mem::zeroed::<libc::tm>() };
    let resolved = call_resolved!(runtime::real_gmtime_r, timer, &mut host_result);

    if resolved.is_null() {
        core::ptr::null_mut()
    } else {
        unsafe { copy_host_tm_into_badge(&mut *result, &host_result) };
        result
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn inet_aton(cp: *const c_char, inp: *mut in_addr) -> c_int {
    call_resolved!(runtime::real_inet_aton, cp, inp)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn inet_ntoa(addr: in_addr) -> *mut c_char {
    call_resolved!(runtime::real_inet_ntoa, addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn iconv_close(cd: iconv_t) -> c_int {
    call_resolved!(runtime::real_iconv_close, cd)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn iconv_open(tocode: *const c_char, fromcode: *const c_char) -> iconv_t {
    call_resolved!(runtime::real_iconv_open, tocode, fromcode)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn isalnum(value: c_int) -> c_int {
    call_resolved!(runtime::real_isalnum, value)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn isalpha(value: c_int) -> c_int {
    call_resolved!(runtime::real_isalpha, value)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn isblank(value: c_int) -> c_int {
    call_resolved!(runtime::real_isblank, value)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn iscntrl(value: c_int) -> c_int {
    call_resolved!(runtime::real_iscntrl, value)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn isdigit(value: c_int) -> c_int {
    call_resolved!(runtime::real_isdigit, value)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn isgraph(value: c_int) -> c_int {
    call_resolved!(runtime::real_isgraph, value)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn islower(value: c_int) -> c_int {
    call_resolved!(runtime::real_islower, value)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn isprint(value: c_int) -> c_int {
    call_resolved!(runtime::real_isprint, value)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ispunct(value: c_int) -> c_int {
    call_resolved!(runtime::real_ispunct, value)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn isatty(fd: c_int) -> c_int {
    call_resolved!(runtime::real_isatty, fd)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn isspace(value: c_int) -> c_int {
    call_resolved!(runtime::real_isspace, value)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn isupper(value: c_int) -> c_int {
    call_resolved!(runtime::real_isupper, value)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn isxdigit(value: c_int) -> c_int {
    call_resolved!(runtime::real_isxdigit, value)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn isascii(value: c_int) -> c_int {
    call_resolved!(runtime::real_isascii, value)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn listen(sockfd: c_int, backlog: c_int) -> c_int {
    call_resolved!(runtime::real_listen, sockfd, backlog)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - Upstream `why_localtime` calls `localtime_r` into a task-local `tm`.
/// - Host forwarding keeps libc `localtime()` storage semantics instead of BadgeVMS's per-task struct.
pub unsafe extern "C" fn localtime(timer: *const time_t) -> *mut tm {
    call_resolved!(runtime::real_localtime, timer)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn localtime_r(timer: *const time_t, result: *mut tm) -> *mut tm {
    if result.is_null() {
        return core::ptr::null_mut();
    }

    let mut host_result = unsafe { core::mem::zeroed::<libc::tm>() };
    let resolved = call_resolved!(runtime::real_localtime_r, timer, &mut host_result);

    if resolved.is_null() {
        core::ptr::null_mut()
    } else {
        unsafe { copy_host_tm_into_badge(&mut *result, &host_result) };
        result
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mktime(timeptr: *mut tm) -> time_t {
    if timeptr.is_null() {
        return call_resolved!(runtime::real_mktime, core::ptr::null_mut());
    }

    let mut host_tm = badge_tm_to_host(unsafe { &*timeptr });
    let result = call_resolved!(runtime::real_mktime, &mut host_tm);
    unsafe { copy_host_tm_into_badge(&mut *timeptr, &host_tm) };
    result
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn clock_gettime(clock_id: clockid_t, tp: *mut timespec) -> c_int {
    if tp.is_null() {
        return call_resolved!(runtime::real_clock_gettime, clock_id, core::ptr::null_mut());
    }

    let mut host_tp = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    let result = call_resolved!(runtime::real_clock_gettime, clock_id, &mut host_tp);

    if result == 0 {
        unsafe { *tp = host_timespec_to_badge(&host_tp) };
    }

    result
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strftime(
    value: *mut c_char,
    maxsize: usize,
    fmt: *const c_char,
    tblock: *const tm,
) -> usize {
    if tblock.is_null() {
        return call_resolved!(
            runtime::real_strftime,
            value,
            maxsize,
            fmt,
            core::ptr::null()
        );
    }

    let host_tm = badge_tm_to_host(unsafe { &*tblock });
    call_resolved!(runtime::real_strftime, value, maxsize, fmt, &host_tm)
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
pub unsafe extern "C" fn strptime(
    input: *const c_char,
    fmt: *const c_char,
    result: *mut tm,
) -> *mut c_char {
    if result.is_null() {
        return call_resolved!(runtime::real_strptime, input, fmt, core::ptr::null_mut());
    }

    let mut host_tm = badge_tm_to_host(unsafe { &*result });
    let parsed = call_resolved!(runtime::real_strptime, input, fmt, &mut host_tm);
    unsafe { copy_host_tm_into_badge(&mut *result, &host_tm) };
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
pub unsafe extern "C" fn wcsftime(
    value: *mut wchar_t,
    maxsize: usize,
    fmt: *const wchar_t,
    tblock: *const tm,
) -> usize {
    if tblock.is_null() {
        return call_resolved!(
            runtime::real_wcsftime,
            value,
            maxsize,
            fmt,
            core::ptr::null()
        );
    }

    let host_tm = badge_tm_to_host(unsafe { &*tblock });
    call_resolved!(runtime::real_wcsftime, value, maxsize, fmt, &host_tm)
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
pub unsafe extern "C" fn memccpy(
    dst: *mut c_void,
    src: *const c_void,
    needle: c_int,
    count: c_uint,
) -> *mut c_void {
    call_resolved!(runtime::real_memccpy, dst, src, needle, count)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memchr(value: *const c_void, needle: c_int, count: c_uint) -> *mut c_void {
    call_resolved!(runtime::real_memchr, value, needle, count)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcmp(left: *const c_void, right: *const c_void, count: c_uint) -> c_int {
    call_resolved!(runtime::real_memcmp, left, right, count)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(
    dst: *mut c_void,
    src: *const c_void,
    count: c_uint,
) -> *mut c_void {
    call_resolved!(runtime::real_memcpy, dst, src, count)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memmem(
    haystack: *const c_void,
    haystack_len: usize,
    needle: *const c_void,
    needle_len: usize,
) -> *mut c_void {
    call_resolved!(
        runtime::real_memmem,
        haystack,
        haystack_len,
        needle,
        needle_len
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memmove(
    dst: *mut c_void,
    src: *const c_void,
    count: c_uint,
) -> *mut c_void {
    call_resolved!(runtime::real_memmove, dst, src, count)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mempcpy(
    dst: *mut c_void,
    src: *const c_void,
    count: c_uint,
) -> *mut c_void {
    call_resolved!(runtime::real_mempcpy, dst, src, count)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memrchr(value: *const c_void, needle: c_int, count: usize) -> *mut c_void {
    call_resolved!(runtime::real_memrchr, value, needle, count)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(dst: *mut c_void, value: c_int, count: c_uint) -> *mut c_void {
    call_resolved!(runtime::real_memset, dst, value, count)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rawmemchr(value: *const c_void, needle: c_int) -> *mut c_void {
    call_resolved!(runtime::real_rawmemchr, value, needle)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lseek(fd: c_int, offset: off_t, whence: c_int) -> off_t {
    call_resolved!(runtime::real_lseek, fd, offset, whence)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mkdir(path: *const c_char, mode: mode_t) -> c_int {
    call_resolved!(runtime::real_mkdir, path, mode)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn open(path: *const c_char, flags: c_int, mut args: ...) -> c_int {
    let mode = if flags & (libc::O_CREAT | libc::O_TMPFILE) != 0 {
        let mode: mode_t = unsafe { args.arg() };
        mode
    } else {
        0
    };

    unsafe { runtime::open_impl(path, flags, mode) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn opendir(name: *const c_char) -> *mut DIR {
    call_resolved!(runtime::real_opendir, name)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn putchar(value: c_int) -> c_int {
    call_resolved!(runtime::real_putchar, value)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn puts(value: *const c_char) -> c_int {
    call_resolved!(runtime::real_puts, value)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - Upstream `why_rand` uses `rand_r(&task_info->seed)`, so RNG state is per-task.
/// - Host forwarding keeps libc `rand()` semantics and its process-global RNG state.
pub unsafe extern "C" fn rand() -> c_int {
    call_resolved!(runtime::real_rand)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - Upstream `why_random` is just task-local `rand_r` widened to `long`.
/// - Host forwarding keeps libc `random()` semantics, which may use a different generator and global state.
pub unsafe extern "C" fn random() -> c_long {
    call_resolved!(runtime::real_random)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn read(fd: c_int, buf: *mut c_void, count: usize) -> isize {
    call_resolved!(runtime::real_read, fd, buf, count)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn readdir(dir: *mut DIR) -> *mut crate::dirent {
    call_resolved!(runtime::real_readdir, dir)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn regcomp(
    preg: *mut regex_t,
    pattern: *const c_char,
    cflags: c_int,
) -> c_int {
    unsafe { runtime::regcomp_impl(preg, pattern, cflags) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn regerror(
    errcode: c_int,
    preg: *const regex_t,
    errbuf: *mut c_char,
    errbuf_size: usize,
) -> usize {
    unsafe { runtime::regerror_impl(errcode, preg, errbuf, errbuf_size) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn regexec(
    preg: *const regex_t,
    text: *const c_char,
    nmatch: usize,
    pmatch: *mut [crate::regmatch_t; 0usize],
    eflags: c_int,
) -> c_int {
    unsafe { runtime::regexec_impl(preg, text, nmatch, pmatch, eflags) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn regfree(preg: *mut regex_t) {
    unsafe { runtime::regfree_impl(preg) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn remove(path: *const c_char) -> c_int {
    call_resolved!(runtime::real_remove, path)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rename(old: *const c_char, new: *const c_char) -> c_int {
    call_resolved!(runtime::real_rename, old, new)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rewind(stream: *mut FILE) {
    call_resolved!(runtime::real_rewind, stream)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rewinddir(dir: *mut DIR) {
    call_resolved!(runtime::real_rewinddir, dir)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmdir(path: *const c_char) -> c_int {
    call_resolved!(runtime::real_rmdir, path)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn setbuf(stream: *mut FILE, buf: *mut c_char) {
    call_resolved!(runtime::real_setbuf, stream, buf)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn setbuffer(stream: *mut FILE, buf: *mut c_char, size: usize) {
    call_resolved!(runtime::real_setbuffer, stream, buf, size)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn setlinebuf(stream: *mut FILE) {
    call_resolved!(runtime::real_setlinebuf, stream)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn setvbuf(
    stream: *mut FILE,
    buf: *mut c_char,
    mode: c_int,
    size: usize,
) -> c_int {
    call_resolved!(runtime::real_setvbuf, stream, buf, mode, size)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn select(
    n: c_int,
    readfds: *mut fd_set,
    writefds: *mut fd_set,
    exceptfds: *mut fd_set,
    timeout: *mut timeval,
) -> c_int {
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

    let result = call_resolved!(
        runtime::real_select,
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
    );

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
pub unsafe extern "C" fn socket(domain: c_int, ty: c_int, protocol: c_int) -> c_int {
    call_resolved!(runtime::real_socket, domain, ty, protocol)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - Upstream `why_srand` only seeds `task_info->seed` for the current task.
/// - Host forwarding seeds libc's global `rand()` state instead.
pub unsafe extern "C" fn srand(seed: c_uint) {
    call_resolved!(runtime::real_srand, seed)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - Upstream `why_srandom` is another setter for the same task-local `seed` used by `why_random`.
/// - Host forwarding seeds libc's global `random()` state instead.
pub unsafe extern "C" fn srandom(seed: c_uint) {
    call_resolved!(runtime::real_srandom, seed)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn stat(path: *const c_char, buf: *mut stat_t) -> c_int {
    call_resolved!(runtime::real_stat, path, buf)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn stpcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char {
    call_resolved!(runtime::real_stpcpy, dst, src)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn stpncpy(
    dst: *mut c_char,
    src: *const c_char,
    count: c_uint,
) -> *mut c_char {
    call_resolved!(runtime::real_stpncpy, dst, src, count)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strcasestr(haystack: *const c_char, needle: *const c_char) -> *mut c_char {
    call_resolved!(runtime::real_strcasestr, haystack, needle)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strcat(dst: *mut c_char, src: *const c_char) -> *mut c_char {
    call_resolved!(runtime::real_strcat, dst, src)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strchr(value: *const c_char, needle: c_int) -> *mut c_char {
    call_resolved!(runtime::real_strchr, value, needle)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strchrnul(value: *const c_char, needle: c_int) -> *mut c_char {
    call_resolved!(runtime::real_strchrnul, value, needle)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strcmp(left: *const c_char, right: *const c_char) -> c_int {
    call_resolved!(runtime::real_strcmp, left, right)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char {
    call_resolved!(runtime::real_strcpy, dst, src)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strcspn(value: *const c_char, reject: *const c_char) -> c_uint {
    call_resolved!(runtime::real_strcspn, value, reject)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - Upstream `why_strdup` returns `NULL` for `NULL` input; canonical libc leaves that undefined.
/// - Upstream allocates with `why_malloc`; host forwarding allocates from libc and must be freed with host `free()`.
pub unsafe extern "C" fn strdup(value: *const c_char) -> *mut c_char {
    call_resolved!(runtime::real_strdup, value)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - Upstream `why_strerror` formats into a task-local buffer via `strerror_r`.
/// - Host forwarding keeps libc `strerror()` storage semantics instead of BadgeVMS's per-task buffer.
pub unsafe extern "C" fn strerror(errnum: c_int) -> *mut c_char {
    call_resolved!(runtime::real_strerror, errnum)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_strerror_r`; badge callers mostly use `why_strerror` instead.
/// - These bindings expose the GNU `char *` form, so host forwarding matches glibc rather than the POSIX `int` variant some libcs provide.
pub unsafe extern "C" fn strerror_r(errnum: c_int, buf: *mut c_char, size: usize) -> *mut c_char {
    call_resolved!(runtime::real_strerror_r, errnum, buf, size)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_strlcat`; badge behavior comes straight from whatever libc exports this BSD extension.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn strlcat(dst: *mut c_char, src: *const c_char, size: c_uint) -> c_uint {
    call_resolved!(runtime::real_strlcat, dst, src, size)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strlen(value: *const c_char) -> c_uint {
    call_resolved!(runtime::real_strlen, value)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_strlcpy`; badge behavior comes straight from whatever libc exports this BSD extension.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn strlcpy(dst: *mut c_char, src: *const c_char, size: c_uint) -> c_uint {
    call_resolved!(runtime::real_strlcpy, dst, src, size)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strncat(
    dst: *mut c_char,
    src: *const c_char,
    count: c_uint,
) -> *mut c_char {
    call_resolved!(runtime::real_strncat, dst, src, count)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strncmp(
    left: *const c_char,
    right: *const c_char,
    count: c_uint,
) -> c_int {
    call_resolved!(runtime::real_strncmp, left, right, count)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strncpy(
    dst: *mut c_char,
    src: *const c_char,
    count: c_uint,
) -> *mut c_char {
    call_resolved!(runtime::real_strncpy, dst, src, count)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - Upstream `why_strndup` returns `NULL` for `NULL` input; canonical libc leaves that undefined.
/// - Upstream allocates with `why_malloc`; host forwarding allocates from libc and must be freed with host `free()`.
pub unsafe extern "C" fn strndup(value: *const c_char, count: c_uint) -> *mut c_char {
    call_resolved!(runtime::real_strndup, value, count)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strnlen(value: *const c_char, count: usize) -> usize {
    call_resolved!(runtime::real_strnlen, value, count)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strpbrk(value: *const c_char, accept: *const c_char) -> *mut c_char {
    call_resolved!(runtime::real_strpbrk, value, accept)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strrchr(value: *const c_char, needle: c_int) -> *mut c_char {
    call_resolved!(runtime::real_strrchr, value, needle)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strsep(stringp: *mut *mut c_char, delim: *const c_char) -> *mut c_char {
    call_resolved!(runtime::real_strsep, stringp, delim)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strspn(value: *const c_char, accept: *const c_char) -> c_uint {
    call_resolved!(runtime::real_strspn, value, accept)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char {
    call_resolved!(runtime::real_strstr, haystack, needle)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - Upstream `why_strtok` is a task-local wrapper around `strtok_r` using `task_info->strtok_saveptr`.
/// - Host forwarding keeps libc `strtok()` semantics and its hidden tokenizer state instead.
pub unsafe extern "C" fn strtok(value: *mut c_char, delim: *const c_char) -> *mut c_char {
    call_resolved!(runtime::real_strtok, value, delim)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strtok_r(
    value: *mut c_char,
    delim: *const c_char,
    saveptr: *mut *mut c_char,
) -> *mut c_char {
    call_resolved!(runtime::real_strtok_r, value, delim, saveptr)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_strverscmp`; badge behavior comes straight from the badge libc's GNU extension.
/// - Host forwarding matches host glibc's `strverscmp()` semantics and availability instead of a badge-specific wrapper.
pub unsafe extern "C" fn strverscmp(left: *const c_char, right: *const c_char) -> c_int {
    call_resolved!(runtime::real_strverscmp, left, right)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - Upstream `why_system` is a stub that returns `0` without executing anything.
/// - Host forwarding uses libc `system()` and therefore runs a real host shell command.
pub unsafe extern "C" fn system(command: *const c_char) -> c_int {
    call_resolved!(runtime::real_system, command)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - Upstream `why_wcsdup` returns `NULL` for `NULL` input; canonical libc leaves that undefined.
/// - Upstream allocates with `why_malloc`; host forwarding allocates from libc and must be freed with host `free()`.
pub unsafe extern "C" fn wcsdup(value: *const wchar_t) -> *mut wchar_t {
    call_resolved!(runtime::real_wcsdup, value)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_wcschr`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn wcschr(value: *const wchar_t, needle: c_int) -> *mut wchar_t {
    call_resolved!(runtime::real_wcschr, value, needle)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_wcscat`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn wcscat(dst: *mut wchar_t, src: *const wchar_t) -> *mut wchar_t {
    call_resolved!(runtime::real_wcscat, dst, src)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_wcscasecmp`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability and host locale handling rather than a badge-specific shim.
pub unsafe extern "C" fn wcscasecmp(left: *const wchar_t, right: *const wchar_t) -> c_int {
    call_resolved!(runtime::real_wcscasecmp, left, right)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_wcscmp`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn wcscmp(left: *const wchar_t, right: *const wchar_t) -> c_int {
    call_resolved!(runtime::real_wcscmp, left, right)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_wcscpy`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn wcscpy(dst: *mut wchar_t, src: *const wchar_t) -> *mut wchar_t {
    call_resolved!(runtime::real_wcscpy, dst, src)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_wcscspn`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn wcscspn(value: *const wchar_t, reject: *const wchar_t) -> usize {
    call_resolved!(runtime::real_wcscspn, value, reject)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_wcslen`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn wcslen(value: *const wchar_t) -> c_uint {
    call_resolved!(runtime::real_wcslen, value)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_wcsncasecmp`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability and host locale handling rather than a badge-specific shim.
pub unsafe extern "C" fn wcsncasecmp(
    left: *const wchar_t,
    right: *const wchar_t,
    count: usize,
) -> c_int {
    call_resolved!(runtime::real_wcsncasecmp, left, right, count)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_wcsncmp`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn wcsncmp(
    left: *const wchar_t,
    right: *const wchar_t,
    count: c_uint,
) -> c_int {
    call_resolved!(runtime::real_wcsncmp, left, right, count)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_wcsncpy`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn wcsncpy(
    dst: *mut wchar_t,
    src: *const wchar_t,
    count: usize,
) -> *mut wchar_t {
    call_resolved!(runtime::real_wcsncpy, dst, src, count)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_wcsnlen`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn wcsnlen(value: *const wchar_t, count: usize) -> usize {
    call_resolved!(runtime::real_wcsnlen, value, count)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_wcsncat`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn wcsncat(
    dst: *mut wchar_t,
    src: *const wchar_t,
    count: usize,
) -> *mut wchar_t {
    call_resolved!(runtime::real_wcsncat, dst, src, count)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_wcspbrk`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn wcspbrk(value: *const wchar_t, accept: *const wchar_t) -> *mut wchar_t {
    call_resolved!(runtime::real_wcspbrk, value, accept)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_wcsrchr`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn wcsrchr(value: *const wchar_t, needle: wchar_t) -> *mut wchar_t {
    call_resolved!(runtime::real_wcsrchr, value, needle)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_wcsspn`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn wcsspn(value: *const wchar_t, accept: *const wchar_t) -> usize {
    call_resolved!(runtime::real_wcsspn, value, accept)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_wcsstr`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn wcsstr(haystack: *const wchar_t, needle: *const wchar_t) -> *mut wchar_t {
    call_resolved!(runtime::real_wcsstr, haystack, needle)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_wcswidth`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability and host width tables rather than a badge-specific shim.
pub unsafe extern "C" fn wcswidth(value: *const wchar_t, count: usize) -> c_int {
    call_resolved!(runtime::real_wcswidth, value, count)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_wcstok`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn wcstok(
    value: *mut wchar_t,
    delim: *const wchar_t,
    saveptr: *mut *mut wchar_t,
) -> *mut wchar_t {
    call_resolved!(runtime::real_wcstok, value, delim, saveptr)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_wmemcmp`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn wmemcmp(
    left: *const wchar_t,
    right: *const wchar_t,
    count: c_uint,
) -> c_int {
    call_resolved!(runtime::real_wmemcmp, left, right, count)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_wmemchr`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn wmemchr(
    value: *const wchar_t,
    needle: c_int,
    count: c_uint,
) -> *mut wchar_t {
    call_resolved!(runtime::real_wmemchr, value, needle, count)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_wmemcpy`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn wmemcpy(
    dst: *mut wchar_t,
    src: *const wchar_t,
    count: c_uint,
) -> *mut wchar_t {
    call_resolved!(runtime::real_wmemcpy, dst, src, count)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_wmemmove`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn wmemmove(
    dst: *mut wchar_t,
    src: *const wchar_t,
    count: c_uint,
) -> *mut wchar_t {
    call_resolved!(runtime::real_wmemmove, dst, src, count)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_wmempcpy`; badge behavior comes straight from the badge libc's wide-memory extension.
/// - Host forwarding matches host libc availability for that extension instead of a badge-specific shim.
pub unsafe extern "C" fn wmempcpy(
    dst: *mut wchar_t,
    src: *const wchar_t,
    count: usize,
) -> *mut wchar_t {
    call_resolved!(runtime::real_wmempcpy, dst, src, count)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_wmemset`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn wmemset(dst: *mut wchar_t, value: wchar_t, count: usize) -> *mut wchar_t {
    call_resolved!(runtime::real_wmemset, dst, value, count)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_iswalnum`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn iswalnum(value: wint_t) -> c_int {
    call_resolved!(runtime::real_iswalnum, value)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_wctob`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability and host multibyte conversion rules rather than a badge-specific shim.
pub unsafe extern "C" fn wctob(value: wint_t) -> c_int {
    call_resolved!(runtime::real_wctob, value)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_wcwidth`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability and host width tables rather than a badge-specific shim.
pub unsafe extern "C" fn wcwidth(value: wchar_t) -> c_int {
    call_resolved!(runtime::real_wcwidth, value)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_iswalpha`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn iswalpha(value: wint_t) -> c_int {
    call_resolved!(runtime::real_iswalpha, value)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_iswblank`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn iswblank(value: wint_t) -> c_int {
    call_resolved!(runtime::real_iswblank, value)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_iswcntrl`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn iswcntrl(value: wint_t) -> c_int {
    call_resolved!(runtime::real_iswcntrl, value)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_iswctype`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn iswctype(value: wint_t, desc: wctype_t) -> c_int {
    call_resolved!(runtime::real_iswctype, value, desc)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_iswdigit`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn iswdigit(value: wint_t) -> c_int {
    call_resolved!(runtime::real_iswdigit, value)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_iswgraph`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn iswgraph(value: wint_t) -> c_int {
    call_resolved!(runtime::real_iswgraph, value)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_iswlower`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn iswlower(value: wint_t) -> c_int {
    call_resolved!(runtime::real_iswlower, value)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_iswprint`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn iswprint(value: wint_t) -> c_int {
    call_resolved!(runtime::real_iswprint, value)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_iswpunct`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn iswpunct(value: wint_t) -> c_int {
    call_resolved!(runtime::real_iswpunct, value)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_iswspace`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn iswspace(value: wint_t) -> c_int {
    call_resolved!(runtime::real_iswspace, value)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_iswupper`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn iswupper(value: wint_t) -> c_int {
    call_resolved!(runtime::real_iswupper, value)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_iswxdigit`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn iswxdigit(value: wint_t) -> c_int {
    call_resolved!(runtime::real_iswxdigit, value)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_towlower`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn towlower(value: wint_t) -> wint_t {
    call_resolved!(runtime::real_towlower, value)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_towupper`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn towupper(value: wint_t) -> wint_t {
    call_resolved!(runtime::real_towupper, value)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_wctrans`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn wctrans(name: *const c_char) -> wctrans_t {
    call_resolved!(runtime::real_wctrans, name)
}

#[unsafe(no_mangle)]
/// Differences from upstream BadgeVMS:
/// - The vendored firmware tree has no project-local `why_wctype`; badge behavior comes straight from the badge libc wide-char implementation.
/// - Host forwarding therefore depends on host-libc availability rather than a badge-specific shim.
pub unsafe extern "C" fn wctype(name: *const c_char) -> wctype_t {
    call_resolved!(runtime::real_wctype, name)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tcgetattr(fd: c_int, termios_p: *mut termios) -> c_int {
    call_resolved!(runtime::real_tcgetattr, fd, termios_p)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tcsetattr(fd: c_int, action: c_int, termios_p: *const termios) -> c_int {
    call_resolved!(runtime::real_tcsetattr, fd, action, termios_p)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tolower(value: c_int) -> c_int {
    call_resolved!(runtime::real_tolower, value)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn toascii(value: c_int) -> c_int {
    call_resolved!(runtime::real_toascii, value)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn toupper(value: c_int) -> c_int {
    call_resolved!(runtime::real_toupper, value)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ungetc(value: c_int, stream: *mut FILE) -> c_int {
    call_resolved!(runtime::real_ungetc, value, stream)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn unlink(path: *const c_char) -> c_int {
    call_resolved!(runtime::real_unlink, path)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn write(fd: c_int, buf: *const c_void, count: usize) -> isize {
    call_resolved!(runtime::real_write, fd, buf, count)
}
