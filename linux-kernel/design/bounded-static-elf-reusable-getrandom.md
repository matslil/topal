# Bounded static ELF reusable `getrandom`

## Decision

The randomized static-process owner may refine initial-random delivery into one
process-lifetime affine random service. Before the provider destroys its
entropy workspace, it moves both the already-reserved 16-byte output and the
generation-two ChaCha20 key into provider-private process storage. The global
key, output, generator, pool, sample, DMA, transport, and device resources are
still wiped and destroyed before process entry. The service is the only owner
of the moved key and reservation.

The first successful nonempty request consumes the reserved output exactly as
the existing initial request does. A later successful request first performs
the already-qualified ChaCha20 extraction and fast-key-erasure transition: the
persistent key is replaced before any output publication, the selected prefix
is copied and verified, and the complete temporary output is wiped and
verified. No unused remainder survives a call.

The service accepts zero length without state change, lengths from one through
16, and current Linux flag bits except the invalid `GRND_RANDOM |
GRND_INSECURE` combination. Unknown flags, the invalid combination, or an
oversized request return `-EINVAL` without consuming output or evolving the
key. A candidate must name a complete overflow-free extent in the validated
writable static mapping; failure returns `-EFAULT` without state change.

The clean-room x86-64 qualification preserves the first observed eight-byte
nonblocking request, clears its scratch destination, then performs a second
eight-byte nonblocking request at `0x005f5e70`. It observes the opaque result
without comparing it to a constant and clears the destination before terminal
dispatch. The second success proves on-demand extraction after initial-output
consumption.

Terminal handling accepts a service after at least the observed first request,
wipes and verifies the live key, reserved output, temporary output, counters,
and provider-private references, and clears the service marker before process
restoration. Qualification additionally requires the ordered repeated-request
marker. Hardware Accessed/Dirty status is normalized for the complete writable
static segment because any accepted result extent may set those provider-owned
page-table bits.

Source names a reusable process random service, not keys, output bytes,
addresses, syscall numbers, registers, page tables, or ChaCha20 mechanics. The
portable boundary is exclusive generator ownership, validation before
evolution, key replacement before publication, complete per-call output wipe,
and terminal destruction. Architecture providers own native ABI selection,
fault containment, private storage, stack switching, and translation status.

This systems-only refinement does not add requests larger than 16 bytes,
blocking before readiness, signal interruption, reseeding, concurrency,
fork/exec inheritance, `/dev/random`, `/dev/urandom`, vDSO delivery,
redistributed BusyBox, or a prompt.

## Alternatives rejected

Pre-extracting another fixed output was rejected because it would only extend
the one-shot bound and keep more secret material alive before demand. Reusing
the consumed reservation was rejected because it would repeat bytes. Keeping
the provider-global key was rejected because it would outlive its acquisition
owner and weaken rollback. Running extraction on the user stack was rejected
because privileged scratch must not depend on writable application state.
