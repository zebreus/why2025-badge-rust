# Domain Docs

How the engineering skills should consume this repo's domain documentation when exploring the codebase.

## Before exploring, read these

- **`CONTEXT.md`** at the repo root, if it exists.
- **`docs/adr/`** for architectural decisions relevant to the area being changed.

If these files do not exist yet, proceed silently. Do not flag their absence or suggest creating them upfront.

## File structure

This repo is configured as a single-context repo.

```
/
├── CONTEXT.md
├── docs/adr/
│   ├── 0001-app-owned-badge-linking.md
│   ├── 0002-ctype-export-normalization.md
│   └── 0003-use-dlsym-for-host-libc-overlap.md
└── src/
```

Use the root `docs/adr/` directory as the canonical location for ADRs in this repo. Do not add new crate-local `docs/adr/` directories for this single-context layout.

## Use the glossary's vocabulary

When your output names a domain concept, use the term as defined in `CONTEXT.md` when that file exists. Avoid inventing synonyms for existing project language.

## Flag ADR conflicts

If your output contradicts an existing ADR, surface it explicitly instead of silently overriding it.