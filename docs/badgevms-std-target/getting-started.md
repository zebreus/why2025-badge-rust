# Getting started with the BadgeVMS std target

The BadgeVMS std target is incubating. Use the patched Rust toolchain bundled with this
superproject together with local `rustup toolchain link` first. Hosted rustup distribution is
intentionally deferred until the local flow and conformance tests are stable.

## Prerequisites

- Rustup.
- A checkout of this repository with the bundled `why2025-badge-rust-toolchain` submodule
  initialized.
- BadgeVMS/ESP RISC-V linker tools, normally including `riscv32-esp-elf-gcc` or the verified BadgeVMS SDK linker driver.
- `rust-src`, Python, CMake, Ninja, Git, and ELF inspection tools such as `readelf`.

The development shell in [flake.nix](../../flake.nix) includes common host-side tools and leaves the patched BadgeVMS std toolchain selectable through rustup.

## Build and link a local toolchain

Initialize the bundled Rust checkout submodule:

```sh
git submodule update --init why2025-badge-rust-toolchain
```

The tooling uses that bundled checkout automatically. Public documentation treats the superproject
as the supported entrypoint for the BadgeVMS std target.

Build the bundled toolchain:

```sh
./tools/badgevms-std/build-toolchain.sh
```

Link it into rustup:

```sh
stage2=$(./tools/badgevms-std/common-stage2-path.sh)
./tools/badgevms-std/link-toolchain.sh "$stage2" "$BADGEVMS_TOOLCHAIN_NAME"
```

Verify it:

```sh
./tools/badgevms-std/verify-toolchain.sh "$BADGEVMS_TOOLCHAIN_NAME"
```

## Build an example App

```sh
cargo +badgevms-std badgevms-std-build --manifest-path examples/std-hello-world/Cargo.toml
```

If the active shell provides a non-rustup `cargo` before rustup's proxy in `PATH`, use the smoke
wrapper or set `RUSTC=$(rustup which --toolchain badgevms-std rustc)` explicitly. The wrapper does
this automatically.

Or use the smoke wrapper:

```sh
./tools/badgevms-std/run-smoke.sh badgevms-std examples/std-hello-world/Cargo.toml
```

## What this does not do

- It does not use `why2025-badge-build` for std Apps.
- It does not use `riscv32imafc-unknown-none-elf`.
- It does not run Host builds using Emulation.
- It does not imply stable-channel support.

Use the existing `badge-*` aliases for no_std Apps and the existing `emu-*` aliases for Host builds using Emulation.
