# Bounded general standard-input read

This record resolves `TK-DEC-072` as a systems-profile refinement. It replaces
the exact input byte, candidate, return site, and length oracle of the initial
stdin qualification with a bounded Linux-compatible adapter for caller-
selected writable user memory. It changes no core-language rule and remains
narrower than the pinned Linux `read(2)` interface.

## Decision

The initial-process personality accepts `read` when:

- descriptor is zero;
- the return instruction lies in the finalized executable mapping;
- length is from one through 256 bytes;
- the complete half-open destination extent is overflow-free and contained in
  a provider-known writable user mapping; and
- the user context is live in the dispatchable state.

Invalid descriptor or length returns the Linux-compatible `-EBADF` or
`-EINVAL` result selected by the personality. An invalid destination returns
`-EFAULT` before input consumption or user-memory access. A valid request
waits for exactly the requested bounded count and stages every received byte
in at least 256 bytes of private kernel storage. Only after complete reception
does it copy the staged prefix into user memory under the generated recovery
protocol, discard the candidate, and return the requested length.

The qualification harness injects the deterministic four-byte sequence
`read`, and the initial ELF validates it after return. Neither the disposition
nor the provider embeds or compares those input bytes. Structural evidence
must prove that copy length and result come from the checked syscall argument.

## Isolation and failure contract

The size cap bounds kernel stack use and one dispatch's input consumption.
Range checks reject arithmetic wrap, zero length, cross-hole extents, read-only
or non-user ranges, and buffers extending past a writable mapping before the
provider polls input. Complete private reception precedes the first user-memory
write; a recovered copy fault publishes no successful result.

Mapping change is impossible during this single-context profile. Unexpected
faults after a range was proved writable contradict the closed address-space
state; the generated recovery remains fail-closed for that invariant breach.
The pointer and private staging do not survive return.

## Alternatives considered

A one-byte loop in application code would preserve the provider oracle and
would not qualify argument-derived extents. Returning whatever bytes are
immediately available would require a readiness, partial-result, and signal
policy not yet present. The selected exact-fill operation is a legal bounded
blocking behavior and keeps those later choices explicit.

## Cross-architecture pressure

The portable contract is an exact-fill bounded standard-input read into a
proved writable user extent, with private staging before user publication. It
does not prescribe syscall numbers, registers, instructions, page-table
layout, copy-loop encoding, UART type, polling, interrupts, or harness
transport. Each provider validates the return continuation and destination
against its own address-space representation.

This increment does not admit descriptor tables, end-of-file, partial reads,
nonblocking mode, signals, restart, terminal line discipline, readiness
polling, files, pipes, sockets, vectored I/O, asynchronous I/O, or requests
above 256 bytes. Those remain explicit compatibility gaps.
