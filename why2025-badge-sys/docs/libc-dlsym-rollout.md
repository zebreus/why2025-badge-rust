# Host libc dlsym rollout

This document tracks the upstream wrapped libc-shaped BadgeVMS exports and the current Linux host
plan for representing them through raw-name exports that forward to the real libc with
`dlsym(RTLD_NEXT)`.

Decision reference: [ADR 0003](../../docs/adr/0003-use-dlsym-for-host-libc-overlap.md)

## Explicit local exports that are not part of the dlsym rollout

- `__errno`
- `asnprintf`
- `die`
- `funopen`
- `gcvtf`
- `gcvtl`

## Implemented mediated local exports

- `regcomp`
- `regerror`
- `regexec`
- `regfree`

These regex exports do not use direct `dlsym` forwarding because badge `regex_t` is not ABI-compatible
with the host libc `regex_t`. The host build now stores a crate-owned bridge behind the badge
`regex_t.re_g` pointer and translates through the host regex ABI internally.

## Implemented dlsym interposition

- `accept`
- `asctime`
- `bind`
- `close`
- `closedir`
- `clearerr`
- `clearerr_unlocked`
- `connect`
- `ctime`
- `fclose`
- `fdopen`
- `feof`
- `ferror`
- `fflush`
- `fgetc`
- `fgetpos`
- `fgets`
- `fileno`
- `fmemopen`
- `fopen`
- `fputc`
- `fputs`
- `fread`
- `freeaddrinfo`
- `freopen`
- `fseek`
- `fseeko`
- `fstat`
- `ftell`
- `ftello`
- `fwrite`
- `getaddrinfo`
- `getc`
- `getdelim`
- `getchar`
- `getchar_unlocked`
- `getenv`
- `getline`
- `gets`
- `getpid`
- `gmtime`
- `inet_aton`
- `inet_ntoa`
- `iconv_close`
- `iconv_open`
- `isatty`
- `listen`
- `localtime`
- `lseek`
- `mkdir`
- `open`
- `opendir`
- `putchar`
- `puts`
- `read`
- `readdir`
- `remove`
- `rename`
- `rewind`
- `rewinddir`
- `rmdir`
- `setbuf`
- `setbuffer`
- `setlinebuf`
- `setvbuf`
- `socket`
- `stat`
- `system`
- `tcgetattr`
- `tcsetattr`
- `ungetc`
- `unlink`
- `write`

## Additional manifest-driven hosted forwarders beyond `wrapped_function`

- `freeaddrinfo`
- `getaddrinfo`
- `inet_aton`
- `inet_ntoa`

## Remaining wrapped functions

### Process and lifetime

- `_exit`
- `_Exit`
- `abort`
- `atexit`
- `exit`

### Time, regex, and iconv

### Formatting and scanning variadics

- `asprintf`
- `fprintf`
- `fscanf`
- `gcvt`
- `printf`
- `scanf`
- `snprintf`
- `sprintf`
- `sscanf`
- `vasprintf`
- `vfprintf`
- `vfscanf`
- `vprintf`
- `vscanf`
- `vsnprintf`
- `vsprintf`
- `vsscanf`

### Allocation, randomness, and string helpers

- `calloc`
- `free`
- `malloc`
- `rand`
- `random`
- `realloc`
- `reallocarray`
- `srand`
- `srandom`
- `strdup`
- `strerror`
- `strndup`
- `strtok`
- `wcsdup`

## Implemented wrapped objects

- `environ`
- `stderr`
- `stdin`
- `stdout`

## Progress plan

1. Land low-risk fixed-signature wrappers first. This change covers the direct syscall and
   file-descriptor/socket family because those are easy to validate in Rust, `staticlib`, and
   `cdylib` consumers.
2. Add non-variadic FILE and directory wrappers next. These are still straightforward forwarders,
   but they need more coverage around `FILE *`, `DIR *`, and pointer lifetime expectations.
3. Add time, regex, and iconv wrappers after that. This phase is now complete for the current
   fixed-signature set, with regex handled through a mediated host bridge rather than raw host
   struct forwarding.
4. Handle variadic formatting and scanning separately. They need explicit c-variadic trampolines
   and should be validated from both Rust and C call sites.
5. Treat allocator and process-lifetime symbols as a dedicated phase. `malloc` and friends, plus
   `abort`/`exit`/`_Exit`/`atexit`, interact with Rust `std`, process teardown, and error handling
   more deeply than the current batch.
6. Handle wrapped objects last. This phase is now complete on Linux host builds: `stdin`, `stdout`,
   `stderr`, and `environ` are exported as crate-owned data symbols and initialized from the real
   host globals during library startup.
