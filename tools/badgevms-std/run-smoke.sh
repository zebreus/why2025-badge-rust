#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/common.sh"

need_cmd rustup

toolchain=${1:-$BADGEVMS_TOOLCHAIN_NAME}
manifest=${2:-examples/std-hello-world/Cargo.toml}
package_dir=$(cd "$(dirname "$manifest")" && pwd)
package_name=$(sed -n 's/^name *= *"\(.*\)"/\1/p' "$manifest" | head -n1)
[[ -n "$package_name" ]] || fail "could not read package name from $manifest"

args=(
    build
    --manifest-path "$manifest"
    --target "$BADGEVMS_STD_TARGET"
)

if [[ ${BADGEVMS_BUILD_STD:-0} == 1 ]]; then
    args+=("-Zbuild-std=core,alloc,std,panic_abort,compiler_builtins")
fi

cargo_toolchain "$toolchain" "${args[@]}"

artifact="$package_dir/target/$BADGEVMS_STD_TARGET/debug/$package_name"
if [[ ! -f "$artifact" ]]; then
    artifact="$PROJECT_ROOT/target/$BADGEVMS_STD_TARGET/debug/$package_name"
fi
[[ -f "$artifact" ]] || fail "could not locate built artifact for $package_name"

"$PROJECT_ROOT/tools/badgevms-std/inspect-elf.sh" "$artifact"
