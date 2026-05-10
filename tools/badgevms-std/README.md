# BadgeVMS std tooling

These scripts support the BadgeVMS std workflow in this superproject. They assume the bundled
`why2025-badge-rust-toolchain` checkout is present. See
[ADR 0005](../../docs/adr/0005-support-badgevms-std-through-the-superproject.md) for why this is
the supported entrypoint.

## Scripts

- `build-toolchain.sh` — build the bundled patched Rust checkout.
- `link-toolchain.sh` — link a stage2 toolchain into rustup.
- `verify-toolchain.sh` — verify target cfg for `riscv32imafc-unknown-badgevms`.
- `run-smoke.sh` — build a std example and inspect the ELF artifact.
- `inspect-elf.sh` — verify BadgeVMS shared-object shape and closed exports.
- `ci-smoke.sh` — run repository-side checks that do not require BadgeVMS hardware.

Set `BADGEVMS_TOOLCHAIN_NAME` to the local rustup name when needed.
