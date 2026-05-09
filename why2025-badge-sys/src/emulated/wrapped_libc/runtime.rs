use crate::{
    __compar_fn_t, _ssize_t, DIR, FILE, VISIT, addrinfo, clock_t, clockid_t, dirent, div_t, fpos_t,
    iconv_t, imaxdiv_t, in_addr, intmax_t, lconv, ldiv_t, lldiv_t, mbstate_t, mode_t, nl_item,
    off_t, option, pid_t, re_guts, regex_t, sockaddr, socklen_t, stat as stat_t, termios, time_t,
    tm, tms, uintmax_t, useconds_t, wchar_t, wctrans_t, wctype_t, wint_t,
};
use core::ffi::{c_char, c_int, c_long, c_longlong, c_uint, c_ulong, c_ulonglong, c_void};
use core::mem;
use core::sync::atomic::{AtomicUsize, Ordering};
use std::io::{self, Write};
use std::sync::OnceLock;

static GETPID_INTERPOSE_CALLS: AtomicUsize = AtomicUsize::new(0);
static GETENV_INTERPOSE_CALLS: AtomicUsize = AtomicUsize::new(0);

const HOST_REGEX_MAGIC: c_uint = 0x5752_4558;

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

type HostRegcompFn = unsafe extern "C" fn(*mut HostRegex, *const c_char, c_int) -> c_int;
type HostRegerrorFn = unsafe extern "C" fn(c_int, *const HostRegex, *mut c_char, usize) -> usize;
type HostRegexecFn =
    unsafe extern "C" fn(*const HostRegex, *const c_char, usize, *mut HostRegmatch, c_int) -> c_int;
type HostRegfreeFn = unsafe extern "C" fn(*mut HostRegex);

pub(crate) fn abort_missing_symbol(symbol: &str) -> ! {
    let _ = writeln!(
        io::stderr(),
        "why2025-badge-sys could not resolve {symbol} via RTLD_NEXT"
    );
    std::process::abort();
}

pub(crate) fn abort_incompatible_symbol(symbol: &str, reason: &str) -> ! {
    let _ = writeln!(
        io::stderr(),
        "why2025-badge-sys cannot host-forward {symbol}: {reason}"
    );
    std::process::abort();
}

fn resolve_symbol(symbol: &'static [u8]) -> *mut c_void {
    unsafe {
        libc::dlerror();
        let resolved = libc::dlsym(libc::RTLD_NEXT, symbol.as_ptr().cast::<c_char>());
        let error = libc::dlerror();

        if !error.is_null() || resolved.is_null() {
            let name = core::str::from_utf8(&symbol[..symbol.len() - 1]).unwrap_or("<invalid>");
            abort_missing_symbol(name);
        }

        resolved
    }
}

unsafe fn resolve_object_value<T: Copy>(symbol: &'static [u8]) -> T {
    unsafe { *resolve_symbol(symbol).cast::<T>() }
}

macro_rules! dlsym_resolver {
    (
        $slot:ident,
        $resolver:ident,
        $symbol:literal,
        fn $name:ident($($arg:ident : $arg_ty:ty),* $(,)?) -> $ret:ty
    ) => {
        static $slot: OnceLock<unsafe extern "C" fn($($arg_ty),*) -> $ret> = OnceLock::new();

        pub(crate) fn $resolver() -> unsafe extern "C" fn($($arg_ty),*) -> $ret {
            *$slot.get_or_init(|| unsafe {
                mem::transmute::<*mut c_void, unsafe extern "C" fn($($arg_ty),*) -> $ret>(
                    resolve_symbol($symbol),
                )
            })
        }
    };
}

macro_rules! call_resolved {
    ($resolver:path $(, $arg:expr )* $(,)?) => {{
        unsafe { $resolver()($($arg),*) }
    }};
}

pub(crate) use call_resolved;

macro_rules! dlsym_resolver_group {
    ($(
        $slot:ident,
        $resolver:ident,
        $symbol:literal,
        fn $name:ident($($arg:ident : $arg_ty:ty),* $(,)?) -> $ret:ty;
    )+) => {
        $(dlsym_resolver!($slot, $resolver, $symbol, fn $name($($arg : $arg_ty),*) -> $ret);)+
    };
}

dlsym_resolver!(REAL__EXIT_CAP, real_exit_cap, b"_Exit\0", fn exit_cap(status: c_int) -> !);
dlsym_resolver!(REAL__EXIT, real_exit_underscore, b"_exit\0", fn exit_underscore(status: c_int) -> !);
dlsym_resolver!(REAL_ABORT, real_abort, b"abort\0", fn abort() -> !);
dlsym_resolver!(REAL_ACCEPT, real_accept, b"accept\0", fn accept(sockfd: c_int, addr: *mut sockaddr, addrlen: *mut socklen_t) -> c_int);
dlsym_resolver!(REAL_ASCTIME, real_asctime, b"asctime\0", fn asctime(tblock: *const tm) -> *mut c_char);
dlsym_resolver!(REAL_BIND, real_bind, b"bind\0", fn bind(sockfd: c_int, addr: *const sockaddr, addrlen: socklen_t) -> c_int);
dlsym_resolver!(REAL_CLOSE, real_close, b"close\0", fn close(fd: c_int) -> c_int);
dlsym_resolver!(REAL_CLOSEDIR, real_closedir, b"closedir\0", fn closedir(dir: *mut DIR) -> c_int);
dlsym_resolver!(REAL_CLEARERR, real_clearerr, b"clearerr\0", fn clearerr(stream: *mut FILE) -> ());
dlsym_resolver!(REAL_CLEARERR_UNLOCKED, real_clearerr_unlocked, b"clearerr_unlocked\0", fn clearerr_unlocked(stream: *mut FILE) -> ());
dlsym_resolver!(REAL_CONNECT, real_connect, b"connect\0", fn connect(sockfd: c_int, addr: *const sockaddr, addrlen: socklen_t) -> c_int);
dlsym_resolver!(REAL_CTIME, real_ctime, b"ctime\0", fn ctime(timer: *const time_t) -> *mut c_char);
dlsym_resolver!(REAL_EXIT, real_exit, b"exit\0", fn exit(status: c_int) -> !);
dlsym_resolver!(REAL_FCLOSE, real_fclose, b"fclose\0", fn fclose(stream: *mut FILE) -> c_int);
dlsym_resolver!(REAL_FDOPEN, real_fdopen, b"fdopen\0", fn fdopen(fd: c_int, mode: *const c_char) -> *mut FILE);
dlsym_resolver!(REAL_FEOF, real_feof, b"feof\0", fn feof(stream: *mut FILE) -> c_int);
dlsym_resolver!(REAL_FERROR, real_ferror, b"ferror\0", fn ferror(stream: *mut FILE) -> c_int);
dlsym_resolver!(REAL_FFLUSH, real_fflush, b"fflush\0", fn fflush(stream: *mut FILE) -> c_int);
dlsym_resolver!(REAL_FGETC, real_fgetc, b"fgetc\0", fn fgetc(stream: *mut FILE) -> c_int);
dlsym_resolver!(REAL_FGETPOS, real_fgetpos, b"fgetpos\0", fn fgetpos(stream: *mut FILE, pos: *mut fpos_t) -> c_int);
dlsym_resolver!(REAL_FGETS, real_fgets, b"fgets\0", fn fgets(buf: *mut c_char, size: c_int, stream: *mut FILE) -> *mut c_char);
dlsym_resolver!(REAL_FILENO, real_fileno, b"fileno\0", fn fileno(stream: *mut FILE) -> c_int);
dlsym_resolver!(REAL_FMEMOPEN, real_fmemopen, b"fmemopen\0", fn fmemopen(buf: *mut c_void, size: usize, mode: *const c_char) -> *mut FILE);
dlsym_resolver!(REAL_FOPEN, real_fopen, b"fopen\0", fn fopen(path: *const c_char, mode: *const c_char) -> *mut FILE);
dlsym_resolver!(REAL_FPUTC, real_fputc, b"fputc\0", fn fputc(value: c_int, stream: *mut FILE) -> c_int);
dlsym_resolver!(REAL_FPUTS, real_fputs, b"fputs\0", fn fputs(value: *const c_char, stream: *mut FILE) -> c_int);
dlsym_resolver!(REAL_FREAD, real_fread, b"fread\0", fn fread(ptr: *mut c_void, size: c_uint, nmemb: c_uint, stream: *mut FILE) -> c_uint);
dlsym_resolver!(REAL_FREOPEN, real_freopen, b"freopen\0", fn freopen(path: *const c_char, mode: *const c_char, stream: *mut FILE) -> *mut FILE);
dlsym_resolver!(REAL_FSEEK, real_fseek, b"fseek\0", fn fseek(stream: *mut FILE, offset: c_long, whence: c_int) -> c_int);
dlsym_resolver!(REAL_FSEEKO, real_fseeko, b"fseeko\0", fn fseeko(stream: *mut FILE, offset: off_t, whence: c_int) -> c_int);
dlsym_resolver!(REAL_FSTAT, real_fstat, b"fstat\0", fn fstat(fd: c_int, buf: *mut stat_t) -> c_int);
dlsym_resolver!(REAL_FREEADDRINFO, real_freeaddrinfo, b"freeaddrinfo\0", fn freeaddrinfo(ai: *mut addrinfo) -> ());
dlsym_resolver!(REAL_FTELL, real_ftell, b"ftell\0", fn ftell(stream: *mut FILE) -> c_long);
dlsym_resolver!(REAL_FTELLO, real_ftello, b"ftello\0", fn ftello(stream: *mut FILE) -> off_t);
dlsym_resolver!(REAL_FWRITE, real_fwrite, b"fwrite\0", fn fwrite(ptr: *const c_void, size: c_uint, nmemb: c_uint, stream: *mut FILE) -> c_uint);
dlsym_resolver!(REAL_GETADDRINFO, real_getaddrinfo, b"getaddrinfo\0", fn getaddrinfo(nodename: *const c_char, servname: *const c_char, hints: *const addrinfo, res: *mut *mut addrinfo) -> c_int);
dlsym_resolver!(REAL_GETDELIM, real_getdelim, b"getdelim\0", fn getdelim(lineptr: *mut *mut c_char, n: *mut usize, delim: c_int, stream: *mut FILE) -> _ssize_t);
dlsym_resolver!(REAL_GETC, real_getc, b"getc\0", fn getc(stream: *mut FILE) -> c_int);
dlsym_resolver!(REAL_GETCHAR, real_getchar, b"getchar\0", fn getchar() -> c_int);
dlsym_resolver!(REAL_GETCHAR_UNLOCKED, real_getchar_unlocked, b"getchar_unlocked\0", fn getchar_unlocked() -> c_int);
dlsym_resolver!(REAL_GETENV, real_getenv, b"getenv\0", fn getenv(name: *const c_char) -> *mut c_char);
dlsym_resolver!(REAL_GETSUBOPT, real_getsubopt, b"getsubopt\0", fn getsubopt(optionp: *mut *mut c_char, tokens: *const *mut c_char, valuep: *mut *mut c_char) -> c_int);
dlsym_resolver!(REAL_GETLINE, real_getline, b"getline\0", fn getline(lineptr: *mut *mut c_char, n: *mut usize, stream: *mut FILE) -> _ssize_t);
dlsym_resolver!(REAL_GETPID, real_getpid, b"getpid\0", fn getpid() -> pid_t);
dlsym_resolver!(REAL_GETS, real_gets, b"gets\0", fn gets(buf: *mut c_char) -> *mut c_char);
dlsym_resolver!(REAL_GETTIMEOFDAY, real_gettimeofday, b"gettimeofday\0", fn gettimeofday(value: *mut libc::timeval, tz: *mut c_void) -> c_int);
dlsym_resolver!(REAL_GMTIME, real_gmtime, b"gmtime\0", fn gmtime(timer: *const time_t) -> *mut tm);
dlsym_resolver!(REAL_GMTIME_R, real_gmtime_r, b"gmtime_r\0", fn gmtime_r(timer: *const time_t, result: *mut libc::tm) -> *mut libc::tm);
dlsym_resolver!(REAL_INET_ATON, real_inet_aton, b"inet_aton\0", fn inet_aton(cp: *const c_char, inp: *mut in_addr) -> c_int);
dlsym_resolver!(REAL_INET_NTOA, real_inet_ntoa, b"inet_ntoa\0", fn inet_ntoa(addr: in_addr) -> *mut c_char);
dlsym_resolver!(REAL_ICONV_CLOSE, real_iconv_close, b"iconv_close\0", fn iconv_close(cd: iconv_t) -> c_int);
dlsym_resolver!(REAL_ICONV_OPEN, real_iconv_open, b"iconv_open\0", fn iconv_open(tocode: *const c_char, fromcode: *const c_char) -> iconv_t);
dlsym_resolver!(REAL_ISALNUM, real_isalnum, b"isalnum\0", fn isalnum(value: c_int) -> c_int);
dlsym_resolver!(REAL_ISALPHA, real_isalpha, b"isalpha\0", fn isalpha(value: c_int) -> c_int);
dlsym_resolver!(REAL_ISBLANK, real_isblank, b"isblank\0", fn isblank(value: c_int) -> c_int);
dlsym_resolver!(REAL_ISCNTRL, real_iscntrl, b"iscntrl\0", fn iscntrl(value: c_int) -> c_int);
dlsym_resolver!(REAL_ISATTY, real_isatty, b"isatty\0", fn isatty(fd: c_int) -> c_int);
dlsym_resolver!(REAL_ISDIGIT, real_isdigit, b"isdigit\0", fn isdigit(value: c_int) -> c_int);
dlsym_resolver!(REAL_ISGRAPH, real_isgraph, b"isgraph\0", fn isgraph(value: c_int) -> c_int);
dlsym_resolver!(REAL_ISLOWER, real_islower, b"islower\0", fn islower(value: c_int) -> c_int);
dlsym_resolver!(REAL_ISPRINT, real_isprint, b"isprint\0", fn isprint(value: c_int) -> c_int);
dlsym_resolver!(REAL_ISPUNCT, real_ispunct, b"ispunct\0", fn ispunct(value: c_int) -> c_int);
dlsym_resolver!(REAL_ISSPACE, real_isspace, b"isspace\0", fn isspace(value: c_int) -> c_int);
dlsym_resolver!(REAL_ISUPPER, real_isupper, b"isupper\0", fn isupper(value: c_int) -> c_int);
dlsym_resolver!(REAL_ISXDIGIT, real_isxdigit, b"isxdigit\0", fn isxdigit(value: c_int) -> c_int);
dlsym_resolver!(REAL_ISASCII, real_isascii, b"isascii\0", fn isascii(value: c_int) -> c_int);
dlsym_resolver!(REAL_LISTEN, real_listen, b"listen\0", fn listen(sockfd: c_int, backlog: c_int) -> c_int);
dlsym_resolver!(REAL_LOCALTIME, real_localtime, b"localtime\0", fn localtime(timer: *const time_t) -> *mut tm);
dlsym_resolver!(REAL_LOCALTIME_R, real_localtime_r, b"localtime_r\0", fn localtime_r(timer: *const time_t, result: *mut libc::tm) -> *mut libc::tm);
dlsym_resolver!(REAL_LSEEK, real_lseek, b"lseek\0", fn lseek(fd: c_int, offset: off_t, whence: c_int) -> off_t);
dlsym_resolver!(REAL_MEMCCPY, real_memccpy, b"memccpy\0", fn memccpy(dst: *mut c_void, src: *const c_void, needle: c_int, count: c_uint) -> *mut c_void);
dlsym_resolver!(REAL_MEMCHR, real_memchr, b"memchr\0", fn memchr(value: *const c_void, needle: c_int, count: c_uint) -> *mut c_void);
dlsym_resolver!(REAL_MEMCMP, real_memcmp, b"memcmp\0", fn memcmp(left: *const c_void, right: *const c_void, count: c_uint) -> c_int);
dlsym_resolver!(REAL_MEMCPY, real_memcpy, b"memcpy\0", fn memcpy(dst: *mut c_void, src: *const c_void, count: c_uint) -> *mut c_void);
dlsym_resolver!(REAL_MEMMEM, real_memmem, b"memmem\0", fn memmem(haystack: *const c_void, haystack_len: usize, needle: *const c_void, needle_len: usize) -> *mut c_void);
dlsym_resolver!(REAL_MEMMOVE, real_memmove, b"memmove\0", fn memmove(dst: *mut c_void, src: *const c_void, count: c_uint) -> *mut c_void);
dlsym_resolver!(REAL_MEMPCPY, real_mempcpy, b"mempcpy\0", fn mempcpy(dst: *mut c_void, src: *const c_void, count: c_uint) -> *mut c_void);
dlsym_resolver!(REAL_MEMRCHR, real_memrchr, b"memrchr\0", fn memrchr(value: *const c_void, needle: c_int, count: usize) -> *mut c_void);
dlsym_resolver!(REAL_MEMSET, real_memset, b"memset\0", fn memset(dst: *mut c_void, value: c_int, count: c_uint) -> *mut c_void);
dlsym_resolver!(REAL_RAWMEMCHR, real_rawmemchr, b"rawmemchr\0", fn rawmemchr(value: *const c_void, needle: c_int) -> *mut c_void);
dlsym_resolver!(REAL_MKDIR, real_mkdir, b"mkdir\0", fn mkdir(path: *const c_char, mode: mode_t) -> c_int);
dlsym_resolver!(REAL_MKTIME, real_mktime, b"mktime\0", fn mktime(timeptr: *mut libc::tm) -> time_t);
dlsym_resolver!(REAL_OPENDIR, real_opendir, b"opendir\0", fn opendir(name: *const c_char) -> *mut DIR);
dlsym_resolver!(REAL_PUTCHAR, real_putchar, b"putchar\0", fn putchar(value: c_int) -> c_int);
dlsym_resolver!(REAL_PUTS, real_puts, b"puts\0", fn puts(value: *const c_char) -> c_int);
dlsym_resolver!(REAL_RAND, real_rand, b"rand\0", fn rand() -> c_int);
dlsym_resolver!(REAL_RANDOM, real_random, b"random\0", fn random() -> c_long);
dlsym_resolver!(REAL_READ, real_read, b"read\0", fn read(fd: c_int, buf: *mut c_void, count: usize) -> isize);
dlsym_resolver!(REAL_READDIR, real_readdir, b"readdir\0", fn readdir(dir: *mut DIR) -> *mut dirent);
dlsym_resolver!(REAL_REMOVE, real_remove, b"remove\0", fn remove(path: *const c_char) -> c_int);
dlsym_resolver!(REAL_RENAME, real_rename, b"rename\0", fn rename(old: *const c_char, new: *const c_char) -> c_int);
dlsym_resolver!(REAL_REWIND, real_rewind, b"rewind\0", fn rewind(stream: *mut FILE) -> ());
dlsym_resolver!(REAL_REWINDDIR, real_rewinddir, b"rewinddir\0", fn rewinddir(dir: *mut DIR) -> ());
dlsym_resolver!(REAL_RMDIR, real_rmdir, b"rmdir\0", fn rmdir(path: *const c_char) -> c_int);
dlsym_resolver!(REAL_SELECT, real_select, b"select\0", fn select(n: c_int, readfds: *mut libc::fd_set, writefds: *mut libc::fd_set, exceptfds: *mut libc::fd_set, timeout: *mut libc::timeval) -> c_int);
dlsym_resolver!(REAL_SETBUF, real_setbuf, b"setbuf\0", fn setbuf(stream: *mut FILE, buf: *mut c_char) -> ());
dlsym_resolver!(REAL_SETBUFFER, real_setbuffer, b"setbuffer\0", fn setbuffer(stream: *mut FILE, buf: *mut c_char, size: usize) -> ());
dlsym_resolver!(REAL_SETLINEBUF, real_setlinebuf, b"setlinebuf\0", fn setlinebuf(stream: *mut FILE) -> ());
dlsym_resolver!(REAL_SETVBUF, real_setvbuf, b"setvbuf\0", fn setvbuf(stream: *mut FILE, buf: *mut c_char, mode: c_int, size: usize) -> c_int);
dlsym_resolver!(REAL_SOCKET, real_socket, b"socket\0", fn socket(domain: c_int, ty: c_int, protocol: c_int) -> c_int);
dlsym_resolver!(REAL_SRAND, real_srand, b"srand\0", fn srand(seed: c_uint) -> ());
dlsym_resolver!(REAL_SRANDOM, real_srandom, b"srandom\0", fn srandom(seed: c_uint) -> ());
dlsym_resolver!(REAL_STAT, real_stat, b"stat\0", fn stat(path: *const c_char, buf: *mut stat_t) -> c_int);
dlsym_resolver!(REAL_STPCPY, real_stpcpy, b"stpcpy\0", fn stpcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char);
dlsym_resolver!(REAL_STPNCPY, real_stpncpy, b"stpncpy\0", fn stpncpy(dst: *mut c_char, src: *const c_char, count: c_uint) -> *mut c_char);
dlsym_resolver!(REAL_STRCASESTR, real_strcasestr, b"strcasestr\0", fn strcasestr(haystack: *const c_char, needle: *const c_char) -> *mut c_char);
dlsym_resolver!(REAL_STRCAT, real_strcat, b"strcat\0", fn strcat(dst: *mut c_char, src: *const c_char) -> *mut c_char);
dlsym_resolver!(REAL_STRCHR, real_strchr, b"strchr\0", fn strchr(value: *const c_char, needle: c_int) -> *mut c_char);
dlsym_resolver!(REAL_STRCHRNUL, real_strchrnul, b"strchrnul\0", fn strchrnul(value: *const c_char, needle: c_int) -> *mut c_char);
dlsym_resolver!(REAL_STRCMP, real_strcmp, b"strcmp\0", fn strcmp(left: *const c_char, right: *const c_char) -> c_int);
dlsym_resolver!(REAL_STRCPY, real_strcpy, b"strcpy\0", fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char);
dlsym_resolver!(REAL_STRCSPN, real_strcspn, b"strcspn\0", fn strcspn(value: *const c_char, reject: *const c_char) -> c_uint);
dlsym_resolver!(REAL_STRFTIME, real_strftime, b"strftime\0", fn strftime(value: *mut c_char, maxsize: usize, fmt: *const c_char, tblock: *const libc::tm) -> usize);
dlsym_resolver!(REAL_STRDUP, real_strdup, b"strdup\0", fn strdup(value: *const c_char) -> *mut c_char);
dlsym_resolver!(REAL_STRERROR, real_strerror, b"strerror\0", fn strerror(errnum: c_int) -> *mut c_char);
dlsym_resolver!(REAL_STRERROR_R, real_strerror_r, b"strerror_r\0", fn strerror_r(errnum: c_int, buf: *mut c_char, size: usize) -> *mut c_char);
dlsym_resolver!(REAL_STRPTIME, real_strptime, b"strptime\0", fn strptime(input: *const c_char, fmt: *const c_char, result: *mut libc::tm) -> *mut c_char);
dlsym_resolver!(REAL_STRLCAT, real_strlcat, b"strlcat\0", fn strlcat(dst: *mut c_char, src: *const c_char, size: c_uint) -> c_uint);
dlsym_resolver!(REAL_STRLEN, real_strlen, b"strlen\0", fn strlen(value: *const c_char) -> c_uint);
dlsym_resolver!(REAL_STRLCPY, real_strlcpy, b"strlcpy\0", fn strlcpy(dst: *mut c_char, src: *const c_char, size: c_uint) -> c_uint);
dlsym_resolver!(REAL_STRNCAT, real_strncat, b"strncat\0", fn strncat(dst: *mut c_char, src: *const c_char, count: c_uint) -> *mut c_char);
dlsym_resolver!(REAL_STRNCMP, real_strncmp, b"strncmp\0", fn strncmp(left: *const c_char, right: *const c_char, count: c_uint) -> c_int);
dlsym_resolver!(REAL_STRNCPY, real_strncpy, b"strncpy\0", fn strncpy(dst: *mut c_char, src: *const c_char, count: c_uint) -> *mut c_char);
dlsym_resolver!(REAL_STRNDUP, real_strndup, b"strndup\0", fn strndup(value: *const c_char, count: c_uint) -> *mut c_char);
dlsym_resolver!(REAL_STRNLEN, real_strnlen, b"strnlen\0", fn strnlen(value: *const c_char, count: usize) -> usize);
dlsym_resolver!(REAL_STRPBRK, real_strpbrk, b"strpbrk\0", fn strpbrk(value: *const c_char, accept: *const c_char) -> *mut c_char);
dlsym_resolver!(REAL_STRRCHR, real_strrchr, b"strrchr\0", fn strrchr(value: *const c_char, needle: c_int) -> *mut c_char);
dlsym_resolver!(REAL_STRSEP, real_strsep, b"strsep\0", fn strsep(stringp: *mut *mut c_char, delim: *const c_char) -> *mut c_char);
dlsym_resolver!(REAL_STRSPN, real_strspn, b"strspn\0", fn strspn(value: *const c_char, accept: *const c_char) -> c_uint);
dlsym_resolver!(REAL_STRSTR, real_strstr, b"strstr\0", fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char);
dlsym_resolver!(REAL_STRTOK, real_strtok, b"strtok\0", fn strtok(value: *mut c_char, delim: *const c_char) -> *mut c_char);
dlsym_resolver!(REAL_STRTOK_R, real_strtok_r, b"strtok_r\0", fn strtok_r(value: *mut c_char, delim: *const c_char, saveptr: *mut *mut c_char) -> *mut c_char);
dlsym_resolver!(REAL_STRVERSCMP, real_strverscmp, b"strverscmp\0", fn strverscmp(left: *const c_char, right: *const c_char) -> c_int);
dlsym_resolver!(REAL_SYSTEM, real_system, b"system\0", fn system(command: *const c_char) -> c_int);
dlsym_resolver!(REAL_TCGETATTR, real_tcgetattr, b"tcgetattr\0", fn tcgetattr(fd: c_int, termios_p: *mut termios) -> c_int);
dlsym_resolver!(REAL_TCSETATTR, real_tcsetattr, b"tcsetattr\0", fn tcsetattr(fd: c_int, action: c_int, termios_p: *const termios) -> c_int);
dlsym_resolver!(REAL_TOLOWER, real_tolower, b"tolower\0", fn tolower(value: c_int) -> c_int);
dlsym_resolver!(REAL_TOASCII, real_toascii, b"toascii\0", fn toascii(value: c_int) -> c_int);
dlsym_resolver!(REAL_TOUPPER, real_toupper, b"toupper\0", fn toupper(value: c_int) -> c_int);
dlsym_resolver!(REAL_UNGETC, real_ungetc, b"ungetc\0", fn ungetc(value: c_int, stream: *mut FILE) -> c_int);
dlsym_resolver!(REAL_UNLINK, real_unlink, b"unlink\0", fn unlink(path: *const c_char) -> c_int);
dlsym_resolver!(REAL_WCSDUP, real_wcsdup, b"wcsdup\0", fn wcsdup(value: *const wchar_t) -> *mut wchar_t);
dlsym_resolver!(REAL_WCSCHR, real_wcschr, b"wcschr\0", fn wcschr(value: *const wchar_t, needle: c_int) -> *mut wchar_t);
dlsym_resolver!(REAL_WCSCAT, real_wcscat, b"wcscat\0", fn wcscat(dst: *mut wchar_t, src: *const wchar_t) -> *mut wchar_t);
dlsym_resolver!(REAL_WCSCASECMP, real_wcscasecmp, b"wcscasecmp\0", fn wcscasecmp(left: *const wchar_t, right: *const wchar_t) -> c_int);
dlsym_resolver!(REAL_WCSCMP, real_wcscmp, b"wcscmp\0", fn wcscmp(left: *const wchar_t, right: *const wchar_t) -> c_int);
dlsym_resolver!(REAL_WSCPY, real_wcscpy, b"wcscpy\0", fn wcscpy(dst: *mut wchar_t, src: *const wchar_t) -> *mut wchar_t);
dlsym_resolver!(REAL_WCSCSPN, real_wcscspn, b"wcscspn\0", fn wcscspn(value: *const wchar_t, reject: *const wchar_t) -> usize);
dlsym_resolver!(REAL_WCSLEN, real_wcslen, b"wcslen\0", fn wcslen(value: *const wchar_t) -> c_uint);
dlsym_resolver!(REAL_WCSNCASECMP, real_wcsncasecmp, b"wcsncasecmp\0", fn wcsncasecmp(left: *const wchar_t, right: *const wchar_t, count: usize) -> c_int);
dlsym_resolver!(REAL_WCSNCMP, real_wcsncmp, b"wcsncmp\0", fn wcsncmp(left: *const wchar_t, right: *const wchar_t, count: c_uint) -> c_int);
dlsym_resolver!(REAL_WCSNCPY, real_wcsncpy, b"wcsncpy\0", fn wcsncpy(dst: *mut wchar_t, src: *const wchar_t, count: usize) -> *mut wchar_t);
dlsym_resolver!(REAL_WCSNLEN, real_wcsnlen, b"wcsnlen\0", fn wcsnlen(value: *const wchar_t, count: usize) -> usize);
dlsym_resolver!(REAL_WCSNCAT, real_wcsncat, b"wcsncat\0", fn wcsncat(dst: *mut wchar_t, src: *const wchar_t, count: usize) -> *mut wchar_t);
dlsym_resolver!(REAL_WCSPBRK, real_wcspbrk, b"wcspbrk\0", fn wcspbrk(value: *const wchar_t, accept: *const wchar_t) -> *mut wchar_t);
dlsym_resolver!(REAL_WCSRCHR, real_wcsrchr, b"wcsrchr\0", fn wcsrchr(value: *const wchar_t, needle: wchar_t) -> *mut wchar_t);
dlsym_resolver!(REAL_WCSSPN, real_wcsspn, b"wcsspn\0", fn wcsspn(value: *const wchar_t, accept: *const wchar_t) -> usize);
dlsym_resolver!(REAL_WCSSTR, real_wcsstr, b"wcsstr\0", fn wcsstr(haystack: *const wchar_t, needle: *const wchar_t) -> *mut wchar_t);
dlsym_resolver!(REAL_WCSFTIME, real_wcsftime, b"wcsftime\0", fn wcsftime(value: *mut wchar_t, maxsize: usize, fmt: *const wchar_t, tblock: *const libc::tm) -> usize);
dlsym_resolver!(REAL_WCSWIDTH, real_wcswidth, b"wcswidth\0", fn wcswidth(value: *const wchar_t, count: usize) -> c_int);
dlsym_resolver!(REAL_WCSTOK, real_wcstok, b"wcstok\0", fn wcstok(value: *mut wchar_t, delim: *const wchar_t, saveptr: *mut *mut wchar_t) -> *mut wchar_t);
dlsym_resolver!(REAL_WCTOB, real_wctob, b"wctob\0", fn wctob(value: wint_t) -> c_int);
dlsym_resolver!(REAL_WCWIDTH, real_wcwidth, b"wcwidth\0", fn wcwidth(value: wchar_t) -> c_int);
dlsym_resolver!(REAL_WMEMCMP, real_wmemcmp, b"wmemcmp\0", fn wmemcmp(left: *const wchar_t, right: *const wchar_t, count: c_uint) -> c_int);
dlsym_resolver!(REAL_WMEMCHR, real_wmemchr, b"wmemchr\0", fn wmemchr(value: *const wchar_t, needle: c_int, count: c_uint) -> *mut wchar_t);
dlsym_resolver!(REAL_WMEMCPY, real_wmemcpy, b"wmemcpy\0", fn wmemcpy(dst: *mut wchar_t, src: *const wchar_t, count: c_uint) -> *mut wchar_t);
dlsym_resolver!(REAL_WMEMMOVE, real_wmemmove, b"wmemmove\0", fn wmemmove(dst: *mut wchar_t, src: *const wchar_t, count: c_uint) -> *mut wchar_t);
dlsym_resolver!(REAL_WMEMPCPY, real_wmempcpy, b"wmempcpy\0", fn wmempcpy(dst: *mut wchar_t, src: *const wchar_t, count: usize) -> *mut wchar_t);
dlsym_resolver!(REAL_WMEMSET, real_wmemset, b"wmemset\0", fn wmemset(dst: *mut wchar_t, value: wchar_t, count: usize) -> *mut wchar_t);
dlsym_resolver!(REAL_ISWALNUM, real_iswalnum, b"iswalnum\0", fn iswalnum(value: wint_t) -> c_int);
dlsym_resolver!(REAL_ISWALPHA, real_iswalpha, b"iswalpha\0", fn iswalpha(value: wint_t) -> c_int);
dlsym_resolver!(REAL_ISWBLANK, real_iswblank, b"iswblank\0", fn iswblank(value: wint_t) -> c_int);
dlsym_resolver!(REAL_ISWCNTRL, real_iswcntrl, b"iswcntrl\0", fn iswcntrl(value: wint_t) -> c_int);
dlsym_resolver!(REAL_ISWCTYPE, real_iswctype, b"iswctype\0", fn iswctype(value: wint_t, desc: wctype_t) -> c_int);
dlsym_resolver!(REAL_ISWDIGIT, real_iswdigit, b"iswdigit\0", fn iswdigit(value: wint_t) -> c_int);
dlsym_resolver!(REAL_ISWGRAPH, real_iswgraph, b"iswgraph\0", fn iswgraph(value: wint_t) -> c_int);
dlsym_resolver!(REAL_ISWLOWER, real_iswlower, b"iswlower\0", fn iswlower(value: wint_t) -> c_int);
dlsym_resolver!(REAL_ISWPRINT, real_iswprint, b"iswprint\0", fn iswprint(value: wint_t) -> c_int);
dlsym_resolver!(REAL_ISWPUNCT, real_iswpunct, b"iswpunct\0", fn iswpunct(value: wint_t) -> c_int);
dlsym_resolver!(REAL_ISWSPACE, real_iswspace, b"iswspace\0", fn iswspace(value: wint_t) -> c_int);
dlsym_resolver!(REAL_ISWUPPER, real_iswupper, b"iswupper\0", fn iswupper(value: wint_t) -> c_int);
dlsym_resolver!(REAL_ISWXDIGIT, real_iswxdigit, b"iswxdigit\0", fn iswxdigit(value: wint_t) -> c_int);
dlsym_resolver!(REAL_TOWLOWER, real_towlower, b"towlower\0", fn towlower(value: wint_t) -> wint_t);
dlsym_resolver!(REAL_TOWUPPER, real_towupper, b"towupper\0", fn towupper(value: wint_t) -> wint_t);
dlsym_resolver!(REAL_WCTRANS, real_wctrans, b"wctrans\0", fn wctrans(name: *const c_char) -> wctrans_t);
dlsym_resolver!(REAL_WCTYPE, real_wctype, b"wctype\0", fn wctype(name: *const c_char) -> wctype_t);
dlsym_resolver_group! {
    REAL_A64L, real_a64l, b"a64l\0", fn a64l(input: *const c_char) -> c_long;
    REAL_ABS, real_abs, b"abs\0", fn abs(value: c_int) -> c_int;
    REAL_ACOS, real_acos, b"acos\0", fn acos(value: f64) -> f64;
    REAL_ACOSF, real_acosf, b"acosf\0", fn acosf(value: f32) -> f32;
    REAL_ACOSH, real_acosh, b"acosh\0", fn acosh(value: f64) -> f64;
    REAL_ACOSHF, real_acoshf, b"acoshf\0", fn acoshf(value: f32) -> f32;
    REAL_ASIN, real_asin, b"asin\0", fn asin(value: f64) -> f64;
    REAL_ASINF, real_asinf, b"asinf\0", fn asinf(value: f32) -> f32;
    REAL_ASINH, real_asinh, b"asinh\0", fn asinh(value: f64) -> f64;
    REAL_ASINHF, real_asinhf, b"asinhf\0", fn asinhf(value: f32) -> f32;
    REAL_ASCTIME_R, real_asctime_r, b"asctime_r\0", fn asctime_r(timer: *const tm, buf: *mut [c_char; 26usize]) -> *mut c_char;
    REAL_ATAN, real_atan, b"atan\0", fn atan(value: f64) -> f64;
    REAL_ATAN2, real_atan2, b"atan2\0", fn atan2(left: f64, right: f64) -> f64;
    REAL_ATAN2F, real_atan2f, b"atan2f\0", fn atan2f(left: f32, right: f32) -> f32;
    REAL_ATANF, real_atanf, b"atanf\0", fn atanf(value: f32) -> f32;
    REAL_ATANH, real_atanh, b"atanh\0", fn atanh(value: f64) -> f64;
    REAL_ATANHF, real_atanhf, b"atanhf\0", fn atanhf(value: f32) -> f32;
    REAL_ATOF, real_atof, b"atof\0", fn atof(value: *const c_char) -> f64;
    REAL_ATOI, real_atoi, b"atoi\0", fn atoi(value: *const c_char) -> c_int;
    REAL_ATOL, real_atol, b"atol\0", fn atol(value: *const c_char) -> c_long;
    REAL_ATOLL, real_atoll, b"atoll\0", fn atoll(value: *const c_char) -> c_longlong;
    REAL_BCMP, real_bcmp, b"bcmp\0", fn bcmp(left: *const c_void, right: *const c_void, count: c_uint) -> c_int;
    REAL_BCOPY, real_bcopy, b"bcopy\0", fn bcopy(src: *const c_void, dst: *mut c_void, count: c_uint) -> ();
    REAL_BSEARCH, real_bsearch, b"bsearch\0", fn bsearch(key: *const c_void, base: *const c_void, nmemb: usize, size: usize, compar: __compar_fn_t) -> *mut c_void;
    REAL_BTOWC, real_btowc, b"btowc\0", fn btowc(value: c_int) -> wint_t;
    REAL_BZERO, real_bzero, b"bzero\0", fn bzero(ptr: *mut c_void, count: c_uint) -> ();
    REAL_CALLOC, real_calloc, b"calloc\0", fn calloc(count: c_uint, size: c_uint) -> *mut c_void;
    REAL_CBRT, real_cbrt, b"cbrt\0", fn cbrt(value: f64) -> f64;
    REAL_CBRTF, real_cbrtf, b"cbrtf\0", fn cbrtf(value: f32) -> f32;
    REAL_CEIL, real_ceil, b"ceil\0", fn ceil(value: f64) -> f64;
    REAL_CEILF, real_ceilf, b"ceilf\0", fn ceilf(value: f32) -> f32;
    REAL_CLOCK, real_clock, b"clock\0", fn clock() -> clock_t;
    REAL_COPYSIGN, real_copysign, b"copysign\0", fn copysign(left: f64, right: f64) -> f64;
    REAL_COPYSIGNF, real_copysignf, b"copysignf\0", fn copysignf(left: f32, right: f32) -> f32;
    REAL_COS, real_cos, b"cos\0", fn cos(value: f64) -> f64;
    REAL_COSF, real_cosf, b"cosf\0", fn cosf(value: f32) -> f32;
    REAL_COSH, real_cosh, b"cosh\0", fn cosh(value: f64) -> f64;
    REAL_COSHF, real_coshf, b"coshf\0", fn coshf(value: f32) -> f32;
    REAL_CTIME_R, real_ctime_r, b"ctime_r\0", fn ctime_r(timer: *const time_t, buf: *mut [c_char; 26usize]) -> *mut c_char;
    REAL_DIFFTIME, real_difftime, b"difftime\0", fn difftime(time2: time_t, time1: time_t) -> f64;
    REAL_DIV, real_div, b"div\0", fn div(numer: c_int, denom: c_int) -> div_t;
    REAL_DREM, real_drem, b"drem\0", fn drem(left: f64, right: f64) -> f64;
    REAL_DREMF, real_dremf, b"dremf\0", fn dremf(left: f32, right: f32) -> f32;
    REAL_ERF, real_erf, b"erf\0", fn erf(value: f64) -> f64;
    REAL_ERFC, real_erfc, b"erfc\0", fn erfc(value: f64) -> f64;
    REAL_ERFCF, real_erfcf, b"erfcf\0", fn erfcf(value: f32) -> f32;
    REAL_ERFF, real_erff, b"erff\0", fn erff(value: f32) -> f32;
    REAL_EXP, real_exp, b"exp\0", fn exp(value: f64) -> f64;
    REAL_EXP2, real_exp2, b"exp2\0", fn exp2(value: f64) -> f64;
    REAL_EXP2F, real_exp2f, b"exp2f\0", fn exp2f(value: f32) -> f32;
    REAL_EXPF, real_expf, b"expf\0", fn expf(value: f32) -> f32;
    REAL_EXPLICIT_BZERO, real_explicit_bzero, b"explicit_bzero\0", fn explicit_bzero(ptr: *mut c_void, count: usize) -> ();
    REAL_EXPM1, real_expm1, b"expm1\0", fn expm1(value: f64) -> f64;
    REAL_EXPM1F, real_expm1f, b"expm1f\0", fn expm1f(value: f32) -> f32;
    REAL_FABS, real_fabs, b"fabs\0", fn fabs(value: f64) -> f64;
    REAL_FABSF, real_fabsf, b"fabsf\0", fn fabsf(value: f32) -> f32;
    REAL_FDIM, real_fdim, b"fdim\0", fn fdim(left: f64, right: f64) -> f64;
    REAL_FDIMF, real_fdimf, b"fdimf\0", fn fdimf(left: f32, right: f32) -> f32;
    REAL_FINITE, real_finite, b"finite\0", fn finite(value: f64) -> c_int;
    REAL_FINITEF, real_finitef, b"finitef\0", fn finitef(value: f32) -> c_int;
    REAL_FFS, real_ffs, b"ffs\0", fn ffs(value: c_int) -> c_int;
    REAL_FFSL, real_ffsl, b"ffsl\0", fn ffsl(value: c_long) -> c_int;
    REAL_FFSLL, real_ffsll, b"ffsll\0", fn ffsll(value: c_longlong) -> c_int;
    REAL_FLOOR, real_floor, b"floor\0", fn floor(value: f64) -> f64;
    REAL_FLOORF, real_floorf, b"floorf\0", fn floorf(value: f32) -> f32;
    REAL_FMA, real_fma, b"fma\0", fn fma(left: f64, right: f64, value: f64) -> f64;
    REAL_FMAF, real_fmaf, b"fmaf\0", fn fmaf(left: f32, right: f32, value: f32) -> f32;
    REAL_FMAX, real_fmax, b"fmax\0", fn fmax(left: f64, right: f64) -> f64;
    REAL_FMAXF, real_fmaxf, b"fmaxf\0", fn fmaxf(left: f32, right: f32) -> f32;
    REAL_FMIN, real_fmin, b"fmin\0", fn fmin(left: f64, right: f64) -> f64;
    REAL_FMINF, real_fminf, b"fminf\0", fn fminf(left: f32, right: f32) -> f32;
    REAL_FMOD, real_fmod, b"fmod\0", fn fmod(left: f64, right: f64) -> f64;
    REAL_FMODF, real_fmodf, b"fmodf\0", fn fmodf(left: f32, right: f32) -> f32;
    REAL_FNMATCH, real_fnmatch, b"fnmatch\0", fn fnmatch(pattern: *const c_char, value: *const c_char, flags: c_int) -> c_int;
    REAL_FREE, real_free, b"free\0", fn free(ptr: *mut c_void) -> ();
    REAL_FREXP, real_frexp, b"frexp\0", fn frexp(value: f64, exp: *mut c_int) -> f64;
    REAL_FREXPF, real_frexpf, b"frexpf\0", fn frexpf(value: f32, exp: *mut c_int) -> f32;
    REAL_FWIDE, real_fwide, b"fwide\0", fn fwide(stream: *mut FILE, mode: c_int) -> c_int;
    REAL_GAMMA, real_gamma, b"gamma\0", fn gamma(value: f64) -> f64;
    REAL_GAMMAF, real_gammaf, b"gammaf\0", fn gammaf(value: f32) -> f32;
    REAL_GCVT, real_gcvt, b"gcvt\0", fn gcvt(value: f64, ndigit: c_int, buf: *mut c_char) -> *mut c_char;
    REAL_GETENTROPY, real_getentropy, b"getentropy\0", fn getentropy(buf: *mut c_void, count: usize) -> c_int;
    REAL_GETOPT, real_getopt, b"getopt\0", fn getopt(argc: c_int, argv: *const [*mut c_char; 0usize], optstring: *const c_char) -> c_int;
    REAL_GETOPT_LONG, real_getopt_long, b"getopt_long\0", fn getopt_long(argc: c_int, argv: *const [*mut c_char; 0usize], shortopts: *const c_char, longopts: *const option, longind: *mut c_int) -> c_int;
    REAL_GETOPT_LONG_ONLY, real_getopt_long_only, b"getopt_long_only\0", fn getopt_long_only(argc: c_int, argv: *const [*mut c_char; 0usize], shortopts: *const c_char, longopts: *const option, longind: *mut c_int) -> c_int;
    REAL_HYPOT, real_hypot, b"hypot\0", fn hypot(left: f64, right: f64) -> f64;
    REAL_HYPOTF, real_hypotf, b"hypotf\0", fn hypotf(left: f32, right: f32) -> f32;
    REAL_ICONV, real_iconv, b"iconv\0", fn iconv(cd: iconv_t, inbuf: *mut *mut c_char, inbytesleft: *mut usize, outbuf: *mut *mut c_char, outbytesleft: *mut usize) -> usize;
    REAL_ILOGB, real_ilogb, b"ilogb\0", fn ilogb(value: f64) -> c_int;
    REAL_ILOGBF, real_ilogbf, b"ilogbf\0", fn ilogbf(value: f32) -> c_int;
    REAL_IMAXABS, real_imaxabs, b"imaxabs\0", fn imaxabs(value: intmax_t) -> intmax_t;
    REAL_IMAXDIV, real_imaxdiv, b"imaxdiv\0", fn imaxdiv(numer: intmax_t, denom: intmax_t) -> imaxdiv_t;
    REAL_INDEX, real_index, b"index\0", fn index(value: *const c_char, needle: c_int) -> *mut c_char;
    REAL_ISINF, real_isinf, b"isinf\0", fn isinf(value: f64) -> c_int;
    REAL_ISINFF, real_isinff, b"isinff\0", fn isinff(value: f32) -> c_int;
    REAL_ISNAN, real_isnan, b"isnan\0", fn isnan(value: f64) -> c_int;
    REAL_ISNANF, real_isnanf, b"isnanf\0", fn isnanf(value: f32) -> c_int;
    REAL_J0, real_j0, b"j0\0", fn j0(value: f64) -> f64;
    REAL_J0F, real_j0f, b"j0f\0", fn j0f(value: f32) -> f32;
    REAL_J1, real_j1, b"j1\0", fn j1(value: f64) -> f64;
    REAL_J1F, real_j1f, b"j1f\0", fn j1f(value: f32) -> f32;
    REAL_JN, real_jn, b"jn\0", fn jn(order: c_int, value: f64) -> f64;
    REAL_JNF, real_jnf, b"jnf\0", fn jnf(order: c_int, value: f32) -> f32;
    REAL_L64A, real_l64a, b"l64a\0", fn l64a(value: c_long) -> *mut c_char;
    REAL_LABS, real_labs, b"labs\0", fn labs(value: c_long) -> c_long;
    REAL_LDEXP, real_ldexp, b"ldexp\0", fn ldexp(value: f64, exp: c_int) -> f64;
    REAL_LDEXPF, real_ldexpf, b"ldexpf\0", fn ldexpf(value: f32, exp: c_int) -> f32;
    REAL_LDIV, real_ldiv, b"ldiv\0", fn ldiv(numer: c_long, denom: c_long) -> ldiv_t;
    REAL_LINK, real_link, b"link\0", fn link(path1: *const c_char, path2: *const c_char) -> c_int;
    REAL_LGAMMA, real_lgamma, b"lgamma\0", fn lgamma(value: f64) -> f64;
    REAL_LGAMMA_R, real_lgamma_r, b"lgamma_r\0", fn lgamma_r(value: f64, signgamp: *mut c_int) -> f64;
    REAL_LGAMMAF, real_lgammaf, b"lgammaf\0", fn lgammaf(value: f32) -> f32;
    REAL_LGAMMAF_R, real_lgammaf_r, b"lgammaf_r\0", fn lgammaf_r(value: f32, signgamp: *mut c_int) -> f32;
    REAL_LLABS, real_llabs, b"llabs\0", fn llabs(value: c_longlong) -> c_longlong;
    REAL_LLDIV, real_lldiv, b"lldiv\0", fn lldiv(numer: c_longlong, denom: c_longlong) -> lldiv_t;
    REAL_LLRINT, real_llrint, b"llrint\0", fn llrint(value: f64) -> c_longlong;
    REAL_LLRINTF, real_llrintf, b"llrintf\0", fn llrintf(value: f32) -> c_longlong;
    REAL_LLROUND, real_llround, b"llround\0", fn llround(value: f64) -> c_longlong;
    REAL_LLROUNDF, real_llroundf, b"llroundf\0", fn llroundf(value: f32) -> c_longlong;
    REAL_LOCALECONV, real_localeconv, b"localeconv\0", fn localeconv() -> *mut lconv;
    REAL_LOG, real_log, b"log\0", fn log(value: f64) -> f64;
    REAL_LOG10, real_log10, b"log10\0", fn log10(value: f64) -> f64;
    REAL_LOG10F, real_log10f, b"log10f\0", fn log10f(value: f32) -> f32;
    REAL_LOG1P, real_log1p, b"log1p\0", fn log1p(value: f64) -> f64;
    REAL_LOG1PF, real_log1pf, b"log1pf\0", fn log1pf(value: f32) -> f32;
    REAL_LOG2, real_log2, b"log2\0", fn log2(value: f64) -> f64;
    REAL_LOG2F, real_log2f, b"log2f\0", fn log2f(value: f32) -> f32;
    REAL_LOGB, real_logb, b"logb\0", fn logb(value: f64) -> f64;
    REAL_LOGBF, real_logbf, b"logbf\0", fn logbf(value: f32) -> f32;
    REAL_LOGF, real_logf, b"logf\0", fn logf(value: f32) -> f32;
    REAL_LRINT, real_lrint, b"lrint\0", fn lrint(value: f64) -> c_long;
    REAL_LRINTF, real_lrintf, b"lrintf\0", fn lrintf(value: f32) -> c_long;
    REAL_LROUND, real_lround, b"lround\0", fn lround(value: f64) -> c_long;
    REAL_LROUNDF, real_lroundf, b"lroundf\0", fn lroundf(value: f32) -> c_long;
    REAL_MALLOC, real_malloc, b"malloc\0", fn malloc(size: c_uint) -> *mut c_void;
    REAL_MBLEN, real_mblen, b"mblen\0", fn mblen(value: *const c_char, count: usize) -> c_int;
    REAL_MBRLEN, real_mbrlen, b"mbrlen\0", fn mbrlen(value: *const c_char, count: usize, state: *mut mbstate_t) -> usize;
    REAL_MBRTOWC, real_mbrtowc, b"mbrtowc\0", fn mbrtowc(pwc: *mut wchar_t, value: *const c_char, count: usize, state: *mut mbstate_t) -> usize;
    REAL_MBSINIT, real_mbsinit, b"mbsinit\0", fn mbsinit(state: *const mbstate_t) -> c_int;
    REAL_MBSNRTOWCS, real_mbsnrtowcs, b"mbsnrtowcs\0", fn mbsnrtowcs(dst: *mut wchar_t, src: *mut *const c_char, nwc: usize, len: usize, state: *mut mbstate_t) -> usize;
    REAL_MBSRTOWCS, real_mbsrtowcs, b"mbsrtowcs\0", fn mbsrtowcs(dst: *mut wchar_t, src: *mut *const c_char, len: usize, state: *mut mbstate_t) -> usize;
    REAL_MBSTOWCS, real_mbstowcs, b"mbstowcs\0", fn mbstowcs(dst: *mut wchar_t, src: *const c_char, len: usize) -> usize;
    REAL_MBTOWC, real_mbtowc, b"mbtowc\0", fn mbtowc(pwc: *mut wchar_t, value: *const c_char, count: usize) -> c_int;
    REAL_MODF, real_modf, b"modf\0", fn modf(value: f64, iptr: *mut f64) -> f64;
    REAL_MODFF, real_modff, b"modff\0", fn modff(value: f32, iptr: *mut f32) -> f32;
    REAL_NAN, real_nan, b"nan\0", fn nan(tagp: *const c_char) -> f64;
    REAL_NANF, real_nanf, b"nanf\0", fn nanf(tagp: *const c_char) -> f32;
    REAL_NEARBYINT, real_nearbyint, b"nearbyint\0", fn nearbyint(value: f64) -> f64;
    REAL_NEARBYINTF, real_nearbyintf, b"nearbyintf\0", fn nearbyintf(value: f32) -> f32;
    REAL_NEXTAFTER, real_nextafter, b"nextafter\0", fn nextafter(left: f64, right: f64) -> f64;
    REAL_NEXTAFTERF, real_nextafterf, b"nextafterf\0", fn nextafterf(left: f32, right: f32) -> f32;
    REAL_NL_LANGINFO, real_nl_langinfo, b"nl_langinfo\0", fn nl_langinfo(item: nl_item) -> *mut c_char;
    REAL_POW, real_pow, b"pow\0", fn pow(left: f64, right: f64) -> f64;
    REAL_POWF, real_powf, b"powf\0", fn powf(left: f32, right: f32) -> f32;
    REAL_QSORT, real_qsort, b"qsort\0", fn qsort(base: *mut c_void, nmemb: usize, size: usize, compar: __compar_fn_t) -> ();
    REAL_QSORT_R, real_qsort_r, b"qsort_r\0", fn qsort_r(base: *mut c_void, nmemb: usize, size: usize, compar: ::core::option::Option<unsafe extern "C" fn(*const c_void, *const c_void, *mut c_void) -> c_int>, thunk: *mut c_void) -> ();
    REAL_RAND_R, real_rand_r, b"rand_r\0", fn rand_r(seed: *mut c_uint) -> c_int;
    REAL_REALLOC, real_realloc, b"realloc\0", fn realloc(ptr: *mut c_void, size: c_uint) -> *mut c_void;
    REAL_REALLOCARRAY, real_reallocarray, b"reallocarray\0", fn reallocarray(ptr: *mut c_void, nmemb: usize, size: usize) -> *mut c_void;
    REAL_REMAINDER, real_remainder, b"remainder\0", fn remainder(left: f64, right: f64) -> f64;
    REAL_REMAINDERF, real_remainderf, b"remainderf\0", fn remainderf(left: f32, right: f32) -> f32;
    REAL_REMQUO, real_remquo, b"remquo\0", fn remquo(left: f64, right: f64, quo: *mut c_int) -> f64;
    REAL_REMQUOF, real_remquof, b"remquof\0", fn remquof(left: f32, right: f32, quo: *mut c_int) -> f32;
    REAL_RINDEX, real_rindex, b"rindex\0", fn rindex(value: *const c_char, needle: c_int) -> *mut c_char;
    REAL_RINT, real_rint, b"rint\0", fn rint(value: f64) -> f64;
    REAL_RINTF, real_rintf, b"rintf\0", fn rintf(value: f32) -> f32;
    REAL_ROUND, real_round, b"round\0", fn round(value: f64) -> f64;
    REAL_ROUNDF, real_roundf, b"roundf\0", fn roundf(value: f32) -> f32;
    REAL_RPMATCH, real_rpmatch, b"rpmatch\0", fn rpmatch(response: *const c_char) -> c_int;
    REAL_SCALBLN, real_scalbln, b"scalbln\0", fn scalbln(value: f64, exp: c_long) -> f64;
    REAL_SCALBLNF, real_scalblnf, b"scalblnf\0", fn scalblnf(value: f32, exp: c_long) -> f32;
    REAL_SCALBN, real_scalbn, b"scalbn\0", fn scalbn(value: f64, exp: c_int) -> f64;
    REAL_SCALBNF, real_scalbnf, b"scalbnf\0", fn scalbnf(value: f32, exp: c_int) -> f32;
    REAL_SIN, real_sin, b"sin\0", fn sin(value: f64) -> f64;
    REAL_SINCOS, real_sincos, b"sincos\0", fn sincos(value: f64, sinp: *mut f64, cosp: *mut f64) -> ();
    REAL_SINCOSF, real_sincosf, b"sincosf\0", fn sincosf(value: f32, sinp: *mut f32, cosp: *mut f32) -> ();
    REAL_SINF, real_sinf, b"sinf\0", fn sinf(value: f32) -> f32;
    REAL_SINH, real_sinh, b"sinh\0", fn sinh(value: f64) -> f64;
    REAL_SINHF, real_sinhf, b"sinhf\0", fn sinhf(value: f32) -> f32;
    REAL_SLEEP, real_sleep, b"sleep\0", fn sleep(seconds: c_uint) -> c_uint;
    REAL_SQRT, real_sqrt, b"sqrt\0", fn sqrt(value: f64) -> f64;
    REAL_SQRTF, real_sqrtf, b"sqrtf\0", fn sqrtf(value: f32) -> f32;
    REAL_STRCASECMP, real_strcasecmp, b"strcasecmp\0", fn strcasecmp(left: *const c_char, right: *const c_char) -> c_int;
    REAL_STRCOLL, real_strcoll, b"strcoll\0", fn strcoll(left: *const c_char, right: *const c_char) -> c_int;
    REAL_STRNCASECMP, real_strncasecmp, b"strncasecmp\0", fn strncasecmp(left: *const c_char, right: *const c_char, count: c_uint) -> c_int;
    REAL_STRTOD, real_strtod, b"strtod\0", fn strtod(value: *const c_char, end_ptr: *mut *mut c_char) -> f64;
    REAL_STRTOF, real_strtof, b"strtof\0", fn strtof(value: *const c_char, end_ptr: *mut *mut c_char) -> f32;
    REAL_STRTOIMAX, real_strtoimax, b"strtoimax\0", fn strtoimax(value: *const c_char, end_ptr: *mut *mut c_char, base: c_int) -> intmax_t;
    REAL_STRTOL, real_strtol, b"strtol\0", fn strtol(value: *const c_char, end_ptr: *mut *mut c_char, base: c_int) -> c_long;
    REAL_STRTOLL, real_strtoll, b"strtoll\0", fn strtoll(value: *const c_char, end_ptr: *mut *mut c_char, base: c_int) -> c_longlong;
    REAL_STRTOUL, real_strtoul, b"strtoul\0", fn strtoul(value: *const c_char, end_ptr: *mut *mut c_char, base: c_int) -> c_ulong;
    REAL_STRTOULL, real_strtoull, b"strtoull\0", fn strtoull(value: *const c_char, end_ptr: *mut *mut c_char, base: c_int) -> c_ulonglong;
    REAL_STRTOUMAX, real_strtoumax, b"strtoumax\0", fn strtoumax(value: *const c_char, end_ptr: *mut *mut c_char, base: c_int) -> uintmax_t;
    REAL_STRXFRM, real_strxfrm, b"strxfrm\0", fn strxfrm(dst: *mut c_char, src: *const c_char, size: c_uint) -> c_uint;
    REAL_SWAB, real_swab, b"swab\0", fn swab(src: *const c_void, dst: *mut c_void, count: isize) -> ();
    REAL_TAN, real_tan, b"tan\0", fn tan(value: f64) -> f64;
    REAL_TANF, real_tanf, b"tanf\0", fn tanf(value: f32) -> f32;
    REAL_TANH, real_tanh, b"tanh\0", fn tanh(value: f64) -> f64;
    REAL_TANHF, real_tanhf, b"tanhf\0", fn tanhf(value: f32) -> f32;
    REAL_TDELETE, real_tdelete, b"tdelete\0", fn tdelete(key: *const c_void, rootp: *mut *mut c_void, compar: __compar_fn_t) -> *mut c_void;
    REAL_TDESTROY, real_tdestroy, b"tdestroy\0", fn tdestroy(root: *mut c_void, freefct: ::core::option::Option<unsafe extern "C" fn(*mut c_void)>) -> ();
    REAL_TFIND, real_tfind, b"tfind\0", fn tfind(key: *const c_void, rootp: *mut *mut c_void, compar: __compar_fn_t) -> *mut c_void;
    REAL_TGAMMA, real_tgamma, b"tgamma\0", fn tgamma(value: f64) -> f64;
    REAL_TGAMMAF, real_tgammaf, b"tgammaf\0", fn tgammaf(value: f32) -> f32;
    REAL_TIME, real_time, b"time\0", fn time(timer: *mut time_t) -> time_t;
    REAL_TIMES, real_times, b"times\0", fn times(buf: *mut tms) -> clock_t;
    REAL_TSEARCH, real_tsearch, b"tsearch\0", fn tsearch(key: *const c_void, rootp: *mut *mut c_void, compar: __compar_fn_t) -> *mut c_void;
    REAL_TRUNC, real_trunc, b"trunc\0", fn trunc(value: f64) -> f64;
    REAL_TRUNCF, real_truncf, b"truncf\0", fn truncf(value: f32) -> f32;
    REAL_TWALK, real_twalk, b"twalk\0", fn twalk(root: *const c_void, action: ::core::option::Option<unsafe extern "C" fn(*const c_void, VISIT, c_int)>) -> ();
    REAL_USLEEP, real_usleep, b"usleep\0", fn usleep(useconds: useconds_t) -> c_int;
    REAL_WCPCPY, real_wcpcpy, b"wcpcpy\0", fn wcpcpy(dst: *mut wchar_t, src: *const wchar_t) -> *mut wchar_t;
    REAL_WCPNCPY, real_wcpncpy, b"wcpncpy\0", fn wcpncpy(dst: *mut wchar_t, src: *const wchar_t, count: usize) -> *mut wchar_t;
    REAL_WCRTOMB, real_wcrtomb, b"wcrtomb\0", fn wcrtomb(dst: *mut c_char, wc: wchar_t, state: *mut mbstate_t) -> usize;
    REAL_WCSCOLL, real_wcscoll, b"wcscoll\0", fn wcscoll(left: *const wchar_t, right: *const wchar_t) -> c_int;
    REAL_WCSNRTOMBS, real_wcsnrtombs, b"wcsnrtombs\0", fn wcsnrtombs(dst: *mut c_char, src: *mut *const wchar_t, nwc: usize, len: usize, state: *mut mbstate_t) -> usize;
    REAL_WCSRTOMBS, real_wcsrtombs, b"wcsrtombs\0", fn wcsrtombs(dst: *mut c_char, src: *mut *const wchar_t, len: usize, state: *mut mbstate_t) -> usize;
    REAL_WCSTOD, real_wcstod, b"wcstod\0", fn wcstod(value: *const wchar_t, end_ptr: *mut *mut wchar_t) -> f64;
    REAL_WCSTOF, real_wcstof, b"wcstof\0", fn wcstof(value: *const wchar_t, end_ptr: *mut *mut wchar_t) -> f32;
    REAL_WCSTOIMAX, real_wcstoimax, b"wcstoimax\0", fn wcstoimax(value: *const wchar_t, end_ptr: *mut *mut wchar_t, base: c_int) -> intmax_t;
    REAL_WCSTOL, real_wcstol, b"wcstol\0", fn wcstol(value: *const wchar_t, end_ptr: *mut *mut wchar_t, base: c_int) -> c_long;
    REAL_WCSTOLL, real_wcstoll, b"wcstoll\0", fn wcstoll(value: *const wchar_t, end_ptr: *mut *mut wchar_t, base: c_int) -> c_longlong;
    REAL_WCSTOMBS, real_wcstombs, b"wcstombs\0", fn wcstombs(dst: *mut c_char, src: *const wchar_t, len: usize) -> usize;
    REAL_WCSTOUL, real_wcstoul, b"wcstoul\0", fn wcstoul(value: *const wchar_t, end_ptr: *mut *mut wchar_t, base: c_int) -> c_ulong;
    REAL_WCSTOULL, real_wcstoull, b"wcstoull\0", fn wcstoull(value: *const wchar_t, end_ptr: *mut *mut wchar_t, base: c_int) -> c_ulonglong;
    REAL_WCSTOUMAX, real_wcstoumax, b"wcstoumax\0", fn wcstoumax(value: *const wchar_t, end_ptr: *mut *mut wchar_t, base: c_int) -> uintmax_t;
    REAL_WCSXFRM, real_wcsxfrm, b"wcsxfrm\0", fn wcsxfrm(dst: *mut wchar_t, src: *const wchar_t, size: usize) -> usize;
    REAL_WCTOMB, real_wctomb, b"wctomb\0", fn wctomb(dst: *mut c_char, wc: wchar_t) -> c_int;
    REAL_Y0, real_y0, b"y0\0", fn y0(value: f64) -> f64;
    REAL_Y0F, real_y0f, b"y0f\0", fn y0f(value: f32) -> f32;
    REAL_Y1, real_y1, b"y1\0", fn y1(value: f64) -> f64;
    REAL_Y1F, real_y1f, b"y1f\0", fn y1f(value: f32) -> f32;
    REAL_YN, real_yn, b"yn\0", fn yn(order: c_int, value: f64) -> f64;
    REAL_YNF, real_ynf, b"ynf\0", fn ynf(order: c_int, value: f32) -> f32;
}
dlsym_resolver!(REAL_CLOCK_GETTIME, real_clock_gettime, b"clock_gettime\0", fn clock_gettime(clock_id: clockid_t, tp: *mut libc::timespec) -> c_int);
dlsym_resolver!(REAL_WRITE, real_write, b"write\0", fn write(fd: c_int, buf: *const c_void, count: usize) -> isize);

static REAL_OPEN: OnceLock<unsafe extern "C" fn(*const c_char, c_int, ...) -> c_int> =
    OnceLock::new();
static REAL_REGCOMP: OnceLock<HostRegcompFn> = OnceLock::new();
static REAL_REGERROR: OnceLock<HostRegerrorFn> = OnceLock::new();
static REAL_REGEXEC: OnceLock<HostRegexecFn> = OnceLock::new();
static REAL_REGFREE: OnceLock<HostRegfreeFn> = OnceLock::new();

fn real_open() -> unsafe extern "C" fn(*const c_char, c_int, ...) -> c_int {
    *REAL_OPEN.get_or_init(|| unsafe {
        mem::transmute::<*mut c_void, unsafe extern "C" fn(*const c_char, c_int, ...) -> c_int>(
            resolve_symbol(b"open\0"),
        )
    })
}

fn real_host_regcomp() -> HostRegcompFn {
    *REAL_REGCOMP.get_or_init(|| unsafe {
        mem::transmute::<*mut c_void, HostRegcompFn>(resolve_symbol(b"regcomp\0"))
    })
}

fn real_host_regerror() -> HostRegerrorFn {
    *REAL_REGERROR.get_or_init(|| unsafe {
        mem::transmute::<*mut c_void, HostRegerrorFn>(resolve_symbol(b"regerror\0"))
    })
}

fn real_host_regexec() -> HostRegexecFn {
    *REAL_REGEXEC.get_or_init(|| unsafe {
        mem::transmute::<*mut c_void, HostRegexecFn>(resolve_symbol(b"regexec\0"))
    })
}

fn real_host_regfree() -> HostRegfreeFn {
    *REAL_REGFREE.get_or_init(|| unsafe {
        mem::transmute::<*mut c_void, HostRegfreeFn>(resolve_symbol(b"regfree\0"))
    })
}

pub(crate) extern "C" fn init_wrapped_objects() {
    unsafe {
        super::stdin = resolve_object_value::<*mut FILE>(b"stdin\0");
        super::stdout = resolve_object_value::<*mut FILE>(b"stdout\0");
        super::stderr = resolve_object_value::<*mut FILE>(b"stderr\0");
        super::environ = resolve_object_value::<*mut *mut c_char>(b"environ\0");
    }
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

unsafe fn require_regex_bridge<'a>(preg: *const regex_t, symbol: &str) -> &'a RegexBridge {
    let bridge = unsafe { regex_bridge_ptr(preg) }.unwrap_or_else(|| {
        abort_incompatible_symbol(
            symbol,
            "regex_t was not initialized by why2025-badge-sys::regcomp",
        )
    });
    unsafe { &*bridge }
}

pub(crate) unsafe fn getpid_impl() -> pid_t {
    GETPID_INTERPOSE_CALLS.fetch_add(1, Ordering::Relaxed);
    call_resolved!(real_getpid)
}

pub(crate) unsafe fn getenv_impl(name: *const c_char) -> *mut c_char {
    GETENV_INTERPOSE_CALLS.fetch_add(1, Ordering::Relaxed);
    call_resolved!(real_getenv, name)
}

pub(crate) unsafe fn open_impl(path: *const c_char, flags: c_int, mode: mode_t) -> c_int {
    unsafe { real_open()(path, flags, mode) }
}

pub(crate) unsafe fn regcomp_impl(
    preg: *mut regex_t,
    pattern: *const c_char,
    cflags: c_int,
) -> c_int {
    if preg.is_null() {
        abort_incompatible_symbol("regcomp", "null regex_t pointer")
    }

    if let Some(mut existing) = unsafe { take_regex_bridge(preg) } {
        unsafe { real_host_regfree()(&mut existing.compiled) };
    }

    let mut bridge = Box::new(RegexBridge {
        compiled: unsafe { mem::zeroed::<HostRegex>() },
    });
    let status = unsafe { real_host_regcomp()(&mut bridge.compiled, pattern, cflags) };
    let raw_bridge = Box::into_raw(bridge);
    unsafe { install_regex_bridge(preg, raw_bridge) };

    status
}

pub(crate) unsafe fn regerror_impl(
    errcode: c_int,
    preg: *const regex_t,
    errbuf: *mut c_char,
    errbuf_size: usize,
) -> usize {
    let host_preg = unsafe { regex_bridge_ptr(preg) }
        .map(|bridge| unsafe { &(*bridge).compiled as *const HostRegex })
        .unwrap_or(core::ptr::null());
    unsafe { real_host_regerror()(errcode, host_preg, errbuf, errbuf_size) }
}

pub(crate) unsafe fn regexec_impl(
    preg: *const regex_t,
    text: *const c_char,
    nmatch: usize,
    pmatch: *mut [crate::regmatch_t; 0usize],
    eflags: c_int,
) -> c_int {
    let bridge = unsafe { require_regex_bridge(preg, "regexec") };
    let copy_matches = nmatch > 0 && !pmatch.is_null();
    let mut host_matches = if copy_matches {
        vec![
            HostRegmatch {
                rm_so: -1,
                rm_eo: -1
            };
            nmatch
        ]
    } else {
        Vec::new()
    };

    let status = unsafe {
        real_host_regexec()(
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
        let badge_matches = pmatch.cast::<crate::regmatch_t>();
        for (index, matched) in host_matches.iter().copied().enumerate() {
            unsafe {
                badge_matches.add(index).write(crate::regmatch_t {
                    rm_so: matched.rm_so as isize,
                    rm_eo: matched.rm_eo as isize,
                });
            }
        }
    }

    status
}

pub(crate) unsafe fn regfree_impl(preg: *mut regex_t) {
    if let Some(mut bridge) = unsafe { take_regex_bridge(preg) } {
        unsafe { real_host_regfree()(&mut bridge.compiled) };
    } else {
        unsafe { clear_badge_regex(preg) };
    }
}

#[cfg(test)]
fn reset_interpose_call_counters() {
    GETPID_INTERPOSE_CALLS.store(0, Ordering::Relaxed);
    GETENV_INTERPOSE_CALLS.store(0, Ordering::Relaxed);
}

#[cfg(test)]
fn getpid_interpose_calls() -> usize {
    GETPID_INTERPOSE_CALLS.load(Ordering::Relaxed)
}

#[cfg(test)]
fn getenv_interpose_calls() -> usize {
    GETENV_INTERPOSE_CALLS.load(Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emulated::wrapped_libc as exports;
    use core::slice;
    use std::ffi::{CStr, CString};
    use std::os::unix::ffi::OsStrExt;
    use std::ptr;
    #[cfg(unix)]
    use std::{
        fs,
        os::unix::process::ExitStatusExt,
        path::Path,
        process::{Command, Output},
    };

    #[cfg(unix)]
    const PROCESS_LIFETIME_ENV: &str = "WHY2025_BADGE_WRAPPED_LIBC_PROCESS_LIFETIME";
    #[cfg(unix)]
    const PROCESS_LIFETIME_MODE_ENV: &str = "WHY2025_BADGE_WRAPPED_LIBC_PROCESS_LIFETIME_MODE";
    #[cfg(unix)]
    const PROCESS_LIFETIME_MARKER_ENV: &str = "WHY2025_BADGE_WRAPPED_LIBC_PROCESS_LIFETIME_MARKER";

    #[cfg(unix)]
    unsafe extern "C" fn write_process_lifetime_marker() {
        if let Some(path) = std::env::var_os(PROCESS_LIFETIME_MARKER_ENV) {
            fs::write(path, b"atexit").expect("write atexit marker");
        }
    }

    #[cfg(unix)]
    fn spawn_process_lifetime_child(
        test_name: &str,
        mode: &str,
        marker_path: Option<&Path>,
    ) -> Output {
        let mut command = Command::new(std::env::current_exe().expect("current test binary path"));
        command
            .arg("--exact")
            .arg(test_name)
            .env(PROCESS_LIFETIME_ENV, "1")
            .env(PROCESS_LIFETIME_MODE_ENV, mode);

        if let Some(marker_path) = marker_path {
            command.env(PROCESS_LIFETIME_MARKER_ENV, marker_path);
        }

        command.output().expect("spawn child test process")
    }

    #[test]
    fn host_getpid_interposes_direct_badge_calls() {
        reset_interpose_call_counters();

        let badge_pid = unsafe { exports::getpid() };
        let host_pid = unsafe { real_getpid()() };

        assert_eq!(badge_pid, host_pid);
        assert_eq!(getpid_interpose_calls(), 1);
    }

    #[test]
    fn host_getpid_interposes_std_process_id() {
        reset_interpose_call_counters();

        let pid = std::process::id() as pid_t;
        let host_pid = unsafe { real_getpid()() };

        assert_eq!(pid, host_pid);
        assert!(getpid_interpose_calls() >= 1);
    }

    #[test]
    fn host_getenv_interposes_direct_badge_calls() {
        reset_interpose_call_counters();

        let key = CString::new("WHY2025_BADGE_DLSYM_PILOT_ENV").unwrap();
        unsafe {
            std::env::set_var("WHY2025_BADGE_DLSYM_PILOT_ENV", "pilot-value");
        }

        let value = unsafe { exports::getenv(key.as_ptr()) };
        let value = unsafe { CStr::from_ptr(value) };

        assert_eq!(value.to_bytes(), b"pilot-value");
        assert!(getenv_interpose_calls() >= 1);
    }

    #[test]
    fn host_ctype_and_string_memory_helpers_roundtrip() {
        assert_ne!(unsafe { exports::isalnum('A' as c_int) }, 0);
        assert_eq!(unsafe { exports::isalnum('@' as c_int) }, 0);
        assert_ne!(unsafe { exports::isalpha('Z' as c_int) }, 0);
        assert_eq!(unsafe { exports::isalpha('7' as c_int) }, 0);
        assert_ne!(unsafe { exports::isblank('\t' as c_int) }, 0);
        assert_eq!(unsafe { exports::isblank('x' as c_int) }, 0);
        assert_ne!(unsafe { exports::iscntrl('\n' as c_int) }, 0);
        assert_eq!(unsafe { exports::iscntrl('x' as c_int) }, 0);
        assert_ne!(unsafe { exports::isdigit('7' as c_int) }, 0);
        assert_eq!(unsafe { exports::isdigit('x' as c_int) }, 0);
        assert_ne!(unsafe { exports::isgraph('!' as c_int) }, 0);
        assert_eq!(unsafe { exports::isgraph(' ' as c_int) }, 0);
        assert_ne!(unsafe { exports::islower('q' as c_int) }, 0);
        assert_eq!(unsafe { exports::islower('Q' as c_int) }, 0);
        assert_ne!(unsafe { exports::isprint(' ' as c_int) }, 0);
        assert_eq!(unsafe { exports::isprint('\n' as c_int) }, 0);
        assert_ne!(unsafe { exports::ispunct('!' as c_int) }, 0);
        assert_eq!(unsafe { exports::ispunct('A' as c_int) }, 0);
        assert_ne!(unsafe { exports::isspace(' ' as c_int) }, 0);
        assert_eq!(unsafe { exports::isspace('x' as c_int) }, 0);
        assert_ne!(unsafe { exports::isupper('Q' as c_int) }, 0);
        assert_eq!(unsafe { exports::isupper('q' as c_int) }, 0);
        assert_ne!(unsafe { exports::isxdigit('f' as c_int) }, 0);
        assert_eq!(unsafe { exports::isxdigit('g' as c_int) }, 0);
        assert_ne!(unsafe { exports::isascii('A' as c_int) }, 0);
        assert_eq!(unsafe { exports::isascii(0x80) }, 0);
        assert_eq!(unsafe { exports::tolower('Q' as c_int) }, 'q' as c_int);
        assert_eq!(unsafe { exports::toascii(0xC1) }, 'A' as c_int);
        assert_eq!(unsafe { exports::toupper('q' as c_int) }, 'Q' as c_int);

        let src = *b"badge";
        let mut dst = [0_u8; 5];
        let copied = unsafe {
            exports::memcpy(
                dst.as_mut_ptr().cast::<c_void>(),
                src.as_ptr().cast::<c_void>(),
                src.len() as c_uint,
            )
        };
        assert_eq!(copied, dst.as_mut_ptr().cast::<c_void>());
        assert_eq!(dst, src);
        assert_eq!(
            unsafe {
                exports::memcmp(
                    dst.as_ptr().cast::<c_void>(),
                    src.as_ptr().cast::<c_void>(),
                    src.len() as c_uint,
                )
            },
            0
        );

        let mut different = dst;
        different[4] = b'!';
        assert_ne!(
            unsafe {
                exports::memcmp(
                    different.as_ptr().cast::<c_void>(),
                    src.as_ptr().cast::<c_void>(),
                    src.len() as c_uint,
                )
            },
            0
        );

        let alpha = CString::new("alpha").unwrap();
        let omega = CString::new("omega").unwrap();
        assert_eq!(
            unsafe { exports::strcmp(alpha.as_ptr(), alpha.as_ptr()) },
            0
        );
        assert!(unsafe { exports::strcmp(alpha.as_ptr(), omega.as_ptr()) } < 0);
    }

    #[test]
    fn host_memory_and_string_pointer_helpers_roundtrip() {
        let mut filled = [0_u8; 8];
        let filled_ptr =
            unsafe { exports::memset(filled.as_mut_ptr().cast::<c_void>(), 'Z' as c_int, 4) };
        assert_eq!(filled_ptr, filled.as_mut_ptr().cast::<c_void>());
        assert_eq!(&filled[..4], b"ZZZZ");

        let mut shifted = [0_u8; 7];
        shifted[..6].copy_from_slice(b"badge\0");
        let moved_ptr = unsafe {
            exports::memmove(
                shifted.as_mut_ptr().add(1).cast::<c_void>(),
                shifted.as_ptr().cast::<c_void>(),
                6,
            )
        };
        assert_eq!(
            moved_ptr,
            unsafe { shifted.as_mut_ptr().add(1) }.cast::<c_void>()
        );
        assert_eq!(&shifted, b"bbadge\0");

        let prefix = CString::new("badge").unwrap();
        let suffix = CString::new("-rust").unwrap();
        let mut buffer = [0 as c_char; 16];
        assert_eq!(
            unsafe { exports::strcpy(buffer.as_mut_ptr(), prefix.as_ptr()) },
            buffer.as_mut_ptr()
        );
        assert_eq!(
            unsafe { exports::strcat(buffer.as_mut_ptr(), suffix.as_ptr()) },
            buffer.as_mut_ptr()
        );

        let combined = unsafe { CStr::from_ptr(buffer.as_ptr()) };
        assert_eq!(combined.to_bytes(), b"badge-rust");

        let needle = unsafe { exports::strchr(buffer.as_ptr(), '-' as c_int) };
        assert!(!needle.is_null());
        let tail = unsafe { CStr::from_ptr(needle) };
        assert_eq!(tail.to_bytes(), b"-rust");
    }

    #[test]
    fn host_search_copy_and_length_helpers_roundtrip() {
        let haystack = *b"bananabadge";
        let needle = *b"badge";

        let found = unsafe {
            exports::memchr(
                haystack.as_ptr().cast::<c_void>(),
                'd' as c_int,
                haystack.len() as c_uint,
            )
        };
        assert!(!found.is_null());
        assert_eq!(
            unsafe { found.cast::<u8>().offset_from(haystack.as_ptr()) },
            8
        );

        let last_a = unsafe {
            exports::memrchr(
                haystack.as_ptr().cast::<c_void>(),
                'a' as c_int,
                haystack.len(),
            )
        };
        assert!(!last_a.is_null());
        assert_eq!(
            unsafe { last_a.cast::<u8>().offset_from(haystack.as_ptr()) },
            7
        );

        let found_subslice = unsafe {
            exports::memmem(
                haystack.as_ptr().cast::<c_void>(),
                haystack.len(),
                needle.as_ptr().cast::<c_void>(),
                needle.len(),
            )
        };
        assert!(!found_subslice.is_null());
        assert_eq!(
            unsafe { found_subslice.cast::<u8>().offset_from(haystack.as_ptr()) },
            6
        );

        let src = *b"badge";
        let mut memccpy_dst = [0_u8; 8];
        let stop = unsafe {
            exports::memccpy(
                memccpy_dst.as_mut_ptr().cast::<c_void>(),
                src.as_ptr().cast::<c_void>(),
                'g' as c_int,
                src.len() as c_uint,
            )
        };
        assert_eq!(
            unsafe { stop.cast::<u8>().offset_from(memccpy_dst.as_ptr()) },
            4
        );
        assert_eq!(&memccpy_dst[..4], b"badg");

        let mut mempcpy_dst = [0_u8; 8];
        let mempcpy_end = unsafe {
            exports::mempcpy(
                mempcpy_dst.as_mut_ptr().cast::<c_void>(),
                src.as_ptr().cast::<c_void>(),
                src.len() as c_uint,
            )
        };
        assert_eq!(
            unsafe { mempcpy_end.cast::<u8>().offset_from(mempcpy_dst.as_ptr()) },
            5
        );
        assert_eq!(&mempcpy_dst[..5], b"badge");

        let c_prefix = CString::new("badge").unwrap();
        let raw_end = unsafe { exports::rawmemchr(c_prefix.as_ptr().cast::<c_void>(), 0) };
        assert_eq!(
            unsafe { raw_end.cast::<c_char>().offset_from(c_prefix.as_ptr()) },
            5
        );

        let mut stpcpy_buf = [0 as c_char; 16];
        let stpcpy_end = unsafe { exports::stpcpy(stpcpy_buf.as_mut_ptr(), c_prefix.as_ptr()) };
        assert_eq!(unsafe { stpcpy_end.offset_from(stpcpy_buf.as_ptr()) }, 5);
        assert_eq!(
            unsafe { CStr::from_ptr(stpcpy_buf.as_ptr()) }.to_bytes(),
            b"badge"
        );

        let mut stpncpy_buf = [0 as c_char; 16];
        let stpncpy_end =
            unsafe { exports::stpncpy(stpncpy_buf.as_mut_ptr(), c_prefix.as_ptr(), 8) };
        assert_eq!(unsafe { stpncpy_end.offset_from(stpncpy_buf.as_ptr()) }, 5);
        assert_eq!(
            unsafe { CStr::from_ptr(stpncpy_buf.as_ptr()) }.to_bytes(),
            b"badge"
        );

        let mixed = CString::new("Badge-Rust").unwrap();
        let rust_lower = CString::new("rust").unwrap();
        let dash = CString::new("-").unwrap();
        let set = CString::new("xyzg").unwrap();
        let badge_set = CString::new("Badge-").unwrap();
        let rust_exact = CString::new("Rust").unwrap();

        let case_insensitive = unsafe { exports::strcasestr(mixed.as_ptr(), rust_lower.as_ptr()) };
        assert!(!case_insensitive.is_null());
        assert_eq!(
            unsafe { CStr::from_ptr(case_insensitive) }.to_bytes(),
            b"Rust"
        );

        let missing = unsafe { exports::strchrnul(mixed.as_ptr(), '!' as c_int) };
        assert_eq!(
            unsafe { missing.offset_from(mixed.as_ptr()) as c_uint },
            unsafe { exports::strlen(mixed.as_ptr()) }
        );
        assert_eq!(
            unsafe { exports::strcspn(mixed.as_ptr(), dash.as_ptr()) },
            5
        );
        assert_eq!(unsafe { exports::strlen(mixed.as_ptr()) }, 10);

        let mut ncat_buf = [0 as c_char; 16];
        assert_eq!(
            unsafe { exports::strcpy(ncat_buf.as_mut_ptr(), c_prefix.as_ptr()) },
            ncat_buf.as_mut_ptr()
        );
        assert_eq!(
            unsafe { exports::strncat(ncat_buf.as_mut_ptr(), dash.as_ptr(), 1) },
            ncat_buf.as_mut_ptr()
        );
        assert_eq!(
            unsafe { exports::strncat(ncat_buf.as_mut_ptr(), rust_exact.as_ptr(), 2) },
            ncat_buf.as_mut_ptr()
        );
        assert_eq!(
            unsafe { CStr::from_ptr(ncat_buf.as_ptr()) }.to_bytes(),
            b"badge-Ru"
        );

        let alphabet = CString::new("alphabet").unwrap();
        let alpha_x = CString::new("alphaX").unwrap();
        assert_eq!(
            unsafe { exports::strncmp(alphabet.as_ptr(), alpha_x.as_ptr(), 5) },
            0
        );
        assert!(unsafe { exports::strncmp(alphabet.as_ptr(), alpha_x.as_ptr(), 6) } > 0);

        let mut strncpy_buf = [0 as c_char; 16];
        assert_eq!(
            unsafe { exports::strncpy(strncpy_buf.as_mut_ptr(), c_prefix.as_ptr(), 8) },
            strncpy_buf.as_mut_ptr()
        );
        assert_eq!(
            unsafe { CStr::from_ptr(strncpy_buf.as_ptr()) }.to_bytes(),
            b"badge"
        );

        assert_eq!(unsafe { exports::strnlen(c_prefix.as_ptr(), 3) }, 3);
        assert_eq!(unsafe { exports::strnlen(c_prefix.as_ptr(), 8) }, 5);

        let first_match = unsafe { exports::strpbrk(mixed.as_ptr(), set.as_ptr()) };
        assert!(!first_match.is_null());
        assert_eq!(
            unsafe { CStr::from_ptr(first_match) }.to_bytes(),
            b"ge-Rust"
        );

        let last_rust_a = CString::new("bananabadge").unwrap();
        let last_match = unsafe { exports::strrchr(last_rust_a.as_ptr(), 'a' as c_int) };
        assert!(!last_match.is_null());
        assert_eq!(unsafe { last_match.offset_from(last_rust_a.as_ptr()) }, 7);

        assert_eq!(
            unsafe { exports::strspn(mixed.as_ptr(), badge_set.as_ptr()) },
            6
        );

        let substring = unsafe { exports::strstr(mixed.as_ptr(), rust_exact.as_ptr()) };
        assert!(!substring.is_null());
        assert_eq!(unsafe { CStr::from_ptr(substring) }.to_bytes(), b"Rust");
    }

    #[test]
    fn host_tokenizing_and_error_helpers_roundtrip() {
        let delim = CString::new(",").unwrap();
        let pipe = CString::new("|").unwrap();

        let mut csv = b"alpha,beta,gamma\0".to_vec();
        let mut csv_cursor = csv.as_mut_ptr().cast::<c_char>();
        let first = unsafe { exports::strsep(&mut csv_cursor, delim.as_ptr()) };
        assert_eq!(unsafe { CStr::from_ptr(first) }.to_bytes(), b"alpha");
        let second = unsafe { exports::strsep(&mut csv_cursor, delim.as_ptr()) };
        assert_eq!(unsafe { CStr::from_ptr(second) }.to_bytes(), b"beta");
        let third = unsafe { exports::strsep(&mut csv_cursor, delim.as_ptr()) };
        assert_eq!(unsafe { CStr::from_ptr(third) }.to_bytes(), b"gamma");
        assert!(unsafe { exports::strsep(&mut csv_cursor, delim.as_ptr()) }.is_null());

        let mut colors = b"red|green|blue\0".to_vec();
        let mut saveptr = core::ptr::null_mut::<c_char>();
        let token1 = unsafe {
            exports::strtok_r(
                colors.as_mut_ptr().cast::<c_char>(),
                pipe.as_ptr(),
                &mut saveptr,
            )
        };
        assert_eq!(unsafe { CStr::from_ptr(token1) }.to_bytes(), b"red");
        let token2 =
            unsafe { exports::strtok_r(core::ptr::null_mut(), pipe.as_ptr(), &mut saveptr) };
        assert_eq!(unsafe { CStr::from_ptr(token2) }.to_bytes(), b"green");
        let token3 =
            unsafe { exports::strtok_r(core::ptr::null_mut(), pipe.as_ptr(), &mut saveptr) };
        assert_eq!(unsafe { CStr::from_ptr(token3) }.to_bytes(), b"blue");
        assert!(
            unsafe { exports::strtok_r(core::ptr::null_mut(), pipe.as_ptr(), &mut saveptr) }
                .is_null()
        );

        let wrapped = unsafe { exports::strerror(libc::ENOENT) };
        let host = unsafe { libc::strerror(libc::ENOENT) };
        assert!(!wrapped.is_null());
        assert!(!host.is_null());
        assert_eq!(
            unsafe { CStr::from_ptr(wrapped) }.to_bytes(),
            unsafe { CStr::from_ptr(host) }.to_bytes()
        );
    }

    #[test]
    fn host_duplication_and_rng_helpers_roundtrip() {
        let text = CString::new("badge-rust").unwrap();

        let duplicated = unsafe { exports::strdup(text.as_ptr()) };
        assert!(!duplicated.is_null());
        assert_eq!(
            unsafe { CStr::from_ptr(duplicated) }.to_bytes(),
            b"badge-rust"
        );
        unsafe { libc::free(duplicated.cast::<c_void>()) };

        let duplicated_prefix = unsafe { exports::strndup(text.as_ptr(), 5) };
        assert!(!duplicated_prefix.is_null());
        assert_eq!(
            unsafe { CStr::from_ptr(duplicated_prefix) }.to_bytes(),
            b"badge"
        );
        unsafe { libc::free(duplicated_prefix.cast::<c_void>()) };

        let wide = ['W' as wchar_t, 'H' as wchar_t, 'Y' as wchar_t, 0];
        let wide_dup = unsafe { exports::wcsdup(wide.as_ptr()) };
        assert!(!wide_dup.is_null());
        let wide_slice = unsafe { slice::from_raw_parts(wide_dup, wide.len()) };
        assert_eq!(wide_slice, &wide);
        unsafe { libc::free(wide_dup.cast::<c_void>()) };

        let comma = CString::new(",").unwrap();
        let mut colors = b"red,green,blue\0".to_vec();
        let token1 =
            unsafe { exports::strtok(colors.as_mut_ptr().cast::<c_char>(), comma.as_ptr()) };
        assert_eq!(unsafe { CStr::from_ptr(token1) }.to_bytes(), b"red");
        let token2 = unsafe { exports::strtok(core::ptr::null_mut(), comma.as_ptr()) };
        assert_eq!(unsafe { CStr::from_ptr(token2) }.to_bytes(), b"green");
        let token3 = unsafe { exports::strtok(core::ptr::null_mut(), comma.as_ptr()) };
        assert_eq!(unsafe { CStr::from_ptr(token3) }.to_bytes(), b"blue");
        assert!(unsafe { exports::strtok(core::ptr::null_mut(), comma.as_ptr()) }.is_null());

        unsafe { exports::srand(0x1234) };
        let badge_rand = [unsafe { exports::rand() }, unsafe { exports::rand() }];
        unsafe { real_srand()(0x1234) };
        let host_rand = [unsafe { real_rand()() }, unsafe { real_rand()() }];
        assert_eq!(badge_rand, host_rand);

        unsafe { exports::srandom(0x4321) };
        let badge_random = [unsafe { exports::random() }, unsafe { exports::random() }];
        unsafe { real_srandom()(0x4321) };
        let host_random = [unsafe { real_random()() }, unsafe { real_random()() }];
        assert_eq!(badge_random, host_random);
    }

    #[test]
    fn host_portability_edge_helpers_roundtrip() {
        let source = CString::new("badge").unwrap();

        let mut wrapped_lcpy = [0 as c_char; 4];
        let mut host_lcpy = [0 as c_char; 4];
        let wrapped_lcpy_len = unsafe {
            exports::strlcpy(
                wrapped_lcpy.as_mut_ptr(),
                source.as_ptr(),
                wrapped_lcpy.len() as c_uint,
            )
        };
        let host_lcpy_len = unsafe {
            real_strlcpy()(
                host_lcpy.as_mut_ptr(),
                source.as_ptr(),
                host_lcpy.len() as c_uint,
            )
        };
        assert_eq!(wrapped_lcpy_len, host_lcpy_len);
        assert_eq!(wrapped_lcpy, host_lcpy);

        let mut wrapped_lcat = [0 as c_char; 6];
        let mut host_lcat = [0 as c_char; 6];
        unsafe {
            exports::strcpy(wrapped_lcat.as_mut_ptr(), b"hi\0".as_ptr().cast());
            real_strcpy()(host_lcat.as_mut_ptr(), b"hi\0".as_ptr().cast());
        }
        let wrapped_lcat_len = unsafe {
            exports::strlcat(
                wrapped_lcat.as_mut_ptr(),
                source.as_ptr(),
                wrapped_lcat.len() as c_uint,
            )
        };
        let host_lcat_len = unsafe {
            real_strlcat()(
                host_lcat.as_mut_ptr(),
                source.as_ptr(),
                host_lcat.len() as c_uint,
            )
        };
        assert_eq!(wrapped_lcat_len, host_lcat_len);
        assert_eq!(wrapped_lcat, host_lcat);

        let mut wrapped_buf = [0 as c_char; 128];
        let mut host_buf = [0 as c_char; 128];
        let wrapped_ptr = unsafe {
            exports::strerror_r(libc::ENOENT, wrapped_buf.as_mut_ptr(), wrapped_buf.len())
        };
        let host_ptr =
            unsafe { real_strerror_r()(libc::ENOENT, host_buf.as_mut_ptr(), host_buf.len()) };
        assert_eq!(
            unsafe { CStr::from_ptr(wrapped_ptr) }.to_bytes(),
            unsafe { CStr::from_ptr(host_ptr) }.to_bytes()
        );

        let file9 = CString::new("file9").unwrap();
        let file10 = CString::new("file10").unwrap();
        let wrapped_cmp = unsafe { exports::strverscmp(file9.as_ptr(), file10.as_ptr()) };
        let host_cmp = unsafe { real_strverscmp()(file9.as_ptr(), file10.as_ptr()) };
        assert_eq!(wrapped_cmp.signum(), host_cmp.signum());
    }

    #[test]
    fn host_wide_string_helpers_roundtrip() {
        let badge = [
            'b' as wchar_t,
            'a' as wchar_t,
            'd' as wchar_t,
            'g' as wchar_t,
            'e' as wchar_t,
            0,
        ];
        let badge_rust = [
            'b' as wchar_t,
            'a' as wchar_t,
            'd' as wchar_t,
            'g' as wchar_t,
            'e' as wchar_t,
            '-' as wchar_t,
            'r' as wchar_t,
            'u' as wchar_t,
            's' as wchar_t,
            't' as wchar_t,
            0,
        ];
        let dash = ['-' as wchar_t, 0];
        let alpha = [
            'a' as wchar_t,
            'l' as wchar_t,
            'p' as wchar_t,
            'h' as wchar_t,
            'a' as wchar_t,
            0,
        ];
        let alpha_x = [
            'a' as wchar_t,
            'l' as wchar_t,
            'p' as wchar_t,
            'h' as wchar_t,
            'a' as wchar_t,
            'X' as wchar_t,
            0,
        ];

        let mut copied = [0 as wchar_t; 8];
        assert_eq!(
            unsafe { exports::wcscpy(copied.as_mut_ptr(), badge.as_ptr()) },
            copied.as_mut_ptr()
        );
        assert_eq!(&copied[..badge.len()], &badge);
        assert_eq!(unsafe { exports::wcslen(copied.as_ptr()) }, 5);
        assert_eq!(unsafe { exports::wcsnlen(copied.as_ptr(), 3) }, 3);
        assert_eq!(
            unsafe { exports::wcscmp(alpha.as_ptr(), alpha.as_ptr()) },
            0
        );
        assert_eq!(
            unsafe { exports::wcsncmp(alpha.as_ptr(), alpha_x.as_ptr(), 5) },
            0
        );
        assert!(unsafe { exports::wcsncmp(alpha.as_ptr(), alpha_x.as_ptr(), 6) } < 0);
        assert_eq!(
            unsafe { exports::wcscspn(badge_rust.as_ptr(), dash.as_ptr()) },
            5
        );

        let found = unsafe { exports::wcschr(badge_rust.as_ptr(), '-' as c_int) };
        assert!(!found.is_null());
        assert_eq!(unsafe { found.offset_from(badge_rust.as_ptr()) }, 5);

        let bananas = [
            'b' as wchar_t,
            'a' as wchar_t,
            'n' as wchar_t,
            'a' as wchar_t,
            'n' as wchar_t,
            'a' as wchar_t,
            0,
        ];
        let last_a = unsafe { exports::wcsrchr(bananas.as_ptr(), 'a' as wchar_t) };
        assert!(!last_a.is_null());
        assert_eq!(unsafe { last_a.offset_from(bananas.as_ptr()) }, 5);

        let mut fixed = [0 as wchar_t; 8];
        assert_eq!(
            unsafe { exports::wcsncpy(fixed.as_mut_ptr(), alpha.as_ptr(), fixed.len()) },
            fixed.as_mut_ptr()
        );
        assert_eq!(&fixed[..alpha.len()], &alpha);

        let why = ['W' as wchar_t, 'H' as wchar_t, 'Y' as wchar_t, 0];
        let mut wide_copy = [0 as wchar_t; 4];
        assert_eq!(
            unsafe { exports::wmemcpy(wide_copy.as_mut_ptr(), why.as_ptr(), why.len() as c_uint) },
            wide_copy.as_mut_ptr()
        );
        assert_eq!(wide_copy, why);
        assert_eq!(
            unsafe { exports::wmemcmp(wide_copy.as_ptr(), why.as_ptr(), why.len() as c_uint) },
            0
        );

        let mut different = wide_copy;
        different[2] = '!' as wchar_t;
        assert_ne!(
            unsafe { exports::wmemcmp(different.as_ptr(), why.as_ptr(), why.len() as c_uint) },
            0
        );
    }

    #[test]
    fn host_additional_wide_string_helpers_roundtrip() {
        let badge = [
            'b' as wchar_t,
            'a' as wchar_t,
            'd' as wchar_t,
            'g' as wchar_t,
            'e' as wchar_t,
            0,
        ];
        let rust = [
            'r' as wchar_t,
            'u' as wchar_t,
            's' as wchar_t,
            't' as wchar_t,
            0,
        ];
        let digits = [
            '2' as wchar_t,
            '0' as wchar_t,
            '2' as wchar_t,
            '5' as wchar_t,
            0,
        ];
        let vowels = [
            'a' as wchar_t,
            'e' as wchar_t,
            'i' as wchar_t,
            'o' as wchar_t,
            'u' as wchar_t,
            0,
        ];
        let delimiters = [':' as wchar_t, '/' as wchar_t, 0];
        let needle = ['r' as wchar_t, 'u' as wchar_t, 0];

        let mut appended = [0 as wchar_t; 16];
        appended[..badge.len()].copy_from_slice(&badge);
        assert_eq!(
            unsafe { exports::wcscat(appended.as_mut_ptr(), rust.as_ptr()) },
            appended.as_mut_ptr()
        );
        assert_eq!(
            &appended[..9],
            &[
                'b' as wchar_t,
                'a' as wchar_t,
                'd' as wchar_t,
                'g' as wchar_t,
                'e' as wchar_t,
                'r' as wchar_t,
                'u' as wchar_t,
                's' as wchar_t,
                't' as wchar_t
            ]
        );

        let mut partial = [0 as wchar_t; 16];
        partial[..badge.len()].copy_from_slice(&badge);
        assert_eq!(
            unsafe { exports::wcsncat(partial.as_mut_ptr(), digits.as_ptr(), 2) },
            partial.as_mut_ptr()
        );
        assert_eq!(
            &partial[..7],
            &[
                'b' as wchar_t,
                'a' as wchar_t,
                'd' as wchar_t,
                'g' as wchar_t,
                'e' as wchar_t,
                '2' as wchar_t,
                '0' as wchar_t
            ]
        );

        let first_vowel = unsafe { exports::wcspbrk(rust.as_ptr(), vowels.as_ptr()) };
        assert!(!first_vowel.is_null());
        assert_eq!(unsafe { first_vowel.offset_from(rust.as_ptr()) }, 1);
        assert_eq!(
            unsafe { exports::wcsspn(badge.as_ptr(), vowels.as_ptr()) },
            0
        );

        let found = unsafe { exports::wcsstr(appended.as_ptr(), needle.as_ptr()) };
        assert!(!found.is_null());
        assert_eq!(unsafe { found.offset_from(appended.as_ptr()) }, 5);

        let mut tokens = [
            'b' as wchar_t,
            'a' as wchar_t,
            'd' as wchar_t,
            'g' as wchar_t,
            'e' as wchar_t,
            ':' as wchar_t,
            'r' as wchar_t,
            'u' as wchar_t,
            's' as wchar_t,
            't' as wchar_t,
            '/' as wchar_t,
            '2' as wchar_t,
            '0' as wchar_t,
            '2' as wchar_t,
            '5' as wchar_t,
            0,
        ];
        let mut save = ptr::null_mut();
        let first = unsafe { exports::wcstok(tokens.as_mut_ptr(), delimiters.as_ptr(), &mut save) };
        let second = unsafe { exports::wcstok(ptr::null_mut(), delimiters.as_ptr(), &mut save) };
        let third = unsafe { exports::wcstok(ptr::null_mut(), delimiters.as_ptr(), &mut save) };
        let fourth = unsafe { exports::wcstok(ptr::null_mut(), delimiters.as_ptr(), &mut save) };
        assert_eq!(unsafe { slice::from_raw_parts(first, 5) }, &badge[..5]);
        assert_eq!(unsafe { slice::from_raw_parts(second, 4) }, &rust[..4]);
        assert_eq!(unsafe { slice::from_raw_parts(third, 4) }, &digits[..4]);
        assert!(fourth.is_null());

        let why = ['W' as wchar_t, 'H' as wchar_t, 'Y' as wchar_t, 0];
        let ch = unsafe { exports::wmemchr(why.as_ptr(), 'H' as c_int, why.len() as c_uint) };
        assert!(!ch.is_null());
        assert_eq!(unsafe { ch.offset_from(why.as_ptr()) }, 1);

        let mut moved = [
            'A' as wchar_t,
            'B' as wchar_t,
            'C' as wchar_t,
            'D' as wchar_t,
            0,
        ];
        assert_eq!(
            unsafe { exports::wmemmove(moved.as_mut_ptr().add(1), moved.as_ptr(), 3) },
            unsafe { moved.as_mut_ptr().add(1) }
        );
        assert_eq!(
            &moved[..4],
            &[
                'A' as wchar_t,
                'A' as wchar_t,
                'B' as wchar_t,
                'C' as wchar_t
            ]
        );

        let mut padded = [0 as wchar_t; 6];
        assert_eq!(
            unsafe { exports::wmemset(padded.as_mut_ptr(), 'Z' as wchar_t, 4) },
            padded.as_mut_ptr()
        );
        assert_eq!(
            &padded[..4],
            &[
                'Z' as wchar_t,
                'Z' as wchar_t,
                'Z' as wchar_t,
                'Z' as wchar_t
            ]
        );

        let mut copied = [0 as wchar_t; 6];
        let tail = unsafe { exports::wmempcpy(copied.as_mut_ptr(), why.as_ptr(), 3) };
        assert_eq!(tail, unsafe { copied.as_mut_ptr().add(3) });
        assert_eq!(&copied[..3], &why[..3]);
    }

    #[test]
    fn host_wide_case_and_width_helpers_roundtrip() {
        let mixed = [
            'B' as wchar_t,
            'a' as wchar_t,
            'D' as wchar_t,
            'g' as wchar_t,
            'E' as wchar_t,
            0,
        ];
        let lowered = [
            'b' as wchar_t,
            'A' as wchar_t,
            'd' as wchar_t,
            'G' as wchar_t,
            'e' as wchar_t,
            0,
        ];
        let lowered_suffix = [
            'b' as wchar_t,
            'A' as wchar_t,
            'x' as wchar_t,
            'G' as wchar_t,
            'e' as wchar_t,
            0,
        ];
        let badge = [
            'b' as wchar_t,
            'a' as wchar_t,
            'd' as wchar_t,
            'g' as wchar_t,
            'e' as wchar_t,
            0,
        ];

        let wrapped_casecmp = unsafe { exports::wcscasecmp(mixed.as_ptr(), lowered.as_ptr()) };
        let host_casecmp = unsafe { real_wcscasecmp()(mixed.as_ptr(), lowered.as_ptr()) };
        assert_eq!(wrapped_casecmp.signum(), host_casecmp.signum());

        let wrapped_ncasecmp =
            unsafe { exports::wcsncasecmp(mixed.as_ptr(), lowered_suffix.as_ptr(), 2) };
        let host_ncasecmp =
            unsafe { real_wcsncasecmp()(mixed.as_ptr(), lowered_suffix.as_ptr(), 2) };
        assert_eq!(wrapped_ncasecmp.signum(), host_ncasecmp.signum());

        let wrapped_ncasecmp_diff =
            unsafe { exports::wcsncasecmp(mixed.as_ptr(), lowered_suffix.as_ptr(), 3) };
        let host_ncasecmp_diff =
            unsafe { real_wcsncasecmp()(mixed.as_ptr(), lowered_suffix.as_ptr(), 3) };
        assert_eq!(wrapped_ncasecmp_diff.signum(), host_ncasecmp_diff.signum());

        let wrapped_wctob = unsafe { exports::wctob('A' as wint_t) };
        let host_wctob_value = unsafe { real_wctob()('A' as wint_t) };
        assert_eq!(wrapped_wctob, host_wctob_value);

        let wrapped_wcwidth = unsafe { exports::wcwidth('A' as wchar_t) };
        let host_wcwidth_value = unsafe { real_wcwidth()('A' as wchar_t) };
        assert_eq!(wrapped_wcwidth, host_wcwidth_value);

        let wrapped_wcswidth = unsafe { exports::wcswidth(badge.as_ptr(), 5) };
        let host_wcswidth_value = unsafe { real_wcswidth()(badge.as_ptr(), 5) };
        assert_eq!(wrapped_wcswidth, host_wcswidth_value);
    }

    #[test]
    fn host_wide_classification_helpers_roundtrip() {
        assert_ne!(unsafe { exports::iswalnum('A' as wint_t) }, 0);
        assert_eq!(unsafe { exports::iswalnum('!' as wint_t) }, 0);
        assert_ne!(unsafe { exports::iswalpha('Q' as wint_t) }, 0);
        assert_eq!(unsafe { exports::iswalpha('7' as wint_t) }, 0);
        assert_ne!(unsafe { exports::iswblank(' ' as wint_t) }, 0);
        assert_eq!(unsafe { exports::iswblank('x' as wint_t) }, 0);
        assert_ne!(unsafe { exports::iswcntrl('\n' as wint_t) }, 0);
        assert_eq!(unsafe { exports::iswcntrl('x' as wint_t) }, 0);
        assert_ne!(unsafe { exports::iswdigit('7' as wint_t) }, 0);
        assert_eq!(unsafe { exports::iswdigit('x' as wint_t) }, 0);
        assert_ne!(unsafe { exports::iswgraph('!' as wint_t) }, 0);
        assert_eq!(unsafe { exports::iswgraph(' ' as wint_t) }, 0);
        assert_ne!(unsafe { exports::iswlower('q' as wint_t) }, 0);
        assert_eq!(unsafe { exports::iswlower('Q' as wint_t) }, 0);
        assert_ne!(unsafe { exports::iswprint(' ' as wint_t) }, 0);
        assert_eq!(unsafe { exports::iswprint('\n' as wint_t) }, 0);
        assert_ne!(unsafe { exports::iswpunct('!' as wint_t) }, 0);
        assert_eq!(unsafe { exports::iswpunct('A' as wint_t) }, 0);
        assert_ne!(unsafe { exports::iswspace(' ' as wint_t) }, 0);
        assert_eq!(unsafe { exports::iswspace('x' as wint_t) }, 0);
        assert_ne!(unsafe { exports::iswupper('Q' as wint_t) }, 0);
        assert_eq!(unsafe { exports::iswupper('q' as wint_t) }, 0);
        assert_ne!(unsafe { exports::iswxdigit('f' as wint_t) }, 0);
        assert_eq!(unsafe { exports::iswxdigit('g' as wint_t) }, 0);
        assert_eq!(unsafe { exports::towlower('Q' as wint_t) }, 'q' as wint_t);
        assert_eq!(unsafe { exports::towupper('q' as wint_t) }, 'Q' as wint_t);
    }

    #[test]
    #[cfg(unix)]
    fn host_exit_family_and_atexit_follow_host_process_semantics() {
        const TEST_NAME: &str = "emulated::wrapped_libc::runtime::tests::host_exit_family_and_atexit_follow_host_process_semantics";

        if std::env::var_os(PROCESS_LIFETIME_ENV).is_some() {
            match std::env::var(PROCESS_LIFETIME_MODE_ENV)
                .expect("process lifetime mode")
                .as_str()
            {
                "exit" => {
                    assert_eq!(
                        crate::emulated::libc_fallback::atexit(Some(write_process_lifetime_marker)),
                        0
                    );
                    unsafe { exports::exit(23) }
                }
                "_exit" => {
                    assert_eq!(
                        crate::emulated::libc_fallback::atexit(Some(write_process_lifetime_marker)),
                        0
                    );
                    unsafe { exports::_exit(24) }
                }
                "_Exit" => {
                    assert_eq!(
                        crate::emulated::libc_fallback::atexit(Some(write_process_lifetime_marker)),
                        0
                    );
                    unsafe { exports::_Exit(25) }
                }
                other => panic!("unexpected process lifetime mode: {other}"),
            }
        }

        let mut exit_marker = std::env::temp_dir();
        exit_marker.push(format!("why2025-badge-atexit-exit-{}", std::process::id()));
        let _ = fs::remove_file(&exit_marker);
        let output = spawn_process_lifetime_child(TEST_NAME, "exit", Some(&exit_marker));
        assert_eq!(output.status.code(), Some(23));
        assert_eq!(fs::read(&exit_marker).expect("read exit marker"), b"atexit");
        let _ = fs::remove_file(&exit_marker);

        let mut underscore_marker = std::env::temp_dir();
        underscore_marker.push(format!("why2025-badge-atexit-_exit-{}", std::process::id()));
        let _ = fs::remove_file(&underscore_marker);
        let output = spawn_process_lifetime_child(TEST_NAME, "_exit", Some(&underscore_marker));
        assert_eq!(output.status.code(), Some(24));
        assert!(!underscore_marker.exists());

        let mut exit_cap_marker = std::env::temp_dir();
        exit_cap_marker.push(format!("why2025-badge-atexit-_Exit-{}", std::process::id()));
        let _ = fs::remove_file(&exit_cap_marker);
        let output = spawn_process_lifetime_child(TEST_NAME, "_Exit", Some(&exit_cap_marker));
        assert_eq!(output.status.code(), Some(25));
        assert!(!exit_cap_marker.exists());
    }

    #[test]
    #[cfg(unix)]
    fn host_abort_follows_host_process_abort_semantics() {
        const TEST_NAME: &str = "emulated::wrapped_libc::runtime::tests::host_abort_follows_host_process_abort_semantics";

        if std::env::var_os(PROCESS_LIFETIME_ENV).is_some() {
            unsafe { exports::abort() }
        }

        let output = spawn_process_lifetime_child(TEST_NAME, "abort", None);
        assert!(!output.status.success());
        assert_eq!(output.status.signal(), Some(libc::SIGABRT));
    }

    #[test]
    fn host_open_read_write_close_and_unlink_roundtrip() {
        let mut path = std::env::temp_dir();
        path.push(format!("why2025-badge-dlsym-{}", std::process::id()));
        let path = CString::new(path.as_os_str().as_bytes()).unwrap();
        let contents = b"badge";

        let fd = unsafe {
            exports::open(
                path.as_ptr(),
                libc::O_CREAT | libc::O_TRUNC | libc::O_RDWR,
                0o600 as mode_t,
            )
        };
        assert!(fd >= 0);

        let written =
            unsafe { exports::write(fd, contents.as_ptr().cast::<c_void>(), contents.len()) };
        assert_eq!(written, contents.len() as isize);
        assert_eq!(unsafe { exports::lseek(fd, 0, libc::SEEK_SET) }, 0);

        let mut buffer = [0_u8; 5];
        let read = unsafe { exports::read(fd, buffer.as_mut_ptr().cast::<c_void>(), buffer.len()) };
        assert_eq!(read, contents.len() as isize);
        assert_eq!(&buffer, contents);

        assert_eq!(unsafe { exports::close(fd) }, 0);
        assert_eq!(unsafe { exports::unlink(path.as_ptr()) }, 0);
    }

    #[test]
    fn host_fopen_fwrite_fread_fclose_roundtrip() {
        let mut path = std::env::temp_dir();
        path.push(format!("why2025-badge-stdio-{}", std::process::id()));
        let path = CString::new(path.as_os_str().as_bytes()).unwrap();
        let mode = CString::new("wb+").unwrap();
        let contents = b"stdio";

        let stream = unsafe { exports::fopen(path.as_ptr(), mode.as_ptr()) };
        assert!(!stream.is_null());

        let written = unsafe {
            exports::fwrite(
                contents.as_ptr().cast::<c_void>(),
                1,
                contents.len() as c_uint,
                stream,
            )
        };
        assert_eq!(written, contents.len() as c_uint);
        assert_eq!(unsafe { exports::fflush(stream) }, 0);
        unsafe { exports::rewind(stream) };

        let mut buffer = [0_u8; 5];
        let read = unsafe {
            exports::fread(
                buffer.as_mut_ptr().cast::<c_void>(),
                1,
                buffer.len() as c_uint,
                stream,
            )
        };
        assert_eq!(read, buffer.len() as c_uint);
        assert_eq!(&buffer, contents);
        assert_eq!(unsafe { exports::fclose(stream) }, 0);
        assert_eq!(unsafe { exports::remove(path.as_ptr()) }, 0);
    }

    #[test]
    fn host_network_name_and_address_resolution_roundtrip() {
        let ip = CString::new("127.0.0.1").unwrap();
        let localhost = CString::new("localhost").unwrap();

        let mut addr = in_addr { s_addr: 0 };
        assert_eq!(unsafe { exports::inet_aton(ip.as_ptr(), &mut addr) }, 1);

        let rendered = unsafe { exports::inet_ntoa(addr) };
        assert!(!rendered.is_null());
        let rendered = unsafe { CStr::from_ptr(rendered) };
        assert_eq!(rendered.to_bytes(), b"127.0.0.1");

        let mut result = core::ptr::null_mut::<addrinfo>();
        let status = unsafe {
            exports::getaddrinfo(
                localhost.as_ptr(),
                core::ptr::null(),
                core::ptr::null(),
                &mut result,
            )
        };
        assert_eq!(status, 0);
        assert!(!result.is_null());

        unsafe { exports::freeaddrinfo(result) };
    }

    #[test]
    fn host_time_iconv_and_line_reader_roundtrip() {
        let epoch: time_t = 0;

        let gmt = unsafe { exports::gmtime(&epoch) };
        assert!(!gmt.is_null());
        let gmt_ref = unsafe { &*gmt };
        assert_eq!(gmt_ref.tm_year, 70);
        assert_eq!(gmt_ref.tm_mday, 1);

        let ascii_time = unsafe { exports::asctime(gmt) };
        assert!(!ascii_time.is_null());
        let ascii_time = unsafe { CStr::from_ptr(ascii_time) };
        assert!(ascii_time.to_bytes().ends_with(b"\n"));
        assert!(
            ascii_time
                .to_bytes()
                .windows(4)
                .any(|window| window == b"1970")
        );

        let ctime_value = unsafe { exports::ctime(&epoch) };
        assert!(!ctime_value.is_null());
        let ctime_value = unsafe { CStr::from_ptr(ctime_value) };
        assert!(
            ctime_value
                .to_bytes()
                .windows(4)
                .any(|window| window == b"1970")
        );

        let local = unsafe { exports::localtime(&epoch) };
        assert!(!local.is_null());

        let utf8 = CString::new("UTF-8").unwrap();
        let converter = unsafe { exports::iconv_open(utf8.as_ptr(), utf8.as_ptr()) };
        assert_ne!(converter as isize, -1);
        assert_eq!(unsafe { exports::iconv_close(converter) }, 0);

        let mut path = std::env::temp_dir();
        path.push(format!("why2025-badge-lines-{}", std::process::id()));
        let path = CString::new(path.as_os_str().as_bytes()).unwrap();
        let mode = CString::new("wb+").unwrap();
        let contents = b"alpha\nbeta|gamma";

        let stream = unsafe { exports::fopen(path.as_ptr(), mode.as_ptr()) };
        assert!(!stream.is_null());
        let written = unsafe {
            exports::fwrite(
                contents.as_ptr().cast::<c_void>(),
                1,
                contents.len() as c_uint,
                stream,
            )
        };
        assert_eq!(written, contents.len() as c_uint);
        assert_eq!(unsafe { exports::fflush(stream) }, 0);
        unsafe { exports::rewind(stream) };

        let mut lineptr = core::ptr::null_mut::<c_char>();
        let mut capacity = 0_usize;
        let line_len = unsafe { exports::getline(&mut lineptr, &mut capacity, stream) };
        assert_eq!(line_len, 6);
        let first_line = unsafe { slice::from_raw_parts(lineptr.cast::<u8>(), line_len as usize) };
        assert_eq!(first_line, b"alpha\n");

        let delim_len =
            unsafe { exports::getdelim(&mut lineptr, &mut capacity, '|' as c_int, stream) };
        assert_eq!(delim_len, 5);
        let second_line =
            unsafe { slice::from_raw_parts(lineptr.cast::<u8>(), delim_len as usize) };
        assert_eq!(second_line, b"beta|");

        unsafe { libc::free(lineptr.cast::<c_void>()) };
        assert_eq!(unsafe { exports::fclose(stream) }, 0);
        assert_eq!(unsafe { exports::remove(path.as_ptr()) }, 0);
    }

    #[test]
    fn host_regex_bridge_compiles_execs_and_formats_errors() {
        let mut regex = unsafe { mem::zeroed::<regex_t>() };
        let pattern = CString::new("ba(d+)e").unwrap();

        assert_eq!(
            unsafe { exports::regcomp(&mut regex, pattern.as_ptr(), libc::REG_EXTENDED) },
            0
        );
        assert_eq!(regex.re_magic, HOST_REGEX_MAGIC);
        assert_eq!(regex.re_nsub, 1);

        let candidate = CString::new("baddde").unwrap();
        let mut matches = [crate::regmatch_t {
            rm_so: -1,
            rm_eo: -1,
        }; 2];
        let status = unsafe {
            exports::regexec(
                &regex,
                candidate.as_ptr(),
                matches.len(),
                matches.as_mut_ptr().cast::<[crate::regmatch_t; 0usize]>(),
                0,
            )
        };
        assert_eq!(status, 0);
        assert_eq!((matches[0].rm_so, matches[0].rm_eo), (0, 6));
        assert_eq!((matches[1].rm_so, matches[1].rm_eo), (2, 5));

        let miss = CString::new("badge!").unwrap();
        assert_ne!(
            unsafe {
                exports::regexec(
                    &regex,
                    miss.as_ptr(),
                    0,
                    core::ptr::null_mut::<[crate::regmatch_t; 0usize]>(),
                    0,
                )
            },
            0
        );

        unsafe { exports::regfree(&mut regex) };
        assert_eq!(regex.re_magic, 0);
        assert!(regex.re_g.is_null());

        let mut bad_regex = unsafe { mem::zeroed::<regex_t>() };
        let bad_pattern = CString::new("[").unwrap();
        let err = unsafe { exports::regcomp(&mut bad_regex, bad_pattern.as_ptr(), 0) };
        assert_ne!(err, 0);

        let mut errbuf = [0 as c_char; 128];
        let size = unsafe { exports::regerror(err, &bad_regex, errbuf.as_mut_ptr(), errbuf.len()) };
        assert!(size > 0);
        let message = unsafe { CStr::from_ptr(errbuf.as_ptr()) };
        assert!(!message.to_bytes().is_empty());

        unsafe { exports::regfree(&mut bad_regex) };
    }

    #[test]
    fn host_wrapped_objects_resolve_real_host_globals() {
        assert!(!unsafe { exports::stdin }.is_null());
        assert!(!unsafe { exports::stdout }.is_null());
        assert!(!unsafe { exports::stderr }.is_null());
        assert!(!unsafe { exports::environ }.is_null());
        assert_eq!(unsafe { exports::fileno(exports::stdin) }, 0);
        assert_eq!(unsafe { exports::fileno(exports::stdout) }, 1);
        assert_eq!(unsafe { exports::fileno(exports::stderr) }, 2);
        assert!(!unsafe { *exports::environ }.is_null());
    }
}
