# Unsupported behavior policy

The BadgeVMS std target is intentionally partial in v1. Unsupported behavior must be explicit, predictable, documented, and tested.

## Rules

1. Never silently ignore an unsupported option.
2. Never emulate a missing kernel or descriptor feature in Rust-only shadow state unless the PRD explicitly allows a temporary shim.
3. Fallible Rust APIs return `io::ErrorKind::Unsupported` with a BadgeVMS-specific message.
4. C/errno surfaces use ENOSYS or ENOTSUP consistently.
5. Infallible Rust APIs use the documented panic/abort path from the central unsupported layer.
6. Unsupported behavior has conformance tests.
7. Temporary shims have replacement seams and comments pointing to the future firmware-backed implementation.

## Message shape

Use a stable message shape so tests and docs can match it:

```text
unsupported on BadgeVMS v1: <feature>
```

Examples:

- `unsupported on BadgeVMS v1: current working directory`
- `unsupported on BadgeVMS v1: process stdio redirection`
- `unsupported on BadgeVMS v1: UDP sockets`

## Explicitly temporary shims

The user-space parker and related timeout waiting are allowed temporary shims in v1. They must remain isolated behind the thread runtime's wait abstraction and must not leak into unrelated subsystems.

## Non-goals that must fail

- Unix `PATH` search for `Command`.
- Installed App ID launch through `std::process`.
- cwd simulation.
- symlink simulation.
- uid/gid or POSIX mode mutation simulation.
- descriptor duplication simulation.
- UDP or Unix-domain socket simulation.
- Unix signal/core-dump child status synthesis.
