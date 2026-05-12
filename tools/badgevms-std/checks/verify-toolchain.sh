#!/usr/bin/env bash
set -euo pipefail

command -v rustc >/dev/null 2>&1 || { printf 'error: missing required command: rustc\n' >&2; exit 1; }
command -v cargo >/dev/null 2>&1 || { printf 'error: missing required command: cargo\n' >&2; exit 1; }
command -v rustup >/dev/null 2>&1 || { printf 'error: missing required command: rustup\n' >&2; exit 1; }

toolchain=${1:-badgevms-std}

rustup run "$toolchain" rustc -Vv

cfg=$(rustup run "$toolchain" rustc --print cfg --target "riscv32imafc-unknown-badgevms" | sort)
printf '%s\n' "$cfg" | grep -qx 'target_os="badgevms"' || { printf 'error: target cfg missing target_os="badgevms"\n' >&2; exit 1; }
printf '%s\n' "$cfg" | grep -qx 'target_family="unix"' || { printf 'error: target cfg missing target_family="unix"\n' >&2; exit 1; }
printf '%s\n' "$cfg" | grep -qx 'target_arch="riscv32"' || { printf 'error: target cfg missing target_arch="riscv32"\n' >&2; exit 1; }
printf '%s\n' "$cfg" | grep -qx 'target_pointer_width="32"' || { printf 'error: target cfg missing 32-bit pointer width\n' >&2; exit 1; }
if printf '%s\n' "$cfg" | grep -Eq '^target_env=".+"$'; then
	printf 'error: BadgeVMS target must not set a non-empty target_env; expected no newlib/libc environment cfg\n' >&2
	exit 1
fi

printf 'BadgeVMS std target cfg looks correct for %s.\n' 'riscv32imafc-unknown-badgevms'
printf '%s\n' "$cfg" | grep -E 'target_(arch|family|os|pointer_width)|panic' || true
