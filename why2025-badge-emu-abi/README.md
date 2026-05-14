# why2025-badge-emu-abi

Experimental Linux/ELF implementation of the BadgeVMS ABI as an exported host artifact.

This crate is intentionally separate from `why2025-badge-sys`. The existing `why2025-badge-sys`
host emulator remains unchanged and is still the behavior oracle for overlapping functionality.

Current scope:

- Production crate is `no_std` with `alloc` available for later phases.
- Host target is Linux/ELF only.
- Crate artifacts include `rlib`, `staticlib`, and `cdylib`.
- A first libc-compatible slice exports errno, ctype normalization symbols, and simple memory/string helpers.
- Graphics, networking, cURL, and Wi-Fi are intentionally deferred and currently abort with diagnostics.

The crate treats exported symbols as the contract. Later phases will add manifest-driven symbol audits,
C/dlopen consumers, and differential tests against the existing emulator.

Validation aliases:

- `cargo emu-abi-check`
- `cargo emu-abi-build`
- `cargo emu-abi-test`
- `tools/emu-abi/checks/inspect-exports.sh`

The test alias intentionally runs the unit-test target only. The crate's production artifacts are
`no_std`, while Cargo's normal test profile builds non-test library artifacts with unwind settings.
