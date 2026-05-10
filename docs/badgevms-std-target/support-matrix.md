# V1 support matrix

This matrix is the implementation and test inventory for the BadgeVMS std target. Supported APIs must be backed by real BadgeVMS behavior. Unsupported APIs must fail explicitly through the shared unsupported layer.

| Area | V1 status | Required behavior |
| --- | --- | --- |
| Target cfg | Supported | `target_os = "badgevms"`, `target_family = "unix"`, 32-bit RISC-V, panic abort. |
| Linking | Supported | Toolchain emits PIC ELF shared object, entry `main`, closed exports. |
| Allocation | Supported | Rust and C share one BadgeVMS process heap; OOM aborts. |
| Panic | Supported | Abort in main and spawned threads; panic output goes to TT01 stderr where possible. |
| `std::thread::spawn` | Supported | Rust-owned runtime on BadgeVMS `thread_create`. |
| `JoinHandle::join` | Supported | Rust-owned completion state; not a direct BadgeVMS `wait` mapping. |
| JoinHandle drop | Supported | Detaches Rust join state only; does not kill the BadgeVMS Thread. |
| TLS destructors | Supported | Run on normal spawned-thread exit. |
| `park` / `unpark` | Supported, temporary | Correct token semantics via user-space shared state and time-based waiting. |
| `available_parallelism` | Supported | Always reports `1`. |
| `Mutex` / `Condvar` | Supported | Built on the same Rust-owned wait/park substrate. |
| `Instant` | Supported | Uses `CLOCK_MONOTONIC`. |
| `SystemTime` | Supported | Uses `CLOCK_REALTIME`, with `gettimeofday` fallback only if needed. |
| `thread::sleep` | Supported, best effort | Best-effort delay primitive; stronger guarantees are not promised. |
| Files | Supported | Native BadgeVMS paths; backed by real fd operations. |
| Directories | Supported | Backed by `opendir`, `readdir`, `rewinddir`, `closedir`. |
| Current directory | Unsupported | `current_dir` and `set_current_dir` fail explicitly. |
| Unix path translation | Unsupported | Rust does not invent slash-path or cwd behavior. |
| Symlink APIs | Unsupported | Explicit unsupported failure. |
| uid/gid or POSIX mode mutation | Unsupported | Explicit unsupported failure unless real firmware support is later added. |
| Raw fd borrowing/ownership | Partially supported | Files, stdio, directories where fd-backed, and TCP sockets. |
| `dup` / `fcntl` / `pipe` | Unsupported | Explicit unsupported failure. |
| Stdio | Supported | fd 0/1/2 map to fresh TT01 handles; terminal detection true. |
| Termios mutation | Placeholder | No-op only if matching firmware behavior is verified; otherwise unsupported. |
| `env::args` | Supported | Captures BadgeVMS loader `argc`/`argv` exactly. |
| `env::var` / `var_os` | Supported | Delegates to `getenv`. |
| `env::vars` / `vars_os` | Partial | Reflects `environ`; may enumerate nothing. |
| Environment mutation | Unsupported | Explicit failure or documented infallible abort/panic path. |
| `Command::spawn` | Supported subset | Raw BadgeVMS executable path, exact argv, default child stdio/environment. |
| `try_wait` / `wait` / `status` | Supported subset | Rust-owned completion cache, normal completion only, synthetic exit code `0`. |
| Command cwd/env/stdio overrides | Unsupported | Fail when launch is attempted. |
| `Command::output` / `kill` / `fork` / `exec` | Unsupported | Explicit failure. |
| Installed App ID launch | Unsupported in std | Must be a separate BadgeVMS-specific API. |
| TCP | Supported | `TcpStream`, `TcpListener`, `SocketAddr`, `ToSocketAddrs`. |
| Address resolution | Supported | Backed by `getaddrinfo` / `freeaddrinfo`. |
| UDP | Unsupported | Explicit unsupported failure. |
| Unix-domain/local sockets | Unsupported | Explicit unsupported failure. |
| Socket options/nonblocking/clone | Unsupported unless backed | Do not emulate in Rust-only state. |

## Release gate

An area may move from planned to supported only when an externally visible conformance test exists and the docs above match the observed behavior.
