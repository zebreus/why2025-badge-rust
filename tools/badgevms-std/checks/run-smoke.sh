#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)

command -v rustup >/dev/null 2>&1 || { printf 'error: missing required command: rustup\n' >&2; exit 1; }

toolchain=${1:-${BADGEVMS_DIST_TOOLCHAIN:-nightly-2099-01-01}}
manifest=${2:-examples/hello-world/Cargo.toml}
target=${BADGEVMS_TARGET:-riscv32imafc-unknown-badgevms}
package_dir=$(cd "$(dirname "$manifest")" && pwd)
package_name=$(sed -n 's/^name *= *"\(.*\)"/\1/p' "$manifest" | head -n1)
[[ -n "$package_name" ]] || { printf 'error: could not read package name from %s\n' "$manifest" >&2; exit 1; }

args=(
    build
    --manifest-path "$manifest"
    --target "$target"
)

if [[ ${BADGEVMS_BUILD_STD:-0} == 1 ]]; then
    args+=("-Zbuild-std=core,alloc,std,panic_abort,compiler_builtins")
fi

toolchain_rustc=$(rustup which --toolchain "$toolchain" rustc)
RUSTC="$toolchain_rustc" rustup run "$toolchain" cargo "${args[@]}"

artifact="$package_dir/target/$target/debug/$package_name"
if [[ ! -f "$artifact" ]]; then
    artifact="$PROJECT_ROOT/target/$target/debug/$package_name"
fi
[[ -f "$artifact" ]] || { printf 'error: could not locate built artifact for %s\n' "$package_name" >&2; exit 1; }

"$PROJECT_ROOT/tools/badgevms-std/checks/inspect-elf.sh" "$artifact"
