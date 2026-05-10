#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/common.sh"

need_cmd cargo
need_cmd rustc

cd "$PROJECT_ROOT"

cargo fmt --check
cargo emu-check --workspace

for manifest in examples/std-*/Cargo.toml; do
    [[ -f "$manifest" ]] || continue
    cargo check --manifest-path "$manifest"
done

if rustup toolchain list 2>/dev/null | grep -q "^$BADGEVMS_TOOLCHAIN_NAME"; then
    "$PROJECT_ROOT/tools/badgevms-std/verify-toolchain.sh" "$BADGEVMS_TOOLCHAIN_NAME"
    "$PROJECT_ROOT/tools/badgevms-std/run-smoke.sh" "$BADGEVMS_TOOLCHAIN_NAME" examples/std-hello-world/Cargo.toml
else
    printf 'skipping BadgeVMS std toolchain verification: rustup toolchain %s is not linked\n' "$BADGEVMS_TOOLCHAIN_NAME"
fi
