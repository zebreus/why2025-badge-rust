# why2025-badge-sys-bindings

Canonical raw generated bindings for the functions exported by the WHY2025 badge firmware.

This crate is the authoritative raw BadgeVMS ABI artifact for both std and no_std consumers in
this repository. The patched BadgeVMS std port consumes it directly. The sibling
`why2025-badge-sys` crate re-exports the same raw surface and adds wrapper-only behavior such as
Host builds using Emulation and no_std badge-link support.

## Regenerating

Run:

```sh
./rebuild-bindings.sh
```

That command regenerates this crate's `src/bindings.rs` and `src/types.rs`. It also refreshes the
wrapper-local linker coverage file at `../why2025-badge-sys/src/linker_test.rs`.
