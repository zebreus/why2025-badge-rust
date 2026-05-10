---
status: accepted
---

# Make the main repo own the canonical BadgeVMS ABI

The BadgeVMS std target remains a patched Rust toolchain target, and the repository root owns the
authoritative raw ABI through `why2025-badge-sys-bindings`.

## Decision

- `why2025-badge-sys-bindings` in this repository is the canonical raw BadgeVMS ABI artifact for
  both std and no_std consumers.
- `why2025-badge-sys` is a thin wrapper over `why2025-badge-sys-bindings` and owns wrapper-only
  behavior such as Host builds using Emulation and no_std badge-link metadata.
- The BadgeVMS std port in the patched Rust toolchain depends on `why2025-badge-sys-bindings`
  directly.
- Rust-specific semantic adaptation stays in the BadgeVMS std PAL inside the patched Rust toolchain.
  Shared repo crates own the raw ABI and wrapper behavior, not Rust std policy.
- The superproject checkout is the supported entrypoint for building and documenting the BadgeVMS
  std target.
- The existing no_std BadgeVMS App workflow remains unchanged apart from consuming the same
  canonical raw ABI source through the workspace crates.

## Consequences

- The repository has one authoritative source for the raw firmware ABI.
- `why2025-badge-sys` documentation must describe a wrapper layer, not the architecture boundary for
  the std port.
- Public setup and workflow documentation must describe the superproject as the supported way to
  build the BadgeVMS std toolchain.
- Future ABI-facing changes start from `why2025-badge-sys-bindings` and then flow outward into the
  wrapper crate, no_std helpers, and the std PAL.