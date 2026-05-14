#!/usr/bin/env bash
set -euo pipefail

artifact="${1:-target/x86_64-unknown-linux-gnu/debug/libwhy2025_badge_emu_abi.so}"
report="${2:-target/x86_64-unknown-linux-gnu/debug/why2025-badge-emu-abi-symbol-report.txt}"
manifest="why2025-badge-sys-bindings/firmware/badgevms/symbols.yml"

if [[ ! -f "$artifact" ]]; then
    echo "error: emu-abi cdylib not found: $artifact" >&2
    exit 1
fi

if [[ ! -f "$manifest" ]]; then
    echo "error: symbol manifest not found: $manifest" >&2
    exit 1
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

awk '
BEGIN { sec = "" }
/^[A-Za-z_][A-Za-z0-9_]*:$/ {
    key = substr($0, 1, length($0) - 1)
    if (key == "simple_function" || key == "simple_function_extern" || key == "wrapped_function" || key == "simple_object" || key == "wrapped_object") {
        sec = key
    } else {
        sec = ""
    }
    next
}
sec != "" && $1 == "-" {
    print sec "\t" $2
}
' "$manifest" | sort -u > "$tmp_dir/manifest.tsv"

cut -f2 "$tmp_dir/manifest.tsv" | sort -u > "$tmp_dir/manifest_symbols.txt"

if command -v nm >/dev/null 2>&1; then
    nm -D --defined-only "$artifact" | awk '{ print $3 }'
elif command -v llvm-nm >/dev/null 2>&1; then
    llvm-nm -D --defined-only "$artifact" | awk '{ print $3 }'
else
    echo "error: need nm or llvm-nm in PATH" >&2
    exit 1
fi | sed '/^$/d' | sort -u > "$tmp_dir/exports.txt"

comm -12 "$tmp_dir/manifest_symbols.txt" "$tmp_dir/exports.txt" > "$tmp_dir/present.txt"
comm -23 "$tmp_dir/manifest_symbols.txt" "$tmp_dir/exports.txt" > "$tmp_dir/missing.txt"
comm -13 "$tmp_dir/manifest_symbols.txt" "$tmp_dir/exports.txt" > "$tmp_dir/extra.txt"

manifest_count="$(wc -l < "$tmp_dir/manifest_symbols.txt")"
present_count="$(wc -l < "$tmp_dir/present.txt")"
missing_count="$(wc -l < "$tmp_dir/missing.txt")"
export_count="$(wc -l < "$tmp_dir/exports.txt")"
extra_count="$(wc -l < "$tmp_dir/extra.txt")"

{
    echo "artifact: $artifact"
    echo "manifest_symbols: $manifest_count"
    echo "exported_symbols: $export_count"
    echo "matching_symbols: $present_count"
    echo "missing_symbols: $missing_count"
    echo "extra_symbols: $extra_count"
    echo
    echo "per_section:"
    for sec in simple_function simple_function_extern wrapped_function simple_object wrapped_object; do
        awk -F $'\t' -v section="$sec" '$1 == section { print $2 }' "$tmp_dir/manifest.tsv" | sort -u > "$tmp_dir/$sec.txt"
        total="$(wc -l < "$tmp_dir/$sec.txt")"
        comm -23 "$tmp_dir/$sec.txt" "$tmp_dir/exports.txt" > "$tmp_dir/$sec-missing.txt"
        sec_missing="$(wc -l < "$tmp_dir/$sec-missing.txt")"
        sec_present="$((total - sec_missing))"
        echo "$sec present=$sec_present missing=$sec_missing"
    done
    echo
    echo "missing:"
    if [[ "$missing_count" -eq 0 ]]; then
        echo "none"
    else
        cat "$tmp_dir/missing.txt"
    fi
    echo
    echo "extra:"
    if [[ "$extra_count" -eq 0 ]]; then
        echo "none"
    else
        cat "$tmp_dir/extra.txt"
    fi
} > "$report"

cat "$report"