# Upstream Rust patch map

This file tracks the Rust fork work that cannot live directly in this repository.

## Target spec

Add a built-in target for `riscv32imafc-unknown-badgevms` with:

- `target_os = "badgevms"`;
- `target_family = "unix"`;
- no non-empty `target_env` value (`newlib` is intentionally absent; rustc still prints
  `target_env=""` for an unspecified environment);
- 32-bit RISC-V IMACF, little endian, `ilp32f` ABI;
- abort panic;
- PIC output;
- `rust-lld` linker driver.

The JSON file in this directory is a review aid only. The final target must be a built-in Rust target because `std` needs target-specific backend code and link orchestration.

The built-in-target and explicit-std-backend decision is recorded in
[ADR 0006](../../docs/adr/0006-implement-badgevms-std-as-a-built-in-rust-target.md).

## Link orchestration

Patch rustc so BadgeVMS std Apps link as shared objects with `main` as the entry and a closed export set. A target JSON alone is not enough if retain-symbols/version-script files must be generated dynamically.

The built-in target must force the executable export list to `main`. The current Rust hook point is `TargetOptions::override_export_symbols`, with the GNU linker export-list path adjusted so BadgeVMS executable links use an LD version script instead of `--dynamic-list`. Without that, `--shared` Apps retain thousands of exported Rust/`std` functions in `.dynsym`.

## Standard library backend

Add BadgeVMS `std` backend selectors in the Rust fork. BadgeVMS keeps
`target_family = "unix"` for ecosystem cfg compatibility, but `std` must route it before generic
Unix/POSIX branches whenever those branches require libc, pthreads, POSIX filesystem/process
semantics, or Unix backtrace support.

The current backend uses dedicated BadgeVMS pieces for allocator, stdio, errno/error mapping, exit,
environment constants, raw OS aliases, and file-descriptor ownership. Unsupported process, fs, net,
time, pipe, random, thread, and path operations intentionally use existing unsupported PAL modules
until the firmware ABI grows those semantics.

The repository/toolchain raw-ABI boundary for that backend is recorded in
[ADR 0004](../../docs/adr/0004-canonical-badgevms-abi-layering.md).

## Raw ABI dependency

`library/std` depends directly on `why2025-badge-sys-bindings` for `target_os = "badgevms"`. The
raw bindings crate exposes a `rustc-dep-of-std` feature that renames `rustc-std-workspace-core` to
`core`, matching upstream std-dependency conventions. Do not reintroduce a BadgeVMS-specific libc
crate or a custom `library/libc` fork.

`library/unwind`, `library/panic_unwind`, `library/std_detect`, and the vendored `backtrace` crate
must exclude BadgeVMS from libc-backed paths. BadgeVMS backtraces currently resolve to the no-op
backend.

## Packaging

Release archives should include prebuilt `rust-std` for `riscv32imafc-unknown-badgevms`, so normal
users can build with `cargo +badgevms-std build --target riscv32imafc-unknown-badgevms` without
`-Zbuild-std`. The supported packaging tree intentionally does not include stage2 link helpers;
release artifacts come from Rust dist component archives only.

When `rust-src` is packaged for maintainers, the canonical `why2025-badge-sys-bindings` crate must
be included in the installed source tree with a manifest whose `rustc-std-workspace-core` path points
inside `lib/rustlib/src/rust/library`. Release packaging must not depend on symlinks back into the
superproject checkout.

The Rust fork patches the `rust-src` dist step to place `why2025-badge-sys-bindings` at
`lib/rustlib/src/why2025-badge-sys-bindings`, matching the relative path used by
`library/std/Cargo.toml`. The copyright generator also treats this out-of-tree path dependency as a
source-tree crate instead of expecting it in the vendored registry directory. Release archives are
assembled from Rust dist component tarballs, not from mutable `stage2` directories.

The release dist config builds Cargo with `build.cargo-native-static = true` so packaged Cargo does
not depend on the maintainer's Nix or system OpenSSL runtime path. The final assembled toolchain
wraps the dist Cargo binary only to set `RUSTC` to the sibling packaged `rustc`; it does not fall
back to a different host Cargo.

## Test selection

Start from upstream std tests for:

- thread spawn/join/park;
- sync mutex/condvar;
- time and sleep;
- filesystem and io;
- process spawn/wait;
- TCP networking;
- unsupported targets.

Skip or adapt tests that assume POSIX cwd, signals, pipes, env mutation, descriptor duplication, Unix-domain sockets, UDP, or full socket options.
