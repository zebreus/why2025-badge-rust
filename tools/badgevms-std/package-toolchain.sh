#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)

if [[ ${1:-} == "-h" || ${1:-} == "--help" ]]; then
    cat <<'USAGE'
usage: package-toolchain.sh [ignored-rust-dist-dir] [out-dir]

Compatibility entrypoint for older CI workflow revisions. The toolchain archive
is built directly by dist-toolchain.sh using Rust's x.py install flow.
USAGE
    exit 0
fi

out_dir=${2:-$PROJECT_ROOT/dist/badgevms-std}
rm -rf "$out_dir"
exec "$PROJECT_ROOT/tools/badgevms-std/dist-toolchain.sh" "$out_dir" "${BADGEVMS_TOOLCHAIN_VERSION:-}"
