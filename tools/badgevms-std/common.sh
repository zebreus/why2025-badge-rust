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

allow_dirty_release_tree() {
    [[ ${BADGEVMS_ALLOW_DIRTY:-0} == 1 ]]
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

git_dirty_status() {
    local repo=$1
    git -C "$repo" status --porcelain --untracked-files=all --ignore-submodules=none
}

release_tree_dirty_report() {
    local rust
    rust=$(rust_repo)

    local status
    status=$(git_dirty_status "$PROJECT_ROOT" || true)
    if [[ -n "$status" ]]; then
        printf 'superproject %s:\n%s\n' "$PROJECT_ROOT" "$status"
    fi

    status=$(git_dirty_status "$rust" || true)
    if [[ -n "$status" ]]; then
        printf 'rust toolchain %s:\n%s\n' "$rust" "$status"
    fi

    local backtrace="$rust/library/backtrace"
    if [[ -e "$backtrace/.git" ]]; then
        status=$(git_dirty_status "$backtrace" || true)
        if [[ -n "$status" ]]; then
            printf 'backtrace submodule %s:\n%s\n' "$backtrace" "$status"
        fi
    fi

    status=$(git -C "$PROJECT_ROOT" submodule status --recursive 2>/dev/null | grep -E '^[+U]' || true)
    if [[ -n "$status" ]]; then
        printf 'mismatched submodules:\n%s\n' "$status"
    fi
}

release_tree_is_dirty() {
    [[ -n "$(release_tree_dirty_report)" ]]
}

ensure_clean_release_tree() {
    need_cmd git

    if allow_dirty_release_tree; then
        printf 'warning: BADGEVMS_ALLOW_DIRTY=1; release cleanliness checks are bypassed\n' >&2
        return
    fi

    local report
    report=$(release_tree_dirty_report)
    if [[ -n "$report" ]]; then
        printf '%s\n' "$report" >&2
        fail 'release packaging requires a clean superproject and submodules; set BADGEVMS_ALLOW_DIRTY=1 only for local experiments'
    fi
}

git_short_rev() {
    local repo=$1
    git -C "$repo" rev-parse --short HEAD 2>/dev/null || printf 'unknown'
}

git_full_rev() {
    local repo=$1
    git -C "$repo" rev-parse HEAD 2>/dev/null || printf 'unknown'
}

git_origin_url() {
    local repo=$1
    git -C "$repo" config --get remote.origin.url 2>/dev/null || true
}

print_target_cfg_summary() {
    local toolchain=$1
    rustc_toolchain "$toolchain" --print cfg --target "$BADGEVMS_STD_TARGET" | sort | grep -E 'target_(arch|family|os|pointer_width)|panic' || true
}
