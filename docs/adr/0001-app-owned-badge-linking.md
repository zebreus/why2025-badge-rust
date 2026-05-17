---
status: superseded
superseded_by: 0008
---

# Make no_std badge final linking app-owned

This ADR records the legacy no_std BadgeVMS App workflow that targeted `riscv32imafc-unknown-none-elf`. The repo's primary no_std workflow moved to `riscv32imafc-unknown-badgevms` in ADR 0008, so this document remains only as historical context for downstreams that still use the old helper path.

This ADR applies only to no_std BadgeVMS Apps targeting `riscv32imafc-unknown-none-elf`. The built-in BadgeVMS target (`riscv32imafc-unknown-badgevms`) now has separate toolchain-owned linking behavior and is out of scope here.

no_std BadgeVMS Apps in this workspace own the final badge-link policy instead of inheriting it from workspace-global target rustflags. We chose this because Cargo lets dependency build scripts publish metadata but not impose final `rustc-link-arg-bins` behavior on consuming binaries, and the old root-level `retain.txt` plus linker-arg flow made the repo awkward to reuse outside this workspace. The settled pattern is: keep only PIC in the root target config, let `why2025-badge-sys` publish badge-link metadata behind `badge-app-link`, and have each App emit the final linker args through a tiny build script that calls `why2025-badge-build`.

## Considered Options

- Keep the workspace-global linker args in `.cargo/config.toml`: rejected because it couples reusable crates and downstream consumers to this repository layout and to a checked-in retain-symbols file.
- Let `why2025-badge-sys` own the final linker args directly for no_std Apps: rejected because dependency build scripts cannot reliably own final binary `rustc-link-arg-bins` behavior for consumers.
- Move final-link ownership to the app boundary with shared helper logic: accepted because it matches Cargo's model while keeping consumer boilerplate small.

## Consequences

- no_std BadgeVMS Apps must opt in explicitly with a tiny build script and a build-dependency on `why2025-badge-build`.
- `why2025-badge-sys` remains the source of badge-link metadata, including the generated retain-symbols file in `OUT_DIR`.
- The root target config for `riscv32imafc-unknown-none-elf` stays limited to shared badge-wide compile settings like PIC instead of final-link policy.
