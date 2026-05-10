#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/common.sh"

need_cmd git
need_cmd python3
need_cmd rustc

repo=$(rust_repo)
[[ -d "$repo/.git" || -f "$repo/.git" ]] || fail "resolved Rust checkout is not a git checkout: $repo"
[[ -x "$repo/x.py" ]] || fail "resolved Rust checkout has no executable x.py: $repo"

cd "$repo"

if ! grep -R -q "badgevms" compiler/rustc_target/src/spec library/std/src library/backtrace/src 2>/dev/null; then
    fail "patched Rust checkout does not appear to contain BadgeVMS target/std backend changes"
fi

[[ -f "$PROJECT_ROOT/why2025-badge-sys-bindings/Cargo.toml" ]] || \
    fail "missing canonical raw BadgeVMS ABI crate: why2025-badge-sys-bindings"

config="$repo/build/badgevms/config.toml"
mkdir -p "$(dirname "$config")"

cat > "$config" <<'CONFIG'
profile = "compiler"
change-id = "ignore"

[llvm]
download-ci-llvm = false
ninja = true
targets = "RISCV;X86"

[build]
extended = true
tools = ["cargo", "rustfmt"]

[rust]
debug = false
incremental = false
# Build and ship the self-contained LLD wrapper used by the BadgeVMS target spec
# (`linker = "rust-lld"`). Bootstrap still skips host-linker override for this
# target so the target-owned linker flavor and arguments are preserved.
lld = true

[target.riscv32imafc-unknown-badgevms]
# The patched target owns link flags. Keep this section available for SDK paths only.

CONFIG

# Stage0 does not know the new built-in BadgeVMS target yet, so bootstrap's
# target sanity check must be skipped until stage1 has been built from this
# patched checkout.
export BOOTSTRAP_SKIP_TARGET_SANITY=1

python3 ./x.py build --config "$config" compiler/rustc --stage 2
python3 ./x.py build --config "$config" library/std --stage 2 --target "$(rustc -vV | sed -n 's/^host: //p')"
python3 ./x.py build --config "$config" library/std --stage 2 --target "$BADGEVMS_STD_TARGET"
python3 ./x.py build --config "$config" cargo

stage2=$(stage2_dir_for_repo "$repo")
[[ -x "$stage2/bin/rustc" ]] || fail "stage2 rustc was not produced at $stage2/bin/rustc"

printf 'built BadgeVMS std toolchain stage2: %s\n' "$stage2"
printf 'developer-link it with:\n  %s/tools/badgevms-std/dev-link-stage2.sh %q %q\n' "$PROJECT_ROOT" "$stage2" "$BADGEVMS_TOOLCHAIN_NAME"
