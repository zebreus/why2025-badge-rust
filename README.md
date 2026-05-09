# why2025-badge-rust

Various support packages for using rust on the WHY2025 badge

## Badge App Linking

Badge apps still need to link as shared objects, keep `main` as the entry point, and prune exports for the loader. That policy now lives at the app boundary instead of the workspace-wide target config.

Keep `-Crelocation-model=pic` in the badge target config, depend on `why2025-badge-app-no-std`, and call its build helper from a tiny local build script. The facade crate consumes the badge-link metadata from `why2025-badge-sys`, so consumer apps do not need a direct dependency on `why2025-badge-build` or a checked-in `retain.txt`.

```toml
[package]
build = "src/build_script.rs"

[dependencies]
why2025-badge-app-no-std.workspace = true

[build-dependencies]
why2025-badge-app-no-std.workspace = true
```

```rust
fn main() {
	why2025_badge_app_no_std::configure_build("src/build_script.rs");
}
```

The example apps in `examples/` use this pattern directly.
