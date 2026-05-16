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
curl -fsSL https://zebreus.github.io/why2025-badge-rust/install.sh | bash
cargo +badgevms build --target riscv32imafc-unknown-badgevms
```

The public installer hides the internal toolchain name and links the installed toolchain locally as
`badgevms`. Under the hood it still installs the pinned `nightly-2099-01-01` toolchain because
stock rustup rejects arbitrary installable channel names such as `badgevms`.

## Maintainer packaging flow

Build a locally served tree for development:

```sh
tools/badgevms-std/dist-toolchain.sh site http://127.0.0.1:8000/dist
python3 -m http.server 8000 --directory site
```

Then validate from another shell:

```sh
tools/badgevms-std/checks/run-dist-smoke.sh http://127.0.0.1:8000 nightly-2099-01-01
```

The published Pages root also contains `install.sh`. You can test the public-facing installer
against a local server by overriding the Pages root:

```sh
BADGEVMS_PAGES_ROOT=http://127.0.0.1:8000 bash site/install.sh
```

The manifest embeds absolute component URLs. CI deploys the public `site/` shape to GitHub Pages,
but does not run rustup smoke tests inside the deploy workflow. After deployment, validate the
public Pages URL locally:

```sh
curl -fsSL https://zebreus.github.io/why2025-badge-rust/install.sh | bash
tools/badgevms-std/checks/run-dist-smoke.sh https://zebreus.github.io/why2025-badge-rust nightly-2099-01-01
```

The std port uses `why2025-badge-sys-bindings` as the raw BadgeVMS ABI source. The Rust fork should
not carry a BadgeVMS-specific `library/libc` fork.
