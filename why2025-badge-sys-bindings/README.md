# why2025-badge-sys-bindings

Generated Rust bindings for the functions exported by the WHY2025 badge firmware.

The repository's raw-ABI boundary around this crate is recorded in
[ADR 0004](../docs/adr/0004-canonical-badgevms-abi-layering.md).

## Regenerating

Run:

```sh
./rebuild-bindings.sh
```

That command regenerates this crate's `src/bindings.rs` and `src/types.rs`. It also refreshes the
wrapper-local linker coverage file at `../why2025-badge-sys/src/linker_test.rs`.
