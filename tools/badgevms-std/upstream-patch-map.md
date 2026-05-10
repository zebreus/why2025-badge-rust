# Upstream Rust patch map

This file tracks the Rust fork work that cannot live directly in this repository.

## Target spec

Add a built-in target for `riscv32imafc-unknown-badgevms` with:

- `target_os = "badgevms"`;
- `target_family = "unix"`;
- 32-bit RISC-V IMACF, little endian, `ilp32f` ABI;
- abort panic;
- PIC output;
- BadgeVMS linker driver.

The JSON file in this directory is a review aid only. The final target must be a built-in Rust target because `std` needs target-specific backend code and link orchestration.

## Link orchestration

Patch rustc so BadgeVMS std Apps link as shared objects with `main` as the entry and a closed export set. A target JSON alone is not enough if retain-symbols/version-script files must be generated dynamically.

The built-in target must force the executable export list to `main`. The current Rust hook point is `TargetOptions::override_export_symbols`, with the GNU linker export-list path adjusted so BadgeVMS executable links use an LD version script instead of `--dynamic-list`. Without that, `--shared` Apps retain thousands of exported Rust/`std` functions in `.dynsym`.

## Standard library backend

Add the BadgeVMS `std` backend modules described in [docs/badgevms-std-target/implementation-map.md](../../docs/badgevms-std-target/implementation-map.md). The current fork may reuse targeted Unix PAL branches where the firmware ABI is fd/socket/process-shaped; move to a dedicated `sys/pal/badgevms` tree if those branches stop being shallow.

## Test selection

Start from upstream std tests for:

- thread spawn/join/park;
- sync mutex/condvar;
- time and sleep;
- filesystem and io;
- process spawn/wait;
- TCP networking;
- unsupported targets.

Skip or adapt tests that assume POSIX cwd, signals, pipes, env mutation, descriptor duplication, Unix-domain sockets, UDP, or full socket options.
