---
status: accepted
---

# Keep the BadgeVMS std target toolchain-owned

BadgeVMS std Apps use a dedicated Rust target and a BadgeVMS `std` backend owned by a patched Rust toolchain. The application repository does not carry a repo-local copy of `libstd`, and std Apps do not use `why2025-badge-build`, `badge-app-link`, or an App-local build script to inject BadgeVMS final-link flags.

The existing no_std App workflow remains unchanged. Current no_std Apps continue to own final link arguments at the App boundary through `why2025-badge-build`, as recorded in [0001-app-owned-badge-linking.md](0001-app-owned-badge-linking.md). This ADR only scopes the new std target.

## Decision

- The BadgeVMS std target is implemented in a patched Rust toolchain.
- The target cfg contract is `target_os = "badgevms"` and `target_family = "unix"`.
- The implementation target triple is `riscv32imafc-unknown-badgevms` unless Rust target plumbing forces a different spelling; the cfg contract is stable either way.
- The toolchain owns std-App linking: PIC shared object, entry `main`, garbage collection, debug stripping where appropriate, local-symbol discard, and closed export pruning equivalent to retaining only `main`.
- The repository provides the canonical raw ABI artifact in `why2025-badge-sys-bindings`, wrapper and Emulation support in `why2025-badge-sys`, examples, docs, toolchain build/link scripts, and conformance harnesses, but not the canonical `std` backend source.
- `why2025-badge-sys-bindings` is the firmware ABI reference artifact in this repository. `why2025-badge-sys` remains the wrapper and Emulation crate for Apps and tests. `std` must not depend on the `why2025-badge-sys` wrapper crate as a Rust dependency.
- Host Emulation decisions around `_ctype_` normalization and selective libc overlap remain settled and are not part of this target decision.

## Consequences

- std Apps feel like normal Rust `std` projects once the BadgeVMS toolchain is installed.
- no_std and std workflows intentionally differ at the final-link boundary.
- Toolchain changes, target spec changes, and standard-library changes move together.
- Repository CI can validate examples, scripts, docs, and smoke artifacts, but full `std` backend validation requires the patched Rust fork and, for runtime behavior, BadgeVMS execution.
- A local `rustup toolchain link` flow is the first supported distribution mechanism; hosted rustup distribution is gated behind conformance results.
