#!/usr/bin/env bash
set -euo pipefail

BADGEVMS_STD_TARGET=${BADGEVMS_STD_TARGET:-riscv32imafc-unknown-badgevms}
BADGEVMS_TOOLCHAIN_NAME=${BADGEVMS_TOOLCHAIN_NAME:-badgevms-std}
PROJECT_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)

rust_repo() {
    local submodule="$PROJECT_ROOT/why2025-badge-rust-toolchain"
    if [[ -d "$submodule/.git" || -f "$submodule/.git" ]]; then
        printf '%s\n' "$submodule"
        return
    fi

    printf 'error: initialize why2025-badge-rust-toolchain\n' >&2
    exit 1
}
