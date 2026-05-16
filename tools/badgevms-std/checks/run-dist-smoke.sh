#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)

dist_server=${1:?usage: run-dist-smoke.sh DIST_SERVER_ROOT [TOOLCHAIN]}
toolchain=${2:-${BADGEVMS_DIST_TOOLCHAIN:-nightly-2099-01-01}}
target=${BADGEVMS_TARGET:-riscv32imafc-unknown-badgevms}

command -v rustup >/dev/null 2>&1 || { printf 'error: missing required command: rustup\n' >&2; exit 1; }
command -v file >/dev/null 2>&1 || { printf 'error: missing required command: file\n' >&2; exit 1; }

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

export RUSTUP_HOME="$tmp/rustup"
export CARGO_HOME="$tmp/cargo"
export PATH="$CARGO_HOME/bin:$PATH"
export RUSTUP_DIST_SERVER="$dist_server"
export RUSTUP_TERM_PROGRESS_WHEN=never

rustup toolchain install "$toolchain" --profile minimal
rustup target add "$target" --toolchain "$toolchain"
rustup component add rust-src --toolchain "$toolchain"

"$PROJECT_ROOT/tools/badgevms-std/checks/verify-toolchain.sh" "$toolchain"

export RUSTC
RUSTC=$(rustup which --toolchain "$toolchain" rustc)

rustup run "$toolchain" cargo new --bin "$tmp/hello-badgevms" >/dev/null
cd "$tmp/hello-badgevms"
rustup run "$toolchain" cargo build --target "$target"

artifact="target/$target/debug/hello-badgevms"
[[ -f "$artifact" ]] || { printf 'error: missing built artifact: %s\n' "$artifact" >&2; exit 1; }
file "$artifact"
"$PROJECT_ROOT/tools/badgevms-std/checks/inspect-elf.sh" "$artifact"