#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/common.sh"

need_cmd rustc
need_cmd cargo
need_cmd rustup

toolchain=${1:-$BADGEVMS_TOOLCHAIN_NAME}

rustc_toolchain "$toolchain" -Vv

cfg=$(rustc_toolchain "$toolchain" --print cfg --target "$BADGEVMS_STD_TARGET" | sort)
printf '%s\n' "$cfg" | grep -qx 'target_os="badgevms"' || fail "target cfg missing target_os=\"badgevms\""
printf '%s\n' "$cfg" | grep -qx 'target_family="unix"' || fail "target cfg missing target_family=\"unix\""
printf '%s\n' "$cfg" | grep -qx 'target_arch="riscv32"' || fail "target cfg missing target_arch=\"riscv32\""
printf '%s\n' "$cfg" | grep -qx 'target_pointer_width="32"' || fail "target cfg missing 32-bit pointer width"

printf 'BadgeVMS std target cfg looks correct for %s.\n' "$BADGEVMS_STD_TARGET"
print_target_cfg_summary "$toolchain"
