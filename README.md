# why2025-badge-rust

Various support packages for using rust on the WHY2025 badge

## Install std

Install `rustup`, then install the BadgeVMS Rust toolchain from this repository's GitHub Pages
rustup dist server. The first published channel is a nightly-only toolchain exposed as
`nightly-2099-01-01`; the future date keeps it separate from official Rust nightly toolchains in a
normal rustup installation.

```sh
# Install rustup if you don't have it already
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
. "$HOME/.cargo/env"

# Install the BadgeVMS Rust toolchain and target from GitHub Pages
export RUSTUP_DIST_SERVER=https://zebreus.github.io/why2025-badge-rust
rustup toolchain install nightly-2099-01-01 --profile minimal
rustup target add riscv32imafc-unknown-badgevms --toolchain nightly-2099-01-01
rustup component add rust-src --toolchain nightly-2099-01-01

# Create and build a crate using the BadgeVMS std target
cargo new --bin hello-badgevms
cd hello-badgevms
cargo +nightly-2099-01-01 build --target riscv32imafc-unknown-badgevms
```
