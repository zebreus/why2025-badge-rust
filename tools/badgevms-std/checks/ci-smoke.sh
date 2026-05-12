#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)

command -v cargo >/dev/null 2>&1 || { printf 'error: missing required command: cargo\n' >&2; exit 1; }
command -v rustc >/dev/null 2>&1 || { printf 'error: missing required command: rustc\n' >&2; exit 1; }

cd "$PROJECT_ROOT"

cargo fmt --check
cargo emu-check --workspace

for manifest in examples/std-*/Cargo.toml; do
    [[ -f "$manifest" ]] || continue
    cargo check --manifest-path "$manifest"
done

if rustup toolchain list 2>/dev/null | grep -q '^badgevms-std'; then
    "$PROJECT_ROOT/tools/badgevms-std/checks/verify-toolchain.sh" badgevms-std
    "$PROJECT_ROOT/tools/badgevms-std/checks/run-smoke.sh" badgevms-std examples/std-hello-world/Cargo.toml
else
    printf 'skipping BadgeVMS std toolchain verification: rustup toolchain %s is not linked\n' badgevms-std
fi
