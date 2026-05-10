# Threading model

BadgeVMS exposes a Thread as a Task created from a function pointer and user data. Rust `std::thread` needs stronger semantics than the raw BadgeVMS `thread_create`/`wait` API provides, so the std backend owns a Rust thread runtime layered on top of BadgeVMS.

## Core rules

- `std::thread::spawn` uses BadgeVMS `thread_create` only as the execution primitive.
- Rust join semantics are owned by Rust state, not by BadgeVMS `wait`.
- Dropping `JoinHandle` detaches Rust join ownership only.
- TLS destructors run during normal spawned-thread exit.
- Panics abort the App in v1.
- `available_parallelism()` returns `1`.

## Thread state

Each spawned thread gets a Rust-owned control block containing:

- unique Rust thread id;
- optional BadgeVMS pid for diagnostics and wait-draining;
- join state and result storage;
- parker token state;
- TLS teardown state;
- detached/completed reference state.

The Rust thread id must not be just the BadgeVMS pid because pids are reusable after task cleanup.

## Join and detach

`JoinHandle::join` waits on the Rust completion state. Completion is published with release ordering by the spawned thread after the closure returns and TLS teardown has run. The joiner observes it with acquire ordering before taking the result.

Dropping a `JoinHandle` releases the joiner's reference. It does not kill the BadgeVMS Thread and does not try to remove the Task from BadgeVMS.

## Wait-draining runtime

BadgeVMS `wait` observes child Tasks, including both Processes and Threads, after firmware cleanup. The Rust runtime must drain these notifications so short-lived Rust Threads do not fill the parent queue.

The wait-draining runtime must:

- classify pids registered by Rust `std::thread` and drop their notifications;
- classify pids registered by `std::process` and complete the corresponding `ChildState`;
- tolerate pid reuse by caching completion by Rust-owned identity;
- document that raw `process_create`/`wait` interop with Rust-managed children is not promised in v1.

## Park and unpark

The v1 parker is intentionally temporary. It uses Rust-owned shared state and bounded time-based waiting because v1 does not expand the firmware ABI with a wake primitive.

Required token semantics:

- `unpark` before `park` leaves one token.
- `park` consumes exactly one token.
- repeated `unpark` calls coalesce to one token.
- timeout waits return after token consumption or elapsed timeout.
- wakeups use generation counters to avoid lost notifications.

Every temporary time-based wait should carry a TODO pointing at the future firmware-backed implementation seam.

## Stack sizes

`Builder::stack_size` policy:

- `0` means BadgeVMS platform default.
- values below the platform minimum round up.
- values above the firmware-representable `u16` range fail explicitly.
- no silent truncation is allowed.

## Tests

Required tests cover spawn/join result transfer, memory synchronization, detach, TLS destructor execution, panic abort, stack-size boundaries, wait queue draining, park/unpark tokens, park timeouts, `Mutex`, and `Condvar` behavior.
