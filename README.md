# why2025-badge-rust

Various support packages for using rust on the WHY2025 badge

## Badge App Linking

Badge apps still need to link as shared objects, keep `main` as the entry point, and prune exports for the loader. That policy now lives at the app boundary instead of the workspace-wide target config.

Keep `-Crelocation-model=pic` in the badge target config, depend on `why2025-badge-app-no-std` for the runtime side, and call `why2025-badge-build` from a tiny local `build.rs`. The facade crate still forwards the badge-link metadata from `why2025-badge-sys`, so consumer apps do not need a checked-in `retain.txt` or a direct dependency on `why2025-badge-sys` in their build script.

`why2025-badge-app-no-std` now provides the default `riscv32` allocator and panic handler for apps. Keep that default on the app dependency. If an app wants to provide its own allocator or panic handler, switch its app dependency to a direct dependency with `default-features = false` and define those items locally.

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

The example apps in `examples/` use this pattern directly.
