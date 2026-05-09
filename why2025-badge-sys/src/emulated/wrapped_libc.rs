use crate::{
    _ssize_t, DIR, FILE, addrinfo, fpos_t, iconv_t, in_addr, mode_t, off_t, pid_t, regex_t,
    sockaddr, socklen_t, stat as stat_t, termios, time_t, tm,
};
use core::ffi::{c_char, c_int, c_long, c_uint, c_void};

mod runtime;

use runtime::call_resolved;

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
pub unsafe extern "C" fn accept(
    sockfd: c_int,
    addr: *mut sockaddr,
    addrlen: *mut socklen_t,
) -> c_int {
    call_resolved!(runtime::real_accept, sockfd, addr, addrlen)
}

#[unsafe(no_mangle)]
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
pub unsafe extern "C" fn ctime(timer: *const time_t) -> *mut c_char {
    call_resolved!(runtime::real_ctime, timer)
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
pub unsafe extern "C" fn gmtime(timer: *const time_t) -> *mut tm {
    call_resolved!(runtime::real_gmtime, timer)
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
pub unsafe extern "C" fn isatty(fd: c_int) -> c_int {
    call_resolved!(runtime::real_isatty, fd)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn listen(sockfd: c_int, backlog: c_int) -> c_int {
    call_resolved!(runtime::real_listen, sockfd, backlog)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn localtime(timer: *const time_t) -> *mut tm {
    call_resolved!(runtime::real_localtime, timer)
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
pub unsafe extern "C" fn socket(domain: c_int, ty: c_int, protocol: c_int) -> c_int {
    call_resolved!(runtime::real_socket, domain, ty, protocol)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn stat(path: *const c_char, buf: *mut stat_t) -> c_int {
    call_resolved!(runtime::real_stat, path, buf)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn system(command: *const c_char) -> c_int {
    call_resolved!(runtime::real_system, command)
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
