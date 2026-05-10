#!/usr/bin/env bash
set -euo pipefail

BADGEVMS_STD_TARGET=${BADGEVMS_STD_TARGET:-riscv32imafc-unknown-badgevms}

fail() {
    printf 'error: %s\n' "$*" >&2
    exit 1
}

need_cmd() {
    command -v "$1" >/dev/null 2>&1 || fail "missing required command: $1"
}

detect_host() {
    if command -v rustc >/dev/null 2>&1; then
        rustc -vV | sed -n 's/^host: //p'
        return
    fi

    case "$(uname -s)-$(uname -m)" in
        Linux-x86_64) printf 'x86_64-unknown-linux-gnu\n' ;;
        *) fail "could not detect host; pass --host" ;;
    esac
}

usage() {
    cat <<'USAGE'
usage: install-toolchain.sh [options]

Options:
  --archive PATH       Install from a local badgevms-std archive.
  --url URL            Download archive from an explicit URL.
  --version VERSION    Release version. Used to form the default download URL.
  --host TRIPLE        Host triple. Defaults to rustc -vV or uname mapping.
  --name NAME          rustup toolchain name. Default: badgevms-std.
  --install-dir DIR    Extraction root. Default: $XDG_DATA_HOME/badgevms-rust/toolchains.
  --force              Replace an existing extracted toolchain directory.
  --print-json         Print a small JSON summary after install.
  -h, --help           Show this help.
USAGE
}

name=${BADGEVMS_TOOLCHAIN_NAME:-badgevms-std}
version=${BADGEVMS_TOOLCHAIN_VERSION:-latest}
host=${BADGEVMS_TOOLCHAIN_HOST:-}
install_dir=${BADGEVMS_TOOLCHAIN_INSTALL_DIR:-${XDG_DATA_HOME:-$HOME/.local/share}/badgevms-rust/toolchains}
archive=${BADGEVMS_TOOLCHAIN_ARCHIVE:-}
url=${BADGEVMS_TOOLCHAIN_URL:-}
force=0
print_json=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --archive) archive=${2:-}; shift 2 ;;
        --url) url=${2:-}; shift 2 ;;
        --version) version=${2:-}; shift 2 ;;
        --host) host=${2:-}; shift 2 ;;
        --name) name=${2:-}; shift 2 ;;
        --install-dir) install_dir=${2:-}; shift 2 ;;
        --force) force=1; shift ;;
        --print-json) print_json=1; shift ;;
        -h|--help) usage; exit 0 ;;
        *) fail "unknown argument: $1" ;;
    esac
done

need_cmd rustup
need_cmd tar
need_cmd sha256sum

host=${host:-$(detect_host)}
[[ -n "$host" ]] || fail "host triple is empty"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

if [[ -z "$archive" ]]; then
    if [[ -z "$url" ]]; then
        if [[ "$version" == "latest" ]]; then
            url="https://github.com/zebreus/why2025-badge-rust/releases/latest/download/badgevms-std-$host.tar.gz"
        else
            url="https://github.com/zebreus/why2025-badge-rust/releases/download/badgevms-std-v$version/badgevms-std-$version-$host.tar.gz"
        fi
    fi

    archive="$tmp/$(basename "$url")"
    if command -v curl >/dev/null 2>&1; then
        curl -fL "$url" -o "$archive"
        if curl -fL "$url.sha256" -o "$archive.sha256"; then
            (cd "$(dirname "$archive")" && sha256sum -c "$(basename "$archive").sha256")
        fi
    elif command -v wget >/dev/null 2>&1; then
        wget -O "$archive" "$url"
        if wget -O "$archive.sha256" "$url.sha256"; then
            (cd "$(dirname "$archive")" && sha256sum -c "$(basename "$archive").sha256")
        fi
    else
        fail "missing downloader: install curl or wget, or pass --archive"
    fi
else
    [[ -f "$archive" ]] || fail "archive does not exist: $archive"
    archive="$(cd "$(dirname "$archive")" && pwd)/$(basename "$archive")"
    if [[ -f "$archive.sha256" ]]; then
        (cd "$(dirname "$archive")" && sha256sum -c "$(basename "$archive").sha256")
    fi
fi

extract="$tmp/extract"
mkdir -p "$extract"
tar -C "$extract" -xzf "$archive"
root_count=$(find "$extract" -mindepth 1 -maxdepth 1 -type d | wc -l)
[[ "$root_count" -eq 1 ]] || fail "archive must contain exactly one top-level directory"
root=$(find "$extract" -mindepth 1 -maxdepth 1 -type d -print -quit)
root_name=$(basename "$root")

mkdir -p "$install_dir"
dest="$install_dir/$root_name"
if [[ -e "$dest" ]]; then
    [[ "$force" -eq 1 ]] || fail "install destination already exists: $dest (pass --force)"
    rm -rf "$dest"
fi
mv "$root" "$dest"

[[ -x "$dest/bin/rustc" ]] || fail "installed toolchain has no bin/rustc"
[[ -x "$dest/bin/cargo" ]] || fail "installed toolchain has no bin/cargo"

rustup toolchain link "$name" "$dest"

cfg=$(rustup run "$name" rustc --target "$BADGEVMS_STD_TARGET" --print cfg | sort)
printf '%s\n' "$cfg" | grep -qx 'target_os="badgevms"' || fail "target cfg missing target_os=\"badgevms\""
printf '%s\n' "$cfg" | grep -qx 'target_family="unix"' || fail "target cfg missing target_family=\"unix\""
if printf '%s\n' "$cfg" | grep -Eq '^target_env=".+"$'; then
    fail "BadgeVMS target must not set a non-empty target_env"
fi

if [[ "$print_json" -eq 1 ]]; then
    printf '{"toolchain":"%s","path":"%s","host":"%s","target":"%s"}\n' \
        "$name" "$dest" "$host" "$BADGEVMS_STD_TARGET"
else
    printf 'installed BadgeVMS Rust toolchain %s at %s\n' "$name" "$dest"
fi
