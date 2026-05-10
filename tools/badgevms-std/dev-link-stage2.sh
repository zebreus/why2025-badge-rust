#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/common.sh"

cat >&2 <<'NOTE'
warning: dev-link-stage2.sh is a local developer shortcut only.
Release toolchains must be built with dist-toolchain.sh and package-toolchain.sh.
NOTE

need_cmd rustup

stage2=${1:-}
toolchain=${2:-$BADGEVMS_TOOLCHAIN_NAME}

[[ -n "$stage2" ]] || fail "usage: $0 /path/to/rust/build/<host>/stage2 [toolchain-name]"
[[ -x "$stage2/bin/rustc" ]] || fail "stage2 dir does not contain bin/rustc: $stage2"
stage2=$(cd "$stage2" && pwd)
host=$(host_triple_from_rustc "$stage2/bin/rustc")
[[ -n "$host" ]] || fail "could not determine host triple from $stage2/bin/rustc"

host_rustlib="$stage2/lib/rustlib/$host"
if ! find "$host_rustlib/lib" -maxdepth 1 -name 'libstd-*.rlib' -print -quit 2>/dev/null | grep -q .; then
    stage1="$(dirname "$stage2")/stage1"
    stage1_host_rustlib="$stage1/lib/rustlib/$host"
    [[ -d "$stage1_host_rustlib/lib" ]] || fail "stage2 host std is missing and stage1 host std was not found at $stage1_host_rustlib"
    mkdir -p "$host_rustlib/lib"
    cp -a "$stage1_host_rustlib/lib/." "$host_rustlib/lib/"
fi

cat > "$stage2/bin/cargo" <<'WRAPPER'
#!/usr/bin/env bash
set -euo pipefail
toolchain_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
export RUSTC="$toolchain_dir/bin/rustc"
for toolchain in ${BADGEVMS_CARGO_FALLBACK_TOOLCHAINS:-nightly stable beta}; do
    cargo=$(rustup which --toolchain "$toolchain" cargo 2>/dev/null || true)
    if [[ -x "$cargo" && "$cargo" != "$toolchain_dir/bin/cargo" ]]; then
        exec "$cargo" "$@"
    fi
done
printf 'error: no fallback Cargo found; install one with `rustup toolchain install stable`\n' >&2
exit 1
WRAPPER
chmod +x "$stage2/bin/cargo"

src_root="$stage2/lib/rustlib/src"
[[ -d "$src_root/rust/library/std" ]] || fail "stage2 dir does not contain rust-src at $src_root/rust"

bindings_dir="$src_root/why2025-badge-sys-bindings"
bindings_manifest="$PROJECT_ROOT/why2025-badge-sys-bindings/Cargo.toml"
[[ -f "$bindings_manifest" ]] || fail "missing why2025-badge-sys-bindings manifest"
rm -rf "$bindings_dir"
mkdir -p "$bindings_dir"
ln -s "$PROJECT_ROOT/why2025-badge-sys-bindings/src" "$bindings_dir/src"
sed 's#path = "../why2025-badge-rust-toolchain/library/rustc-std-workspace-core"#path = "../rust/library/rustc-std-workspace-core"#' \
    "$bindings_manifest" > "$bindings_dir/Cargo.toml"

rustup toolchain link "$toolchain" "$stage2"

printf 'linked developer toolchain %s -> %s\n' "$toolchain" "$stage2"
"$PROJECT_ROOT/tools/badgevms-std/verify-toolchain.sh" "$toolchain"
