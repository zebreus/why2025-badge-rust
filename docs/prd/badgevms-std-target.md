## Problem Statement

As a WHY2025 badge developer, I can currently choose between two very different paths:

- a pragmatic host-emulation path that uses Rust `std` on Linux for development
- the actual badge target, which is still a `no_std` flow built around `riscv32imafc-unknown-none-elf`

That means I cannot write a real on-device BadgeVMS application against Rust `std`, even though BadgeVMS already provides an operating-system-like environment with managed processes, shared-address-space threads, file I/O, networking, clocks, and terminal I/O.

The current gap is not just crate ergonomics. The badge runtime has a distinct execution model:

- BadgeVMS applications are PIC ELF shared objects loaded by BadgeVMS
- the entrypoint is `main`
- the SDK/sysroot is BadgeVMS-specific
- there is no `dlopen()`-style general shared-library model
- BadgeVMS paths are VMS-like rather than Unix-like

From the user perspective, the missing feature is simple: "I want to install a toolchain, target the actual badge, and use Rust `std` on-device, with explicit and predictable failure for any `std` feature BadgeVMS does not support yet."

## Solution

Provide a real BadgeVMS Rust target for on-device apps, backed by a patched Rust toolchain and a BadgeVMS-specific `std` backend, instead of trying to stretch the current `unknown-none` target or keeping a repo-local copy of `libstd`.

From the user perspective, the finished solution looks like this:

- developers install a BadgeVMS-capable Rust toolchain via `rustup`
- developers build against a BadgeVMS target triple rather than `unknown-none`
- BadgeVMS apps can opt into Rust `std` on the real badge hardware
- supported APIs such as `std::thread`, `std::time`, `std::fs`, `std::net`, and stdio use the BadgeVMS ABI
- unsupported APIs fail explicitly and predictably, rather than silently pretending to work
- the existing Linux host-emulation path remains available for fast iteration and testing, but it is clearly separated from the true on-device target

The implementation will be staged:

1. introduce a BadgeVMS target definition and toolchain build flow
2. implement a minimal but correct BadgeVMS `std` backend with partial support
3. ship local `rustup toolchain link` installation first
4. optionally add a hosted rustup-compatible distribution once the target is stable enough to publish

## V1 Platform Contract

- The v1 target supports real on-device `std::thread` and is not allowed to defer threading to a later rewrite.
- The target uses a distinct BadgeVMS OS identity while still declaring the Unix family for compatibility. The intent is `target_os = "badgevms"` with `target_family = "unix"`, not pretending to be Linux or another existing Unix OS.
- The target remains `panic=abort` for v1, including panics in spawned Rust threads.
- The toolchain owns BadgeVMS application linking directly in v1. Std badge apps do not require `why2025-badge-build`, `badge-app-link`, or another helper crate to inject `--shared`, `--entry=main`, or retain-style export pruning.
- The entry contract is fixed in v1: badge std applications export `main`, link as BadgeVMS shared objects, and use closed export pruning equivalent to the current retain-symbols-file workflow. User-defined extra exports and alternate entry symbols are out of scope.
- `std::thread` is implemented through a Rust-owned thread runtime layered on top of BadgeVMS `thread_create`, not by directly exposing BadgeVMS `wait` semantics as Rust join semantics.
- V1 does not expand the firmware ABI for threading. Temporary shims such as the userspace parker must be isolated behind explicit replacement seams and comments that point to the future firmware-backed implementation.
- `JoinHandle::join` uses Rust-owned per-thread completion state and unique join ownership. Dropping a `JoinHandle` detaches only the Rust join state, not the underlying BadgeVMS thread itself.
- `thread::park`, `park_timeout`, and `Thread::unpark` are implemented in user space with correct token semantics, backed by shared state plus time-based waiting. This is intentionally temporary and may be inefficient.
- Thread-local destructors are required for spawned Rust threads in v1 and must run through the Rust-owned thread runtime on normal thread exit.
- `std::thread::Builder::stack_size` follows a strict policy: `0` means platform default, requests below the platform minimum are rounded up, and requests above the BadgeVMS representable range fail instead of being silently truncated.
- `std::thread::available_parallelism()` reports `1` in v1, matching the current BadgeVMS app execution model rather than raw chip core count.
- `std::fs` and related syscall-backed path operations use native BadgeVMS path semantics directly. Rust does not invent Unix path translation or a synthetic current working directory.
- `std::env::current_dir()` and `std::env::set_current_dir()` are unsupported in v1.
- `std::process::Command` is supported in a narrow, explicit subset: BadgeVMS executable-path launch, exact BadgeVMS argv shaping, `spawn`, `try_wait`, blocking wait, and `status`, using default BadgeVMS child stdio and environment behavior.
- `Command::new` stays at the raw executable-path layer. It accepts BadgeVMS paths and uses BadgeVMS logical-name resolution; it does not search Unix `PATH`, host executables, or Installed App IDs.
- Launching an Installed App by App ID remains a separate BadgeVMS-specific higher-level API rather than part of `std::process`.
- When no explicit argv is supplied, child launch matches raw BadgeVMS `process_create` behavior by presenting `argv[0]` as the executable path.
- Spawned child processes receive fresh TT01-backed `stdin`, `stdout`, and `stderr` just like raw BadgeVMS processes. V1 does not inherit the parent file table or synthesize alternate defaults for process launch.
- The child-facing launch contract is intentionally aligned with raw BadgeVMS process launch so unaware C child Apps remain supported under `std::process` and observe the same lookup, argv, and default stdio behavior.
- Unsupported `Command` builder features such as environment mutation, cwd override, stdio redirection, and pipes fail explicitly when process launch is attempted. Other unsupported process APIs such as `output`, `kill`, `fork`, `exec`, and richer child-status control are also explicit failures in v1 rather than being silently ignored.
- Child completion tracking is Rust-owned and caches terminal completion state after first observation so repeated `wait`, `try_wait`, and `status` calls stay coherent even if BadgeVMS later recycles the pid.
- `std::process` may manage any child process it spawned, whether the child itself is written in Rust, C, or another BadgeVMS-native language. V1 does not promise same-parent interoperability between raw `process_create`/`wait` calls and Rust-managed child waiting.
- V1 child status models only BadgeVMS-supported normal completion. Unix signal termination, stop, continue, and core-dump reporting are unsupported and must not be synthesized in Rust-owned shadow state.
- V1 does not synthesize Installed App identity, `APP:`-relative runtime context, or other app-manifest-derived launch state from an executable path, even when the path points into `APPS:`.
- V1 does not add a Rust-only exit-status side channel. Completed child processes currently surface a synthetic exit code of `0`, with an explicit TODO to switch to real exit codes once BadgeVMS exposes them.
- Environment lookups use BadgeVMS as the source of truth per API. `var()` and `var_os()` delegate to `getenv()`, while `vars()` and `vars_os()` reflect `environ` and may therefore enumerate nothing until the firmware exposes a real environment array.
- `std::net` is frozen to TCP plus address resolution in v1: `TcpStream`, `TcpListener`, `SocketAddr`, `ToSocketAddrs`, and `getaddrinfo`-backed lookup are supported; `UdpSocket`, Unix-domain/local sockets, and datagram-dependent features are unsupported.
- `Instant` uses `CLOCK_MONOTONIC`, `SystemTime` uses wall-clock time from `CLOCK_REALTIME` or `gettimeofday`, and `thread::sleep` is supported as a best-effort delay primitive. Stronger time guarantees remain undocumented.
- `stdin`, `stdout`, `stderr`, `print!`, `eprint!`, and panic output all map to the TT01 terminal device in v1. Terminal detection reports true, while termios-style configuration remains a placeholder no-op with an explicit upgrade path.
- Rust and C interop share one BadgeVMS process heap. The target does not introduce a second allocator domain for Rust std, and out-of-memory is treated as an aborting fatal error.
- Unix raw-fd integration is supported as far as BadgeVMS really has fd-backed resources. Files, directories, stdio, and TCP sockets may participate in raw-fd ownership and borrowing APIs, but missing dup/fcntl/pipe-style descriptor management remains unsupported rather than emulated.
- Unix filesystem extensions are exposed only where they map onto real BadgeVMS operations. Unsupported Unix extras such as symlink handling and uid/gid or POSIX mode mutation fail explicitly instead of being simulated in Rust-owned shadow state.
- Temporary gaps and fake implementations are centralized behind one shared capability/error layer so they fail consistently and remain easy to replace later.
- `std::sync::Mutex` and `std::sync::Condvar` are required in v1 and are implemented on top of the same Rust-owned runtime substrate as thread parking and wakeup.

## User Stories

1. As a badge app author, I want to compile for a real BadgeVMS target triple, so that I can write on-device programs without staying in `no_std`.
2. As a badge app author, I want to use Rust `std` on the actual badge, so that I can use common Rust libraries and programming patterns.
3. As a badge app author, I want `cargo` commands for the badge target to rebuild `std` as part of the build, so that my app and the standard library are compiled for the same runtime assumptions.
4. As a badge app author, I want the BadgeVMS linker and loader constraints to be encoded in the target, so that I do not need to rediscover the required PIC/shared-object flags by trial and error.
5. As a badge app author, I want `std::thread::spawn` to work on-device, so that I can structure concurrent work as standard Rust threads.
6. As a badge app author, I want `JoinHandle::join` to behave correctly, so that thread completion synchronizes with my caller in the same way it does on supported Rust targets.
7. As a badge app author, I want dropping a `JoinHandle` to detach correctly, so that standard Rust thread lifecycle semantics remain intact on BadgeVMS.
8. As a badge app author, I want thread-local destructors and thread teardown to run in a defined way, so that `std` thread lifecycle does not leak or corrupt state.
9. As a badge app author, I want `thread::park` and `Thread::unpark` to work correctly, so that standard synchronization primitives and parking-based coordination do not deadlock.
10. As a badge app author, I want `std::time` to use BadgeVMS clocks, so that sleeping, deadlines, timeouts, and elapsed-time measurement work on-device.
11. As a badge app author, I want `std::fs::File` and related APIs to work with BadgeVMS filesystems, so that I can read and write files from Rust `std` code.
12. As a badge app author, I want BadgeVMS path behavior to be documented and explicit, so that I understand how VMS-style paths interact with Rust path APIs.
13. As a badge app author, I want `std::net` to use BadgeVMS sockets and name resolution, so that I can write networked badge apps with standard Rust networking code.
14. As a badge app author, I want `stdin`, `stdout`, `stderr`, `print!`, and panic output to map to BadgeVMS terminal/device behavior, so that diagnostics are visible on the device.
15. As a badge app author, I want allocator and process-exit behavior to be target-defined, so that memory allocation and fatal failure behave consistently with BadgeVMS expectations.
16. As a badge app author, I want unsupported APIs such as partially implemented process-management features to fail explicitly, so that I can recognize platform limits quickly.
17. As a badge app author, I want unsupported `std` behavior to return platform errors or use documented abort behavior, so that failures are predictable and testable.
18. As a library author, I want my crate to compile for the badge `std` target with minimal conditional compilation, so that library code can be reused between host and badge builds.
19. As a library author, I want to depend on the target’s documented capability set, so that I know when to use `std`, when to gate features, and when to surface platform-specific limitations.
20. As a contributor, I want the BadgeVMS `std` backend to be organized into deep modules, so that core runtime decisions are encapsulated behind small, stable interfaces.
21. As a contributor, I want a dedicated BadgeVMS ABI facade for filesystem, networking, time, stdio, process, and threading operations, so that raw firmware bindings do not leak through the whole backend.
22. As a contributor, I want a dedicated thread lifecycle module, so that spawn, join, detach, TLS teardown, and park/unpark logic can be tested in isolation.
23. As a contributor, I want unsupported-platform behavior centralized, so that partial-`std` decisions stay consistent across subsystems.
24. As a maintainer, I want the new target to preserve the current host-emulation route, so that fast Linux-based development does not regress while on-device `std` is being built out.
25. As a maintainer, I want existing host-side ADRs around selective libc overlap and symbol interposition to remain settled, so that this work does not accidentally reopen unrelated host-emulation design decisions.
26. As a maintainer, I want the BadgeVMS target to be distributed as a patched toolchain rather than a repo-local `libstd` fork, so that compiler, target spec, and standard library changes move together.
27. As a maintainer, I want a local-install story based on `rustup toolchain link`, so that developers can adopt the target before any hosted distribution exists.
28. As a release engineer, I want the option to publish a rustup-compatible hosted distribution later, so that users can eventually install the target with normal rustup flows instead of local linking.
29. As a CI maintainer, I want the toolchain build and badge-target smoke tests to be automatable, so that regressions in the target, sysroot, or distribution pipeline are caught early.
30. As a CI maintainer, I want host and badge tests to remain clearly separated, so that emulation validation and on-device target validation do not get conflated.
31. As a documentation author, I want installation and usage docs for the BadgeVMS `std` target, so that users understand the staged support model and required nightly/build-std flow.
32. As a documentation author, I want supported and unsupported `std` areas to be listed explicitly, so that users can judge whether the target is ready for their app.
33. As a project maintainer, I want the target to reflect BadgeVMS process and thread semantics rather than pretending to be generic Unix, so that the runtime model stays honest.
34. As a project maintainer, I want the badge target to encode BadgeVMS-specific path, loader, and process constraints, so that future contributors are guided by the platform model instead of cargo-culting other targets.
35. As a badge app author, I want to use the same language and tooling workflow as other Rust `std` projects where practical, so that building real badge apps feels like Rust rather than a special-case porting exercise.

## Implementation Decisions

- Introduce a dedicated BadgeVMS target triple instead of continuing to model the badge as `unknown-none`.
- Treat this as a real toolchain-and-standard-library port, not as a crate feature and not as a standalone `libstd` copy inside the application repository.
- Keep the target partial by design in the first implementation. Unsupported `std` features are acceptable, but their behavior must be explicit and documented.
- Encode the BadgeVMS load model directly in the target and backend assumptions: PIC application objects, shared-object linking model, a fixed `main` entrypoint, closed export pruning, BadgeVMS SDK/sysroot expectations, and abort-first failure behavior.
- Make the target Unix-family for compatibility, but keep a distinct BadgeVMS OS identity instead of impersonating Linux or another existing Unix target.
- Preserve the existing host-emulation path as a separate development mode. Host emulation and on-device `std` are related, but they are not the same product and must not be collapsed into one abstraction.
- Keep the canonical raw firmware bindings in a low-level artifact (`why2025-badge-sys-bindings`) and keep wrapper plus Emulation behavior in `why2025-badge-sys`, so the ABI reference remains explicit instead of accumulating more policy in the wrapper crate.
- Build a deep BadgeVMS ABI facade module that owns the translation between raw firmware exports and the needs of the Rust standard library.
- Build a deep thread runtime module on top of BadgeVMS task/thread primitives. This module owns spawn, join, detach, lifecycle state, TLS teardown, and park/unpark semantics.
- Do not implement `std::thread` as a thin direct mapping to the existing parent/child wait queue model. Rust thread semantics require unique join ownership, detach-on-drop, and defined synchronization guarantees.
- Keep v1 on `panic=abort`, including child threads, so the platform does not need to solve unwind transport or cross-thread unwinding semantics in the first release.
- Do not expand the firmware ABI in v1 to add Rust-specific thread helpers. Temporary thread waiting and wakeup behavior lives in replaceable Rust-owned modules with explicit TODOs for future firmware-backed implementations.
- Require spawned-thread TLS teardown in v1 rather than deferring it, even though the initial implementation may be Rust-owned rather than exposed directly through a first-class BadgeVMS TLS ABI.
- Clamp or reject thread stack sizes in Rust-facing code instead of inheriting the firmware's current silent truncation behavior.
- Report `available_parallelism()` as `1` in v1, matching the current BadgeVMS application scheduling contract.
- Own badge-app linking in the toolchain itself for std apps. The helper-crate workflow remains for current `no_std` consumers, but it is not part of the std target story.
- Centralize unsupported behavior behind a shared capability layer so that all partial-`std` decisions are consistent across subsystems.
- Keep BadgeVMS path semantics native and explicit. Do not invent a fake Unix path model for APIs whose underlying platform is not Unix-shaped.
- Keep `std::process::Command` at the raw executable-path layer. Installed App launch by App ID is a separate BadgeVMS-specific concern and is not folded into `std`.
- Use BadgeVMS path and logical-name resolution for `Command::new`. Do not add Unix `PATH` search or host-executable fallback to the on-device target contract.
- Support a minimal but real `std::process::Command` subset in the MVP: path-plus-argv launch, `spawn`, `try_wait`, blocking wait, and `status`. Full `Command` parity is not required and should not block the target.
- Match raw BadgeVMS child-launch semantics for argv and default stdio. In particular, absent explicit argv the child sees `argv[0] = executable path`, and each spawned process starts with fresh TT01 stdio rather than inherited parent descriptors.
- Keep the child-facing process contract honest for unaware C child Apps. If `std::process` launches a BadgeVMS-native child, that child should observe the same launch semantics it would see under raw BadgeVMS process creation.
- Keep child management Rust-owned once launched. Cache terminal completion state to survive pid reuse, allow Rust to manage any child it spawned regardless of implementation language, and do not promise same-parent interop with raw `process_create`/`wait`.
- Model only normal process completion in v1. Do not fabricate Unix signal/core-dump status or app-manifest-derived runtime context such as inferred Installed App identity or synthetic `APP:` bindings.
- Use BadgeVMS APIs as the source of truth for environment behavior, even when that leaves v1 with asymmetric lookup-versus-enumeration semantics.
- Freeze v1 networking to TCP plus name resolution. Do not advertise UDP, Unix-domain sockets, or datagram semantics before the platform actually exposes them.
- Map stdio directly onto the single TT01 console device in v1. Treat termios support as an explicit placeholder rather than as a fully supported feature.
- Use one BadgeVMS process heap for both Rust and C allocation behavior, and treat OOM as fatal.
- Expose as much Unix raw-fd behavior as existing BadgeVMS handles justify, but do not fabricate missing descriptor-management operations in Rust-only space.
- Expose Unix filesystem extensions only where BadgeVMS has a real backing operation. Do not invent Rust-side shims for symlinks, ownership changes, or POSIX permission control that the platform does not currently expose.
- Ship `Mutex` and `Condvar` in v1 on top of the same Rust-owned wait and park substrate used by the threading runtime.
- Keep existing accepted ADR decisions around host-side libc overlap and `_ctype_` normalization unchanged. This PRD is about the badge `std` target, not about reopening host interposition policy.
- Add the target in the Rust toolchain and standard library source flow first, then integrate it into this repository’s developer workflow and documentation.
- Stage distribution in two phases: local custom-toolchain linking first, hosted rustup-compatible distribution second.
- Keep the target on nightly during incubation and rely on `build-std` while the backend remains experimental.

## Testing Decisions

- Good tests must assert external behavior and platform contracts, not implementation details. The target should be tested by what developers can observe from compiled badge programs, not by the internal arrangement of helper types or state machines.
- A good test for this work proves user-visible semantics such as successful installation, correct linking, correct thread lifecycle behavior, explicit unsupported errors, correct I/O behavior, and correct timeout/synchronization behavior.
- The BadgeVMS ABI facade should be tested thoroughly because it is the main deep module that shields the rest of the backend from raw firmware details.
- The thread runtime module should be tested thoroughly because it is the highest-risk part of the partial-`std` port and carries correctness requirements around join, detach, TLS teardown, and park/unpark semantics.
- The target/link configuration should have smoke tests that prove BadgeVMS-compatible artifacts are produced and load as BadgeVMS applications.
- Time, stdio, filesystem, and networking integrations should each have behavior-focused tests that verify the Rust `std` contract actually maps onto BadgeVMS behavior.
- Process tests should lock in the partial `Command` contract, including BadgeVMS-path-only lookup, exact argv defaulting, `try_wait`, fresh child stdio defaults, unaware C child coverage, explicit failure for unsupported builder options and non-goal APIs, coherent post-exit observation across pid reuse, and the temporary synthetic zero exit status.
- Process-status tests should prove that v1 models only normal completion and does not synthesize Unix signal or core-dump outcomes.
- Environment tests should lock in the BadgeVMS source-of-truth rule, including the current lookup-versus-enumeration asymmetry.
- Networking tests should verify TCP and address-resolution support while explicitly proving UDP-style and Unix-domain-socket APIs fail in the documented way.
- Filesystem tests should lock in native BadgeVMS path semantics and explicit unsupported behavior for current-directory APIs.
- Unix-family fd tests should verify ownership and borrowing on supported handle types and explicit unsupported behavior for missing dup/fcntl-style operations.
- Unix filesystem-extension tests should prove that only directly backed operations are exposed and that unsupported symlink, ownership, and POSIX mode-control APIs fail in the documented way.
- Unsupported areas such as incomplete process management and descriptor management should have explicit tests that lock in the chosen failure mode.
- Distribution and installation should have smoke tests for the supported developer flow, starting with a `rustup toolchain link` workflow and later expanding to hosted rustup distribution if that phase is implemented.
- Prior art for these tests already exists in two places: the repository’s host-emulation tests and extracted runtime helpers show the preferred pattern of isolating complex emulation/runtime logic behind dedicated modules and validating behavior through subprocesses and externally visible outcomes; upstream Rust `std` tests for threads, joining, parking, timeouts, stdio, and unsupported targets provide the semantic expectations that the BadgeVMS backend should match where support is claimed.

## Out of Scope

- Full upstreaming of the BadgeVMS target to `rust-lang/rust` in the first delivery.
- Stable-channel support in the first delivery.
- Full POSIX or generic Unix compatibility.
- Replacing or removing the existing Linux host-emulation development path.
- General shared-library support or a BadgeVMS loader redesign.
- Alternate entry symbols or user-configurable export whitelists in the MVP.
- Reworking BadgeVMS path syntax into Unix syntax.
- Synthetic current-working-directory behavior for path resolution.
- Blanket host libc interposition or revisiting accepted ADRs unrelated to the badge-side target.
- Complete `std` feature parity, especially in areas where BadgeVMS currently has weak or nonstandard semantics.
- A fully featured `std::process::Command` implementation in the MVP.
- Environment mutation, child stdio redirection, pipes, `output`, `kill`, `fork`, `exec`, and richer child-process control in the MVP.
- App-ID-based launch or synthesized `APP:` runtime context through `std::process` in the MVP.
- Unix `PATH` or host-executable fallback for `std::process::Command` on the on-device target.
- Same-parent interoperability between raw `process_create`/`wait` and `std::process` in the MVP.
- Unix signal/core-dump-style child-status modeling in the MVP.
- UDP sockets, Unix-domain/local sockets, or general datagram networking in the MVP.
- Descriptor duplication and flag-mutation APIs such as dup/fcntl-style behavior in the MVP.
- Unix filesystem features without a real BadgeVMS backing operation, including symlink APIs and uid/gid or POSIX permission mutation in the MVP.
- Real child exit-code propagation before the firmware exposes it.
- Shipping a hosted rustup distribution before the local custom-toolchain workflow is proven and documented.

## Further Notes

- BadgeVMS already looks much more like an operating-system target than a bare-metal `unknown-none` target, but it still has a distinctive runtime model that should be represented honestly.
- The right abstraction boundary is "BadgeVMS target plus BadgeVMS `std` backend consuming a canonical raw ABI artifact", not "more features on the `why2025-badge-sys` wrapper crate".
- For v1, some Unix-facing behavior is intentionally pragmatic rather than pure: the target is Unix-family for compatibility, but still has a distinct BadgeVMS OS identity and explicitly documented non-Unix edges.
- For v1 process launch, `std` stays at the raw BadgeVMS executable-path layer. It does not become an Installed App launcher or infer extra app-manifest context from an `APPS:` path.
- The repository should continue to support two complementary workflows: fast host emulation for iteration and true BadgeVMS `std` for on-device applications.
- The first successful milestone is not full completeness. It is a trustworthy partial target that developers can install, compile against, and reason about.
