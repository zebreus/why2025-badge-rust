---
status: accepted
---

# Support the BadgeVMS std target through the superproject checkout

The supported way to build and document the BadgeVMS std target is the superproject checkout that
contains the patched Rust toolchain, workspace crates, scripts, tests, and ADRs together. We chose
this because the target currently depends on coordinated changes across the toolchain fork and this
repository, and treating detached pieces as first-class entrypoints would duplicate setup and let
the documented workflow drift from the implementation.
