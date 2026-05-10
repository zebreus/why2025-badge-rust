# BadgeVMS std conformance tests

This directory defines the runtime conformance suites for the BadgeVMS std target. The actual executable test Apps live in the patched Rust fork and in std example crates as they become runnable on BadgeVMS.

The conformance inventory and runner record prefix live in [conformance.toml](conformance.toml).

Suites are split so CI does not conflate Host builds using Emulation, no_std BadgeVMS builds, compile-only std checks, hardware-required std checks, and network-required std checks.
