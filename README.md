# why2025-badge-rust
Various support packages for using rust on the WHY2025 badge

## Badge App Linking

Badge apps still need to link as shared objects, keep `main` as the entry point, and prune exports for the loader. That policy now lives at the app boundary instead of the workspace-wide target config.

Keep `-Crelocation-model=pic` in the badge target config, enable `why2025-badge-sys` with `badge-app-link`, add `why2025-badge-build` as a build dependency, and call the helper from a tiny build script. The retain-symbols file is generated in `OUT_DIR`, so consumers no longer need a checked-in `retain.txt`.

```toml
[package]
build = "src/build_script.rs"

[dependencies]
why2025-badge-sys = { workspace = true, features = ["badge-app-link"] }

[build-dependencies]
why2025-badge-build.workspace = true
```

```rust
fn main() {
	why2025_badge_build::configure("src/build_script.rs");
}
```

The example apps in `examples/` use this pattern directly.
