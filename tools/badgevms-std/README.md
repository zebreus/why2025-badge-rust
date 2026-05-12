# BadgeVMS std tooling

These scripts support the BadgeVMS std workflow in this superproject. They assume the bundled
`why2025-badge-rust-toolchain` checkout is present. See
[ADR 0005](../../docs/adr/0005-support-badgevms-std-through-the-superproject.md) for why this is
the supported entrypoint.

## Files

- `dist-toolchain.sh` — build a rustup-linkable archive with Rust's standard `x.py install` flow
  plus the standard `rust-src` dist component installer.
- `install-toolchain.sh` — install a packaged archive and register it with rustup.

## Checks

- `checks/verify-toolchain.sh` — verify target cfg for `riscv32imafc-unknown-badgevms`, including
  no non-empty `target_env` such as `newlib`.
- `checks/run-smoke.sh` — build a std example and inspect the ELF artifact.
- `checks/inspect-elf.sh` — verify BadgeVMS shared-object shape and closed exports.
- `checks/ci-smoke.sh` — run repository-side checks that do not require BadgeVMS hardware.

The std port uses `why2025-badge-sys-bindings` as the raw BadgeVMS ABI source. The Rust fork should
not carry a BadgeVMS-specific `library/libc` fork.

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
`BADGEVMS_BUILD_STD=1 tools/badgevms-std/checks/run-smoke.sh badgevms-std` only when validating
packaged `rust-src` support.

Release packages are archived Rust install prefixes produced by `x.py install`, with `rust-src`
added from Rust's own dist component installer. They include real `rustc`, Cargo, `rustfmt`, host
std, BadgeVMS std, and `rust-src`. The packaged Cargo entrypoint sets `RUSTC` to the sibling
packaged `rustc` before executing the real Cargo binary, so `rustup run badgevms-std cargo ...` does
not accidentally use the user's ambient compiler.

## Maintainer local flow

For local compiler iteration, use Rust's standard custom-toolchain path from inside the bundled Rust
checkout: build a stage sysroot with `x.py`, then point rustup at it with `rustup toolchain link`.
The exact `x.py` command depends on what you are changing; the important bit is that rustup links a
Rust build sysroot directly instead of using the release archive path.

## Maintainer packaging flow

```sh
tools/badgevms-std/dist-toolchain.sh dist/badgevms-std
tools/badgevms-std/install-toolchain.sh --archive dist/badgevms-std/*.tar.gz --force
tools/badgevms-std/checks/run-smoke.sh badgevms-std examples/std-hello-world/Cargo.toml
BADGEVMS_BUILD_STD=1 tools/badgevms-std/checks/run-smoke.sh badgevms-std examples/std-hello-world/Cargo.toml
```

Real release artifacts are produced by CI. Local maintainer runs of `dist-toolchain.sh` are allowed
on a dirty checkout; the script no longer gates that case.

## Troubleshooting

- If `install-toolchain.sh --version latest` fails, check that the GitHub Release contains the
  `badgevms-std-<host>.tar.gz` alias and matching `.sha256` file for your host.
- If a build accidentally uses the ambient `rustc`, invoke Cargo through `rustup run badgevms-std`
  or set `RUSTC` to `$(rustup which --toolchain badgevms-std rustc)`.
- If `BADGEVMS_BUILD_STD=1` fails, inspect the packaged `rust-src` tree and verify
  `lib/rustlib/src/why2025-badge-sys-bindings/Cargo.toml` points at
  `../rust/library/rustc-std-workspace-core`.
