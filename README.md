# why2025-badge-rust

Various support packages for using rust on the WHY2025 badge

## Workflows

This repository now documents three separate workflows:

- **no_std BadgeVMS Apps** use `riscv32imafc-unknown-none-elf`, `why2025-badge-app-no-std`, and the App-owned link helper described below.
- **Host builds using Emulation** use the host target and the `emu-*` cargo aliases for fast iteration against host-side BadgeVMS behavior.
- **BadgeVMS std Apps** use the incubating `riscv32imafc-unknown-badgevms` target from the patched
  Rust toolchain bundled with this superproject. The toolchain owns `std`, panic/allocator
  integration, and final App linking; std Apps do not use `why2025-badge-build` for link flags.

See [docs/adr/0004-canonical-badgevms-abi-layering.md](docs/adr/0004-canonical-badgevms-abi-layering.md)
for the authoritative architecture decision, [docs/prd/badgevms-std-target.md](docs/prd/badgevms-std-target.md)
for the product requirements, and [docs/badgevms-std-target/index.md](docs/badgevms-std-target/index.md)
for implementation docs, support matrix, examples, scripts, and test gates.

The canonical raw BadgeVMS firmware bindings live in `why2025-badge-sys-bindings`. The sibling
`why2025-badge-sys` crate is the thin wrapper over that ABI and adds Host builds using Emulation
plus no_std app-link behavior.

## App Linking

no_std BadgeVMS Apps still need to link as shared objects, keep `main` as the entry point, and prune exports for the loader. That policy lives at the App boundary instead of the workspace-wide target config.

Keep `-Crelocation-model=pic` in the BadgeVMS target config, depend on `why2025-badge-app-no-std` for the runtime side, and call `why2025-badge-build` from a tiny local `build.rs`. The facade crate still forwards the badge-link metadata from `why2025-badge-sys`, so consumer Apps do not need a checked-in `retain.txt` or a direct dependency on `why2025-badge-sys` in their build script.

`why2025-badge-app-no-std` now provides the default `riscv32` allocator and panic handler for Apps. Keep that default on the App dependency. If an App wants to provide its own allocator or panic handler, switch its App dependency to a direct dependency with `default-features = false` and define those items locally.

```toml
[dependencies]
why2025-badge-app-no-std.workspace = true

[build-dependencies]
why2025-badge-build.workspace = true
```

```rust
fn main() {
	why2025_badge_build::configure();
}
```

The example Apps in `examples/` use this pattern directly.
