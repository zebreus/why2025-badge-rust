---
status: superseded-by-0007-for-distribution
---

# Support the BadgeVMS std target through the superproject checkout

The supported way to build and document the BadgeVMS std target is the superproject checkout that
contains the patched Rust toolchain, workspace crates, scripts, tests, and ADRs together. We chose
this because the target currently depends on coordinated changes across the toolchain fork and this
repository, and treating detached pieces as first-class entrypoints would duplicate setup and let
the documented workflow drift from the implementation.

The superproject is the supported producer entrypoint. End users should consume toolchain artifacts
generated from that checkout, so they do not need the Rust fork, submodules, or local `rust-src`
patching to build BadgeVMS std Apps.

ADR 0007 supersedes this ADR's original archive-based distribution decision. Consumer distribution
now uses a GitHub Pages-backed rustup dist server instead of an archived install prefix plus
`rustup toolchain link`. Mutable `stage1`/`stage2` directories remain local Rust-fork iteration
interfaces only.
