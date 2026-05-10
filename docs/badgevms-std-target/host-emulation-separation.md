# Host build and BadgeVMS std separation

The current Host build using Emulation and the new on-device BadgeVMS std target are complementary workflows, not one abstraction.

## Host build using Emulation

- Target: host, currently `x86_64-unknown-linux-gnu`.
- Purpose: fast iteration and behavior checks against a host-side implementation of BadgeVMS.
- Existing aliases: `emu-check`, `emu-build`, `emu-run`.
- Implementation: [why2025-badge-sys/src/emulated.rs](../../why2025-badge-sys/src/emulated.rs).

## no_std BadgeVMS App

- Target: `riscv32imafc-unknown-none-elf`.
- Purpose: current on-device App flow without Rust `std`.
- Existing aliases: `badge-check`, `badge-build`, `badge-run`.
- Linking: App-owned helper flow from [0001-app-owned-badge-linking.md](../adr/0001-app-owned-badge-linking.md).

## std BadgeVMS App

- Target: `riscv32imafc-unknown-badgevms`.
- Purpose: on-device Rust `std` Apps.
- Linking: toolchain-owned, no App helper crate.
- ABI source: `why2025-badge-sys-bindings` from this superproject.
- Rust semantics: BadgeVMS std PAL code in the patched Rust toolchain.
- Distribution: local custom toolchain first.

Do not use Host build behavior to justify unsupported on-device std behavior. The target must reflect BadgeVMS process, path, thread, and loader semantics honestly.
