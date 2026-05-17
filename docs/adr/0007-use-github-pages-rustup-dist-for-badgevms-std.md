---
status: accepted
---

# Use a GitHub Pages rustup dist server for BadgeVMS std

BadgeVMS std toolchains are distributed as a rustup dist server hosted from GitHub Pages instead of
GitHub Release archives registered with `rustup toolchain link`.

The producer remains the superproject checkout. It owns the patched Rust checkout, the canonical
BadgeVMS ABI crates, distribution scripts, and validation. The distribution script uses Rust's
standard `x.py dist` flow to produce component tarballs and Rust's `build-manifest` tool to generate
the v2 rustup channel manifest under a top-level `dist/` directory.

The first supported train is nightly-only and the first supported host is
`x86_64-unknown-linux-gnu`. The BadgeVMS target is `riscv32imafc-unknown-badgevms` and is installed
with `rustup target add`, matching rustup's standard cross-target model. `rust-src` is published as
an installable component for build-std and source-based tooling workflows.

Stock rustup rejects arbitrary installable channel names such as `badgevms`; those names are only
supported for linked custom toolchains. To coexist with official Rust toolchains in a normal rustup
home without a wrapper, the public install name is `nightly-2099-01-01`. The future-dated nightly
name is accepted by stock rustup, routes to the repository's own dist server via `RUSTUP_DIST_SERVER`,
and avoids colliding with official nightly installs.

The supported consumer flow is:

```sh
export RUSTUP_DIST_SERVER=https://zebreus.github.io/why2025-badge-rust
rustup toolchain install nightly-2099-01-01 --profile minimal
rustup target add riscv32imafc-unknown-badgevms --toolchain nightly-2099-01-01
rustup component add rust-src --toolchain nightly-2099-01-01
```

The old archive installer path is removed as a public distribution mechanism after the Pages-backed
dist install passes local and post-deployment Ubuntu smoke tests.
