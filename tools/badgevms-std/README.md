# BadgeVMS std tooling

These scripts support the BadgeVMS std workflow in this superproject. They assume the bundled
`why2025-badge-rust-toolchain` checkout is present. See
[ADR 0005](../../docs/adr/0005-support-badgevms-std-through-the-superproject.md) for why this is
the supported entrypoint.

## Scripts

- `build-toolchain.sh` — build the bundled patched Rust checkout.
- `dist-toolchain.sh` — build Rust dist artifacts for release packaging.
- `package-toolchain.sh` — assemble a relocatable rustup-linkable toolchain archive from Rust dist artifacts.
- `install-toolchain.sh` — install a packaged archive and register it with rustup.
- `dev-link-stage2.sh` — developer-only shortcut for linking a mutable stage2 tree.
- `verify-toolchain.sh` — verify target cfg for `riscv32imafc-unknown-badgevms`, including no
	non-empty `target_env` such as `newlib`.
- `run-smoke.sh` — build a std example and inspect the ELF artifact.
- `inspect-elf.sh` — verify BadgeVMS shared-object shape and closed exports.
- `ci-smoke.sh` — run repository-side checks that do not require BadgeVMS hardware.

The std port uses `why2025-badge-sys-bindings` as the raw BadgeVMS ABI source. The Rust fork should
not carry a BadgeVMS-specific `library/libc` fork.

Set `BADGEVMS_TOOLCHAIN_NAME` to the local rustup name when needed.

## Install a packaged toolchain

The release path is a relocatable sysroot archive plus a small rustup registration script:

```sh
tools/badgevms-std/install-toolchain.sh --archive dist/badgevms-std/<archive>.tar.gz
rustup run badgevms-std cargo build --target riscv32imafc-unknown-badgevms \
	--manifest-path examples/std-hello-world/Cargo.toml
```

Archives include prebuilt BadgeVMS `std`, so normal users should not need `-Zbuild-std`. Use
`BADGEVMS_BUILD_STD=1 tools/badgevms-std/run-smoke.sh badgevms-std` only when validating packaged
`rust-src` support.

Release packages are assembled from Rust dist artifacts and include real `rustc`, `cargo`,
`rustfmt`, host std, BadgeVMS std, and `rust-src`. The packaged `cargo` entrypoint is a thin
wrapper around the dist Cargo binary that points Cargo at the sibling packaged `rustc`, which keeps
`rustup run badgevms-std cargo ...` independent of the user's ambient `PATH`.

## Maintainer packaging flow

```sh
tools/badgevms-std/dist-toolchain.sh
tools/badgevms-std/package-toolchain.sh why2025-badge-rust-toolchain/build/dist
tools/badgevms-std/install-toolchain.sh --archive dist/badgevms-std/*.tar.gz --force
tools/badgevms-std/run-smoke.sh badgevms-std examples/std-hello-world/Cargo.toml
BADGEVMS_BUILD_STD=1 tools/badgevms-std/run-smoke.sh badgevms-std examples/std-hello-world/Cargo.toml
```

## Developer stage2 shortcut

`build-toolchain.sh` and `dev-link-stage2.sh` remain local iteration shortcuts for compiler work.
They are not release packaging inputs and may patch a mutable stage2 tree for local `build-std`
testing. Packaged releases must use `dist-toolchain.sh`, `package-toolchain.sh`, and
`install-toolchain.sh` so the installed toolchain is independent of the source checkout.
