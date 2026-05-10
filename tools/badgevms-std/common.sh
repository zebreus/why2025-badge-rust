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
    if [[ -n "${BADGEVMS_RUST_REPO:-}" ]]; then
        printf '%s\n' "$BADGEVMS_RUST_REPO"
        return
    fi

    local metadata="$PROJECT_ROOT/docs/badgevms-std-target/toolchain-metadata.toml"
    if [[ -f "$metadata" ]]; then
        local value
        value=$(sed -n 's/^rust_repo *= *"\(.*\)"/\1/p' "$metadata" | head -n1)
        if [[ -n "$value" ]]; then
            printf '%s\n' "$value"
            return
        fi
    fi

    local submodule="$PROJECT_ROOT/why2025-badge-rust-toolchain"
    if [[ -d "$submodule/.git" || -f "$submodule/.git" ]]; then
        printf '%s\n' "$submodule"
        return
    fi

    fail 'set BADGEVMS_RUST_REPO, initialize why2025-badge-rust-toolchain, or create docs/badgevms-std-target/toolchain-metadata.toml'
}

stage2_dir_for_repo() {
    local repo=$1
    local host
    host=$(rustc -vV | sed -n 's/^host: //p')
    printf '%s/build/%s/stage2\n' "$repo" "$host"
}

print_target_cfg_summary() {
    local toolchain=$1
    rustc_toolchain "$toolchain" --print cfg --target "$BADGEVMS_STD_TARGET" | sort | grep -E 'target_(arch|family|os|pointer_width)|panic' || true
}
