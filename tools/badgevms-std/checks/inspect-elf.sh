#!/usr/bin/env bash
set -euo pipefail

command -v readelf >/dev/null 2>&1 || { printf 'error: missing required command: readelf\n' >&2; exit 1; }

artifact=${1:-}
[[ -n "$artifact" ]] || { printf 'error: usage: %s /path/to/app.elf\n' "$0" >&2; exit 1; }
[[ -f "$artifact" ]] || { printf 'error: artifact does not exist: %s\n' "$artifact" >&2; exit 1; }

header=$(readelf -h "$artifact")
printf '%s\n' "$header" | grep -q 'Type:.*DYN' || { printf 'error: artifact is not an ELF shared object (ET_DYN)\n' >&2; exit 1; }
printf '%s\n' "$header" | grep -q 'Machine:.*RISC-V' || { printf 'error: artifact is not RISC-V\n' >&2; exit 1; }

symbols=$(readelf -W --dyn-syms "$artifact")
defined_funcs=$(printf '%s\n' "$symbols" | awk '$4 == "FUNC" && $7 != "UND" { print $8 }')
grep -qx 'main' <<<"$defined_funcs" || { printf 'error: exported/defined main symbol not found\n' >&2; exit 1; }

unexpected_exports=$(printf '%s\n' "$defined_funcs" | grep -v '^main$' || true)
if [[ -n "$unexpected_exports" ]]; then
    printf 'unexpected global function exports:\n%s\n' "$unexpected_exports" >&2
    printf 'error: closed export pruning failed\n' >&2
    exit 1
fi

printf 'BadgeVMS ELF inspection passed: %s\n' "$artifact"
