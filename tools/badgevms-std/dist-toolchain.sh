#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/common.sh"

need_cmd git
need_cmd python3
need_cmd rustc

for arg in "$@"; do
    case "$arg" in
        --allow-dirty)
            export BADGEVMS_ALLOW_DIRTY=1
            ;;
        -h|--help)
            cat <<'USAGE'
usage: dist-toolchain.sh [--allow-dirty]

Build Rust dist artifacts for the BadgeVMS std toolchain release package.
Release builds require a clean superproject and submodules unless --allow-dirty
or BADGEVMS_ALLOW_DIRTY=1 is set for local experiments.
USAGE
            exit 0
            ;;
        *)
            fail "unknown argument: $arg"
            ;;
    esac
done

repo=$(rust_repo)
[[ -d "$repo/.git" || -f "$repo/.git" ]] || fail "resolved Rust checkout is not a git checkout: $repo"
[[ -x "$repo/x.py" ]] || fail "resolved Rust checkout has no executable x.py: $repo"
ensure_clean_release_tree

for submodule in library/backtrace src/llvm-project src/tools/cargo; do
    [[ -e "$repo/$submodule/.git" ]] || \
        fail "missing required Rust submodule $submodule; run: git -C $repo submodule update --init $submodule"
done

host=$(host_triple_from_rustc rustc)
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
target = ["$host", "$BADGEVMS_STD_TARGET"]
extended = true
tools = ["cargo", "rustfmt"]
cargo-native-static = true

[rust]
debug = false
incremental = false
lld = true

[target.$BADGEVMS_STD_TARGET]
# The patched built-in target owns linker flags. Keep this section available for SDK paths only.

CONFIG

cd "$repo"

[[ -f "$PROJECT_ROOT/why2025-badge-sys-bindings/Cargo.toml" ]] || \
    fail "missing canonical raw BadgeVMS ABI crate: why2025-badge-sys-bindings"

# Stage0 does not know the new built-in BadgeVMS target yet, so bootstrap's target sanity check
# must be skipped until stage1 has been built from this patched checkout.
export BOOTSTRAP_SKIP_TARGET_SANITY=1

python3 ./x.py dist --config "$config" rustc rust-std cargo rustfmt rust-src

dist="$repo/build/dist"
for pattern in \
    "rustc-*-$host.tar.*" \
    "rust-std-*-$host.tar.*" \
    "rust-std-*-$BADGEVMS_STD_TARGET.tar.*" \
    "cargo-*-$host.tar.*" \
    "rustfmt-*-$host.tar.*" \
    "rust-src-*.tar.*"; do
    find "$dist" -maxdepth 1 -type f -name "$pattern" -print -quit | grep -q . || \
        fail "dist output missing expected artifact matching $pattern in $dist"
done

printf 'built BadgeVMS dist artifacts in: %s/build/dist\n' "$repo"
printf 'package them with:\n  %s/tools/badgevms-std/package-toolchain.sh %q\n' "$PROJECT_ROOT" "$dist"
