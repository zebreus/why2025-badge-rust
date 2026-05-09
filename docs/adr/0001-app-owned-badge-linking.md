---
status: accepted
---

# Make badge final linking app-owned

Badge applications in this workspace own the final badge-link policy instead of inheriting it from workspace-global target rustflags. We chose this because Cargo lets dependency build scripts publish metadata but not impose final `rustc-link-arg-bins` behavior on consuming binaries, and the old root-level `retain.txt` plus linker-arg flow made the repo awkward to reuse outside this workspace. The settled pattern is: keep only PIC in the root target config, let `why2025-badge-sys` publish badge-link metadata behind `badge-app-link`, and have each badge app emit the final linker args through a tiny build script that calls `why2025-badge-build`.

## Considered Options

- Keep the workspace-global linker args in `.cargo/config.toml`: rejected because it couples reusable crates and downstream consumers to this repository layout and to a checked-in retain-symbols file.
- Let `why2025-badge-sys` own the final linker args directly: rejected because dependency build scripts cannot reliably own final binary `rustc-link-arg-bins` behavior for consumers.
- Move final-link ownership to the app boundary with shared helper logic: accepted because it matches Cargo's model while keeping consumer boilerplate small.

## Consequences

- Badge apps must opt in explicitly with a tiny build script and a build-dependency on `why2025-badge-build`.
- `why2025-badge-sys` remains the source of badge-link metadata, including the generated retain-symbols file in `OUT_DIR`.
- The root target config stays limited to shared badge-wide compile settings like PIC instead of final-link policy.