# Process model

`std::process::Command` is supported in v1 only as a narrow raw BadgeVMS executable-path launcher.

## Supported subset

- `Command::new` with a BadgeVMS executable path.
- Exact argv construction.
- `spawn`.
- `try_wait`.
- blocking `wait`.
- `status`.
- default BadgeVMS child stdio and environment behavior.

## Path lookup

`Command::new` accepts the raw path provided by the caller. It uses BadgeVMS path and logical-name resolution through `process_create`.

It does not:

- search Unix `PATH`;
- launch host executables;
- interpret an Installed App ID;
- infer `APP:` context;
- read an App manifest.

Installed App launch by App ID belongs in a separate BadgeVMS-specific API, not in `std::process`.

## Argv shaping

The child-facing contract matches raw BadgeVMS `process_create`:

- if no explicit args are supplied, call `process_create` with `argc = 0` and `argv = NULL`, so firmware presents `argv[0]` as the executable path;
- if explicit args are supplied, pass `argv[0] = executable path` followed by the exact arguments;
- do not implement Unix `arg0` overrides unless the override preserves this contract.

## Child state

Rust owns child completion tracking after launch:

- register the child by Rust-owned child identity and BadgeVMS pid;
- central wait draining observes completion;
- completion is cached after first observation;
- repeated `try_wait`, `wait`, and `status` stay coherent even if BadgeVMS recycles the pid.

V1 does not promise same-parent interoperability between raw `process_create`/`wait` calls and Rust-managed children.

## Exit status

BadgeVMS currently does not expose real child exit codes to this contract. V1 reports normal completion with synthetic exit code `0` and carries an explicit TODO to switch to real exit codes when firmware supports them.

Do not synthesize Unix signal termination, stop, continue, or core-dump status.

## Unsupported features

These fail explicitly when process launch or use is attempted:

- environment mutation;
- cwd override;
- stdio redirection;
- pipes;
- `output`;
- `kill`;
- `fork`;
- `exec`;
- richer child-status control.

## Tests

Required tests cover raw path launch, no-arg `argv[0]`, explicit args, Rust and C child Apps, `spawn`, `try_wait`, `wait`, `status`, repeated post-exit observation, pid reuse, default TT01 stdio, unsupported builders, no App-ID launch, no PATH search, synthetic zero status, and no signal/core-dump fabrication.
