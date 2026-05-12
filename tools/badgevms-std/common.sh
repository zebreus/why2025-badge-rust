#!/usr/bin/env bash
set -euo pipefail

BADGEVMS_STD_TARGET=${BADGEVMS_STD_TARGET:-riscv32imafc-unknown-badgevms}
BADGEVMS_TOOLCHAIN_NAME=${BADGEVMS_TOOLCHAIN_NAME:-badgevms-std}
PROJECT_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)

fail() {
    printf 'error: %s\n' "$*" >&2
    exit 1
}

need_cmd() {
    command -v "$1" >/dev/null 2>&1 || fail "missing required command: $1"
}

host_triple_from_rustc() {
    local rustc=${1:-rustc}
    "$rustc" -vV | sed -n 's/^host: //p'
}

rustc_toolchain() {
    local toolchain=$1
    shift
    rustup run "$toolchain" rustc "$@"
}

cargo_toolchain() {
    local toolchain=$1
    shift
    local rustc
    rustc=$(rustup which --toolchain "$toolchain" rustc)
    RUSTC="$rustc" rustup run "$toolchain" cargo "$@"
}

rust_repo() {
    local submodule="$PROJECT_ROOT/why2025-badge-rust-toolchain"
    if [[ -d "$submodule/.git" || -f "$submodule/.git" ]]; then
        printf '%s\n' "$submodule"
        return
    fi

    fail 'initialize why2025-badge-rust-toolchain'
}
