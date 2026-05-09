---
status: accepted
---

# Normalize `_ctype_` to `_ctype_b`

The upstream BadgeVMS app-import ABI manifest in `firmware/badgevms/symbols.yml` exports both `_ctype_` and `_ctype_b`, but the public C headers only declare `_ctype_b` and expose `_ctype_` as a macro alias. We chose to represent the upstream `_ctype_` export through the real declared backing object `_ctype_b` because a header-driven binding generator can observe `_ctype_b` without inventing declarations, and fabricating extra libc-shaped exports in host `std` builds risks unsafe interposition on the host process. The settled pattern is: binding generation normalizes `_ctype_` to `_ctype_b`, host emulation exports `_ctype_b` instead of a synthetic `_ctype_` symbol, and audits treat `_ctype_` as an intentional normalization rather than an accidental omission.

## Considered Options

- Represent the upstream `_ctype_` export through the real declared backing object `_ctype_b`: accepted because it matches the public headers and avoids inventing linkage the headers do not describe.
- Export a synthetic `_ctype_` symbol for exact manifest-name parity: rejected because the headers do not declare a real `_ctype_` object and a fabricated host export would have unclear linkage semantics.
- Attempt broad host-side override of libc-owned symbols in `std` builds: rejected because `std` links libc into the host process, so blanket replacement would change host process behavior rather than only BadgeVMS guest resolution.

## Consequences

- The raw Rust surface is not an exact one-name-for-one-name mirror of the upstream manifest for this specific symbol.
- The representation matches the upstream header reality and avoids fabricating a host export whose behavior is unclear.
- Future work on exact-name libc-like exports must be selective and explicit rather than a blanket host libc override.
- A future selective wrapper approach can revisit whether `_ctype_` should become an exact exported alias instead of a normalization.