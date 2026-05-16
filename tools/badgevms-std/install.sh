#!/usr/bin/env bash
set -euo pipefail

command -v rustup >/dev/null 2>&1 || { echo You need rustup. Install it with 'curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh' ; exit 1; }

export RUSTUP_DIST_SERVER="https://zebreus.github.io/why2025-badge-rust"

rustup toolchain install "nightly-2099-01-01" --profile minimal
rustup target add "riscv32imafc-unknown-badgevms" --toolchain "nightly-2099-01-01"
rustup component add rust-src --toolchain "nightly-2099-01-01"

rm -rf "$(rustup show home)/toolchains/badgevms"
rustup toolchain link badgevms "$(dirname "$(dirname "$(rustup which --toolchain "nightly-2099-01-01" rustc)")")"