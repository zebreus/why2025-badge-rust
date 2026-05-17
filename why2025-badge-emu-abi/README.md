# why2025-badge-emu-abi

Experimental Linux/ELF implementation of the BadgeVMS ABI as an exported host artifact.

This crate is intentionally separate from `why2025-badge-sys`. The existing `why2025-badge-sys`
host emulator remains unchanged and is still the behavior oracle for overlapping functionality.

This crate is not intended to be a Rust-facing API. Rust consumers should import the canonical
raw ABI from `why2025-badge-sys-bindings` and let `why2025-badge-emu-abi` act only as the host-side
C-symbol provider.

Current scope:

- Production crate is `no_std` with `alloc` available for later phases.
- Host target is Linux/ELF only.
- The main crate stays `rlib`-safe in the Rust dependency graph so host consumers can link it as a C-symbol provider.
- The crate now exports the full manifest surface from `symbols.yml`.
- Easy host-backed wrapped exports are forwarded with `RTLD_NEXT`.
- Remaining not-yet-implemented families are still exported, but many currently terminate via deterministic generated abort stubs.

The crate treats exported symbols as the contract. Later phases will add manifest-driven symbol audits,
C/dlopen consumers, and differential tests against the existing emulator.

Validation aliases:

- `cargo emu-abi-check`
- `cargo emu-abi-test`
- `tools/emu-abi/checks/report-symbol-coverage.sh`
- `tools/emu-abi/checks/inspect-exports.sh`

The test alias intentionally runs the unit-test target only. The crate's production artifacts are
`no_std`, while Cargo's normal test profile builds non-test library artifacts with unwind settings.
