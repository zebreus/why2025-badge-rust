#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)

usage() {
    cat <<'USAGE'
usage: dist-toolchain.sh [out-dir] [version]

Build and package a rustup-linkable BadgeVMS std toolchain with Rust's standard
`x.py install` flow. The archive contains an installed Rust prefix with rustc,
host std, BadgeVMS std, cargo, rustfmt, and the standard `rust-src` component.
USAGE
}

command -v python3 >/dev/null 2>&1 || { printf 'error: missing required command: python3\n' >&2; exit 1; }
command -v rustc >/dev/null 2>&1 || { printf 'error: missing required command: rustc\n' >&2; exit 1; }
command -v git >/dev/null 2>&1 || { printf 'error: missing required command: git\n' >&2; exit 1; }
command -v tar >/dev/null 2>&1 || { printf 'error: missing required command: tar\n' >&2; exit 1; }
command -v sha256sum >/dev/null 2>&1 || { printf 'error: missing required command: sha256sum\n' >&2; exit 1; }

positionals=()
for arg in "$@"; do
    case "$arg" in
        -h|--help)
            usage
            exit 0
            ;;
        --)
            shift
            while [[ $# -gt 0 ]]; do
                positionals+=("$1")
                shift
            done
            break
            ;;
        --*)
            printf 'error: unknown argument: %s\n' "$arg" >&2
            exit 1
            ;;
        *)
            positionals+=("$arg")
            ;;
    esac
    shift || true
done

repo="$PROJECT_ROOT/why2025-badge-rust-toolchain"
[[ -d "$repo/.git" || -f "$repo/.git" ]] || { printf 'error: initialize why2025-badge-rust-toolchain\n' >&2; exit 1; }
[[ -x "$repo/x.py" ]] || { printf 'error: resolved Rust checkout has no executable x.py: %s\n' "$repo" >&2; exit 1; }

for submodule in library/backtrace src/llvm-project src/tools/cargo; do
    [[ -e "$repo/$submodule/.git" ]] || \
        { printf 'error: missing required Rust submodule %s; run: git -C %s submodule update --init %s\n' "$submodule" "$repo" "$submodule" >&2; exit 1; }
done

host=$(rustc -vV | sed -n 's/^host: //p')
out_dir=${positionals[0]:-$PROJECT_ROOT/dist/badgevms-std}
version=${positionals[1]:-${BADGEVMS_TOOLCHAIN_VERSION:-}}

config="$repo/build/badgevms-dist/config.toml"
mkdir -p "$(dirname "$config")"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
image="$tmp/toolchain"

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

[install]
prefix = "$image"
sysconfdir = "etc"

[target.riscv32imafc-unknown-badgevms]
# The patched built-in target owns linker flags. Keep this section available for SDK paths only.

CONFIG

cd "$repo"

[[ -f "$PROJECT_ROOT/why2025-badge-sys-bindings/Cargo.toml" ]] || \
    { printf 'error: missing canonical raw BadgeVMS ABI crate: why2025-badge-sys-bindings\n' >&2; exit 1; }

python3 ./x.py install --config "$config" compiler/rustc library/std cargo rustfmt
python3 ./x.py dist --config "$config" rust-src

rust_src_artifact=$(find "$repo/build/dist" -maxdepth 1 -type f -name 'rust-src-*.tar.*' | sort | tail -n1)
[[ -n "$rust_src_artifact" ]] || { printf 'error: dist output missing rust-src artifact in %s\n' "$repo/build/dist" >&2; exit 1; }
rust_src_extract="$tmp/rust-src"
mkdir -p "$rust_src_extract"
tar -C "$rust_src_extract" -xf "$rust_src_artifact"
rust_src_top=$(find "$rust_src_extract" -mindepth 1 -maxdepth 1 -type d -print)
if [[ $(printf '%s\n' "$rust_src_top" | sed '/^$/d' | wc -l) -ne 1 ]]; then
    printf 'error: rust-src artifact must contain exactly one top-level directory: %s\n' "$rust_src_artifact" >&2
    exit 1
fi
rust_src_install="$rust_src_top/install.sh"
[[ -x "$rust_src_install" ]] || { printf 'error: rust-src artifact has no executable install.sh: %s\n' "$rust_src_artifact" >&2; exit 1; }
"$rust_src_install" --prefix="$image" --disable-ldconfig >/dev/null

mv "$image/bin/cargo" "$image/bin/cargo-real"
cat > "$image/bin/cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

bin_dir=$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
if [[ -z ${RUSTC:-} ]]; then
    export RUSTC="$bin_dir/rustc"
fi
exec "$bin_dir/cargo-real" "$@"
EOF
chmod +x "$image/bin/cargo"

[[ -x "$image/bin/rustc" ]] || { printf 'error: installed toolchain is missing bin/rustc\n' >&2; exit 1; }
[[ -x "$image/bin/cargo" ]] || { printf 'error: installed toolchain is missing bin/cargo\n' >&2; exit 1; }
[[ -x "$image/bin/cargo-real" ]] || { printf 'error: installed toolchain is missing bin/cargo-real\n' >&2; exit 1; }
[[ -x "$image/bin/rustfmt" ]] || { printf 'error: installed toolchain is missing bin/rustfmt\n' >&2; exit 1; }
find "$image/lib/rustlib/$host/lib" -maxdepth 1 -name 'libstd-*.rlib' -print -quit 2>/dev/null | grep -q . || \
    { printf 'error: installed toolchain is missing host std\n' >&2; exit 1; }
find "$image/lib/rustlib/riscv32imafc-unknown-badgevms/lib" -maxdepth 1 -name 'libstd-*.rlib' -print -quit 2>/dev/null | grep -q . || \
    { printf 'error: installed toolchain is missing BadgeVMS std\n' >&2; exit 1; }
[[ -f "$image/lib/rustlib/src/why2025-badge-sys-bindings/Cargo.toml" ]] || \
    { printf 'error: installed rust-src is missing why2025-badge-sys-bindings\n' >&2; exit 1; }

cfg=$("$image/bin/rustc" --target "riscv32imafc-unknown-badgevms" --print cfg | sort)
printf '%s\n' "$cfg" | grep -qx 'target_os="badgevms"' || { printf 'error: installed rustc target cfg missing target_os="badgevms"\n' >&2; exit 1; }
printf '%s\n' "$cfg" | grep -qx 'target_family="unix"' || { printf 'error: installed rustc target cfg missing target_family="unix"\n' >&2; exit 1; }
if printf '%s\n' "$cfg" | grep -Eq '^target_env=".+"$'; then
    printf 'error: installed BadgeVMS target must not set a non-empty target_env\n' >&2
    exit 1
fi

if [[ -z "$version" ]]; then
    rust_version=$("$image/bin/rustc" -V | awk '{ print $2 }')
    git_rev=$(git -C "$PROJECT_ROOT" rev-parse --short HEAD 2>/dev/null || printf 'unknown')
    version="$rust_version-$git_rev"
fi

source_repo=$(git -C "$PROJECT_ROOT" config --get remote.origin.url 2>/dev/null || true)
source_rev=$(git -C "$PROJECT_ROOT" rev-parse HEAD 2>/dev/null || printf 'unknown')
rust_repo_url=$(git -C "$repo" config --get remote.origin.url 2>/dev/null || true)
rust_rev=$(git -C "$repo" rev-parse HEAD 2>/dev/null || printf 'unknown')
built_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)
rustc_release=$("$image/bin/rustc" -V | awk '{ print $2 }')

cat > "$image/badgevms-toolchain.env" <<EOF
BADGEVMS_TOOLCHAIN_VERSION=$version
BADGEVMS_TOOLCHAIN_HOST=$host
BADGEVMS_STD_TARGET=riscv32imafc-unknown-badgevms
BADGEVMS_SOURCE_REPO=$source_repo
BADGEVMS_SOURCE_REV=$source_rev
BADGEVMS_RUST_REPO=$rust_repo_url
BADGEVMS_RUST_REV=$rust_rev
BADGEVMS_BUILT_AT=$built_at
EOF

VERSION="$version" \
HOST="$host" \
TARGET="riscv32imafc-unknown-badgevms" \
BUILT_AT="$built_at" \
RUSTC_RELEASE="$rustc_release" \
SOURCE_REPO="$source_repo" \
SOURCE_REV="$source_rev" \
RUST_REPO="$rust_repo_url" \
RUST_REV="$rust_rev" \
python3 - "$image/badgevms-toolchain.json" <<'PY'
import json
import os
import sys

data = {
    "version": os.environ["VERSION"],
    "host": os.environ["HOST"],
    "target": os.environ["TARGET"],
    "built_at": os.environ["BUILT_AT"],
    "rustc_release": os.environ["RUSTC_RELEASE"],
    "source": {
        "repo": os.environ["SOURCE_REPO"],
        "rev": os.environ["SOURCE_REV"],
    },
    "rust_toolchain": {
        "repo": os.environ["RUST_REPO"],
        "rev": os.environ["RUST_REV"],
    },
}
with open(sys.argv[1], "w", encoding="utf-8") as f:
    json.dump(data, f, indent=4)
    f.write("\n")
PY

mkdir -p "$out_dir"
out_dir=$(cd "$out_dir" && pwd)
package="badgevms-std-$version-$host"
package_root="$tmp/$package"
mv "$image" "$package_root"

archive="$out_dir/$package.tar.gz"
metadata="$out_dir/$package.json"
cp "$package_root/badgevms-toolchain.json" "$metadata"
tar -C "$tmp" -czf "$archive" "$package"
(cd "$out_dir" && sha256sum "$(basename "$archive")" > "$(basename "$archive").sha256")

printf 'packaged BadgeVMS toolchain with x.py install: %s\n' "$archive"
printf 'checksum: %s.sha256\n' "$archive"
printf 'metadata: %s\n' "$metadata"
