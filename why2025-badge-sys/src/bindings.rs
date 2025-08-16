use crate::types::*;

unsafe extern "C" {
    pub fn cacos(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn cacosf(arg1: __BindgenComplex<f32>) -> __BindgenComplex<f32>;
    pub fn casin(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn casinf(arg1: __BindgenComplex<f32>) -> __BindgenComplex<f32>;
    pub fn catan(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn catanf(arg1: __BindgenComplex<f32>) -> __BindgenComplex<f32>;
    pub fn ccos(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn ccosf(arg1: __BindgenComplex<f32>) -> __BindgenComplex<f32>;
    pub fn csin(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn csinf(arg1: __BindgenComplex<f32>) -> __BindgenComplex<f32>;
    pub fn ctan(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn ctanf(arg1: __BindgenComplex<f32>) -> __BindgenComplex<f32>;
    pub fn cacosh(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn cacoshf(arg1: __BindgenComplex<f32>) -> __BindgenComplex<f32>;
    pub fn casinh(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn casinhf(arg1: __BindgenComplex<f32>) -> __BindgenComplex<f32>;
    pub fn catanh(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn catanhf(arg1: __BindgenComplex<f32>) -> __BindgenComplex<f32>;
    pub fn ccosh(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn ccoshf(arg1: __BindgenComplex<f32>) -> __BindgenComplex<f32>;
    pub fn csinh(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn csinhf(arg1: __BindgenComplex<f32>) -> __BindgenComplex<f32>;
    pub fn ctanh(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn ctanhf(arg1: __BindgenComplex<f32>) -> __BindgenComplex<f32>;
    pub fn cexp(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn cexpf(arg1: __BindgenComplex<f32>) -> __BindgenComplex<f32>;
    pub fn clog(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn clogf(arg1: __BindgenComplex<f32>) -> __BindgenComplex<f32>;
    pub fn cabs(arg1: __BindgenComplex<f64>) -> f64;
    pub fn cabsf(arg1: __BindgenComplex<f32>) -> f32;
    pub fn cpow(arg1: __BindgenComplex<f64>, arg2: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn cpowf(arg1: __BindgenComplex<f32>, arg2: __BindgenComplex<f32>)
    -> __BindgenComplex<f32>;
    pub fn csqrt(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn csqrtf(arg1: __BindgenComplex<f32>) -> __BindgenComplex<f32>;
    pub fn carg(arg1: __BindgenComplex<f64>) -> f64;
    pub fn cargf(arg1: __BindgenComplex<f32>) -> f32;
    pub fn cimag(arg1: __BindgenComplex<f64>) -> f64;
    pub fn cimagf(arg1: __BindgenComplex<f32>) -> f32;
    pub fn conj(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn conjf(arg1: __BindgenComplex<f32>) -> __BindgenComplex<f32>;
    pub fn cproj(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn cprojf(arg1: __BindgenComplex<f32>) -> __BindgenComplex<f32>;
    pub fn creal(arg1: __BindgenComplex<f64>) -> f64;
    pub fn crealf(arg1: __BindgenComplex<f32>) -> f32;
    pub fn clog10(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn clog10f(arg1: __BindgenComplex<f32>) -> __BindgenComplex<f32>;
    pub fn csqrtl(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn cabsl(arg1: __BindgenComplex<f64>) -> u128;
    pub fn cprojl(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn creall(arg1: __BindgenComplex<f64>) -> u128;
    pub fn conjl(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn cimagl(arg1: __BindgenComplex<f64>) -> u128;
    pub fn cargl(arg1: __BindgenComplex<f64>) -> u128;
    pub fn casinl(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn cacosl(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn catanl(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn ccosl(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn csinl(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn ctanl(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn cacoshl(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn casinhl(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn catanhl(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn ccoshl(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn csinhl(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn ctanhl(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn cexpl(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn clogl(arg1: __BindgenComplex<f64>) -> __BindgenComplex<f64>;
    pub fn cpowl(arg1: __BindgenComplex<f64>, arg2: __BindgenComplex<f64>)
    -> __BindgenComplex<f64>;
    pub fn isalnum(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn isalpha(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn iscntrl(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn isdigit(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn isgraph(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn islower(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn isprint(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn ispunct(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn isspace(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn isupper(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn isxdigit(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn tolower(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn toupper(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn isblank(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn isascii(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn toascii(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn isalnum_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int;
    pub fn isalpha_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int;
    pub fn isblank_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int;
    pub fn iscntrl_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int;
    pub fn isdigit_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int;
    pub fn isgraph_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int;
    pub fn islower_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int;
    pub fn isprint_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int;
    pub fn ispunct_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int;
    pub fn isspace_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int;
    pub fn isupper_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int;
    pub fn isxdigit_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int;
    pub fn tolower_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int;
    pub fn toupper_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int;
    pub fn isascii_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int;
    pub fn toascii_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int;
    pub static _ctype_b: [::core::ffi::c_char; 0usize];
    pub fn feclearexcept(excepts: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn fegetexceptflag(
        flagp: *mut fexcept_t,
        excepts: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn fesetexceptflag(
        flagp: *const fexcept_t,
        excepts: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn feraiseexcept(excepts: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn fetestexcept(excepts: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn fegetround() -> ::core::ffi::c_int;
    pub fn fesetround(rounding_mode: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn fegetenv(envp: *mut fenv_t) -> ::core::ffi::c_int;
    pub fn feholdexcept(envp: *mut fenv_t) -> ::core::ffi::c_int;
    pub fn fesetenv(envp: *const fenv_t) -> ::core::ffi::c_int;
    pub fn feupdateenv(envp: *const fenv_t) -> ::core::ffi::c_int;
    pub fn fnmatch(
        arg1: *const ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
        arg3: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn getopt(
        __argc: ::core::ffi::c_int,
        __argv: *const [*mut ::core::ffi::c_char; 0usize],
        __optstring: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    pub fn getopt_long(
        __argc: ::core::ffi::c_int,
        __argv: *const [*mut ::core::ffi::c_char; 0usize],
        __shortopts: *const ::core::ffi::c_char,
        __longopts: *const option,
        __longind: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn getopt_long_only(
        __argc: ::core::ffi::c_int,
        __argv: *const [*mut ::core::ffi::c_char; 0usize],
        __shortopts: *const ::core::ffi::c_char,
        __longopts: *const option,
        __longind: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn iconv_open(
        arg1: *const ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
    ) -> iconv_t;
    pub fn iconv(
        arg1: iconv_t,
        arg2: *mut *mut ::core::ffi::c_char,
        arg3: *mut size_t,
        arg4: *mut *mut ::core::ffi::c_char,
        arg5: *mut size_t,
    ) -> size_t;
    pub fn iconv_close(arg1: iconv_t) -> ::core::ffi::c_int;
    pub fn fpgetround() -> fp_rnd;
    pub fn fpsetround(arg1: fp_rnd) -> fp_rnd;
    pub fn fpgetmask() -> fp_except;
    pub fn fpsetmask(arg1: fp_except) -> fp_except;
    pub fn fpgetsticky() -> fp_except;
    pub fn fpsetsticky(arg1: fp_except) -> fp_except;
    pub fn imaxabs(arg1: intmax_t) -> intmax_t;
    pub fn imaxdiv(__numer: intmax_t, __denomer: intmax_t) -> imaxdiv_t;
    pub fn strtoimax(
        arg1: *const ::core::ffi::c_char,
        arg2: *mut *mut ::core::ffi::c_char,
        arg3: ::core::ffi::c_int,
    ) -> intmax_t;
    pub fn strtoumax(
        arg1: *const ::core::ffi::c_char,
        arg2: *mut *mut ::core::ffi::c_char,
        arg3: ::core::ffi::c_int,
    ) -> uintmax_t;
    pub fn wcstoimax(
        arg1: *const _wchar_t,
        arg2: *mut *mut _wchar_t,
        arg3: ::core::ffi::c_int,
    ) -> intmax_t;
    pub fn wcstoumax(
        arg1: *const _wchar_t,
        arg2: *mut *mut _wchar_t,
        arg3: ::core::ffi::c_int,
    ) -> uintmax_t;
    pub fn strtoimax_l(
        arg1: *const ::core::ffi::c_char,
        _restrict: *mut *mut ::core::ffi::c_char,
        arg2: ::core::ffi::c_int,
        arg3: locale_t,
    ) -> intmax_t;
    pub fn strtoumax_l(
        arg1: *const ::core::ffi::c_char,
        _restrict: *mut *mut ::core::ffi::c_char,
        arg2: ::core::ffi::c_int,
        arg3: locale_t,
    ) -> uintmax_t;
    pub fn wcstoimax_l(
        arg1: *const _wchar_t,
        _restrict: *mut *mut _wchar_t,
        arg2: ::core::ffi::c_int,
        arg3: locale_t,
    ) -> intmax_t;
    pub fn wcstoumax_l(
        arg1: *const _wchar_t,
        _restrict: *mut *mut _wchar_t,
        arg2: ::core::ffi::c_int,
        arg3: locale_t,
    ) -> uintmax_t;
    pub fn nl_langinfo(arg1: nl_item) -> *mut ::core::ffi::c_char;
    pub fn nl_langinfo_l(arg1: nl_item, arg2: locale_t) -> *mut ::core::ffi::c_char;
    pub fn localeconv() -> *mut lconv;
    pub fn uselocale(arg1: locale_t) -> locale_t;
    pub fn atan(arg1: f64) -> f64;
    pub fn cos(arg1: f64) -> f64;
    pub fn sin(arg1: f64) -> f64;
    pub fn tan(arg1: f64) -> f64;
    pub fn tanh(arg1: f64) -> f64;
    pub fn frexp(arg1: f64, arg2: *mut ::core::ffi::c_int) -> f64;
    pub fn modf(arg1: f64, arg2: *mut f64) -> f64;
    pub fn ceil(arg1: f64) -> f64;
    pub fn fabs(arg1: f64) -> f64;
    pub fn floor(arg1: f64) -> f64;
    pub fn acos(arg1: f64) -> f64;
    pub fn asin(arg1: f64) -> f64;
    pub fn atan2(arg1: f64, arg2: f64) -> f64;
    pub fn cosh(arg1: f64) -> f64;
    pub fn sinh(arg1: f64) -> f64;
    pub fn exp(arg1: f64) -> f64;
    pub fn ldexp(arg1: f64, arg2: ::core::ffi::c_int) -> f64;
    pub fn log(arg1: f64) -> f64;
    pub fn log10(arg1: f64) -> f64;
    pub fn pow(arg1: f64, arg2: f64) -> f64;
    pub fn sqrt(arg1: f64) -> f64;
    pub fn fmod(arg1: f64, arg2: f64) -> f64;
    pub fn finite(arg1: f64) -> ::core::ffi::c_int;
    pub fn finitef(arg1: f32) -> ::core::ffi::c_int;
    pub fn isinf(arg1: f64) -> ::core::ffi::c_int;
    pub fn isinff(arg1: f32) -> ::core::ffi::c_int;
    pub fn isnan(arg1: f64) -> ::core::ffi::c_int;
    pub fn isnanf(arg1: f32) -> ::core::ffi::c_int;
    pub fn finitel(arg1: u128) -> ::core::ffi::c_int;
    pub fn infinity() -> f64;
    pub fn nan(arg1: *const ::core::ffi::c_char) -> f64;
    pub fn copysign(arg1: f64, arg2: f64) -> f64;
    pub fn logb(arg1: f64) -> f64;
    pub fn ilogb(arg1: f64) -> ::core::ffi::c_int;
    pub fn asinh(arg1: f64) -> f64;
    pub fn cbrt(arg1: f64) -> f64;
    pub fn nextafter(arg1: f64, arg2: f64) -> f64;
    pub fn rint(arg1: f64) -> f64;
    pub fn scalbn(arg1: f64, arg2: ::core::ffi::c_int) -> f64;
    pub fn exp2(arg1: f64) -> f64;
    pub fn scalbln(arg1: f64, arg2: ::core::ffi::c_long) -> f64;
    pub fn tgamma(arg1: f64) -> f64;
    pub fn nearbyint(arg1: f64) -> f64;
    pub fn lrint(arg1: f64) -> ::core::ffi::c_long;
    pub fn llrint(arg1: f64) -> ::core::ffi::c_longlong;
    pub fn round(arg1: f64) -> f64;
    pub fn lround(arg1: f64) -> ::core::ffi::c_long;
    pub fn llround(arg1: f64) -> ::core::ffi::c_longlong;
    pub fn trunc(arg1: f64) -> f64;
    pub fn remquo(arg1: f64, arg2: f64, arg3: *mut ::core::ffi::c_int) -> f64;
    pub fn fdim(arg1: f64, arg2: f64) -> f64;
    pub fn fmax(arg1: f64, arg2: f64) -> f64;
    pub fn fmin(arg1: f64, arg2: f64) -> f64;
    pub fn fma(arg1: f64, arg2: f64, arg3: f64) -> f64;
    pub fn log1p(arg1: f64) -> f64;
    pub fn expm1(arg1: f64) -> f64;
    pub fn acosh(arg1: f64) -> f64;
    pub fn atanh(arg1: f64) -> f64;
    pub fn remainder(arg1: f64, arg2: f64) -> f64;
    pub fn gamma(arg1: f64) -> f64;
    pub fn lgamma(arg1: f64) -> f64;
    pub fn erf(arg1: f64) -> f64;
    pub fn erfc(arg1: f64) -> f64;
    pub fn log2(arg1: f64) -> f64;
    pub fn hypot(arg1: f64, arg2: f64) -> f64;
    pub fn atanf(arg1: f32) -> f32;
    pub fn cosf(arg1: f32) -> f32;
    pub fn sinf(arg1: f32) -> f32;
    pub fn tanf(arg1: f32) -> f32;
    pub fn tanhf(arg1: f32) -> f32;
    pub fn frexpf(arg1: f32, arg2: *mut ::core::ffi::c_int) -> f32;
    pub fn modff(arg1: f32, arg2: *mut f32) -> f32;
    pub fn ceilf(arg1: f32) -> f32;
    pub fn fabsf(arg1: f32) -> f32;
    pub fn floorf(arg1: f32) -> f32;
    pub fn acosf(arg1: f32) -> f32;
    pub fn asinf(arg1: f32) -> f32;
    pub fn atan2f(arg1: f32, arg2: f32) -> f32;
    pub fn coshf(arg1: f32) -> f32;
    pub fn sinhf(arg1: f32) -> f32;
    pub fn expf(arg1: f32) -> f32;
    pub fn ldexpf(arg1: f32, arg2: ::core::ffi::c_int) -> f32;
    pub fn logf(arg1: f32) -> f32;
    pub fn log10f(arg1: f32) -> f32;
    pub fn powf(arg1: f32, arg2: f32) -> f32;
    pub fn sqrtf(arg1: f32) -> f32;
    pub fn fmodf(arg1: f32, arg2: f32) -> f32;
    pub fn exp2f(arg1: f32) -> f32;
    pub fn scalblnf(arg1: f32, arg2: ::core::ffi::c_long) -> f32;
    pub fn tgammaf(arg1: f32) -> f32;
    pub fn nearbyintf(arg1: f32) -> f32;
    pub fn lrintf(arg1: f32) -> ::core::ffi::c_long;
    pub fn llrintf(arg1: f32) -> ::core::ffi::c_longlong;
    pub fn roundf(arg1: f32) -> f32;
    pub fn lroundf(arg1: f32) -> ::core::ffi::c_long;
    pub fn llroundf(arg1: f32) -> ::core::ffi::c_longlong;
    pub fn truncf(arg1: f32) -> f32;
    pub fn remquof(arg1: f32, arg2: f32, arg3: *mut ::core::ffi::c_int) -> f32;
    pub fn fdimf(arg1: f32, arg2: f32) -> f32;
    pub fn fmaxf(arg1: f32, arg2: f32) -> f32;
    pub fn fminf(arg1: f32, arg2: f32) -> f32;
    pub fn fmaf(arg1: f32, arg2: f32, arg3: f32) -> f32;
    pub fn infinityf() -> f32;
    pub fn nanf(arg1: *const ::core::ffi::c_char) -> f32;
    pub fn copysignf(arg1: f32, arg2: f32) -> f32;
    pub fn logbf(arg1: f32) -> f32;
    pub fn ilogbf(arg1: f32) -> ::core::ffi::c_int;
    pub fn asinhf(arg1: f32) -> f32;
    pub fn cbrtf(arg1: f32) -> f32;
    pub fn nextafterf(arg1: f32, arg2: f32) -> f32;
    pub fn rintf(arg1: f32) -> f32;
    pub fn scalbnf(arg1: f32, arg2: ::core::ffi::c_int) -> f32;
    pub fn log1pf(arg1: f32) -> f32;
    pub fn expm1f(arg1: f32) -> f32;
    pub fn acoshf(arg1: f32) -> f32;
    pub fn atanhf(arg1: f32) -> f32;
    pub fn remainderf(arg1: f32, arg2: f32) -> f32;
    pub fn gammaf(arg1: f32) -> f32;
    pub fn lgammaf(arg1: f32) -> f32;
    pub fn erff(arg1: f32) -> f32;
    pub fn erfcf(arg1: f32) -> f32;
    pub fn log2f(arg1: f32) -> f32;
    pub fn hypotf(arg1: f32, arg2: f32) -> f32;
    pub fn hypotl(arg1: u128, arg2: u128) -> u128;
    pub fn sqrtl(arg1: u128) -> u128;
    pub fn frexpl(arg1: u128, arg2: *mut ::core::ffi::c_int) -> u128;
    pub fn scalbnl(arg1: u128, arg2: ::core::ffi::c_int) -> u128;
    pub fn scalblnl(arg1: u128, arg2: ::core::ffi::c_long) -> u128;
    pub fn rintl(arg1: u128) -> u128;
    pub fn lrintl(arg1: u128) -> ::core::ffi::c_long;
    pub fn llrintl(arg1: u128) -> ::core::ffi::c_longlong;
    pub fn ilogbl(arg1: u128) -> ::core::ffi::c_int;
    pub fn logbl(arg1: u128) -> u128;
    pub fn ldexpl(arg1: u128, arg2: ::core::ffi::c_int) -> u128;
    pub fn nearbyintl(arg1: u128) -> u128;
    pub fn ceill(arg1: u128) -> u128;
    pub fn fmaxl(arg1: u128, arg2: u128) -> u128;
    pub fn fminl(arg1: u128, arg2: u128) -> u128;
    pub fn roundl(arg1: u128) -> u128;
    pub fn lroundl(arg1: u128) -> ::core::ffi::c_long;
    pub fn llroundl(arg1: u128) -> ::core::ffi::c_longlong;
    pub fn truncl(arg1: u128) -> u128;
    pub fn floorl(arg1: u128) -> u128;
    pub fn fabsl(arg1: u128) -> u128;
    pub fn copysignl(arg1: u128, arg2: u128) -> u128;
    pub fn atanl(arg1: u128) -> u128;
    pub fn cosl(arg1: u128) -> u128;
    pub fn sinl(arg1: u128) -> u128;
    pub fn tanl(arg1: u128) -> u128;
    pub fn tanhl(arg1: u128) -> u128;
    pub fn log1pl(arg1: u128) -> u128;
    pub fn expm1l(arg1: u128) -> u128;
    pub fn acosl(arg1: u128) -> u128;
    pub fn asinl(arg1: u128) -> u128;
    pub fn atan2l(arg1: u128, arg2: u128) -> u128;
    pub fn coshl(arg1: u128) -> u128;
    pub fn sinhl(arg1: u128) -> u128;
    pub fn expl(arg1: u128) -> u128;
    pub fn logl(arg1: u128) -> u128;
    pub fn log10l(arg1: u128) -> u128;
    pub fn powl(arg1: u128, arg2: u128) -> u128;
    pub fn fmodl(arg1: u128, arg2: u128) -> u128;
    pub fn asinhl(arg1: u128) -> u128;
    pub fn cbrtl(arg1: u128) -> u128;
    pub fn nextafterl(arg1: u128, arg2: u128) -> u128;
    pub fn nexttowardf(arg1: f32, arg2: u128) -> f32;
    pub fn nexttoward(arg1: f64, arg2: u128) -> f64;
    pub fn nexttowardl(arg1: u128, arg2: u128) -> u128;
    pub fn log2l(arg1: u128) -> u128;
    pub fn exp2l(arg1: u128) -> u128;
    pub fn tgammal(arg1: u128) -> u128;
    pub fn remquol(arg1: u128, arg2: u128, arg3: *mut ::core::ffi::c_int) -> u128;
    pub fn fdiml(arg1: u128, arg2: u128) -> u128;
    pub fn fmal(arg1: u128, arg2: u128, arg3: u128) -> u128;
    pub fn acoshl(arg1: u128) -> u128;
    pub fn atanhl(arg1: u128) -> u128;
    pub fn remainderl(arg1: u128, arg2: u128) -> u128;
    pub fn lgammal(arg1: u128) -> u128;
    pub fn erfl(arg1: u128) -> u128;
    pub fn erfcl(arg1: u128) -> u128;
    pub fn drem(arg1: f64, arg2: f64) -> f64;
    pub fn dremf(arg1: f32, arg2: f32) -> f32;
    pub fn lgamma_r(arg1: f64, arg2: *mut ::core::ffi::c_int) -> f64;
    pub fn lgammaf_r(arg1: f32, arg2: *mut ::core::ffi::c_int) -> f32;
    pub fn y0(arg1: f64) -> f64;
    pub fn y1(arg1: f64) -> f64;
    pub fn yn(arg1: ::core::ffi::c_int, arg2: f64) -> f64;
    pub fn j0(arg1: f64) -> f64;
    pub fn j1(arg1: f64) -> f64;
    pub fn jn(arg1: ::core::ffi::c_int, arg2: f64) -> f64;
    pub fn y0f(arg1: f32) -> f32;
    pub fn y1f(arg1: f32) -> f32;
    pub fn ynf(arg1: ::core::ffi::c_int, arg2: f32) -> f32;
    pub fn j0f(arg1: f32) -> f32;
    pub fn j1f(arg1: f32) -> f32;
    pub fn jnf(arg1: ::core::ffi::c_int, arg2: f32) -> f32;
    pub fn sincos(arg1: f64, arg2: *mut f64, arg3: *mut f64);
    pub fn sincosf(arg1: f32, arg2: *mut f32, arg3: *mut f32);
    pub fn exp10(arg1: f64) -> f64;
    pub fn pow10(arg1: f64) -> f64;
    pub fn exp10f(arg1: f32) -> f32;
    pub fn pow10f(arg1: f32) -> f32;
    pub fn regcomp(
        arg1: *mut regex_t,
        arg2: *const ::core::ffi::c_char,
        arg3: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn regerror(
        arg1: ::core::ffi::c_int,
        arg2: *const regex_t,
        arg3: *mut ::core::ffi::c_char,
        arg4: size_t,
    ) -> size_t;
    pub fn regexec(
        arg1: *const regex_t,
        arg2: *const ::core::ffi::c_char,
        arg3: size_t,
        arg4: *mut [regmatch_t; 0usize],
        arg5: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn regfree(arg1: *mut regex_t);
    pub fn tdelete(
        arg1: *const ::core::ffi::c_void,
        arg2: *mut *mut ::core::ffi::c_void,
        arg3: __compar_fn_t,
    ) -> *mut ::core::ffi::c_void;
    pub fn tdestroy(
        arg1: *mut ::core::ffi::c_void,
        arg2: ::core::option::Option<unsafe extern "C" fn(arg1: *mut ::core::ffi::c_void)>,
    );
    pub fn tfind(
        arg1: *const ::core::ffi::c_void,
        arg2: *mut *mut ::core::ffi::c_void,
        arg3: __compar_fn_t,
    ) -> *mut ::core::ffi::c_void;
    pub fn tsearch(
        arg1: *const ::core::ffi::c_void,
        arg2: *mut *mut ::core::ffi::c_void,
        arg3: __compar_fn_t,
    ) -> *mut ::core::ffi::c_void;
    pub fn twalk(
        arg1: *const ::core::ffi::c_void,
        arg2: ::core::option::Option<
            unsafe extern "C" fn(
                arg1: *const ::core::ffi::c_void,
                arg2: VISIT,
                arg3: ::core::ffi::c_int,
            ),
        >,
    );
    pub fn longjmp(__jmpb: *mut ::core::ffi::c_longlong, __retval: ::core::ffi::c_int) -> !;
    pub fn setjmp(__jmpb: *mut ::core::ffi::c_longlong) -> ::core::ffi::c_int;
    pub static stdin: *mut FILE;
    pub static stdout: *mut FILE;
    pub static stderr: *mut FILE;
    pub fn fclose(__stream: *mut FILE) -> ::core::ffi::c_int;
    pub fn fflush(stream: *mut FILE) -> ::core::ffi::c_int;
    pub fn fputc(__c: ::core::ffi::c_int, __stream: *mut FILE) -> ::core::ffi::c_int;
    pub fn putchar(__c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn printf(__fmt: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    pub fn fprintf(
        __stream: *mut FILE,
        __fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    pub fn vprintf(
        __fmt: *const ::core::ffi::c_char,
        __ap: __builtin_va_list,
    ) -> ::core::ffi::c_int;
    pub fn vfprintf(
        __stream: *mut FILE,
        __fmt: *const ::core::ffi::c_char,
        __ap: __builtin_va_list,
    ) -> ::core::ffi::c_int;
    pub fn sprintf(
        __s: *mut ::core::ffi::c_char,
        __fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    pub fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __n: ::core::ffi::c_uint,
        __fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    pub fn vsprintf(
        __s: *mut ::core::ffi::c_char,
        __fmt: *const ::core::ffi::c_char,
        ap: __builtin_va_list,
    ) -> ::core::ffi::c_int;
    pub fn vsnprintf(
        __s: *mut ::core::ffi::c_char,
        __n: ::core::ffi::c_uint,
        __fmt: *const ::core::ffi::c_char,
        ap: __builtin_va_list,
    ) -> ::core::ffi::c_int;
    pub fn asprintf(
        strp: *mut *mut ::core::ffi::c_char,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    pub fn asnprintf(
        str_: *mut ::core::ffi::c_char,
        lenp: *mut size_t,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> *mut ::core::ffi::c_char;
    pub fn vasprintf(
        strp: *mut *mut ::core::ffi::c_char,
        fmt: *const ::core::ffi::c_char,
        ap: __gnuc_va_list,
    ) -> ::core::ffi::c_int;
    pub fn fputs(__str: *const ::core::ffi::c_char, __stream: *mut FILE) -> ::core::ffi::c_int;
    pub fn puts(__str: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    pub fn fwrite(
        __ptr: *const ::core::ffi::c_void,
        __size: ::core::ffi::c_uint,
        __nmemb: ::core::ffi::c_uint,
        __stream: *mut FILE,
    ) -> ::core::ffi::c_uint;
    pub fn fgetc(__stream: *mut FILE) -> ::core::ffi::c_int;
    pub fn getc(__stream: *mut FILE) -> ::core::ffi::c_int;
    pub fn getchar() -> ::core::ffi::c_int;
    pub fn ungetc(__c: ::core::ffi::c_int, __stream: *mut FILE) -> ::core::ffi::c_int;
    pub fn scanf(__fmt: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    pub fn fscanf(
        __stream: *mut FILE,
        __fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    pub fn vscanf(__fmt: *const ::core::ffi::c_char, __ap: __builtin_va_list)
    -> ::core::ffi::c_int;
    pub fn vfscanf(
        __stream: *mut FILE,
        __fmt: *const ::core::ffi::c_char,
        __ap: __builtin_va_list,
    ) -> ::core::ffi::c_int;
    pub fn sscanf(
        __buf: *const ::core::ffi::c_char,
        __fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    pub fn vsscanf(
        __buf: *const ::core::ffi::c_char,
        __fmt: *const ::core::ffi::c_char,
        ap: __builtin_va_list,
    ) -> ::core::ffi::c_int;
    pub fn fgets(
        __str: *mut ::core::ffi::c_char,
        __size: ::core::ffi::c_int,
        __stream: *mut FILE,
    ) -> *mut ::core::ffi::c_char;
    pub fn fread(
        __ptr: *mut ::core::ffi::c_void,
        __size: ::core::ffi::c_uint,
        __nmemb: ::core::ffi::c_uint,
        __stream: *mut FILE,
    ) -> ::core::ffi::c_uint;
    pub fn clearerr(__stream: *mut FILE);
    pub fn ferror(__stream: *mut FILE) -> ::core::ffi::c_int;
    pub fn feof(__stream: *mut FILE) -> ::core::ffi::c_int;
    pub fn clearerr_unlocked(__stream: *mut FILE);
    pub fn fgetpos(stream: *mut FILE, pos: *mut fpos_t) -> ::core::ffi::c_int;
    pub fn fopen(path: *const ::core::ffi::c_char, mode: *const ::core::ffi::c_char) -> *mut FILE;
    pub fn freopen(
        path: *const ::core::ffi::c_char,
        mode: *const ::core::ffi::c_char,
        stream: *mut FILE,
    ) -> *mut FILE;
    pub fn fdopen(arg1: ::core::ffi::c_int, arg2: *const ::core::ffi::c_char) -> *mut FILE;
    pub fn fmemopen(
        buf: *mut ::core::ffi::c_void,
        size: size_t,
        mode: *const ::core::ffi::c_char,
    ) -> *mut FILE;
    pub fn fseek(
        stream: *mut FILE,
        offset: ::core::ffi::c_long,
        whence: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn fseeko(
        stream: *mut FILE,
        offset: __off_t,
        whence: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn ftell(stream: *mut FILE) -> ::core::ffi::c_long;
    pub fn ftello(stream: *mut FILE) -> __off_t;
    pub fn fileno(arg1: *mut FILE) -> ::core::ffi::c_int;
    pub fn remove(pathname: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    pub fn rename(
        oldpath: *const ::core::ffi::c_char,
        newpath: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    pub fn rewind(stream: *mut FILE);
    pub fn setbuf(stream: *mut FILE, buf: *mut ::core::ffi::c_char);
    pub fn setbuffer(stream: *mut FILE, buf: *mut ::core::ffi::c_char, size: size_t);
    pub fn setlinebuf(stream: *mut FILE);
    pub fn setvbuf(
        stream: *mut FILE,
        buf: *mut ::core::ffi::c_char,
        mode: ::core::ffi::c_int,
        size: size_t,
    ) -> ::core::ffi::c_int;
    pub fn getline(
        lineptr: *mut *mut ::core::ffi::c_char,
        n: *mut size_t,
        stream: *mut FILE,
    ) -> _ssize_t;
    pub fn getdelim(
        lineptr: *mut *mut ::core::ffi::c_char,
        n: *mut size_t,
        delim: ::core::ffi::c_int,
        stream: *mut FILE,
    ) -> _ssize_t;
    pub fn funopen(
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
    ) -> *mut FILE;
    pub fn getchar_unlocked() -> ::core::ffi::c_int;
    pub fn free(arg1: *mut ::core::ffi::c_void);
    pub fn _Exit(__status: ::core::ffi::c_int) -> !;
    pub fn a64l(__input: *const ::core::ffi::c_char) -> ::core::ffi::c_long;
    pub fn abort() -> !;
    pub fn abs(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn atexit(__func: ::core::option::Option<unsafe extern "C" fn()>) -> ::core::ffi::c_int;
    pub fn atof(__nptr: *const ::core::ffi::c_char) -> f64;
    pub fn atoff(__nptr: *const ::core::ffi::c_char) -> f32;
    pub fn atoi(__nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    pub fn atol(__nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_long;
    pub fn atoll(__nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_longlong;
    pub fn bsearch(
        __key: *const ::core::ffi::c_void,
        __base: *const ::core::ffi::c_void,
        __nmemb: size_t,
        __size: size_t,
        _compar: __compar_fn_t,
    ) -> *mut ::core::ffi::c_void;
    pub fn calloc(arg1: ::core::ffi::c_uint, arg2: ::core::ffi::c_uint)
    -> *mut ::core::ffi::c_void;
    pub fn div(__numer: ::core::ffi::c_int, __denom: ::core::ffi::c_int) -> div_t;
    pub fn exit(__status: ::core::ffi::c_int) -> !;
    pub fn getenv(__string: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    pub fn getsubopt(
        arg1: *mut *mut ::core::ffi::c_char,
        arg2: *const *mut ::core::ffi::c_char,
        arg3: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    pub fn labs(arg1: ::core::ffi::c_long) -> ::core::ffi::c_long;
    pub fn ldiv(__numer: ::core::ffi::c_long, __denom: ::core::ffi::c_long) -> ldiv_t;
    pub fn llabs(arg1: ::core::ffi::c_longlong) -> ::core::ffi::c_longlong;
    pub fn lldiv(__numer: ::core::ffi::c_longlong, __denom: ::core::ffi::c_longlong) -> lldiv_t;
    pub fn malloc(arg1: ::core::ffi::c_uint) -> *mut ::core::ffi::c_void;
    pub fn mblen(arg1: *const ::core::ffi::c_char, arg2: size_t) -> ::core::ffi::c_int;
    pub fn mbstowcs(arg1: *mut wchar_t, arg2: *const ::core::ffi::c_char, arg3: size_t) -> size_t;
    pub fn mbtowc(
        arg1: *mut wchar_t,
        arg2: *const ::core::ffi::c_char,
        arg3: size_t,
    ) -> ::core::ffi::c_int;
    pub fn qsort(
        __base: *mut ::core::ffi::c_void,
        __nmemb: size_t,
        __size: size_t,
        _compar: __compar_fn_t,
    );
    pub fn qsort_r(
        __base: *mut ::core::ffi::c_void,
        __nmemb: size_t,
        __size: size_t,
        _compar: ::core::option::Option<
            unsafe extern "C" fn(
                arg1: *const ::core::ffi::c_void,
                arg2: *const ::core::ffi::c_void,
                arg3: *mut ::core::ffi::c_void,
            ) -> ::core::ffi::c_int,
        >,
        __thunk: *mut ::core::ffi::c_void,
    );
    pub fn rand() -> ::core::ffi::c_int;
    pub fn rand_r(__seed: *mut ::core::ffi::c_uint) -> ::core::ffi::c_int;
    pub fn random() -> ::core::ffi::c_long;
    pub fn realloc(
        arg1: *mut ::core::ffi::c_void,
        arg2: ::core::ffi::c_uint,
    ) -> *mut ::core::ffi::c_void;
    pub fn reallocarray(
        arg1: *mut ::core::ffi::c_void,
        arg2: size_t,
        arg3: size_t,
    ) -> *mut ::core::ffi::c_void;
    pub fn rpmatch(response: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    pub fn srand(__seed: ::core::ffi::c_uint);
    pub fn srandom(arg1: ::core::ffi::c_uint);
    pub fn strtod(__n: *const ::core::ffi::c_char, __end_PTR: *mut *mut ::core::ffi::c_char)
    -> f64;
    pub fn strtof(__n: *const ::core::ffi::c_char, __end_PTR: *mut *mut ::core::ffi::c_char)
    -> f32;
    pub fn strtold(
        __n: *const ::core::ffi::c_char,
        __end_PTR: *mut *mut ::core::ffi::c_char,
    ) -> u128;
    pub fn strtol(
        __n: *const ::core::ffi::c_char,
        __end_PTR: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> ::core::ffi::c_long;
    pub fn strtoll(
        __n: *const ::core::ffi::c_char,
        __end_PTR: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> ::core::ffi::c_longlong;
    pub fn strtoul(
        __n: *const ::core::ffi::c_char,
        __end_PTR: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> ::core::ffi::c_ulong;
    pub fn strtoull(
        __n: *const ::core::ffi::c_char,
        __end_PTR: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> ::core::ffi::c_ulonglong;
    pub fn strtod_l(
        arg1: *const ::core::ffi::c_char,
        arg2: *mut *mut ::core::ffi::c_char,
        arg3: locale_t,
    ) -> f64;
    pub fn strtof_l(
        arg1: *const ::core::ffi::c_char,
        arg2: *mut *mut ::core::ffi::c_char,
        arg3: locale_t,
    ) -> f32;
    pub fn strtold_l(
        arg1: *const ::core::ffi::c_char,
        arg2: *mut *mut ::core::ffi::c_char,
        arg3: locale_t,
    ) -> u128;
    pub fn strtol_l(
        arg1: *const ::core::ffi::c_char,
        arg2: *mut *mut ::core::ffi::c_char,
        arg3: ::core::ffi::c_int,
        arg4: locale_t,
    ) -> ::core::ffi::c_long;
    pub fn strtoul_l(
        arg1: *const ::core::ffi::c_char,
        arg2: *mut *mut ::core::ffi::c_char,
        arg3: ::core::ffi::c_int,
        __loc: locale_t,
    ) -> ::core::ffi::c_ulong;
    pub fn strtoll_l(
        arg1: *const ::core::ffi::c_char,
        arg2: *mut *mut ::core::ffi::c_char,
        arg3: ::core::ffi::c_int,
        arg4: locale_t,
    ) -> ::core::ffi::c_longlong;
    pub fn strtoull_l(
        arg1: *const ::core::ffi::c_char,
        arg2: *mut *mut ::core::ffi::c_char,
        arg3: ::core::ffi::c_int,
        __loc: locale_t,
    ) -> ::core::ffi::c_ulonglong;
    pub fn system(__string: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    pub fn wcstombs(arg1: *mut ::core::ffi::c_char, arg2: *const wchar_t, arg3: size_t) -> size_t;
    pub fn wctomb(arg1: *mut ::core::ffi::c_char, arg2: wchar_t) -> ::core::ffi::c_int;
    pub fn l64a(__input: ::core::ffi::c_long) -> *mut ::core::ffi::c_char;
    pub fn gcvt(
        arg1: f64,
        arg2: ::core::ffi::c_int,
        arg3: *mut ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn gcvtf(
        arg1: f32,
        arg2: ::core::ffi::c_int,
        arg3: *mut ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn gcvtl(
        arg1: u128,
        arg2: ::core::ffi::c_int,
        arg3: *mut ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn itoa(
        arg1: ::core::ffi::c_int,
        arg2: *mut ::core::ffi::c_char,
        arg3: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    pub fn utoa(
        arg1: ::core::ffi::c_uint,
        arg2: *mut ::core::ffi::c_char,
        arg3: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    pub fn bcmp(
        arg1: *const ::core::ffi::c_void,
        arg2: *const ::core::ffi::c_void,
        arg3: ::core::ffi::c_uint,
    ) -> ::core::ffi::c_int;
    pub fn bcopy(
        arg1: *const ::core::ffi::c_void,
        arg2: *mut ::core::ffi::c_void,
        arg3: ::core::ffi::c_uint,
    );
    pub fn bzero(arg1: *mut ::core::ffi::c_void, arg2: ::core::ffi::c_uint);
    pub fn explicit_bzero(arg1: *mut ::core::ffi::c_void, arg2: size_t);
    pub fn ffs(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn ffsl(arg1: ::core::ffi::c_long) -> ::core::ffi::c_int;
    pub fn ffsll(arg1: ::core::ffi::c_longlong) -> ::core::ffi::c_int;
    pub fn fls(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn flsl(arg1: ::core::ffi::c_long) -> ::core::ffi::c_int;
    pub fn flsll(arg1: ::core::ffi::c_longlong) -> ::core::ffi::c_int;
    pub fn index(
        arg1: *const ::core::ffi::c_char,
        arg2: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    pub fn rindex(
        arg1: *const ::core::ffi::c_char,
        arg2: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    pub fn strcasecmp(
        arg1: *const ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    pub fn strncasecmp(
        arg1: *const ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
        arg3: ::core::ffi::c_uint,
    ) -> ::core::ffi::c_int;
    pub fn strcasecmp_l(
        arg1: *const ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
        arg3: locale_t,
    ) -> ::core::ffi::c_int;
    pub fn strncasecmp_l(
        arg1: *const ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
        arg3: size_t,
        arg4: locale_t,
    ) -> ::core::ffi::c_int;
    pub fn memccpy(
        arg1: *mut ::core::ffi::c_void,
        arg2: *const ::core::ffi::c_void,
        arg3: ::core::ffi::c_int,
        arg4: ::core::ffi::c_uint,
    ) -> *mut ::core::ffi::c_void;
    pub fn memchr(
        arg1: *const ::core::ffi::c_void,
        arg2: ::core::ffi::c_int,
        arg3: ::core::ffi::c_uint,
    ) -> *mut ::core::ffi::c_void;
    pub fn memcmp(
        arg1: *const ::core::ffi::c_void,
        arg2: *const ::core::ffi::c_void,
        arg3: ::core::ffi::c_uint,
    ) -> ::core::ffi::c_int;
    pub fn memcpy(
        arg1: *mut ::core::ffi::c_void,
        arg2: *const ::core::ffi::c_void,
        arg3: ::core::ffi::c_uint,
    ) -> *mut ::core::ffi::c_void;
    pub fn memmem(
        arg1: *const ::core::ffi::c_void,
        arg2: size_t,
        arg3: *const ::core::ffi::c_void,
        arg4: size_t,
    ) -> *mut ::core::ffi::c_void;
    pub fn memmove(
        arg1: *mut ::core::ffi::c_void,
        arg2: *const ::core::ffi::c_void,
        arg3: ::core::ffi::c_uint,
    ) -> *mut ::core::ffi::c_void;
    pub fn mempcpy(
        arg1: *mut ::core::ffi::c_void,
        arg2: *const ::core::ffi::c_void,
        arg3: ::core::ffi::c_uint,
    ) -> *mut ::core::ffi::c_void;
    pub fn memrchr(
        arg1: *const ::core::ffi::c_void,
        arg2: ::core::ffi::c_int,
        arg3: size_t,
    ) -> *mut ::core::ffi::c_void;
    pub fn memset(
        arg1: *mut ::core::ffi::c_void,
        arg2: ::core::ffi::c_int,
        arg3: ::core::ffi::c_uint,
    ) -> *mut ::core::ffi::c_void;
    pub fn rawmemchr(
        arg1: *const ::core::ffi::c_void,
        arg2: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_void;
    pub fn stpcpy(
        arg1: *mut ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn stpncpy(
        arg1: *mut ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
        arg3: ::core::ffi::c_uint,
    ) -> *mut ::core::ffi::c_char;
    pub fn strcasestr(
        arg1: *const ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn strcat(
        arg1: *mut ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn strchr(
        arg1: *const ::core::ffi::c_char,
        arg2: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    pub fn strchrnul(
        arg1: *const ::core::ffi::c_char,
        arg2: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    pub fn strcmp(
        arg1: *const ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    pub fn strcoll(
        arg1: *const ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    pub fn strcoll_l(
        arg1: *const ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
        arg3: locale_t,
    ) -> ::core::ffi::c_int;
    pub fn strcpy(
        arg1: *mut ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn strcspn(
        arg1: *const ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_uint;
    pub fn strdup(arg1: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    pub fn strerror(arg1: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    pub fn strerror_r(
        arg1: ::core::ffi::c_int,
        arg2: *mut ::core::ffi::c_char,
        arg3: size_t,
    ) -> *mut ::core::ffi::c_char;
    pub fn strlcat(
        arg1: *mut ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
        arg3: ::core::ffi::c_uint,
    ) -> ::core::ffi::c_uint;
    pub fn strlen(arg1: *const ::core::ffi::c_char) -> ::core::ffi::c_uint;
    pub fn strlcpy(
        arg1: *mut ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
        arg3: ::core::ffi::c_uint,
    ) -> ::core::ffi::c_uint;
    pub fn strlwr(arg1: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    pub fn strncat(
        arg1: *mut ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
        arg3: ::core::ffi::c_uint,
    ) -> *mut ::core::ffi::c_char;
    pub fn strncmp(
        arg1: *const ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
        arg3: ::core::ffi::c_uint,
    ) -> ::core::ffi::c_int;
    pub fn strncpy(
        arg1: *mut ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
        arg3: ::core::ffi::c_uint,
    ) -> *mut ::core::ffi::c_char;
    pub fn strndup(
        arg1: *const ::core::ffi::c_char,
        arg2: ::core::ffi::c_uint,
    ) -> *mut ::core::ffi::c_char;
    pub fn strnlen(arg1: *const ::core::ffi::c_char, arg2: size_t) -> size_t;
    pub fn strnstr(
        arg1: *const ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
        arg3: size_t,
    ) -> *mut ::core::ffi::c_char;
    pub fn strpbrk(
        arg1: *const ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn strrchr(
        arg1: *const ::core::ffi::c_char,
        arg2: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    pub fn strsep(
        arg1: *mut *mut ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn strspn(
        arg1: *const ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_uint;
    pub fn strstr(
        arg1: *const ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn strtok(
        arg1: *mut ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn strtok_r(
        arg1: *mut ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
        arg3: *mut *mut ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn strupr(arg1: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    pub fn strverscmp(
        arg1: *const ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    pub fn strxfrm(
        arg1: *mut ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
        arg3: ::core::ffi::c_uint,
    ) -> ::core::ffi::c_uint;
    pub fn strxfrm_l(
        arg1: *mut ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
        arg3: size_t,
        arg4: locale_t,
    ) -> size_t;
    pub fn timingsafe_bcmp(
        arg1: *const ::core::ffi::c_void,
        arg2: *const ::core::ffi::c_void,
        arg3: size_t,
    ) -> ::core::ffi::c_int;
    pub fn timingsafe_memcmp(
        arg1: *const ::core::ffi::c_void,
        arg2: *const ::core::ffi::c_void,
        arg3: size_t,
    ) -> ::core::ffi::c_int;
    pub fn select(
        __n: ::core::ffi::c_int,
        __readfds: *mut fd_set,
        __writefds: *mut fd_set,
        __exceptfds: *mut fd_set,
        __timeout: *mut timeval,
    ) -> ::core::ffi::c_int;
    pub fn gettimeofday(__p: *mut timeval, __tz: *mut ::core::ffi::c_void) -> ::core::ffi::c_int;
    pub fn times(arg1: *mut tms) -> clock_t;
    pub fn asctime(_tblock: *const tm) -> *mut ::core::ffi::c_char;
    pub fn asctime_r(
        arg1: *const tm,
        arg2: *mut [::core::ffi::c_char; 26usize],
    ) -> *mut ::core::ffi::c_char;
    pub fn clock() -> clock_t;
    pub fn clock_gettime(clock_id: clockid_t, tp: *mut timespec) -> ::core::ffi::c_int;
    pub fn ctime(_time: *const time_t) -> *mut ::core::ffi::c_char;
    pub fn ctime_r(
        arg1: *const time_t,
        arg2: *mut [::core::ffi::c_char; 26usize],
    ) -> *mut ::core::ffi::c_char;
    pub fn difftime(_time2: time_t, _time1: time_t) -> f64;
    pub fn gmtime(_timer: *const time_t) -> *mut tm;
    pub fn gmtime_r(arg1: *const time_t, arg2: *mut tm) -> *mut tm;
    pub fn localtime(_timer: *const time_t) -> *mut tm;
    pub fn localtime_r(arg1: *const time_t, arg2: *mut tm) -> *mut tm;
    pub fn mktime(_timeptr: *mut tm) -> time_t;
    pub fn strftime(
        _s: *mut ::core::ffi::c_char,
        _maxsize: size_t,
        _fmt: *const ::core::ffi::c_char,
        _t: *const tm,
    ) -> size_t;
    pub fn strftime_l(
        _s: *mut ::core::ffi::c_char,
        _maxsize: size_t,
        _fmt: *const ::core::ffi::c_char,
        _t: *const tm,
        _l: locale_t,
    ) -> size_t;
    pub fn strptime(
        arg1: *const ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
        arg3: *mut tm,
    ) -> *mut ::core::ffi::c_char;
    pub fn strptime_l(
        arg1: *const ::core::ffi::c_char,
        arg2: *const ::core::ffi::c_char,
        arg3: *mut tm,
        arg4: locale_t,
    ) -> *mut ::core::ffi::c_char;
    pub fn time(_timer: *mut time_t) -> time_t;
    pub static mut environ: *mut *mut ::core::ffi::c_char;
    pub fn close(__fildes: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn getentropy(arg1: *mut ::core::ffi::c_void, arg2: size_t) -> ::core::ffi::c_int;
    pub fn getpid() -> pid_t;
    pub fn isatty(__fildes: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn link(
        __path1: *const ::core::ffi::c_char,
        __path2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    pub fn lseek(
        __fildes: ::core::ffi::c_int,
        __offset: off_t,
        __whence: ::core::ffi::c_int,
    ) -> off_t;
    pub fn read(
        __fd: ::core::ffi::c_int,
        __buf: *mut ::core::ffi::c_void,
        __nbyte: size_t,
    ) -> ssize_t;
    pub fn rmdir(__path: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    pub fn sleep(__seconds: ::core::ffi::c_uint) -> ::core::ffi::c_uint;
    pub fn swab(arg1: *const ::core::ffi::c_void, arg2: *mut ::core::ffi::c_void, arg3: ssize_t);
    pub fn unlink(__path: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    pub fn usleep(__useconds: useconds_t) -> ::core::ffi::c_int;
    pub fn write(
        __fd: ::core::ffi::c_int,
        __buf: *const ::core::ffi::c_void,
        __nbyte: size_t,
    ) -> ssize_t;
    pub fn btowc(arg1: ::core::ffi::c_int) -> wint_t;
    pub fn fwide(arg1: *mut __FILE, arg2: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn mbrlen(arg1: *const ::core::ffi::c_char, arg2: size_t, arg3: *mut mbstate_t) -> size_t;
    pub fn mbrtowc(
        arg1: *mut wchar_t,
        arg2: *const ::core::ffi::c_char,
        arg3: size_t,
        arg4: *mut mbstate_t,
    ) -> size_t;
    pub fn mbsinit(arg1: *const mbstate_t) -> ::core::ffi::c_int;
    pub fn mbsnrtowcs(
        arg1: *mut wchar_t,
        arg2: *mut *const ::core::ffi::c_char,
        arg3: size_t,
        arg4: size_t,
        arg5: *mut mbstate_t,
    ) -> size_t;
    pub fn mbsrtowcs(
        arg1: *mut wchar_t,
        arg2: *mut *const ::core::ffi::c_char,
        arg3: size_t,
        arg4: *mut mbstate_t,
    ) -> size_t;
    pub fn wcpcpy(arg1: *mut wchar_t, arg2: *const wchar_t) -> *mut wchar_t;
    pub fn wcpncpy(arg1: *mut wchar_t, arg2: *const wchar_t, arg3: size_t) -> *mut wchar_t;
    pub fn wcrtomb(arg1: *mut ::core::ffi::c_char, arg2: wchar_t, arg3: *mut mbstate_t) -> size_t;
    pub fn wcscasecmp(arg1: *const wchar_t, arg2: *const wchar_t) -> ::core::ffi::c_int;
    pub fn wcscasecmp_l(
        arg1: *const wchar_t,
        arg2: *const wchar_t,
        arg3: locale_t,
    ) -> ::core::ffi::c_int;
    pub fn wcscat(arg1: *mut wchar_t, arg2: *const wchar_t) -> *mut wchar_t;
    pub fn wcschr(
        arg1: *const ::core::ffi::c_int,
        arg2: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_int;
    pub fn wcscmp(
        arg1: *const ::core::ffi::c_int,
        arg2: *const ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn wcscoll(arg1: *const wchar_t, arg2: *const wchar_t) -> ::core::ffi::c_int;
    pub fn wcscoll_l(
        arg1: *const wchar_t,
        arg2: *const wchar_t,
        arg3: locale_t,
    ) -> ::core::ffi::c_int;
    pub fn wcscpy(arg1: *mut wchar_t, arg2: *const wchar_t) -> *mut wchar_t;
    pub fn wcscspn(arg1: *const wchar_t, arg2: *const wchar_t) -> size_t;
    pub fn wcsdup(arg1: *const wchar_t) -> *mut wchar_t;
    pub fn wcsftime(
        arg1: *mut wchar_t,
        arg2: size_t,
        arg3: *const wchar_t,
        arg4: *const tm,
    ) -> size_t;
    pub fn wcsftime_l(
        arg1: *mut wchar_t,
        arg2: size_t,
        arg3: *const wchar_t,
        arg4: *const tm,
        arg5: locale_t,
    ) -> size_t;
    pub fn wcslcat(arg1: *mut wchar_t, arg2: *const wchar_t, arg3: size_t) -> size_t;
    pub fn wcslcpy(arg1: *mut wchar_t, arg2: *const wchar_t, arg3: size_t) -> size_t;
    pub fn wcslen(arg1: *const ::core::ffi::c_int) -> ::core::ffi::c_uint;
    pub fn wcsncasecmp(
        arg1: *const wchar_t,
        arg2: *const wchar_t,
        arg3: size_t,
    ) -> ::core::ffi::c_int;
    pub fn wcsncasecmp_l(
        arg1: *const wchar_t,
        arg2: *const wchar_t,
        arg3: size_t,
        arg4: locale_t,
    ) -> ::core::ffi::c_int;
    pub fn wcsncat(arg1: *mut wchar_t, arg2: *const wchar_t, arg3: size_t) -> *mut wchar_t;
    pub fn wcsncmp(
        arg1: *const ::core::ffi::c_int,
        arg2: *const ::core::ffi::c_int,
        arg3: ::core::ffi::c_uint,
    ) -> ::core::ffi::c_int;
    pub fn wcsncpy(arg1: *mut wchar_t, arg2: *const wchar_t, arg3: size_t) -> *mut wchar_t;
    pub fn wcsnlen(arg1: *const wchar_t, arg2: size_t) -> size_t;
    pub fn wcsnrtombs(
        arg1: *mut ::core::ffi::c_char,
        arg2: *mut *const wchar_t,
        arg3: size_t,
        arg4: size_t,
        arg5: *mut mbstate_t,
    ) -> size_t;
    pub fn wcspbrk(arg1: *const wchar_t, arg2: *const wchar_t) -> *mut wchar_t;
    pub fn wcsrchr(arg1: *const wchar_t, arg2: wchar_t) -> *mut wchar_t;
    pub fn wcsrtombs(
        arg1: *mut ::core::ffi::c_char,
        arg2: *mut *const wchar_t,
        arg3: size_t,
        arg4: *mut mbstate_t,
    ) -> size_t;
    pub fn wcsspn(arg1: *const wchar_t, arg2: *const wchar_t) -> size_t;
    pub fn wcsstr(arg1: *const wchar_t, arg2: *const wchar_t) -> *mut wchar_t;
    pub fn wcstod(arg1: *const wchar_t, arg2: *mut *mut wchar_t) -> f64;
    pub fn wcstod_l(arg1: *const wchar_t, arg2: *mut *mut wchar_t, arg3: locale_t) -> f64;
    pub fn wcstof(arg1: *const wchar_t, arg2: *mut *mut wchar_t) -> f32;
    pub fn wcstof_l(arg1: *const wchar_t, arg2: *mut *mut wchar_t, arg3: locale_t) -> f32;
    pub fn wcstok(
        arg1: *mut wchar_t,
        arg2: *const wchar_t,
        arg3: *mut *mut wchar_t,
    ) -> *mut wchar_t;
    pub fn wcstol(
        arg1: *const wchar_t,
        arg2: *mut *mut wchar_t,
        arg3: ::core::ffi::c_int,
    ) -> ::core::ffi::c_long;
    pub fn wcstol_l(
        arg1: *const wchar_t,
        arg2: *mut *mut wchar_t,
        arg3: ::core::ffi::c_int,
        arg4: locale_t,
    ) -> ::core::ffi::c_long;
    pub fn wcstold(arg1: *const wchar_t, arg2: *mut *mut wchar_t) -> u128;
    pub fn wcstold_l(arg1: *const wchar_t, arg2: *mut *mut wchar_t, arg3: locale_t) -> u128;
    pub fn wcstoll(
        arg1: *const wchar_t,
        arg2: *mut *mut wchar_t,
        arg3: ::core::ffi::c_int,
    ) -> ::core::ffi::c_longlong;
    pub fn wcstoll_l(
        arg1: *const wchar_t,
        arg2: *mut *mut wchar_t,
        arg3: ::core::ffi::c_int,
        arg4: locale_t,
    ) -> ::core::ffi::c_longlong;
    pub fn wcstoul(
        arg1: *const wchar_t,
        arg2: *mut *mut wchar_t,
        arg3: ::core::ffi::c_int,
    ) -> ::core::ffi::c_ulong;
    pub fn wcstoul_l(
        arg1: *const wchar_t,
        arg2: *mut *mut wchar_t,
        arg3: ::core::ffi::c_int,
        arg4: locale_t,
    ) -> ::core::ffi::c_ulong;
    pub fn wcstoull(
        arg1: *const wchar_t,
        arg2: *mut *mut wchar_t,
        arg3: ::core::ffi::c_int,
    ) -> ::core::ffi::c_ulonglong;
    pub fn wcstoull_l(
        arg1: *const wchar_t,
        arg2: *mut *mut wchar_t,
        arg3: ::core::ffi::c_int,
        arg4: locale_t,
    ) -> ::core::ffi::c_ulonglong;
    pub fn wcswidth(arg1: *const wchar_t, arg2: size_t) -> ::core::ffi::c_int;
    pub fn wcsxfrm(arg1: *mut wchar_t, arg2: *const wchar_t, arg3: size_t) -> size_t;
    pub fn wcsxfrm_l(
        arg1: *mut wchar_t,
        arg2: *const wchar_t,
        arg3: size_t,
        arg4: locale_t,
    ) -> size_t;
    pub fn wctob(arg1: wint_t) -> ::core::ffi::c_int;
    pub fn wcwidth(arg1: wchar_t) -> ::core::ffi::c_int;
    pub fn wmemchr(
        arg1: *const ::core::ffi::c_int,
        arg2: ::core::ffi::c_int,
        arg3: ::core::ffi::c_uint,
    ) -> *mut ::core::ffi::c_int;
    pub fn wmemcmp(
        arg1: *const ::core::ffi::c_int,
        arg2: *const ::core::ffi::c_int,
        arg3: ::core::ffi::c_uint,
    ) -> ::core::ffi::c_int;
    pub fn wmemcpy(
        arg1: *mut ::core::ffi::c_int,
        arg2: *const ::core::ffi::c_int,
        arg3: ::core::ffi::c_uint,
    ) -> *mut ::core::ffi::c_int;
    pub fn wmemmove(
        arg1: *mut ::core::ffi::c_int,
        arg2: *const ::core::ffi::c_int,
        arg3: ::core::ffi::c_uint,
    ) -> *mut ::core::ffi::c_int;
    pub fn wmempcpy(arg1: *mut wchar_t, arg2: *const wchar_t, arg3: size_t) -> *mut wchar_t;
    pub fn wmemset(arg1: *mut wchar_t, arg2: wchar_t, arg3: size_t) -> *mut wchar_t;
    pub fn iswalnum(arg1: wint_t) -> ::core::ffi::c_int;
    pub fn iswalpha(arg1: wint_t) -> ::core::ffi::c_int;
    pub fn iswblank(arg1: wint_t) -> ::core::ffi::c_int;
    pub fn iswcntrl(arg1: wint_t) -> ::core::ffi::c_int;
    pub fn iswctype(arg1: wint_t, arg2: wctype_t) -> ::core::ffi::c_int;
    pub fn iswdigit(arg1: wint_t) -> ::core::ffi::c_int;
    pub fn iswgraph(arg1: wint_t) -> ::core::ffi::c_int;
    pub fn iswlower(arg1: wint_t) -> ::core::ffi::c_int;
    pub fn iswprint(arg1: wint_t) -> ::core::ffi::c_int;
    pub fn iswpunct(arg1: wint_t) -> ::core::ffi::c_int;
    pub fn iswspace(arg1: wint_t) -> ::core::ffi::c_int;
    pub fn iswupper(arg1: wint_t) -> ::core::ffi::c_int;
    pub fn iswxdigit(arg1: wint_t) -> ::core::ffi::c_int;
    pub fn towupper(arg1: wint_t) -> wint_t;
    pub fn towlower(arg1: wint_t) -> wint_t;
    pub fn wctrans(arg1: *const ::core::ffi::c_char) -> wctrans_t;
    pub fn wctype(arg1: *const ::core::ffi::c_char) -> wctype_t;
    pub fn iswalnum_l(arg1: wint_t, arg2: locale_t) -> ::core::ffi::c_int;
    pub fn iswalpha_l(arg1: wint_t, arg2: locale_t) -> ::core::ffi::c_int;
    pub fn iswblank_l(arg1: wint_t, arg2: locale_t) -> ::core::ffi::c_int;
    pub fn iswcntrl_l(arg1: wint_t, arg2: locale_t) -> ::core::ffi::c_int;
    pub fn iswctype_l(arg1: wint_t, arg2: wctype_t, arg3: locale_t) -> ::core::ffi::c_int;
    pub fn iswdigit_l(arg1: wint_t, arg2: locale_t) -> ::core::ffi::c_int;
    pub fn iswgraph_l(arg1: wint_t, arg2: locale_t) -> ::core::ffi::c_int;
    pub fn iswlower_l(arg1: wint_t, arg2: locale_t) -> ::core::ffi::c_int;
    pub fn iswprint_l(arg1: wint_t, arg2: locale_t) -> ::core::ffi::c_int;
    pub fn iswpunct_l(arg1: wint_t, arg2: locale_t) -> ::core::ffi::c_int;
    pub fn iswspace_l(arg1: wint_t, arg2: locale_t) -> ::core::ffi::c_int;
    pub fn iswupper_l(arg1: wint_t, arg2: locale_t) -> ::core::ffi::c_int;
    pub fn iswxdigit_l(arg1: wint_t, arg2: locale_t) -> ::core::ffi::c_int;
    pub fn towupper_l(arg1: wint_t, arg2: locale_t) -> wint_t;
    pub fn towlower_l(arg1: wint_t, arg2: locale_t) -> wint_t;
    pub fn wctrans_l(arg1: *const ::core::ffi::c_char, arg2: locale_t) -> wctrans_t;
    pub fn wctype_l(arg1: *const ::core::ffi::c_char, arg2: locale_t) -> wctype_t;
    pub fn parse_path(path: *const ::core::ffi::c_char, result: *mut path_t)
    -> path_parse_result_t;
    pub fn path_free(path: *mut path_t);
    pub fn mkdir_p(path: *const ::core::ffi::c_char) -> bool;
    pub fn rm_rf(path: *const ::core::ffi::c_char) -> bool;
    pub fn path_dirname(path: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    pub fn path_basename(path: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    pub fn path_devname(path: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    pub fn path_dirconcat(
        path: *const ::core::ffi::c_char,
        subdir: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn path_fileconcat(
        path: *const ::core::ffi::c_char,
        filename: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn path_concat(
        base_path: *const ::core::ffi::c_char,
        append_path: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn application_launch(unique_identifier: *const ::core::ffi::c_char) -> pid_t;
    pub fn application_create(
        unique_identifier: *const ::core::ffi::c_char,
        name: *const ::core::ffi::c_char,
        author: *const ::core::ffi::c_char,
        version: *const ::core::ffi::c_char,
        interpreter: *const ::core::ffi::c_char,
        source: application_source_t,
    ) -> *mut application_t;
    pub fn application_set_metadata(
        application: *mut application_t,
        metadata_file: *const ::core::ffi::c_char,
    ) -> bool;
    pub fn application_set_binary_path(
        application: *mut application_t,
        binary_path: *const ::core::ffi::c_char,
    ) -> bool;
    pub fn application_set_version(
        application: *mut application_t,
        version: *const ::core::ffi::c_char,
    ) -> bool;
    pub fn application_set_author(
        application: *mut application_t,
        author: *const ::core::ffi::c_char,
    ) -> bool;
    pub fn application_set_name(
        application: *mut application_t,
        name: *const ::core::ffi::c_char,
    ) -> bool;
    pub fn application_set_interpreter(
        application: *mut application_t,
        interpreter: *const ::core::ffi::c_char,
    ) -> bool;
    pub fn application_destroy(application: *mut application_t) -> bool;
    pub fn application_create_file(
        application: *mut application_t,
        file_path: *const ::core::ffi::c_char,
    ) -> *mut FILE;
    pub fn application_create_file_string(
        application: *mut application_t,
        file_path: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn application_list(out: *mut *mut application_t) -> application_list_handle;
    pub fn application_list_get_next(list: application_list_handle) -> *mut application_t;
    pub fn application_list_close(list: application_list_handle);
    pub fn application_get(unique_identifier: *const ::core::ffi::c_char) -> *mut application_t;
    pub fn application_free(application: *mut application_t);
    pub fn window_create(
        title: *const ::core::ffi::c_char,
        size: window_size_t,
        flags: window_flag_t,
    ) -> window_handle_t;
    pub fn window_framebuffer_create(
        window: window_handle_t,
        size: window_size_t,
        pixel_format: pixel_format_t,
    ) -> *mut framebuffer_t;
    pub fn window_destroy(window: window_handle_t);
    pub fn window_title_get(window: window_handle_t) -> *const ::core::ffi::c_char;
    pub fn window_title_set(window: window_handle_t, title: *const ::core::ffi::c_char);
    pub fn window_position_get(window: window_handle_t) -> window_coords_t;
    pub fn window_position_set(window: window_handle_t, coords: window_coords_t)
    -> window_coords_t;
    pub fn window_size_get(window: window_handle_t) -> window_size_t;
    pub fn window_size_set(window: window_handle_t, size: window_size_t) -> window_size_t;
    pub fn window_flags_get(window: window_handle_t) -> window_flag_t;
    pub fn window_flags_set(window: window_handle_t, flags: window_flag_t) -> window_flag_t;
    pub fn window_framebuffer_size_get(window: window_handle_t) -> window_size_t;
    pub fn window_framebuffer_size_set(
        window: window_handle_t,
        size: window_size_t,
    ) -> window_size_t;
    pub fn window_framebuffer_format_get(window: window_handle_t) -> pixel_format_t;
    pub fn window_framebuffer_get(window: window_handle_t) -> *mut framebuffer_t;
    pub fn window_present(
        window: window_handle_t,
        block: bool,
        rects: *mut window_rect_t,
        num_rects: ::core::ffi::c_int,
    );
    pub fn window_event_poll(window: window_handle_t, block: bool, timeout_msec: u32) -> event_t;
    pub fn get_screen_info(
        width: *mut ::core::ffi::c_int,
        height: *mut ::core::ffi::c_int,
        format: *mut pixel_format_t,
        refresh_rate: *mut f32,
    );
    pub fn opendir(name: *const ::core::ffi::c_char) -> *mut DIR;
    pub fn readdir(pdir: *mut DIR) -> *mut dirent;
    pub fn rewinddir(pdir: *mut DIR);
    pub fn closedir(pdir: *mut DIR) -> ::core::ffi::c_int;
    pub fn fstat(__fd: ::core::ffi::c_int, __sbuf: *mut stat) -> ::core::ffi::c_int;
    pub fn mkdir(_path: *const ::core::ffi::c_char, __mode: mode_t) -> ::core::ffi::c_int;
    pub fn stat(__path: *const ::core::ffi::c_char, __sbuf: *mut stat) -> ::core::ffi::c_int;
    pub fn device_get(name: *const ::core::ffi::c_char) -> *mut device_t;
    pub fn die(reason: *const ::core::ffi::c_char);
    pub fn vaddr_to_paddr(vaddr: u32) -> u32;
    pub fn get_mac_address() -> *const ::core::ffi::c_char;
    pub fn ota_session_open() -> ota_handle_t;
    pub fn ota_write(
        session: ota_handle_t,
        buffer: *mut ::core::ffi::c_void,
        block_size: ::core::ffi::c_int,
    ) -> bool;
    pub fn ota_session_commit(session: ota_handle_t) -> bool;
    pub fn ota_session_abort(session: ota_handle_t) -> bool;
    pub fn ota_get_running_version(version: *mut ::core::ffi::c_char) -> bool;
    pub fn ota_get_invalid_version(version: *mut ::core::ffi::c_char) -> bool;
    pub fn process_create(
        path: *const ::core::ffi::c_char,
        stack_size: size_t,
        argc: ::core::ffi::c_int,
        argv: *mut *mut ::core::ffi::c_char,
    ) -> pid_t;
    pub fn thread_create(
        thread_entry: ::core::option::Option<
            unsafe extern "C" fn(user_data: *mut ::core::ffi::c_void),
        >,
        user_data: *mut ::core::ffi::c_void,
        stack_size: u16,
    ) -> pid_t;
    pub fn wait(block: bool, timeout_msec: u32) -> pid_t;
    pub fn task_priority_lower();
    pub fn task_priority_restore();
    pub fn get_num_tasks() -> u32;
    pub fn wifi_get_status() -> wifi_status_t;
    pub fn wifi_get_connection_status() -> wifi_connection_status_t;
    pub fn wifi_get_connection_station() -> wifi_station_handle;
    pub fn wifi_connect() -> wifi_connection_status_t;
    pub fn wifi_disconnect() -> wifi_connection_status_t;
    pub fn wifi_scan_free_station(station: wifi_station_handle);
    pub fn wifi_scan_get_num_results() -> ::core::ffi::c_int;
    pub fn wifi_scan_get_result(num: ::core::ffi::c_int) -> wifi_station_handle;
    pub fn wifi_station_get_ssid(station: wifi_station_handle) -> *const ::core::ffi::c_char;
    pub fn wifi_station_get_bssid(station: wifi_station_handle) -> *mut mac_address_t;
    pub fn wifi_station_get_primary_channel(station: wifi_station_handle) -> ::core::ffi::c_int;
    pub fn wifi_station_get_secondary_channel(station: wifi_station_handle) -> ::core::ffi::c_int;
    pub fn wifi_station_get_rssi(station: wifi_station_handle) -> ::core::ffi::c_int;
    pub fn wifi_station_get_mode(station: wifi_station_handle) -> wifi_auth_mode_t;
    pub fn wifi_station_wps(station: wifi_station_handle) -> bool;
    pub fn curl_easy_init() -> *mut CURL;
    pub fn curl_easy_setopt(curl: *mut CURL, option: CURLoption, ...) -> CURLcode;
    pub fn curl_easy_perform(curl: *mut CURL) -> CURLcode;
    pub fn curl_easy_cleanup(curl: *mut CURL);
    pub fn curl_easy_getinfo(curl: *mut CURL, info: curl_easy_info_t, ...) -> CURLcode;
    pub fn curl_easy_strerror(error: CURLcode) -> *const ::core::ffi::c_char;
    pub fn curl_slist_append(
        list: *mut curl_slist,
        string: *const ::core::ffi::c_char,
    ) -> *mut curl_slist;
    pub fn curl_slist_free_all(list: *mut curl_slist);
    pub fn curl_global_init(flags: ::core::ffi::c_long) -> CURLcode;
    pub fn curl_global_cleanup();
    pub fn inet_ntoa(__in: in_addr) -> *mut ::core::ffi::c_char;
    pub fn inet_aton(__cp: *const ::core::ffi::c_char, __inp: *mut in_addr) -> ::core::ffi::c_int;
    pub fn accept(
        s: ::core::ffi::c_int,
        addr: *mut sockaddr,
        addrlen: *mut socklen_t,
    ) -> ::core::ffi::c_int;
    pub fn bind(
        s: ::core::ffi::c_int,
        name: *const sockaddr,
        namelen: socklen_t,
    ) -> ::core::ffi::c_int;
    pub fn connect(
        s: ::core::ffi::c_int,
        name: *const sockaddr,
        namelen: socklen_t,
    ) -> ::core::ffi::c_int;
    pub fn listen(s: ::core::ffi::c_int, backlog: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn socket(
        domain: ::core::ffi::c_int,
        type_: ::core::ffi::c_int,
        protocol: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn freeaddrinfo(ai: *mut addrinfo);
    pub fn getaddrinfo(
        nodename: *const ::core::ffi::c_char,
        servname: *const ::core::ffi::c_char,
        hints: *const addrinfo,
        res: *mut *mut addrinfo,
    ) -> ::core::ffi::c_int;
    #[doc = " The signature seems to be like this. idk why it's not in our headers"]
    pub fn diprintf(
        a: ::core::ffi::c_int,
        b: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    #[doc = " The signature seems to be like lgamma_r. I have no idea what it does"]
    pub fn gamma_r(arg1: f64, arg2: *mut ::core::ffi::c_int) -> f64;
    #[doc = " The signature seems to be like lgammaf_r. I have no idea what it does"]
    pub fn gammaf_r(arg1: f32, arg2: *mut ::core::ffi::c_int) -> f32;
    #[doc = " Probably in symbols, because it's in newlib. idk why it is not in our\n headers"]
    pub fn sig2str(
        signum: ::core::ffi::c_int,
        str_: *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    #[doc = " Probably in symbols, because it's in newlib. idk why it is not in our\n headers"]
    pub fn str2sig(
        str_: *const ::core::ffi::c_char,
        pnum: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    #[doc = " @brief Gets the parameters of the terminal\n\n @param fd file descriptor of the terminal\n @param p output termios structure\n @return 0 when successful, -1 otherwise with errno set"]
    pub fn tcgetattr(fd: ::core::ffi::c_int, p: *mut termios) -> ::core::ffi::c_int;
    #[doc = " @brief Sets the parameters of the terminal\n\n @param fd file descriptor of the terminal\n @param optional_actions optional actions\n @param p input termios structure\n @return 0 when successful, -1 otherwise with errno set"]
    pub fn tcsetattr(
        fd: ::core::ffi::c_int,
        optional_actions: ::core::ffi::c_int,
        p: *const termios,
    ) -> ::core::ffi::c_int;
    pub fn __adddf3(a: f64, b: f64) -> f64;
    pub fn __subdf3(a: f64, b: f64) -> f64;
    pub fn __muldf3(a: f64, b: f64) -> f64;
    pub fn __divdf3(a: f64, b: f64) -> f64;
    pub fn __eqdf2(a: f64, b: f64) -> ::core::ffi::c_int;
    pub fn __gedf2(a: f64, b: f64) -> ::core::ffi::c_int;
    pub fn __gtdf2(a: f64, b: f64) -> ::core::ffi::c_int;
    pub fn __ledf2(a: f64, b: f64) -> ::core::ffi::c_int;
    pub fn __ltdf2(a: f64, b: f64) -> ::core::ffi::c_int;
    pub fn __extendsfdf2(a: f32) -> f64;
    pub fn __truncdfsf2(a: f64) -> f32;
    pub fn __extendhfsf2(a: __BindgenFloat16) -> f32;
    pub fn __truncsfhf2(a: f32) -> __BindgenFloat16;
    pub fn __fixdfsi(a: f64) -> i32;
    pub fn __fixdfdi(a: f64) -> i64;
    pub fn __fixunsdfsi(a: f64) -> u32;
    pub fn __floatdisf(a: i64) -> f32;
    pub fn __floatsidf(a: i32) -> f64;
    pub fn __floatunsidf(a: u32) -> f64;
    pub fn __divdi3(a: i64, b: i64) -> i64;
    pub fn __udivdi3(a: u64, b: u64) -> u64;
    pub fn __umoddi3(a: u64, b: u64) -> u64;
    pub fn __clzsi2(a: u32) -> ::core::ffi::c_int;
    pub fn __popcountsi2(a: u32) -> ::core::ffi::c_int;
    pub fn __riscv_save_0();
    pub fn __riscv_save_1();
    pub fn __riscv_save_2();
    pub fn __riscv_save_3();
    pub fn __riscv_save_4();
    pub fn __riscv_save_5();
    pub fn __riscv_save_6();
    pub fn __riscv_save_7();
    pub fn __riscv_save_8();
    pub fn __riscv_save_9();
    pub fn __riscv_save_10();
    pub fn __riscv_save_11();
    pub fn __riscv_save_12();
    pub fn __riscv_restore_0();
    pub fn __riscv_restore_1();
    pub fn __riscv_restore_2();
    pub fn __riscv_restore_3();
    pub fn __riscv_restore_4();
    pub fn __riscv_restore_5();
    pub fn __riscv_restore_6();
    pub fn __riscv_restore_7();
    pub fn __riscv_restore_8();
    pub fn __riscv_restore_9();
    pub fn __riscv_restore_10();
    pub fn __riscv_restore_11();
    pub fn __riscv_restore_12();
}
