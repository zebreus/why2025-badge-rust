# why2025-badge-rust

Various support packages for using rust on the WHY2025 badge

## Install BadgeVMS Toolchain

Install the public BadgeVMS toolchain once, then use the same target for both std and no_std Apps.
The installer bootstraps `rustup` if needed, installs the published BadgeVMS toolchain, adds the
BadgeVMS target and `rust-src`, and links the toolchain locally as `badgevms`.

```sh
curl -fsSL https://zebreus.github.io/why2025-badge-rust/install.sh | bash

# Create and build a crate using the BadgeVMS target
cargo new --bin hello-badgevms
cd hello-badgevms
cargo +badgevms build --target riscv32imafc-unknown-badgevms

# Build one of this workspace's no_std examples on the same target
cargo +badgevms build --manifest-path examples/hello-world-no_std/Cargo.toml --target riscv32imafc-unknown-badgevms
```

The workspace no_std examples no longer need a root `build.rs` or `why2025-badge-build`; the
built-in `riscv32imafc-unknown-badgevms` target owns the BadgeVMS shared-object link behavior.

## Legacy no_std examples without our toolchain

If you need the old stock-nightly `riscv32imafc-unknown-none-elf` flow, see
`examples/hello-world-no_std-none-elf` for the `why2025-badge-build` helper path and
`examples/hello-world-no_std-none-elf-manual-linking` for a fully manual `build.rs` plus checked-in `retain.txt`.
