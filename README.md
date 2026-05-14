# why2025-badge-rust

Various support packages for using rust on the WHY2025 badge

## Workflows

This repository now documents three separate workflows:

- **no_std BadgeVMS Apps** use `riscv32imafc-unknown-none-elf`, `why2025-badge-app-no-std`, and the App-owned link helper described below.
- **Host builds using Emulation** use the host target and the `emu-*` cargo aliases for fast iteration against host-side BadgeVMS behavior.
- **BadgeVMS std Apps** use the incubating `riscv32imafc-unknown-badgevms` target from the patched
  Rust toolchain bundled with this superproject.

See [docs/adr/0004-canonical-badgevms-abi-layering.md](docs/adr/0004-canonical-badgevms-abi-layering.md)
for the raw-ABI boundary, [docs/adr/0005-support-badgevms-std-through-the-superproject.md](docs/adr/0005-support-badgevms-std-through-the-superproject.md)
for the supported std-target entrypoint, [tools/badgevms-std/README.md](tools/badgevms-std/README.md)
for the current scripts.

The incubating BadgeVMS std backend is intentionally partial. It currently owns the
BadgeVMS ABI boundary for allocator, stdio, process launch, threads, sync/TLS,
startup argv capture, environment lookup/enumeration, clocks, best-effort
randomness, and a narrow native filesystem surface for fd-backed files,
metadata, directory iteration, directly backed path-mutating operations,
`std::io::IsTerminal` over BadgeVMS `isatty`, and TCP/IPv4 `std::net` operations
backed by native `socket`, `connect`, `bind`, `listen`, `accept`, `read`,
`write`, and `getaddrinfo` calls. Unsupported Unix-shaped behavior remains
explicit: pipes are out of scope, current-directory APIs are not synthesized,
PATH search is not added, UDP and Unix-domain sockets are not emulated, socket
duplication/options/nonblocking/timeouts remain unsupported without direct
BadgeVMS backing, symlinks and uid/gid or POSIX permission mutation are not
emulated, and randomness should not yet be treated as a cryptographic-strength
firmware guarantee.

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
