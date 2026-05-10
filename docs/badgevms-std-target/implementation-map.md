# Implementation map

## Rust fork changes

The patched Rust fork owns these changes:

1. Built-in target spec for `riscv32imafc-unknown-badgevms`.
2. Linker integration for BadgeVMS std Apps: shared object, entry `main`, closed export pruning.
3. `library/std` backend for `target_os = "badgevms"`.
4. Direct consumption of the canonical raw ABI from `why2025-badge-sys-bindings`.
5. Abort panic and shared allocator integration.
6. Runtime modules for threads, park/unpark, process tracking, filesystem, networking, time,
   stdio, environment, and unsupported features.
7. Selected upstream std tests with BadgeVMS-specific skips for non-goals.

## Backend module layout

The current patched Rust fork uses a small `os::badgevms` public-extension module plus targeted
`target_os = "badgevms"` branches in the existing Unix PAL where BadgeVMS follows
fd/socket/process-shaped APIs. Keep raw ABI consumption concentrated in a dedicated BadgeVMS ABI
layer over `why2025-badge-sys-bindings`. The std port should not route BadgeVMS ABI use through the
`why2025-badge-sys` wrapper crate.

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

`abi.rs` is the only module that should expose raw unsafe C declarations or the narrow adapter over
`why2025-badge-sys-bindings`. Other modules use narrow safe wrappers.

## Repository responsibilities

This repository owns:

- PRD and ADRs.
- Raw ABI reference via [why2025-badge-sys-bindings/src/bindings.rs](../../why2025-badge-sys-bindings/src/bindings.rs).
- Firmware source snapshot under [why2025-badge-sys-bindings/firmware](../../why2025-badge-sys-bindings/firmware).
- The wrapper and Emulation layer in [why2025-badge-sys](../../why2025-badge-sys).
- Host build using Emulation.
- no_std BadgeVMS App workflow.
- Toolchain build/link scripts.
- Std example Apps.
- CI smoke checks.
- Hardware/network conformance harness definitions.

## ABI source of truth

The std backend ABI should be derived from:

- [why2025-badge-sys-bindings/firmware/badgevms/symbols.yml](../../why2025-badge-sys-bindings/firmware/badgevms/symbols.yml)
- [why2025-badge-sys-bindings/firmware/badgevms/include](../../why2025-badge-sys-bindings/firmware/badgevms/include)
- [why2025-badge-sys-bindings/extra-headers](../../why2025-badge-sys-bindings/extra-headers)
- [why2025-badge-sys-bindings/rebuild-bindings.sh](../../why2025-badge-sys-bindings/rebuild-bindings.sh)

The prepared `headers/` tree under `why2025-badge-sys-bindings` is generated staging data for
bindgen, not source-of-truth input.

The std backend consumes `why2025-badge-sys-bindings` directly from this superproject. The
`why2025-badge-sys` wrapper remains for Host builds using Emulation and the no_std badge-link
workflow; it is not the std dependency boundary.

## Linker reference

The existing no_std helper path is still the best local reference for link flags:

- [why2025-badge-sys/src/build_script.rs](../../why2025-badge-sys/src/build_script.rs)
- [why2025-badge-build/src/lib.rs](../../why2025-badge-build/src/lib.rs)

The std target must encode equivalent behavior in the Rust toolchain instead of requiring the App-local helper.
