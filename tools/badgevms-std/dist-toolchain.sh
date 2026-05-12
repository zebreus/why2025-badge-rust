#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)

command -v python3 >/dev/null 2>&1 || { printf 'error: missing required command: python3\n' >&2; exit 1; }
command -v rustc >/dev/null 2>&1 || { printf 'error: missing required command: rustc\n' >&2; exit 1; }

for arg in "$@"; do
    case "$arg" in
        -h|--help)
            cat <<'USAGE'
usage: dist-toolchain.sh

Build Rust dist artifacts for the BadgeVMS std toolchain release package.
USAGE
            exit 0
            ;;
        *)
            printf 'error: unknown argument: %s\n' "$arg" >&2
            exit 1
            ;;
    esac
done

repo="$PROJECT_ROOT/why2025-badge-rust-toolchain"
[[ -d "$repo/.git" || -f "$repo/.git" ]] || { printf 'error: initialize why2025-badge-rust-toolchain\n' >&2; exit 1; }
[[ -x "$repo/x.py" ]] || { printf 'error: resolved Rust checkout has no executable x.py: %s\n' "$repo" >&2; exit 1; }

for submodule in library/backtrace src/llvm-project src/tools/cargo; do
    [[ -e "$repo/$submodule/.git" ]] || \
        { printf 'error: missing required Rust submodule %s; run: git -C %s submodule update --init %s\n' "$submodule" "$repo" "$submodule" >&2; exit 1; }
done

host=$(rustc -vV | sed -n 's/^host: //p')
config="$repo/build/badgevms-dist/config.toml"
mkdir -p "$(dirname "$config")"

cat > "$config" <<CONFIG
profile = "compiler"
change-id = "ignore"

[llvm]
download-ci-llvm = false
ninja = true
targets = "RISCV;X86"

[build]
host = ["$host"]
target = ["$host", "riscv32imafc-unknown-badgevms"]
extended = true
tools = ["cargo", "rustfmt"]
cargo-native-static = true

[rust]
debug = false
incremental = false
lld = true

[target.riscv32imafc-unknown-badgevms]
# The patched built-in target owns linker flags. Keep this section available for SDK paths only.

CONFIG

cd "$repo"

[[ -f "$PROJECT_ROOT/why2025-badge-sys-bindings/Cargo.toml" ]] || \
    { printf 'error: missing canonical raw BadgeVMS ABI crate: why2025-badge-sys-bindings\n' >&2; exit 1; }

# Stage0 does not know the new built-in BadgeVMS target yet, so bootstrap's target sanity check
# must be skipped until stage1 has been built from this patched checkout.
export BOOTSTRAP_SKIP_TARGET_SANITY=1

python3 ./x.py dist --config "$config" rustc rust-std cargo rustfmt rust-src

dist="$repo/build/dist"
for pattern in \
    "rustc-*-$host.tar.*" \
    "rust-std-*-$host.tar.*" \
    "rust-std-*-riscv32imafc-unknown-badgevms.tar.*" \
    "cargo-*-$host.tar.*" \
    "rustfmt-*-$host.tar.*" \
    "rust-src-*.tar.*"; do
    find "$dist" -maxdepth 1 -type f -name "$pattern" -print -quit | grep -q . || \
        { printf 'error: dist output missing expected artifact matching %s in %s\n' "$pattern" "$dist" >&2; exit 1; }
done

printf 'built BadgeVMS dist artifacts in: %s/build/dist\n' "$repo"
printf 'package them with:\n  %s/tools/badgevms-std/package-toolchain.sh %q\n' "$PROJECT_ROOT" "$dist"
