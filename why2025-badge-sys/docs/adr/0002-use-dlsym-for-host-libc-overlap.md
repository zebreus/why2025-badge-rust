---
status: accepted
---

# Use `dlsym(RTLD_NEXT)` for selected host libc overlap on Linux

For Linux host builds, `why2025-badge-sys` will export selected raw libc-shaped symbols itself and forward them to the real host libc with `dlsym(RTLD_NEXT)`. We chose this because it works for direct Rust use and for C consumers of `staticlib` and `cdylib` outputs without requiring consumer-controlled `--wrap` flags, while the `--wrap` approach pushes final-link control onto every consumer and the ELF `.symver` experiment still self-resolved and crashed at runtime.

## Considered Options

- Consumer-controlled `--wrap`: rejected because it requires final-link flags the crate cannot impose for `staticlib` or ordinary downstream binaries.
- ELF versioned libc imports with `.symver`: rejected because the pilot compiled but recursed or segfaulted in direct Rust, `staticlib`, and `cdylib` use.
- Explicit fallbacks only: rejected because it leaves too much of the upstream wrapped libc surface unrepresented on the host.

## Consequences

- This decision is Linux/ELF-oriented and depends on dynamic-loader behavior.
- Interposition remains selective and symbol-by-symbol; it is not a blanket libc replacement policy.
- Some overlapping symbols still need non-dlsym strategies when the badge ABI diverges from the host ABI, such as the mediated host-regex bridge used for the `reg*` family.
- Allocator, termination, and variadic families still need follow-up work.