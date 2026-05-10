---
status: accepted
---

# Implement the BadgeVMS std target as a built-in Rust target

BadgeVMS std is implemented as a built-in Rust target with an explicit BadgeVMS std backend in the Rust fork. That backend depends directly on `why2025-badge-sys-bindings`, keeps `target_family = "unix"` for ecosystem cfg compatibility, keeps `target_env` unspecified instead of advertising `newlib`, and does not use a custom BadgeVMS `libc` fork. We chose this because `std` needs target-specific backend and link orchestration inside rustc, BadgeVMS still needs explicit non-libc branches even while preserving Unix-family cfg compatibility, and keeping the raw ABI in one place avoids splitting BadgeVMS semantics across competing wrapper and libc layers.

## Considered Options

- BadgeVMS-specific `library/libc` fork: rejected because it duplicates raw ABI ownership, invites drift between std and the repo-owned bindings, and pushes BadgeVMS-specific semantics into a layer shared with unrelated targets.
- Generic Unix/libc std routing: rejected because BadgeVMS keeps `target_family = "unix"` only for cfg compatibility, while the generic Unix path assumes libc, pthreads, POSIX process/filesystem semantics, and Unix backtrace support that BadgeVMS does not currently provide.
- `target_env = "newlib"`: rejected because BadgeVMS is not promising a newlib environment, and setting that cfg would steer std and downstream crates toward the wrong libc assumptions.
- External target JSON instead of a built-in target: rejected because BadgeVMS std needs rustc-owned target registration plus link/export orchestration, not just target codegen defaults.
