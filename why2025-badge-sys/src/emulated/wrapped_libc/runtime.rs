use crate::{
    _ssize_t, DIR, FILE, addrinfo, dirent, fpos_t, iconv_t, in_addr, mode_t, off_t, pid_t, re_guts,
    regex_t, sockaddr, socklen_t, stat as stat_t, termios, time_t, tm, wchar_t,
};
use core::ffi::{c_char, c_int, c_long, c_uint, c_void};
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

dlsym_resolver!(REAL_ACCEPT, real_accept, b"accept\0", fn accept(sockfd: c_int, addr: *mut sockaddr, addrlen: *mut socklen_t) -> c_int);
dlsym_resolver!(REAL_ASCTIME, real_asctime, b"asctime\0", fn asctime(tblock: *const tm) -> *mut c_char);
dlsym_resolver!(REAL_BIND, real_bind, b"bind\0", fn bind(sockfd: c_int, addr: *const sockaddr, addrlen: socklen_t) -> c_int);
dlsym_resolver!(REAL_CLOSE, real_close, b"close\0", fn close(fd: c_int) -> c_int);
dlsym_resolver!(REAL_CLOSEDIR, real_closedir, b"closedir\0", fn closedir(dir: *mut DIR) -> c_int);
dlsym_resolver!(REAL_CLEARERR, real_clearerr, b"clearerr\0", fn clearerr(stream: *mut FILE) -> ());
dlsym_resolver!(REAL_CLEARERR_UNLOCKED, real_clearerr_unlocked, b"clearerr_unlocked\0", fn clearerr_unlocked(stream: *mut FILE) -> ());
dlsym_resolver!(REAL_CONNECT, real_connect, b"connect\0", fn connect(sockfd: c_int, addr: *const sockaddr, addrlen: socklen_t) -> c_int);
dlsym_resolver!(REAL_CTIME, real_ctime, b"ctime\0", fn ctime(timer: *const time_t) -> *mut c_char);
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
dlsym_resolver!(REAL_GETLINE, real_getline, b"getline\0", fn getline(lineptr: *mut *mut c_char, n: *mut usize, stream: *mut FILE) -> _ssize_t);
dlsym_resolver!(REAL_GETPID, real_getpid, b"getpid\0", fn getpid() -> pid_t);
dlsym_resolver!(REAL_GETS, real_gets, b"gets\0", fn gets(buf: *mut c_char) -> *mut c_char);
dlsym_resolver!(REAL_GMTIME, real_gmtime, b"gmtime\0", fn gmtime(timer: *const time_t) -> *mut tm);
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
dlsym_resolver!(REAL_STRDUP, real_strdup, b"strdup\0", fn strdup(value: *const c_char) -> *mut c_char);
dlsym_resolver!(REAL_STRERROR, real_strerror, b"strerror\0", fn strerror(errnum: c_int) -> *mut c_char);
dlsym_resolver!(REAL_STRLEN, real_strlen, b"strlen\0", fn strlen(value: *const c_char) -> c_uint);
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
dlsym_resolver!(REAL_SYSTEM, real_system, b"system\0", fn system(command: *const c_char) -> c_int);
dlsym_resolver!(REAL_TCGETATTR, real_tcgetattr, b"tcgetattr\0", fn tcgetattr(fd: c_int, termios_p: *mut termios) -> c_int);
dlsym_resolver!(REAL_TCSETATTR, real_tcsetattr, b"tcsetattr\0", fn tcsetattr(fd: c_int, action: c_int, termios_p: *const termios) -> c_int);
dlsym_resolver!(REAL_TOLOWER, real_tolower, b"tolower\0", fn tolower(value: c_int) -> c_int);
dlsym_resolver!(REAL_TOASCII, real_toascii, b"toascii\0", fn toascii(value: c_int) -> c_int);
dlsym_resolver!(REAL_TOUPPER, real_toupper, b"toupper\0", fn toupper(value: c_int) -> c_int);
dlsym_resolver!(REAL_UNGETC, real_ungetc, b"ungetc\0", fn ungetc(value: c_int, stream: *mut FILE) -> c_int);
dlsym_resolver!(REAL_UNLINK, real_unlink, b"unlink\0", fn unlink(path: *const c_char) -> c_int);
dlsym_resolver!(REAL_WCSDUP, real_wcsdup, b"wcsdup\0", fn wcsdup(value: *const wchar_t) -> *mut wchar_t);
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
        assert_eq!(unsafe { exports::strcmp(alpha.as_ptr(), alpha.as_ptr()) }, 0);
        assert!(unsafe { exports::strcmp(alpha.as_ptr(), omega.as_ptr()) } < 0);
    }

    #[test]
    fn host_memory_and_string_pointer_helpers_roundtrip() {
        let mut filled = [0_u8; 8];
        let filled_ptr = unsafe {
            exports::memset(
                filled.as_mut_ptr().cast::<c_void>(),
                'Z' as c_int,
                4,
            )
        };
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
        assert_eq!(moved_ptr, unsafe { shifted.as_mut_ptr().add(1) }.cast::<c_void>());
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
        assert_eq!(unsafe { found.cast::<u8>().offset_from(haystack.as_ptr()) }, 8);

        let last_a = unsafe {
            exports::memrchr(
                haystack.as_ptr().cast::<c_void>(),
                'a' as c_int,
                haystack.len(),
            )
        };
        assert!(!last_a.is_null());
        assert_eq!(unsafe { last_a.cast::<u8>().offset_from(haystack.as_ptr()) }, 7);

        let found_subslice = unsafe {
            exports::memmem(
                haystack.as_ptr().cast::<c_void>(),
                haystack.len(),
                needle.as_ptr().cast::<c_void>(),
                needle.len(),
            )
        };
        assert!(!found_subslice.is_null());
        assert_eq!(unsafe { found_subslice.cast::<u8>().offset_from(haystack.as_ptr()) }, 6);

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
        assert_eq!(unsafe { stop.cast::<u8>().offset_from(memccpy_dst.as_ptr()) }, 4);
        assert_eq!(&memccpy_dst[..4], b"badg");

        let mut mempcpy_dst = [0_u8; 8];
        let mempcpy_end = unsafe {
            exports::mempcpy(
                mempcpy_dst.as_mut_ptr().cast::<c_void>(),
                src.as_ptr().cast::<c_void>(),
                src.len() as c_uint,
            )
        };
        assert_eq!(unsafe { mempcpy_end.cast::<u8>().offset_from(mempcpy_dst.as_ptr()) }, 5);
        assert_eq!(&mempcpy_dst[..5], b"badge");

        let c_prefix = CString::new("badge").unwrap();
        let raw_end = unsafe { exports::rawmemchr(c_prefix.as_ptr().cast::<c_void>(), 0) };
        assert_eq!(unsafe { raw_end.cast::<c_char>().offset_from(c_prefix.as_ptr()) }, 5);

        let mut stpcpy_buf = [0 as c_char; 16];
        let stpcpy_end = unsafe { exports::stpcpy(stpcpy_buf.as_mut_ptr(), c_prefix.as_ptr()) };
        assert_eq!(unsafe { stpcpy_end.offset_from(stpcpy_buf.as_ptr()) }, 5);
        assert_eq!(unsafe { CStr::from_ptr(stpcpy_buf.as_ptr()) }.to_bytes(), b"badge");

        let mut stpncpy_buf = [0 as c_char; 16];
        let stpncpy_end = unsafe { exports::stpncpy(stpncpy_buf.as_mut_ptr(), c_prefix.as_ptr(), 8) };
        assert_eq!(unsafe { stpncpy_end.offset_from(stpncpy_buf.as_ptr()) }, 5);
        assert_eq!(unsafe { CStr::from_ptr(stpncpy_buf.as_ptr()) }.to_bytes(), b"badge");

        let mixed = CString::new("Badge-Rust").unwrap();
        let rust_lower = CString::new("rust").unwrap();
        let dash = CString::new("-").unwrap();
        let set = CString::new("xyzg").unwrap();
        let badge_set = CString::new("Badge-").unwrap();
        let rust_exact = CString::new("Rust").unwrap();

        let case_insensitive = unsafe { exports::strcasestr(mixed.as_ptr(), rust_lower.as_ptr()) };
        assert!(!case_insensitive.is_null());
        assert_eq!(unsafe { CStr::from_ptr(case_insensitive) }.to_bytes(), b"Rust");

        let missing = unsafe { exports::strchrnul(mixed.as_ptr(), '!' as c_int) };
        assert_eq!(unsafe { missing.offset_from(mixed.as_ptr()) as c_uint }, unsafe {
            exports::strlen(mixed.as_ptr())
        });
        assert_eq!(unsafe { exports::strcspn(mixed.as_ptr(), dash.as_ptr()) }, 5);
        assert_eq!(unsafe { exports::strlen(mixed.as_ptr()) }, 10);

        let mut ncat_buf = [0 as c_char; 16];
        assert_eq!(unsafe { exports::strcpy(ncat_buf.as_mut_ptr(), c_prefix.as_ptr()) }, ncat_buf.as_mut_ptr());
        assert_eq!(unsafe { exports::strncat(ncat_buf.as_mut_ptr(), dash.as_ptr(), 1) }, ncat_buf.as_mut_ptr());
        assert_eq!(unsafe { exports::strncat(ncat_buf.as_mut_ptr(), rust_exact.as_ptr(), 2) }, ncat_buf.as_mut_ptr());
        assert_eq!(unsafe { CStr::from_ptr(ncat_buf.as_ptr()) }.to_bytes(), b"badge-Ru");

        let alphabet = CString::new("alphabet").unwrap();
        let alpha_x = CString::new("alphaX").unwrap();
        assert_eq!(unsafe { exports::strncmp(alphabet.as_ptr(), alpha_x.as_ptr(), 5) }, 0);
        assert!(unsafe { exports::strncmp(alphabet.as_ptr(), alpha_x.as_ptr(), 6) } > 0);

        let mut strncpy_buf = [0 as c_char; 16];
        assert_eq!(unsafe { exports::strncpy(strncpy_buf.as_mut_ptr(), c_prefix.as_ptr(), 8) }, strncpy_buf.as_mut_ptr());
        assert_eq!(unsafe { CStr::from_ptr(strncpy_buf.as_ptr()) }.to_bytes(), b"badge");

        assert_eq!(unsafe { exports::strnlen(c_prefix.as_ptr(), 3) }, 3);
        assert_eq!(unsafe { exports::strnlen(c_prefix.as_ptr(), 8) }, 5);

        let first_match = unsafe { exports::strpbrk(mixed.as_ptr(), set.as_ptr()) };
        assert!(!first_match.is_null());
        assert_eq!(unsafe { CStr::from_ptr(first_match) }.to_bytes(), b"ge-Rust");

        let last_rust_a = CString::new("bananabadge").unwrap();
        let last_match = unsafe { exports::strrchr(last_rust_a.as_ptr(), 'a' as c_int) };
        assert!(!last_match.is_null());
        assert_eq!(unsafe { last_match.offset_from(last_rust_a.as_ptr()) }, 7);

        assert_eq!(unsafe { exports::strspn(mixed.as_ptr(), badge_set.as_ptr()) }, 6);

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
            exports::strtok_r(colors.as_mut_ptr().cast::<c_char>(), pipe.as_ptr(), &mut saveptr)
        };
        assert_eq!(unsafe { CStr::from_ptr(token1) }.to_bytes(), b"red");
        let token2 = unsafe { exports::strtok_r(core::ptr::null_mut(), pipe.as_ptr(), &mut saveptr) };
        assert_eq!(unsafe { CStr::from_ptr(token2) }.to_bytes(), b"green");
        let token3 = unsafe { exports::strtok_r(core::ptr::null_mut(), pipe.as_ptr(), &mut saveptr) };
        assert_eq!(unsafe { CStr::from_ptr(token3) }.to_bytes(), b"blue");
        assert!(unsafe { exports::strtok_r(core::ptr::null_mut(), pipe.as_ptr(), &mut saveptr) }
            .is_null());

        let wrapped = unsafe { exports::strerror(libc::ENOENT) };
        let host = unsafe { libc::strerror(libc::ENOENT) };
        assert!(!wrapped.is_null());
        assert!(!host.is_null());
        assert_eq!(unsafe { CStr::from_ptr(wrapped) }.to_bytes(), unsafe {
            CStr::from_ptr(host)
        }
        .to_bytes());
    }

    #[test]
    fn host_duplication_and_rng_helpers_roundtrip() {
        let text = CString::new("badge-rust").unwrap();

        let duplicated = unsafe { exports::strdup(text.as_ptr()) };
        assert!(!duplicated.is_null());
        assert_eq!(unsafe { CStr::from_ptr(duplicated) }.to_bytes(), b"badge-rust");
        unsafe { libc::free(duplicated.cast::<c_void>()) };

        let duplicated_prefix = unsafe { exports::strndup(text.as_ptr(), 5) };
        assert!(!duplicated_prefix.is_null());
        assert_eq!(unsafe { CStr::from_ptr(duplicated_prefix) }.to_bytes(), b"badge");
        unsafe { libc::free(duplicated_prefix.cast::<c_void>()) };

        let wide = ['W' as wchar_t, 'H' as wchar_t, 'Y' as wchar_t, 0];
        let wide_dup = unsafe { exports::wcsdup(wide.as_ptr()) };
        assert!(!wide_dup.is_null());
        let wide_slice = unsafe { slice::from_raw_parts(wide_dup, wide.len()) };
        assert_eq!(wide_slice, &wide);
        unsafe { libc::free(wide_dup.cast::<c_void>()) };

        let comma = CString::new(",").unwrap();
        let mut colors = b"red,green,blue\0".to_vec();
        let token1 = unsafe { exports::strtok(colors.as_mut_ptr().cast::<c_char>(), comma.as_ptr()) };
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
