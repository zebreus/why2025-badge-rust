# why2025-badge-rust

Various support packages for using rust on the WHY2025 badge

## Install std

Install `rustup`, then register the packaged `badgevms-std` toolchain from the latest GitHub release and build a test crate:

```sh
# Install rustup if you don't have it already
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
. "$HOME/.cargo/env"

# Install the latest badgevms-std toolchain from GitHub releases
curl -fLO https://github.com/zebreus/why2025-badge-rust/releases/latest/download/install-toolchain.sh
bash install-toolchain.sh

# Create and build a crate using the badgevms-std toolchain
cargo new --bin hello-badgevms
cd hello-badgevms
rustup run badgevms-std cargo build --target riscv32imafc-unknown-badgevms
```
