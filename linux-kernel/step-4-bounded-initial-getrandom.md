# Step 4 bounded initial `getrandom` delivery

## Outcome

The Topal kernel now performs a second conditioned CRNG extraction and consumes
opaque output identity two through
`topal.systems.entropy.output.reserve-initial-user-random/1` before entering the
initial process. The x86-64 provider maps that portable reservation to Linux
syscall 318 and retains only a private, single-use reference to the 32-byte
output.

The bounded syscall accepts zero length without consumption and accepts one
nonempty request of at most 16 bytes with any current compatible Linux flag
combination. It rejects an excessive length, an unknown flag bit, or the
`GRND_RANDOM | GRND_INSECURE` combination with `-EINVAL`; it rejects an invalid
complete writable range with `-EFAULT`. Failed validation does not consume the
reservation. Success copies and verifies the requested prefix, discards the
unselected bytes, wipes and verifies the complete provider output, clears the
private reference, advances the process lifecycle, and returns the requested
length.

The ring-3 probe exercises zero length, unknown flags, incompatible flags, a
guard-crossing range, and one successful 16-byte request. It reads both returned
qwords without constraining their secret-derived values. Process teardown is
admitted only after the successful request has consumed the reservation.

## Validation boundary

Rules `TOPAL-SYSTEMS-ENTROPY-RESERVE-001` and
`TOPAL-SYSTEMS-USER-GETRANDOM-001` require the fresh generation-two-to-three
extraction, exact reservation, target ABI mapping, non-consuming error paths,
copy-before-wipe order, and single successful consumption. Negative language,
semantic, and compiler tests reject a malformed reservation, omission,
duplication, reordering, premature entry, output reuse, invalid extents or flag
policy, and teardown with a live reservation.

Provider-object inspection proves native syscall 318, direct GS-relative
private state, complete range checks before copying, all four output-qword
wipes, lifecycle publication, and the five ring-3 calls. Direct GS-relative
state is important on x86-64 because state offset `0x68` contains the retained
page-table backing identity rather than another pointer to the state object.

The checked root contains 143 fatal edges, an exact 305-event semantic trace,
2,787 relocations, and 2,408 UART calls. The sealed provider plan contains 83
operations. Artifact schema `/96`, provider and provider-object schemas `/88`,
artifact-root schema `/94`, boot-adapter schema `/56`, and gate schema `/98`
identify the contract.

## Physical evidence

The gate retains the pinned `pc-q35-10.2` machine, `qemu64-v1` CPU, SeaBIOS,
ISA serial console, and one `virtio-rng-pci-non-transitional` function at
`00:05.0`. `TOPAL_KERNEL_USER_GETRANDOM_OK` is reachable only after the ring-3
argument, guard-range, copy, wipe, consumption, and teardown checks complete.

Two successful independent pinned-QEMU runs reached all 74 ordered
observations. Hashes for the recorded run are:

- linked kernel: `b87c5fc075103057d395d8bfd64a5c284a9dbbacbcea886df9c510867d67d6fd`;
- boot image: `e8cb3277446e371862258ccad862abd9fd1131cee6fb134b43fd8bfaf26fb101`;
- boot resource: `612e211b86636c9fff22ca632cdb3d05a7e27952fca76ab2d1e85e154f3dd7d2`;
- generated image: `bb272c3e453f8df338e072da3e49c500d8357bbe21f6687af2869d2de827d3df`;
- supplied image: `955a0ce0e548aec6afa24fbabe5f29c5677e6e961d1119439c8a4731c1ad36f8`;
- artifact provenance: `23b7b8b547bf4553786098c402db417926f92d10a4c4726c5c0139ecdf2e1d78`;
- boot provenance: `b54cb8955284713073ffe4f31e9294cdbadaf9380c777883f57de050ebef7375`;
- serial output: `a6120938276b4100697ce0a0841a4e0b004dd887e2e366eb20834c1693f09c28`;
- evidence: `36fcb9020c23cf03a150ab2c5f197cce93dcb7801238ec8d287b8cbe708ed5b7`.

## Risk and remaining boundary

Risk is high because this increment exposes secret-derived bytes through a
native user ABI and adds state to syscall entry and teardown. Review covered
affine ownership, generation ordering, flag compatibility, overflow and guard
range checks, non-consuming failures, direct GS-relative state identity,
copy-before-wipe order, complete wiping, one-shot consumption, structural
inspection, and duplicate physical execution.

Repeated successful calls, requests larger than 16 bytes, blocking before CRNG
readiness, signal interruption, `/dev/random`, `/dev/urandom`, vDSO
`getrandom`, multi-block generation, reseeding, per-CPU generators, batching,
queue recycling and renotification, asynchronous interrupt routing, reset, and
recovery remain unavailable.
