#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/common.sh"

need_cmd git
need_cmd python3
need_cmd rustc

repo=$(rust_repo)
[[ -d "$repo/.git" ]] || fail "BADGEVMS_RUST_REPO is not a git checkout: $repo"
[[ -x "$repo/x.py" ]] || fail "Rust checkout has no executable x.py: $repo"

cd "$repo"

if [[ -n "${BADGEVMS_RUST_REV:-}" ]]; then
    git fetch --all --tags
    git checkout "$BADGEVMS_RUST_REV"
fi

if ! grep -R "badgevms" compiler/rustc_target/src/spec library/std/src 2>/dev/null | head -n1 >/dev/null; then
    fail "patched Rust checkout does not appear to contain BadgeVMS target/std backend changes"
fi

cat > config.badgevms.toml <<'CONFIG'
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

[target.riscv32imafc-unknown-badgevms]
# The patched target owns link flags. Keep this section available for SDK paths only.
CONFIG

# Stage0 does not know the new built-in BadgeVMS target yet, so bootstrap's
# target sanity check must be skipped until stage1 has been built from this
# patched checkout.
export BOOTSTRAP_SKIP_TARGET_SANITY=1

python3 ./x.py build --config config.badgevms.toml compiler/rustc --stage 2
python3 ./x.py build --config config.badgevms.toml library/std --stage 2 --target "$(rustc -vV | sed -n 's/^host: //p')"
python3 ./x.py build --config config.badgevms.toml cargo

stage2=$(stage2_dir_for_repo "$repo")
[[ -x "$stage2/bin/rustc" ]] || fail "stage2 rustc was not produced at $stage2/bin/rustc"

printf 'built BadgeVMS std toolchain stage2: %s\n' "$stage2"
printf 'link it with:\n  %s/tools/badgevms-std/link-toolchain.sh %q %q\n' "$PROJECT_ROOT" "$stage2" "$BADGEVMS_TOOLCHAIN_NAME"
