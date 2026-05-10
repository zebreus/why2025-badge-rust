#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/common.sh"

need_cmd rustup

stage2=${1:-}
toolchain=${2:-$BADGEVMS_TOOLCHAIN_NAME}

[[ -n "$stage2" ]] || fail "usage: $0 /path/to/rust/build/<host>/stage2 [toolchain-name]"
[[ -x "$stage2/bin/rustc" ]] || fail "stage2 dir does not contain bin/rustc: $stage2"

rustup toolchain link "$toolchain" "$stage2"

printf 'linked toolchain %s -> %s\n' "$toolchain" "$stage2"
"$PROJECT_ROOT/tools/badgevms-std/verify-toolchain.sh" "$toolchain"
