# BadgeVMS std tooling

These scripts support the BadgeVMS std workflow in this superproject. They assume the bundled
`why2025-badge-rust-toolchain` checkout is present. See
[ADR 0005](../../docs/adr/0005-support-badgevms-std-through-the-superproject.md) for why this is
the supported entrypoint.

## Scripts

- `dist-toolchain.sh` — build Rust dist artifacts for release packaging.
- `package-toolchain.sh` — assemble a relocatable rustup-linkable toolchain archive from Rust dist artifacts.
- `install-toolchain.sh` — install a packaged archive and register it with rustup.
- `verify-toolchain.sh` — verify target cfg for `riscv32imafc-unknown-badgevms`, including no
	non-empty `target_env` such as `newlib`.
- `run-smoke.sh` — build a std example and inspect the ELF artifact.
- `inspect-elf.sh` — verify BadgeVMS shared-object shape and closed exports.
- `ci-smoke.sh` — run repository-side checks that do not require BadgeVMS hardware.

The std port uses `why2025-badge-sys-bindings` as the raw BadgeVMS ABI source. The Rust fork should
not carry a BadgeVMS-specific `library/libc` fork.

Set `BADGEVMS_TOOLCHAIN_NAME` to the local rustup name when needed.

## Install a packaged toolchain

The release path is a relocatable sysroot archive plus a small rustup registration script. From a
published release, a normal user should be able to install and build without checking out this
repository:

```sh
curl -fLO https://github.com/zebreus/why2025-badge-rust/releases/latest/download/install-toolchain.sh
bash install-toolchain.sh --version latest
rustup run badgevms-std cargo build --target riscv32imafc-unknown-badgevms \
	--manifest-path Cargo.toml
```

For a local archive, use:

```sh
tools/badgevms-std/install-toolchain.sh --archive dist/badgevms-std/<archive>.tar.gz
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

Release packaging requires a clean superproject, Rust toolchain submodule, and nested submodules.
For local experiments only, set `BADGEVMS_ALLOW_DIRTY=1` or pass `--allow-dirty` to
`dist-toolchain.sh`/`package-toolchain.sh`; dirty archives must not be published.

## Troubleshooting

- If `install-toolchain.sh --version latest` fails, check that the GitHub Release contains the
	`badgevms-std-<host>.tar.gz` alias and matching `.sha256` file for your host.
- If a build accidentally uses the ambient `rustc`, invoke Cargo through `rustup run badgevms-std`
	or set `RUSTC` to `$(rustup which --toolchain badgevms-std rustc)`.
- If `BADGEVMS_BUILD_STD=1` fails, inspect the packaged `rust-src` tree and verify
	`lib/rustlib/src/why2025-badge-sys-bindings/Cargo.toml` points at
	`../rust/library/rustc-std-workspace-core`.
