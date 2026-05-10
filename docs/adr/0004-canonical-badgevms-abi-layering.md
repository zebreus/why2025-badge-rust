---
status: accepted
---

# Keep the canonical BadgeVMS ABI in `why2025-badge-sys-bindings`

This repository records the raw BadgeVMS firmware ABI in `why2025-badge-sys-bindings`, and both the
no_std workspace crates and the patched BadgeVMS std port build outward from that crate. We chose
this boundary so the raw ABI has one owner, `why2025-badge-sys` stays a wrapper layer for Emulation
and badge-link behavior, and Rust-specific std semantics stay in the BadgeVMS std PAL instead of
leaking into shared repo crates.
