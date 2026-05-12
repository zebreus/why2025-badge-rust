---
status: accepted
---

# Support the BadgeVMS std target through the superproject checkout

The supported way to build and document the BadgeVMS std target is the superproject checkout that
contains the patched Rust toolchain, workspace crates, scripts, tests, and ADRs together. We chose
this because the target currently depends on coordinated changes across the toolchain fork and this
repository, and treating detached pieces as first-class entrypoints would duplicate setup and let
the documented workflow drift from the implementation.

The superproject is the supported producer entrypoint. End users should consume packaged,
rustup-linkable toolchain archives generated from that checkout, so they do not need the Rust fork,
submodules, or local `rust-src` patching to build BadgeVMS std Apps.

Release packaging is produced in the superproject with Rust's standard `x.py install` flow into a
clean prefix. The only separately installed component is Rust's own `rust-src` dist component,
because `x.py install src` also enters target documentation paths that are not part of the consumer
toolchain contract. The installed prefix is archived with checksums and metadata. The public
installer downloads or accepts that archive and registers it with `rustup toolchain link`. Mutable
`stage2` directories and source-tree `rust-src` symlink patching are not supported release or
consumer interfaces; direct `stage1`/`stage2` rustup links are only for local Rust-fork iteration.
