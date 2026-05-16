#!/usr/bin/env bash
set -euo pipefail

pages_root=${BADGEVMS_PAGES_ROOT:-https://zebreus.github.io/why2025-badge-rust}
toolchain=${BADGEVMS_DIST_TOOLCHAIN:-nightly-2099-01-01}
target=${BADGEVMS_TARGET:-riscv32imafc-unknown-badgevms}
alias_name=${BADGEVMS_ALIAS:-badgevms}

run_step() {
    local description=$1
    local log
    shift
    log=$(mktemp)
    if "$@" >"$log" 2>&1; then
        printf '%s\n' "$description"
        rm -f "$log"
        return 0
    fi

    cat "$log" >&2
    rm -f "$log"
    exit 1
}

if [[ $(uname -s) != Linux || $(uname -m) != x86_64 ]]; then
    printf 'error: BadgeVMS installer currently supports Linux x86_64 hosts only\n' >&2
    exit 1
fi

command -v curl >/dev/null 2>&1 || { printf 'error: missing required command: curl\n' >&2; exit 1; }

if ! command -v rustup >/dev/null 2>&1; then
    run_step \
        'Installed rustup.' \
        bash -c "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain none -y --profile minimal"
fi

cargo_env=${CARGO_HOME:-$HOME/.cargo}/env
if [[ -f "$cargo_env" ]]; then
    # shellcheck disable=SC1090
    . "$cargo_env"
fi

command -v rustup >/dev/null 2>&1 || { printf 'error: rustup is not available after installation\n' >&2; exit 1; }

if [[ "$alias_name" == "$toolchain" ]]; then
    printf 'error: BADGEVMS_ALIAS must differ from BADGEVMS_DIST_TOOLCHAIN\n' >&2
    exit 1
fi

pages_root=${pages_root%/}
export RUSTUP_DIST_SERVER="$pages_root"
export RUSTUP_TERM_PROGRESS_WHEN=${RUSTUP_TERM_PROGRESS_WHEN:-never}

run_step 'Installed BadgeVMS toolchain.' rustup toolchain install "$toolchain" --profile minimal
run_step 'Installed BadgeVMS target.' rustup target add "$target" --toolchain "$toolchain"
run_step 'Installed rust-src.' rustup component add rust-src --toolchain "$toolchain"

sysroot=$(dirname "$(dirname "$(rustup which --toolchain "$toolchain" rustc)")")
toolchains_dir=$(rustup show home)/toolchains
alias_path=${toolchains_dir:?}/${alias_name:?}
rm -rf "$alias_path"
run_step 'Linked badgevms alias.' rustup toolchain link "$alias_name" "$sysroot"

printf '\nBadgeVMS install complete.\n'
printf 'Use it with:\n'
printf '  cargo +%s build --target %s\n' "$alias_name" "$target"