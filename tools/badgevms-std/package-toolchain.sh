#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/common.sh"

usage() {
    cat <<'USAGE'
usage: package-toolchain.sh [dist-dir] [out-dir] [version]

Assembles a relocatable, rustup-linkable BadgeVMS toolchain archive from Rust dist artifacts.
The input directory must be produced by dist-toolchain.sh / x.py dist and contain rustc, host
rust-std, BadgeVMS rust-std, cargo, rustfmt, and rust-src component tarballs.
USAGE
}

if [[ ${1:-} == "-h" || ${1:-} == "--help" ]]; then
    usage
    exit 0
fi

need_cmd tar
need_cmd sha256sum

repo=$(rust_repo)
dist_dir=${1:-$repo/build/dist}
out_dir=${2:-$PROJECT_ROOT/dist/badgevms-std}
version=${3:-${BADGEVMS_TOOLCHAIN_VERSION:-}}
host=${BADGEVMS_TOOLCHAIN_HOST:-$(host_triple_from_rustc rustc)}

[[ -d "$dist_dir" ]] || fail "dist directory does not exist: $dist_dir"
dist_dir=$(cd "$dist_dir" && pwd)

find_artifact() {
    local component=$1
    local triple=${2:-}
    local patterns=()

    if [[ -n "$triple" ]]; then
        patterns=("$component-*-$triple.tar.xz" "$component-*-$triple.tar.gz")
    else
        patterns=("$component-*.tar.xz" "$component-*.tar.gz")
    fi

    local pattern artifact
    for pattern in "${patterns[@]}"; do
        artifact=$(find "$dist_dir" -maxdepth 1 -type f -name "$pattern" | sort | tail -n1)
        if [[ -n "$artifact" ]]; then
            printf '%s\n' "$artifact"
            return 0
        fi
    done

    fail "missing dist artifact for $component ${triple:-targetless} in $dist_dir"
}

install_component() {
    local artifact=$1
    local work=$2
    local prefix=$3
    local top install_script

    rm -rf "$work"
    mkdir -p "$work"
    tar -C "$work" -xf "$artifact"

    top=$(find "$work" -mindepth 1 -maxdepth 1 -type d -print)
    if [[ $(printf '%s\n' "$top" | sed '/^$/d' | wc -l) -ne 1 ]]; then
        fail "dist artifact must contain exactly one top-level directory: $artifact"
    fi

    install_script="$top/install.sh"
    [[ -x "$install_script" ]] || fail "dist artifact has no executable install.sh: $artifact"
    "$install_script" --prefix="$prefix" --disable-ldconfig >/dev/null
}

rustc_artifact=$(find_artifact rustc "$host")
host_std_artifact=$(find_artifact rust-std "$host")
target_std_artifact=$(find_artifact rust-std "$BADGEVMS_STD_TARGET")
cargo_artifact=$(find_artifact cargo "$host")
rustfmt_artifact=$(find_artifact rustfmt "$host")
rust_src_artifact=$(find_artifact rust-src)

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
image="$tmp/sysroot"
mkdir -p "$image"

for artifact in \
    "$rustc_artifact" \
    "$host_std_artifact" \
    "$target_std_artifact" \
    "$cargo_artifact" \
    "$rustfmt_artifact" \
    "$rust_src_artifact"; do
    install_component "$artifact" "$tmp/extract" "$image"
done

mv "$image/bin/cargo" "$image/bin/cargo-real"
cat > "$image/bin/cargo" <<'WRAPPER'
#!/usr/bin/env bash
set -euo pipefail
toolchain_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
export RUSTC=${RUSTC:-$toolchain_dir/bin/rustc}
exec "$toolchain_dir/bin/cargo-real" "$@"
WRAPPER
chmod +x "$image/bin/cargo"

[[ -x "$image/bin/rustc" ]] || fail "assembled toolchain is missing bin/rustc"
[[ -x "$image/bin/cargo" ]] || fail "assembled toolchain is missing bin/cargo"
[[ -x "$image/bin/cargo-real" ]] || fail "assembled toolchain is missing bin/cargo-real"
[[ -x "$image/bin/rustfmt" ]] || fail "assembled toolchain is missing bin/rustfmt"
find "$image/lib/rustlib/$host/lib" -maxdepth 1 -name 'libstd-*.rlib' -print -quit 2>/dev/null | grep -q . || \
    fail "assembled toolchain is missing host std"
find "$image/lib/rustlib/$BADGEVMS_STD_TARGET/lib" -maxdepth 1 -name 'libstd-*.rlib' -print -quit 2>/dev/null | grep -q . || \
    fail "assembled toolchain is missing BadgeVMS std"
[[ -f "$image/lib/rustlib/src/why2025-badge-sys-bindings/Cargo.toml" ]] || \
    fail "assembled rust-src is missing why2025-badge-sys-bindings"
grep -q '../rust/library/rustc-std-workspace-core' \
    "$image/lib/rustlib/src/why2025-badge-sys-bindings/Cargo.toml" || \
    fail "packaged why2025-badge-sys-bindings manifest does not use installed rust-src paths"

"$image/bin/rustc" -Vv >/dev/null
"$image/bin/cargo" -V >/dev/null
"$image/bin/rustfmt" -V >/dev/null

if [[ -z "$version" ]]; then
    rust_version=$("$image/bin/rustc" -V | awk '{ print $2 }')
    git_rev=$(git -C "$PROJECT_ROOT" rev-parse --short HEAD 2>/dev/null || printf 'nogit')
    version="$rust_version-$git_rev"
    if ! git -C "$PROJECT_ROOT" diff --quiet --ignore-submodules=none 2>/dev/null; then
        version="$version-dirty"
    fi
fi

cat > "$image/badgevms-toolchain.env" <<EOF
BADGEVMS_TOOLCHAIN_VERSION=$version
BADGEVMS_TOOLCHAIN_HOST=$host
BADGEVMS_STD_TARGET=$BADGEVMS_STD_TARGET
BADGEVMS_SOURCE_REPO=$(git -C "$PROJECT_ROOT" config --get remote.origin.url 2>/dev/null || true)
BADGEVMS_SOURCE_REV=$(git -C "$PROJECT_ROOT" rev-parse HEAD 2>/dev/null || true)
EOF

while IFS= read -r link; do
    target=$(readlink "$link")
    case "$target" in
        /*)
            fail "archive would contain absolute symlink: $link -> $target"
            ;;
        *why2025-badge-rust*|*stage2*|*build*)
            fail "archive would contain workspace/build symlink: $link -> $target"
            ;;
    esac
done < <(find "$image" -type l)

if find "$image" \( -name .git -o -name .gitmodules \) -print -quit | grep -q .; then
    fail "archive would contain git metadata"
fi
checkout_refs=$(grep -RIl --exclude='badgevms-toolchain.env' \
    "$PROJECT_ROOT\|why2025-badge-rust-toolchain/build" "$image" || true)
if [[ -n "$checkout_refs" ]]; then
    printf 'source checkout or stage2 references found in assembled image:\n%s\n' \
        "$(printf '%s\n' "$checkout_refs" | head -n 20)" >&2
    fail "archive would contain source checkout or stage2 references"
fi

mkdir -p "$out_dir"
out_dir=$(cd "$out_dir" && pwd)
package="badgevms-std-$version-$host"
package_root="$tmp/$package"
mv "$image" "$package_root"

archive="$out_dir/$package.tar.gz"
tar -C "$tmp" -czf "$archive" "$package"
sha256sum "$archive" > "$archive.sha256"

printf 'packaged BadgeVMS toolchain: %s\n' "$archive"
printf 'checksum: %s.sha256\n' "$archive"
