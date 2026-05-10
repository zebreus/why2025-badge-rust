# Paths, filesystem, stdio, and fd behavior

BadgeVMS paths are native VMS-style paths such as `FLASH0:[DIR.SUBDIR]FILE`. The std target must use those paths directly.

## Path rules

- Accept BadgeVMS paths directly.
- Reject interior NUL bytes.
- Do not translate Unix slash paths into BadgeVMS paths.
- Do not synthesize a current working directory.
- `std::env::current_dir()` and `std::env::set_current_dir()` fail explicitly.

The existing Emulation path parser in [why2025-badge-sys/src/emulated/badgevms/fs/paths.rs](../../why2025-badge-sys/src/emulated/badgevms/fs/paths.rs) is a reference for syntax and test cases, not a `std` dependency.

## Files and directories

`std::fs::File` is backed by real BadgeVMS fd operations such as `open`, `read`, `write`, `lseek`, `close`, `stat`, and `fstat`.

Directory APIs are backed by `opendir`, `readdir`, `rewinddir`, and `closedir`.

Metadata exposes only fields backed by BadgeVMS. Rust must not invent uid/gid, symlink, or POSIX permission semantics.

## Raw fd

Raw fd ownership and borrowing are supported where BadgeVMS has real fd-backed resources:

- files;
- stdio;
- directories if represented as fds by the firmware surface;
- TCP sockets.

Descriptor management that BadgeVMS does not expose is unsupported:

- `dup`;
- `fcntl` flag mutation;
- `pipe`;
- clone/try_clone where it would require descriptor duplication;
- close-on-exec and nonblocking flags unless the firmware adds real backing support.

## Stdio and terminal

Each BadgeVMS Process starts with fresh TT01-backed fd 0, 1, and 2. The std target maps:

- `stdin` to fd 0;
- `stdout`, `print!`, and `println!` to fd 1;
- `stderr`, `eprint!`, and panic output to fd 2.

Terminal detection reports true for TT01-backed stdio.

Termios configuration remains a placeholder. It may be a documented no-op only if that matches verified firmware behavior; otherwise it must fail unsupported.

## Tests

Required tests cover native BadgeVMS path open/read/write, create/truncate/append, directory listing, metadata, removal, cwd unsupported behavior, raw-fd ownership transfer, unsupported descriptor APIs, unsupported symlink/uid/gid/mode mutation, stdio visibility, terminal detection, and panic output.
