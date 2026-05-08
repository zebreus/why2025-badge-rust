# ADR 0001: Normalize `_ctype_` to `_ctype_b`

## Status

Accepted

## Context

The upstream BadgeVMS app-import ABI manifest in `firmware/badgevms/symbols.yml` exports both
`_ctype_` and `_ctype_b`.

The public C headers do not declare `_ctype_` as a standalone symbol. In upstream `ctype.h`,
`_ctype_` is only a macro alias:

```c
extern const char _ctype_b[];
#define _ctype_ (_ctype_b + _CTYPE_OFFSET)
```

That means a header-driven binding generator can observe `_ctype_b`, but it cannot observe a real
declaration for `_ctype_` without inventing one.

On the host emulator, inventing extra libc-shaped exports is also risky. Rust binaries that use
`std` always link libc, and globally exported replacements for libc names can interpose on the host
process in surprising ways. The crate already has evidence of this class of problem for wrapped
libc-named exports.

More specifically, there is no credible general-purpose plan to "override all C symbols" safely in
the current host build. As long as the host side uses Rust `std`, libc is part of the process, so
exporting replacement globals or functions with libc-owned names changes the behavior of the host
process itself rather than only changing BadgeVMS guest resolution.

## Decision

For now, `why2025-badge-sys` intentionally represents the upstream `_ctype_` export through the
real declared backing object `_ctype_b`.

Concretely:

- binding generation normalizes the manifest entry `_ctype_` to `_ctype_b`
- host emulation exports `_ctype_b`, not a synthetic `_ctype_` symbol
- the project does not attempt broad host-side override of libc-owned symbol names in `std` builds
- audits should treat `_ctype_` as an intentional normalization, not as an accidental missing
  binding

If the project later wants exact exported names for selected libc-like BadgeVMS symbols, the only
acceptable direction is an explicit, opt-in forwarding layer for individual symbols. That would
mean linker- or loader-level interposition such as selected wrapped shims that still call the real
host libc implementation underneath, with Badge-specific behavior added only for narrowly chosen
paths.

## Consequences

- the raw Rust surface is not an exact one-name-for-one-name mirror of the upstream manifest for
  this specific symbol
- the representation matches the upstream header reality and avoids fabricating a host export whose
  linkage semantics are unclear
- future work on exact-name libc-like exports must be selective and explicit, not a blanket host
  libc override
- a future selective wrapper approach can revisit whether `_ctype_` should become an exact exported
  alias instead of a normalization