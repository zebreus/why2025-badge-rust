---
status: accepted
supersedes: 0001
---

# Make `riscv32imafc-unknown-badgevms` the primary no_std target

The repository's primary no_std workflow uses the built-in BadgeVMS target
`riscv32imafc-unknown-badgevms` instead of the repo-owned
`riscv32imafc-unknown-none-elf` plus app-local linker glue. We chose this because the built-in
target already owns the BadgeVMS shared-object contract for both std and no_std Apps, and reusing
that target removes example-local `build.rs` files, removes `why2025-badge-build` from the active
workflow, and gives the repo one public target name for BadgeVMS development.

## Considered Options

- Keep the repo-primary no_std workflow on `riscv32imafc-unknown-none-elf` with app-owned final linking: rejected because it keeps the workspace split across two target names and preserves build-script boilerplate that the built-in BadgeVMS target no longer needs.
- Add a second built-in no_std-specific BadgeVMS target such as `riscv32imafc-unknown-badgevms-elf`: rejected because it creates another public target name without buying a materially different repository workflow.
- Reuse `riscv32imafc-unknown-badgevms` for both std and no_std Apps: accepted because the target already produces the required BadgeVMS ELF shape, including PIC, ET_DYN output, exported `main`, and closed export pruning.

## Consequences

- Active no_std examples build directly with `cargo +badgevms build --target riscv32imafc-unknown-badgevms` and no longer need a root `build.rs` or a `why2025-badge-build` dependency.
- `why2025-badge-app-no-std` remains as app-facing runtime and entry glue, not as a build-script bridge.
- `why2025-badge-build` and `why2025-badge-sys` badge-link metadata remain in the repository only as legacy compatibility for downstreams that still target `riscv32imafc-unknown-none-elf`.
- Repo docs, aliases, and CI treat `riscv32imafc-unknown-badgevms` as the one public BadgeVMS target for both std and no_std work.
- no_std Apps built on the primary path inherit the built-in target cfg surface, including `target_family = "unix"`, because that target is now the shared BadgeVMS contract.
