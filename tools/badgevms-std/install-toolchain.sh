#!/usr/bin/env bash
set -euo pipefail

BADGEVMS_RELEASE_REPO=${BADGEVMS_RELEASE_REPO:-zebreus/why2025-badge-rust}

detect_host() {
    if command -v rustc >/dev/null 2>&1; then
        local rustc_host
        rustc_host=$(rustc -vV 2>/dev/null | sed -n 's/^host: //p' || true)
        if [[ -n "$rustc_host" ]]; then
            printf '%s\n' "$rustc_host"
            return
        fi
    fi

    case "$(uname -s)-$(uname -m)" in
        Linux-x86_64) printf 'x86_64-unknown-linux-gnu\n' ;;
        *) printf 'error: could not detect host; pass --host\n' >&2; exit 1 ;;
    esac
}

usage() {
    cat <<'USAGE'
usage: install-toolchain.sh [options]

Options:
  --archive PATH       Install from a local badgevms-std archive.
  --url URL            Download archive from an explicit URL.
  --version VERSION    Release version. Used to form the default download URL.
                      Use "latest" for the latest GitHub Release asset.
  --host TRIPLE        Host triple. Defaults to rustc -vV or uname mapping.
  --name NAME          rustup toolchain name. Default: badgevms-std.
  --install-dir DIR    Extraction root. Default: $XDG_DATA_HOME/badgevms-rust/toolchains.
  --force              Replace an existing extracted toolchain directory.
  --print-json         Print a small JSON summary after install.
  -h, --help           Show this help.
USAGE
}

name=badgevms-std
version=${BADGEVMS_TOOLCHAIN_VERSION:-latest}
host=${BADGEVMS_TOOLCHAIN_HOST:-}
install_dir=${BADGEVMS_TOOLCHAIN_INSTALL_DIR:-${XDG_DATA_HOME:-$HOME/.local/share}/badgevms-rust/toolchains}
archive=${BADGEVMS_TOOLCHAIN_ARCHIVE:-}
url=${BADGEVMS_TOOLCHAIN_URL:-}
force=0
print_json=0

download_file() {
    local source=$1
    local dest=$2

    if command -v curl >/dev/null 2>&1; then
        curl -fL "$source" -o "$dest"
    elif command -v wget >/dev/null 2>&1; then
        wget -O "$dest" "$source"
    else
        printf 'error: missing downloader: install curl or wget, or pass --archive\n' >&2
        exit 1
    fi
}

release_url_for() {
    local requested_version=$1
    local requested_host=$2

    if [[ "$requested_version" == "latest" ]]; then
        printf 'https://github.com/%s/releases/latest/download/badgevms-std-%s.tar.gz\n' \
            "$BADGEVMS_RELEASE_REPO" "$requested_host"
        return
    fi

    local clean_version=${requested_version#badgevms-std-v}
    printf 'https://github.com/%s/releases/download/badgevms-std-v%s/badgevms-std-%s-%s.tar.gz\n' \
        "$BADGEVMS_RELEASE_REPO" "$clean_version" "$clean_version" "$requested_host"
}

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
        *) printf 'error: unknown argument: %s\n' "$1" >&2; exit 1 ;;
    esac
done

    command -v rustup >/dev/null 2>&1 || { printf 'error: missing required command: rustup\n' >&2; exit 1; }
    command -v tar >/dev/null 2>&1 || { printf 'error: missing required command: tar\n' >&2; exit 1; }
    command -v sha256sum >/dev/null 2>&1 || { printf 'error: missing required command: sha256sum\n' >&2; exit 1; }

host=${host:-$(detect_host)}
    [[ -n "$host" ]] || { printf 'error: host triple is empty\n' >&2; exit 1; }

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

if [[ -z "$archive" ]]; then
    if [[ -z "$url" ]]; then
        url=$(release_url_for "$version" "$host")
    fi

    archive="$tmp/$(basename "$url")"
    download_file "$url" "$archive"
    download_file "$url.sha256" "$archive.sha256"
    (cd "$(dirname "$archive")" && sha256sum -c "$(basename "$archive").sha256")
else
    [[ -f "$archive" ]] || { printf 'error: archive does not exist: %s\n' "$archive" >&2; exit 1; }
    archive="$(cd "$(dirname "$archive")" && pwd)/$(basename "$archive")"
    if [[ -f "$archive.sha256" ]]; then
        (cd "$(dirname "$archive")" && sha256sum -c "$(basename "$archive").sha256")
    fi
fi

extract="$tmp/extract"
mkdir -p "$extract"
tar -C "$extract" -xzf "$archive"
root_count=$(find "$extract" -mindepth 1 -maxdepth 1 -type d | wc -l)
[[ "$root_count" -eq 1 ]] || { printf 'error: archive must contain exactly one top-level directory\n' >&2; exit 1; }
root=$(find "$extract" -mindepth 1 -maxdepth 1 -type d -print -quit)
root_name=$(basename "$root")

mkdir -p "$install_dir"
dest="$install_dir/$root_name"
if [[ -e "$dest" ]]; then
    [[ "$force" -eq 1 ]] || { printf 'error: install destination already exists: %s (pass --force)\n' "$dest" >&2; exit 1; }
    rm -rf "$dest"
fi
mv "$root" "$dest"

[[ -x "$dest/bin/rustc" ]] || { printf 'error: installed toolchain has no bin/rustc\n' >&2; exit 1; }
[[ -x "$dest/bin/cargo" ]] || { printf 'error: installed toolchain has no bin/cargo\n' >&2; exit 1; }

rustup toolchain link "$name" "$dest"

cfg=$(rustup run "$name" rustc --target "riscv32imafc-unknown-badgevms" --print cfg | sort)
printf '%s\n' "$cfg" | grep -qx 'target_os="badgevms"' || { printf 'error: target cfg missing target_os="badgevms"\n' >&2; exit 1; }
printf '%s\n' "$cfg" | grep -qx 'target_family="unix"' || { printf 'error: target cfg missing target_family="unix"\n' >&2; exit 1; }
if printf '%s\n' "$cfg" | grep -Eq '^target_env=".+"$'; then
    printf 'error: BadgeVMS target must not set a non-empty target_env\n' >&2
    exit 1
fi

if [[ "$print_json" -eq 1 ]]; then
    printf '{"toolchain":"%s","path":"%s","host":"%s","target":"%s"}\n' \
        "$name" "$dest" "$host" 'riscv32imafc-unknown-badgevms'
else
    printf 'installed BadgeVMS Rust toolchain %s at %s\n' "$name" "$dest"
    printf 'next steps:\n'
    printf '  rustup run %s cargo build --target %s\n' "$name" 'riscv32imafc-unknown-badgevms'
fi
