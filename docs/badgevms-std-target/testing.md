# Testing and release gates

Tests must assert externally visible platform behavior, not private helper layout.

## Test layers

1. Host checks for this repository: formatting, Host builds using Emulation, no_std BadgeVMS builds, docs/scripts examples.
2. Toolchain compile checks: patched Rust fork builds the BadgeVMS target and `std`.
3. Link artifact checks: generated App is a PIC shared object with entry `main`, closed exports, and expected imports.
4. Compile-only std examples: std example crates type-check for the BadgeVMS target.
5. On-device conformance Apps: run on BadgeVMS and print machine-readable PASS/FAIL records to TT01.
6. Network fixture tests: TCP and DNS tests when hardware/network fixtures are configured.
7. Distribution smoke: fresh clone, local `rustup toolchain link`, build std App, inspect ELF, optionally run on BadgeVMS.

## Required conformance areas

- target cfg identity;
- linker artifact shape;
- thread spawn/join/detach/TLS/park/unpark/stack policy/panic abort;
- `Mutex` and `Condvar`;
- monotonic/system time and sleep;
- native BadgeVMS filesystem paths;
- cwd unsupported behavior;
- stdio and panic output;
- environment lookup and enumeration asymmetry;
- process argv, wait/status, pid reuse, unsupported builders;
- TCP and address resolution;
- explicit failures for UDP, Unix sockets, symlinks, uid/gid, POSIX mode mutation, descriptor duplication, pipes, env mutation, cwd, PATH search, App-ID launch, and signal/core status.

## Release gates

Local toolchain release requires:

- patched toolchain build succeeds;
- std hello-world builds;
- ELF artifact checks pass;
- thread/sync conformance passes;
- time/fs/stdio/env conformance passes;
- process subset conformance passes;
- unsupported behavior conformance passes;
- docs support matrix matches test results.

Hosted rustup distribution requires all local gates plus clean-rustup-home hosted install tests.
