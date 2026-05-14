#!/usr/bin/env bash
set -euo pipefail

artifact="${1:-target/x86_64-unknown-linux-gnu/debug/libwhy2025_badge_emu_abi.so}"

if [[ ! -f "$artifact" ]]; then
    echo "error: emu-abi cdylib not found: $artifact" >&2
    echo "hint: run cargo emu-abi-build first" >&2
    exit 1
fi

if command -v nm >/dev/null 2>&1; then
    symbol_list="$(nm -D --defined-only "$artifact")"
elif command -v llvm-nm >/dev/null 2>&1; then
    symbol_list="$(llvm-nm -D --defined-only "$artifact")"
else
    echo "error: need nm or llvm-nm in PATH" >&2
    exit 1
fi

required_symbols=(
    __errno
    _ctype_
    _ctype_b
    strlen
    memcmp
    memcpy
    memmove
    memset
    memchr
    bzero
    explicit_bzero
    window_create
    wifi_get_status
    curl_easy_init
    socket
)

missing=0
for symbol in "${required_symbols[@]}"; do
    if ! grep -Eq "[[:space:]]${symbol}$" <<<"$symbol_list"; then
        echo "missing required emu-abi export: $symbol" >&2
        missing=1
    fi
done

if [[ "$missing" -ne 0 ]]; then
    exit 1
fi

echo "emu-abi export inspection passed: ${#required_symbols[@]} symbols present in $artifact"
