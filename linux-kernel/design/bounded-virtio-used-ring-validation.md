# Bounded virtio used-ring validation

This record resolves `TK-DEC-102` as the metadata-validation step after the
first acknowledged virtio-rng completion signal. It returns only the shared
virtqueue-control buffer to CPU ownership and validates the exact device-written
used entry without granting access to the entropy payload.

## Decision

After `TK-DEC-101`, one generic DMA completion operation consumes the
device-owned control buffer and produces a CPU-owned control buffer with
provider-completed device-to-CPU visibility. The request retains buffer identity
one, device identity one, translation-domain identity one, bidirectional
direction, provider-managed visibility, and the virtqueue-control role.

Checked source then performs four generic bounded integer read-equals
operations against that CPU-owned buffer:

| Used-ring field | Offset | Width | Expected value |
| --- | ---: | ---: | ---: |
| flags | 148 | 2 | 0 |
| index | 150 | 2 | 1 |
| first element identifier | 152 | 4 | 0 |
| first element length | 156 | 4 | 32 |

The absolute control-buffer offsets follow the queue-size-eight split-ring
layout constructed earlier: the used ring begins at 148, so these fields are
at used-ring-relative offsets 0, 2, 4, and 8.
The reads prove that exactly descriptor zero completed with the requested
32-byte length. The payload buffer remains device-owned throughout, so neither
the completed-length fact nor control-buffer ownership authorizes payload
inspection.

## Architecture pressure test

The ownership transition expresses device-to-CPU visibility rather than an x86
cache instruction. A coherent x86-64 provider may need only ordering and
metadata checks; noncoherent AArch64 or RISC-V providers must perform the
required cache maintenance, DMA synchronization, and barriers before returning
CPU authority. IOMMU and bounce-buffer providers may also synchronize or copy
provider-private storage. The source-visible buffer identity and used-ring
layout remain unchanged.

Integer validation is ordinary CPU-owned DMA-buffer access. It exposes no
address, cache operation, fence, physical register, or provider mapping token.

## Deliberate bounds

This increment admits exactly one control-buffer completion transition followed
by the four reads above. It rejects another buffer, role, device, domain,
direction, visibility policy, offset, width, value, omission, duplication,
reordering, or access before CPU ownership.

It does not return payload ownership, inspect entropy, recycle or renotify the
queue, populate `AT_RANDOM`, implement `getrandom`, configure asynchronous
interrupt routing, or define reset and recovery.
