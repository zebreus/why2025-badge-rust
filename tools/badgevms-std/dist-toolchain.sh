#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)

usage() {
    cat <<'USAGE'
usage: dist-toolchain.sh [out-root] [dist-base-url]

Build a rustup dist-server tree for the BadgeVMS std target.

The output root contains a top-level dist/ directory suitable for GitHub Pages.
Consumers install the toolchain with RUSTUP_DIST_SERVER=<pages-root>.

Defaults:
  out-root:      dist/badgevms-rustup
  dist-base-url: https://zebreus.github.io/why2025-badge-rust/dist

Environment:
BADGEVMS_DIST_DATE       Manifest/archive date. Default: 2099-01-01.
BADGEVMS_DIST_CHANNEL    Rust dist channel. Default: nightly.
BADGEVMS_DIST_TOOLCHAIN  Rustup toolchain users install. Default: nightly-2099-01-01.
BADGEVMS_DIST_BASE_URL   Public /dist URL used inside manifests.
BADGEVMS_TARGET          BadgeVMS target triple. Default: riscv32imafc-unknown-badgevms.
USAGE
}

command -v python3 >/dev/null 2>&1 || { printf 'error: missing required command: python3\n' >&2; exit 1; }
command -v rustc >/dev/null 2>&1 || { printf 'error: missing required command: rustc\n' >&2; exit 1; }
command -v cargo >/dev/null 2>&1 || { printf 'error: missing required command: cargo\n' >&2; exit 1; }
command -v git >/dev/null 2>&1 || { printf 'error: missing required command: git\n' >&2; exit 1; }
command -v sha256sum >/dev/null 2>&1 || { printf 'error: missing required command: sha256sum\n' >&2; exit 1; }

positionals=()
while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help) usage; exit 0 ;;
        --) shift; while [[ $# -gt 0 ]]; do positionals+=("$1"); shift; done; break ;;
        --*) printf 'error: unknown argument: %s\n' "$1" >&2; exit 1 ;;
        *) positionals+=("$1"); shift ;;
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
target=${BADGEVMS_TARGET:-riscv32imafc-unknown-badgevms}
dist_date=${BADGEVMS_DIST_DATE:-2099-01-01}
dist_channel=${BADGEVMS_DIST_CHANNEL:-nightly}
dist_toolchain=${BADGEVMS_DIST_TOOLCHAIN:-nightly-2099-01-01}
out_root=${positionals[0]:-$PROJECT_ROOT/dist/badgevms-rustup}
base_url=${positionals[1]:-${BADGEVMS_DIST_BASE_URL:-https://zebreus.github.io/why2025-badge-rust/dist}}

case "$out_root" in
    /*) ;;
    *) out_root="$PROJECT_ROOT/$out_root" ;;
esac

if [[ "$dist_channel" != nightly ]]; then
    printf 'error: only nightly dist channel is supported for BadgeVMS std for now\n' >&2
    exit 1
fi

if [[ "$dist_toolchain" != nightly-* ]]; then
    printf 'error: BADGEVMS_DIST_TOOLCHAIN must be a rustup-accepted nightly-* name, got %s\n' "$dist_toolchain" >&2
    exit 1
fi

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
target = ["$host", "$target"]
extended = true
tools = ["cargo"]
cargo-native-static = true

[rust]
channel = "nightly"
debug = false
incremental = false
lld = true

[target.$target]
# The patched built-in target owns linker flags. Keep this section available for SDK paths only.
# Avoid requiring a target C compiler for compiler-rt builtins while producing the rust-std dist
# package. Rust implementations are sufficient for this target and keep CI/consumer packaging
# self-contained.
optimized-compiler-builtins = false

CONFIG

[[ -f "$PROJECT_ROOT/why2025-badge-sys-bindings/Cargo.toml" ]] || \
    { printf 'error: missing canonical raw BadgeVMS ABI crate: why2025-badge-sys-bindings\n' >&2; exit 1; }

cd "$repo"

python3 ./x.py dist --config "$config" --target "$host" extended rust-src
python3 ./x.py dist --config "$config" --target "$target" rust-std

rust_dist="$repo/build/dist"
date_dir="$out_root/dist/$dist_date"
rm -rf "$out_root/dist"
mkdir -p "$date_dir"

copy_required_component() {
    local base=$1
    local found=0
    local artifact

    for artifact in "$rust_dist/$base".tar.{gz,xz}; do
        if [[ -f "$artifact" ]]; then
            cp "$artifact" "$date_dir/"
            found=1
        fi
    done

    [[ "$found" -eq 1 ]] || { printf 'error: missing required dist artifact matching %s.tar.{gz,xz}\n' "$base" >&2; exit 1; }
}

copy_required_component "rust-$dist_channel-$host"
copy_required_component "rustc-$dist_channel-$host"
copy_required_component "cargo-$dist_channel-$host"
copy_required_component "rust-std-$dist_channel-$host"
copy_required_component "rust-std-$dist_channel-$target"
copy_required_component "rust-src-$dist_channel"

BUILD_MANIFEST_EXTRA_TARGETS="$target" \
BUILD_MANIFEST_SHIPPED_FILES_PATH="$date_dir/shipped-files.txt" \
    cargo run --manifest-path "$repo/Cargo.toml" --release -p build-manifest -- \
    "$date_dir" "$date_dir" "$dist_date" "$base_url" "$dist_channel"

(
    cd "$date_dir"
    sha256sum "channel-rust-$dist_channel.toml" > "channel-rust-$dist_channel.toml.sha256"
)

cp "$date_dir/channel-rust-$dist_channel.toml" "$out_root/dist/channel-rust-$dist_channel.toml"
cp "$date_dir/channel-rust-$dist_channel.toml.sha256" "$out_root/dist/channel-rust-$dist_channel.toml.sha256"
for suffix in -date.txt -git-commit-hash.txt; do
    if [[ -f "$date_dir/channel-rust-$dist_channel$suffix" ]]; then
        cp "$date_dir/channel-rust-$dist_channel$suffix" "$out_root/dist/channel-rust-$dist_channel$suffix"
    fi
done

grep -q "\[pkg.rust-std.target.$target\]" "$date_dir/channel-rust-$dist_channel.toml" || {
    printf 'error: generated manifest does not contain rust-std for %s\n' "$target" >&2
    exit 1
}
grep -q "\[pkg.rust-src.target.\"\*\"\]" "$date_dir/channel-rust-$dist_channel.toml" || {
    printf 'error: generated manifest does not contain rust-src\n' >&2
    exit 1
}

cat > "$out_root/README.txt" <<EOF
BadgeVMS Rust std rustup dist tree

Install with:

  RUSTUP_DIST_SERVER=<this site root> rustup toolchain install $dist_toolchain --profile minimal
  RUSTUP_DIST_SERVER=<this site root> rustup target add $target --toolchain $dist_toolchain
  RUSTUP_DIST_SERVER=<this site root> rustup component add rust-src --toolchain $dist_toolchain

The deployed site root must contain this file and the dist/ directory next to it.
EOF

printf 'built BadgeVMS rustup dist tree: %s\n' "$out_root"
printf 'dist server root: %s\n' "${base_url%/dist}"
printf 'dist base URL: %s\n' "$base_url"
printf 'rustup toolchain: %s\n' "$dist_toolchain"
printf 'target: %s\n' "$target"
