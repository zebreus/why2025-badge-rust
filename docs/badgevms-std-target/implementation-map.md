# Implementation map

## Rust fork changes

The patched Rust fork owns these changes:

1. Built-in target spec for `riscv32imafc-unknown-badgevms`.
2. Linker integration for BadgeVMS std Apps: shared object, entry `main`, closed export pruning.
3. `library/std` backend for `target_os = "badgevms"`.
4. `library/libc` as a submodule pointing at the separate BadgeVMS libc fork.
5. BadgeVMS ABI declarations required by `std`.
6. Abort panic and shared allocator integration.
7. Runtime modules for threads, park/unpark, process tracking, filesystem, networking, time, stdio, environment, and unsupported features.
8. Selected upstream std tests with BadgeVMS-specific skips for non-goals.

## Backend module layout

The current patched Rust fork uses a small `os::badgevms` public-extension module plus targeted `target_os = "badgevms"` branches in the existing Unix PAL where BadgeVMS follows fd/socket/process-shaped APIs. Keep raw ABI declarations isolated under the Rust fork's BadgeVMS-owned modules and do not import this repository as a Rust dependency.

If the port outgrows those branches, migrate toward this dedicated PAL shape:

```text
library/std/src/sys/pal/badgevms/
  abi.rs
  unsupported.rs
  rt.rs
  thread.rs
  park.rs
  sync.rs
  time.rs
  fs.rs
  fd.rs
  stdio.rs
  env.rs
  process.rs
  net.rs
```

`abi.rs` is the only module that should expose raw unsafe C declarations. Other modules use narrow safe wrappers.

## Repository responsibilities

This repository owns:

- PRD and ADRs.
- Raw ABI reference via [why2025-badge-sys/src/bindings.rs](../../why2025-badge-sys/src/bindings.rs).
- Firmware source snapshot under [why2025-badge-sys/firmware](../../why2025-badge-sys/firmware).
- Host build using Emulation.
- no_std BadgeVMS App workflow.
- Toolchain build/link scripts.
- Std example Apps.
- CI smoke checks.
- Hardware/network conformance harness definitions.

## ABI source of truth

The std backend ABI should be derived from:

- [why2025-badge-sys/firmware/badgevms/symbols.yml](../../why2025-badge-sys/firmware/badgevms/symbols.yml)
- [why2025-badge-sys/firmware/badgevms/include](../../why2025-badge-sys/firmware/badgevms/include)
- [why2025-badge-sys/headers](../../why2025-badge-sys/headers)
- [why2025-badge-sys/rebuild-bindings.sh](../../why2025-badge-sys/rebuild-bindings.sh)

The `libc` crate patches for BadgeVMS live in the separate
[`why2025-badge-rust-libc`](https://github.com/zebreus/why2025-badge-rust-libc)
fork and are consumed by the Rust fork through its `library/libc` submodule.

Do not make `std` import `why2025-badge-sys` as a Rust dependency. That would make the target depend on this application-support workspace and would violate the toolchain-owned design.

## Linker reference

The existing no_std helper path is still the best local reference for link flags:

- [why2025-badge-sys/src/build_script.rs](../../why2025-badge-sys/src/build_script.rs)
- [why2025-badge-build/src/lib.rs](../../why2025-badge-build/src/lib.rs)

The std target must encode equivalent behavior in the Rust toolchain instead of requiring the App-local helper.
