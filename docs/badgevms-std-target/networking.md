# Networking

V1 networking is limited to TCP plus address resolution.

## Supported

- `TcpStream`.
- `TcpListener`.
- `SocketAddr` conversion backed by real BadgeVMS socket address types.
- `ToSocketAddrs` backed by `getaddrinfo` and `freeaddrinfo`.
- fd-backed read/write on TCP sockets.

## Unsupported

- `UdpSocket`.
- Unix-domain/local sockets.
- Datagram APIs.
- Socket options without real BadgeVMS backing.
- Nonblocking mode unless real firmware support exists.
- Descriptor clone unless real descriptor duplication exists.
- `shutdown`, TTL, nodelay, local addr, and peer addr unless corresponding firmware symbols are verified.

Unsupported networking APIs must fail through the shared unsupported layer, not by pretending to work.

## Tests

Tests are split by fixture requirements:

- compile-only unsupported checks always run;
- TCP connect/read/write tests require a reachable fixture;
- listener accept tests require loopback or a BadgeVMS network fixture;
- name-resolution tests require configured DNS or a fixture resolver.

Network-dependent tests must be skipped explicitly when the fixture is unavailable, not silently ignored.
