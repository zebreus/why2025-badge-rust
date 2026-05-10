# BadgeVMS std target

This directory describes the on-device Rust `std` target for BadgeVMS Apps.

The target is a toolchain-and-standard-library port, not a crate feature. The patched Rust
toolchain owns the target spec, `std` backend, panic strategy, allocator integration, and std-App
final linking. This repository owns the BadgeVMS domain model, the canonical raw ABI artifact in
`why2025-badge-sys-bindings`, the thin wrapper and Emulation layer in `why2025-badge-sys`,
examples, smoke tests, installation scripts, and documentation.

The authoritative architecture decision for that split is
[../adr/0004-canonical-badgevms-abi-layering.md](../adr/0004-canonical-badgevms-abi-layering.md).
Use that ADR as the source of truth when this directory and toolchain-side comments disagree.

## Target contract

- Target triple: `riscv32imafc-unknown-badgevms`.
- Required cfg: `target_os = "badgevms"`.
- Required family: `target_family = "unix"`.
- Panic strategy: abort.
- App artifact: PIC ELF shared object loaded by BadgeVMS.
- Entry: exported `main`.
- Exports: closed export set, retaining only `main` in v1.
- Distribution phase 1: local custom toolchain linked with `rustup toolchain link`.
- Distribution phase 2: optional hosted rustup-compatible toolchain after conformance gates pass.

## Documents

- [getting-started.md](getting-started.md) — local toolchain build/link flow and cargo commands.
- [support-matrix.md](support-matrix.md) — supported, partial, and unsupported `std` API areas.
- [implementation-map.md](implementation-map.md) — upstream Rust patch areas and repository integration points.
- [threading-model.md](threading-model.md) — `std::thread`, TLS, park/unpark, `Mutex`, and `Condvar` design.
- [paths-and-io.md](paths-and-io.md) — BadgeVMS paths, filesystem, fd, and stdio behavior.
- [process-model.md](process-model.md) — narrow `std::process::Command` contract.
- [networking.md](networking.md) — TCP and address-resolution support.
- [unsupported-policy.md](unsupported-policy.md) — centralized explicit failure policy.
- [testing.md](testing.md) — smoke, conformance, CI, hardware, and release gates.
