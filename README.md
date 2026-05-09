# why2025-badge-rust

Various support packages for using rust on the WHY2025 badge

## App Linking

BadgeVMS Apps still need to link as shared objects, keep `main` as the entry point, and prune exports for the loader. That policy now lives at the App boundary instead of the workspace-wide target config.

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
