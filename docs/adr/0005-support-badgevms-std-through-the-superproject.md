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

Release packaging is produced from Rust `x.py dist` artifacts in the superproject, then assembled
into a relocatable archive with checksums and metadata. The public installer downloads or accepts
that archive and registers it with `rustup toolchain link`. Mutable `stage2` directories and
source-tree `rust-src` symlink patching are not supported producer or consumer interfaces.
