# why2025-badge-rust

Various support packages for using rust on the WHY2025 badge

## Install std

Install the BadgeVMS std toolchain with the public installer. It bootstraps `rustup` if needed,
installs the published BadgeVMS toolchain, adds the BadgeVMS target and `rust-src`, and links the
toolchain locally as `badgevms`.

```sh
curl -fsSL https://zebreus.github.io/why2025-badge-rust/install.sh | bash

# Create and build a crate using the BadgeVMS std target
cargo new --bin hello-badgevms
cd hello-badgevms
cargo +badgevms build --target riscv32imafc-unknown-badgevms
```
