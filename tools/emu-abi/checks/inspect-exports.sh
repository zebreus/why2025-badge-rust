#!/usr/bin/env bash
set -euo pipefail

artifact="${1:-target/x86_64-unknown-linux-gnu/debug/libwhy2025_badge_emu_abi.so}"
report="${2:-target/x86_64-unknown-linux-gnu/debug/why2025-badge-emu-abi-symbol-report.txt}"

if [[ ! -f "$artifact" ]]; then
    echo "error: emu-abi cdylib not found: $artifact" >&2
    echo "hint: run cargo emu-abi-build first" >&2
    exit 1
fi

script_dir="$(cd -- "$(dirname -- "$0")" && pwd)"
bash "$script_dir/report-symbol-coverage.sh" "$artifact" "$report"

if ! grep -Eq '^missing_symbols:[[:space:]]+0$' "$report"; then
    echo "error: emu-abi is still missing exported manifest symbols" >&2
    exit 1
fi

if ! grep -Eq '^extra_symbols:[[:space:]]+0$' "$report"; then
    echo "error: emu-abi exports symbols that are not present in the BadgeVMS manifest" >&2
    exit 1
fi

echo "emu-abi export inspection passed: exact 1:1 manifest parity present in $artifact"

