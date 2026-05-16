# BadgeVMS std tooling

These scripts build and verify the BadgeVMS std rustup distribution published by this superproject.
See [ADR 0007](../../docs/adr/0007-use-github-pages-rustup-dist-for-badgevms-std.md) for the
dist-server decision and [ADR 0005](../../docs/adr/0005-support-badgevms-std-through-the-superproject.md)
for the broader superproject ownership boundary.

## Files

- `dist-toolchain.sh` — build a GitHub Pages-ready rustup dist tree. It uses the bundled Rust
  checkout, `x.py dist`, and Rust's `build-manifest` tool to produce a top-level `dist/` directory.
- `checks/verify-toolchain.sh` — verify the built-in target cfg and that the BadgeVMS std library is
  installed for `riscv32imafc-unknown-badgevms`.
- `checks/run-smoke.sh` — build a std example using an already-installed toolchain and inspect the
  resulting ELF.
- `checks/run-dist-smoke.sh` — install from a supplied `RUSTUP_DIST_SERVER`, add the BadgeVMS target
  and `rust-src`, build a fresh hello-world crate, and inspect the ELF.

## Consumer install flow

```sh
export RUSTUP_DIST_SERVER=https://zebreus.github.io/why2025-badge-rust
rustup toolchain install nightly-2099-01-01 --profile minimal
rustup target add riscv32imafc-unknown-badgevms --toolchain nightly-2099-01-01
rustup component add rust-src --toolchain nightly-2099-01-01
cargo +nightly-2099-01-01 build --target riscv32imafc-unknown-badgevms
```

The toolchain is intentionally named `nightly-2099-01-01` because stock rustup rejects arbitrary
installable channel names such as `badgevms`. A future-dated nightly name is accepted by stock rustup
and does not collide with official Rust nightly toolchains in a normal rustup home.

## Maintainer packaging flow

Build a Pages-ready tree locally:

```sh
tools/badgevms-std/dist-toolchain.sh site http://127.0.0.1:8000/dist
python3 -m http.server 8000 --directory site
```

Then validate from another shell:

```sh
tools/badgevms-std/checks/run-dist-smoke.sh http://127.0.0.1:8000 nightly-2099-01-01
```

The manifest embeds absolute component URLs. If you smoke-test a locally served tree before
deploying, build it with the local `/dist` URL, then regenerate the deployable tree from the
existing tarballs:

```sh
BADGEVMS_DIST_REUSE_ARTIFACTS=1 \
  tools/badgevms-std/dist-toolchain.sh site https://zebreus.github.io/why2025-badge-rust/dist
```

CI deploys the same `site/` shape to GitHub Pages and then validates the public Pages URL from a
clean Ubuntu container.

The std port uses `why2025-badge-sys-bindings` as the raw BadgeVMS ABI source. The Rust fork should
not carry a BadgeVMS-specific `library/libc` fork.
