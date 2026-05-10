# why2025-badge-sys-bindings

Canonical raw generated bindings for the functions exported by the WHY2025 badge firmware.

## Regenerating

Run:

```sh
./rebuild-bindings.sh
```

That command regenerates this crate's `src/bindings.rs` and `src/types.rs`. It also refreshes the
wrapper-local linker coverage file at `../why2025-badge-sys/src/linker_test.rs`.
