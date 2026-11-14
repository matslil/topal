# Bounded virtio payload completion

This record resolves `TK-DEC-103` as the ownership step after validation of the
first virtio-rng used entry. It returns the completed entropy payload buffer to
CPU ownership without inspecting, interpreting, or exporting any payload byte.

## Decision

After `TK-DEC-102` proves that descriptor zero completed with length 32, one
generic DMA completion operation consumes device-owned payload buffer identity
two and produces the same opaque identity in `CpuOwned` state. The request
retains device identity one, translation-domain identity one,
device-to-memory direction, provider-managed visibility, and the
`EntropyPayload` role.

The prior descriptor binding proves that descriptor zero names payload buffer
two at offset zero, and the used entry proves that the device completed exactly
32 bytes. The completion transition therefore has evidence for the submitted
operation that owned the payload. It does not itself read the bytes or assign
an entropy estimate, random-pool credit, secrecy property, or userspace
interface to them.

The already completed control buffer remains CPU-owned. Returning payload
ownership does not recycle either buffer, advance the available ring, or grant
the device continued access.

## Architecture pressure test

The source operation expresses an ownership and device-to-CPU visibility
transition, not a cache instruction. A coherent x86-64 provider may realize it
with checked ordering and metadata only. A noncoherent AArch64 or RISC-V
provider must invalidate or synchronize the completed payload range before
returning CPU authority. An IOMMU or bounce-buffer provider may also unmap,
synchronize, or copy provider-private storage while preserving the same opaque
buffer identity.

No source-visible address, cache-line size, barrier, IOVA, bounce location, or
physical representation is introduced.

## Alternatives rejected

Returning both buffers at interrupt acknowledgement was rejected because it
would conflate transport signalling, used-ring validation, and payload
visibility. Reading payload bytes during completion was rejected because DMA
ownership and entropy consumption are separate protocols. Treating all 256
bits as credited entropy was rejected because trust and credit policy belongs
to the later entropy-pool design, not to the DMA provider.

## Deliberate bounds

This increment admits exactly one payload-buffer completion transition after
the four used-ring reads. It rejects a different buffer, role, device, domain,
direction, visibility policy, omission, duplication, reordering, or premature
completion.

It does not read or compare payload bytes, estimate or credit entropy, create
an entropy pool, populate `AT_RANDOM`, implement `getrandom`, recycle or
renotify the queue, configure asynchronous interrupt routing, reset the device,
or define failure recovery.
