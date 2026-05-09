# why2025-badge-rust

Various support packages for using rust on the WHY2025 badge

## Badge App Linking

Badge apps still need to link as shared objects, keep `main` as the entry point, and prune exports for the loader. That policy now lives at the app boundary instead of the workspace-wide target config.

Keep `-Crelocation-model=pic` in the badge target config, depend on `why2025-badge-app-no-std`, and call its build helper from a tiny local build script. The facade crate consumes the badge-link metadata from `why2025-badge-sys`, so consumer apps do not need a direct dependency on `why2025-badge-build` or a checked-in `retain.txt`.

`why2025-badge-app-no-std` now provides the default `riscv32` allocator and panic handler for apps. Keep that default on the app dependency, and disable it only in the build-script dependency. If an app wants to provide its own allocator or panic handler, switch its app dependency to a direct dependency with `default-features = false` and define those items locally. In this workspace, the example crates use a direct path dependency in `build-dependencies` because Cargo will not let a workspace-inherited dependency override default features.

```toml
[package]
build = "src/build_script.rs"

[dependencies]
why2025-badge-app-no-std.workspace = true

[build-dependencies]
why2025-badge-app-no-std = { path = "../../why2025-badge-app-no-std", default-features = false }
```

```rust
fn main() {
	why2025_badge_app_no_std::configure_build("src/build_script.rs");
}
```

The example apps in `examples/` use this pattern directly.
