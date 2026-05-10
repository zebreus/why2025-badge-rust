# BadgeVMS std tooling

These scripts support the repository side of the BadgeVMS std target. They assume the standard
library backend lives in the bundled `why2025-badge-rust-toolchain` checkout and that the
canonical raw ABI lives in `why2025-badge-sys-bindings` in this superproject.

## Scripts

- `build-toolchain.sh` — build the bundled patched Rust checkout.
- `link-toolchain.sh` — link a stage2 toolchain into rustup.
- `verify-toolchain.sh` — verify target cfg for `riscv32imafc-unknown-badgevms`.
- `run-smoke.sh` — build a std example and inspect the ELF artifact.
- `inspect-elf.sh` — verify BadgeVMS shared-object shape and closed exports.
- `ci-smoke.sh` — run repository-side checks that do not require BadgeVMS hardware.

Set `BADGEVMS_TOOLCHAIN_NAME` to the local rustup name when needed.
