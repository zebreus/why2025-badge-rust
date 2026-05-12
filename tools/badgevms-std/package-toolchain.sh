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

positionals=()
while [[ $# -gt 0 ]]; do
    case "$1" in
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
            ;;
        --*)
            fail "unknown argument: $1"
            ;;
        *)
            positionals+=("$1")
            shift
            ;;
    esac
done

need_cmd git
need_cmd python3
need_cmd tar
need_cmd sha256sum

repo=$(rust_repo)
dist_dir=${positionals[0]:-$repo/build/dist}
out_dir=${positionals[1]:-$PROJECT_ROOT/dist/badgevms-std}
version=${positionals[2]:-${BADGEVMS_TOOLCHAIN_VERSION:-}}
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

json_quote() {
    python3 -c 'import json, sys; print(json.dumps(sys.argv[1]))' "$1"
}

audit_runtime_dependencies() {
    [[ $(uname -s) == Linux ]] || return 0

    if ! command -v ldd >/dev/null 2>&1; then
        printf 'warning: ldd not found; skipping runtime dependency audit\n' >&2
        return 0
    fi

    local report="$image/badgevms-runtime-deps.txt"
    : > "$report"

    local tool bin deps bad_nix
    for tool in rustc cargo-real rustfmt; do
        bin="$image/bin/$tool"
        [[ -x "$bin" ]] || continue

        deps=$(LD_LIBRARY_PATH="$image/lib:$image/lib/rustlib/$host/lib:${LD_LIBRARY_PATH:-}" ldd "$bin" 2>&1 || true)
        {
            printf '## %s\n' "$tool"
            printf '%s\n\n' "$deps"
        } >> "$report"

        if printf '%s\n' "$deps" | grep -Eiq 'lib(ssl|crypto)\.so|openssl'; then
            fail "packaged $tool has a dynamic OpenSSL dependency"
        fi

        bad_nix=$(printf '%s\n' "$deps" | grep '/nix/store/' | grep -Eiv 'glibc|gcc|libz' || true)
        if [[ -n "$bad_nix" ]]; then
            printf 'unexpected Nix runtime dependencies for %s:\n%s\n' "$tool" "$bad_nix" >&2
            fail "packaged $tool depends on non-allowlisted Nix store libraries"
        fi
    done
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

cfg=$("$image/bin/rustc" --target "$BADGEVMS_STD_TARGET" --print cfg | sort)
printf '%s\n' "$cfg" | grep -qx 'target_os="badgevms"' || fail 'packaged rustc target cfg missing target_os="badgevms"'
printf '%s\n' "$cfg" | grep -qx 'target_family="unix"' || fail 'packaged rustc target cfg missing target_family="unix"'
if printf '%s\n' "$cfg" | grep -Eq '^target_env=".+"$'; then
    fail 'packaged BadgeVMS target must not set a non-empty target_env'
fi

audit_runtime_dependencies

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
BADGEVMS_STD_TARGET=$BADGEVMS_STD_TARGET
BADGEVMS_SOURCE_REPO=$source_repo
BADGEVMS_SOURCE_REV=$source_rev
BADGEVMS_RUST_REPO=$rust_repo_url
BADGEVMS_RUST_REV=$rust_rev
BADGEVMS_BUILT_AT=$built_at
EOF

cat > "$image/badgevms-toolchain.json" <<EOF
{
    "version": $(json_quote "$version"),
    "host": $(json_quote "$host"),
    "target": $(json_quote "$BADGEVMS_STD_TARGET"),
    "built_at": $(json_quote "$built_at"),
    "rustc_release": $(json_quote "$rustc_release"),
    "source": {
        "repo": $(json_quote "$source_repo"),
        "rev": $(json_quote "$source_rev")
    },
    "rust_toolchain": {
        "repo": $(json_quote "$rust_repo_url"),
        "rev": $(json_quote "$rust_rev")
    }
}
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
if grep -RIl 'why2025-badge-rust-libc' "$image" | grep -q .; then
    fail "archive would contain the removed why2025-badge-rust-libc crate"
fi
checkout_refs=$(grep -RIl --exclude='badgevms-toolchain.env' --exclude='badgevms-toolchain.json' \
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
metadata="$out_dir/$package.json"
cp "$package_root/badgevms-toolchain.json" "$metadata"
tar -C "$tmp" -czf "$archive" "$package"
(cd "$out_dir" && sha256sum "$(basename "$archive")" > "$(basename "$archive").sha256")

printf 'packaged BadgeVMS toolchain: %s\n' "$archive"
printf 'checksum: %s.sha256\n' "$archive"
printf 'metadata: %s\n' "$metadata"
