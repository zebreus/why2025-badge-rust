#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../common.sh"

need_cmd readelf

artifact=${1:-}
[[ -n "$artifact" ]] || fail "usage: $0 /path/to/app.elf"
[[ -f "$artifact" ]] || fail "artifact does not exist: $artifact"

header=$(readelf -h "$artifact")
printf '%s\n' "$header" | grep -q 'Type:.*DYN' || fail "artifact is not an ELF shared object (ET_DYN)"
printf '%s\n' "$header" | grep -q 'Machine:.*RISC-V' || fail "artifact is not RISC-V"

symbols=$(readelf -W --dyn-syms "$artifact")
defined_funcs=$(printf '%s\n' "$symbols" | awk '$4 == "FUNC" && $7 != "UND" { print $8 }')
grep -qx 'main' <<<"$defined_funcs" || fail "exported/defined main symbol not found"

unexpected_exports=$(printf '%s\n' "$defined_funcs" | grep -v '^main$' || true)
if [[ -n "$unexpected_exports" ]]; then
    printf 'unexpected global function exports:\n%s\n' "$unexpected_exports" >&2
    fail "closed export pruning failed"
fi

printf 'BadgeVMS ELF inspection passed: %s\n' "$artifact"
