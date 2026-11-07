# Bounded initial `getrandom`

This record resolves `TK-DEC-112` as the first syscall delivery of
cryptographically generated material. It keeps generator extraction, interface
reservation, user-copy validation, and successful affine consumption distinct.

## Decision

After initial `AT_RANDOM` delivery, the qualification root shall perform one
fresh `topal.systems.entropy.generator.extract/1` transition from ready
generator generation two to generation three and opaque output identity two.
It shall then consume that output into one provider-neutral
`topal.systems.entropy.output.reserve-initial-user-random/1` capability for
process identity one, Linux interface identity one, and user-random service
identity one before entering the initial context.

The reservation names source extent 32, selected offset zero and maximum
delivery extent 16, discarded offset 16 and extent 16, the complete Linux flag
mask 7, incompatible flag combination mask 6, and single-use policy. It gives
source no byte, address, syscall number, register, or copy mechanism. A target
provider maps the service to its native Linux ABI; x86-64 maps it to syscall
318.

The initial x86-64 provider shall accept lengths 1 through 16 with any current
Linux flag combination whose bits are within `GRND_NONBLOCK` (1),
`GRND_RANDOM` (2), and `GRND_INSECURE` (4), except combinations containing
both `GRND_RANDOM` and `GRND_INSECURE`. Because the CRNG is already ready, the
valid flags do not change this bounded call's output. Zero length returns zero
without consuming the reservation. Unknown bits, the incompatible combination,
or a request larger than 16 return `-EINVAL` without consumption. An invalid
complete writable extent returns `-EFAULT` without consumption.

For the first valid nonempty request, the provider validates the complete user
extent before copying, copies and verifies exactly the requested prefix,
discards the remainder of the selected 16-byte extent and bytes 16 through 31,
wipes and verifies all 32 provider-output bytes, clears its private reservation
pointer, and returns the requested length. The reservation and output are then
consumed. The ring-3 probe exercises zero length, unknown flags, the invalid
`GRND_RANDOM | GRND_INSECURE` combination, a guard-crossing extent, and one
successful 16-byte request; it reads both returned qwords without comparing
them with a constant.

Current upstream Linux identifies `getrandom` as its primary modern userspace
RNG interface, assigns native x86-64 syscall number 318, admits the same three
flag bits, rejects unknown bits and the insecure/random combination, and uses
CRNG fast key erasure before copying to userspace. This bounded contract keeps
those observable argument results while making its one-output lifetime and
qualification limit explicit.

## Architecture pressure test

The reservation, affine output, maximum extent, flag policy, complete-range
validation, copy-before-wipe ordering, and user-visible results are portable.
An x86-64 provider selects syscall 318 and its entry registers; AArch64 and
RISC-V providers select their own Linux syscall tables and copy-fault
mechanisms. No architecture exposes an output address or instruction to source.

The initial 16-byte bound is independent of pointer width and page-table shape.
A future general service can repeat extraction and accept larger requests
without changing this capability's separation between generator state, opaque
output, ABI reservation, and fault-contained delivery.

## Alternatives rejected

Reusing `AT_RANDOM` bytes was rejected because the stack object belongs to the
process and would couple two independently observable interfaces. Returning raw
virtio-rng payload was rejected because it bypasses conditioning and fast key
erasure. Generating a constant or source-visible byte list was rejected because
it is neither secure nor opaque. Combining the second extraction with the user
copy was rejected because a blocking or faulting interface must not obscure key
replacement. Encoding syscall 318 in portable source was rejected because
Linux syscall numbers are architecture ABI details. Consuming output on an
argument or range error was rejected because failed validation need not destroy
the sole bounded qualification opportunity.

## Deliberate bounds

This increment admits one generation-two-to-three extraction, output identity
two, one reserved initial-process Linux `getrandom` service, at most 16 returned
bytes, every current valid Linux flag combination on an already-ready CRNG, and
one successful nonempty call. It rejects another identity, service, extent,
flag policy, process, premature reservation, omission, duplication, reordering,
partial copy, incomplete wipe, reuse, or live reservation at process teardown.

It does not provide repeated successful calls, requests larger than 16 bytes,
blocking before readiness, signal interruption, `/dev/random`, `/dev/urandom`,
vDSO `getrandom`, multi-block generation, reseeding, per-CPU generators,
batching, queue recycling, asynchronous interrupt routing, reset, or recovery.
