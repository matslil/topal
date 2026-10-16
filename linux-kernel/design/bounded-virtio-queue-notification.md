# Bounded virtio queue notification

This record resolves `TK-DEC-100` as the first device-visible submission event
for the activated virtio-rng queue. It notifies queue zero in the region
described by the previously discovered notification capability, using the
separate configuration-access capability without exposing the assigned BAR
address or adding a virtio-specific provider operation.

## Decision

After exact queue activation, the initial driver performs one source-owned
two-byte write of value zero through configuration-access capability `0x84` to
the region described by notification capability `0x70`: BAR 4, region offset
`0x3000`, register offset zero. The value is queue index zero. The selected
queue reported notification offset zero and the notification capability
reported multiplier four, so the effective notification offset remains zero.

Notification uses the existing checked PCI-region write. The provider resolves
and serializes access to the capability-described region but does not infer
queue identity, notification offset arithmetic, submission policy, or virtio
meaning. No readback is required because the notification register represents
an event rather than retained state.

The control buffer is already device-owned and contains an available index of
one published after its descriptor and available entry. Its earlier device-
preparation transition owns any architecture-required visibility maintenance;
the notification cannot substitute for that ownership boundary.

## Architecture pressure test

Virtio PCI notification arithmetic and queue-index payload are shared across
x86-64, AArch64, and RISC-V PCI hosts. An x86 provider may use the PCI
configuration-access capability gateway, while ECAM and mapped-MMIO providers
use their qualified mechanisms. The source request continues to name only
segment/BDF, capability-relative region facts, width, and value.

Virtio-mmio uses a different transport register and requires a separately
qualified mapping, but it can preserve the same higher-level invariant: fully
published device-owned queue state precedes exactly one submission event.

## Deliberate bounds

This increment admits only the queue-zero notification described above and
only after the full activation sequence. It rejects another access or target
capability, BAR, region, offset, width, value, omission, duplication,
reordering, or premature notification.

It does not poll or interrupt on completion, inspect the used ring, return DMA
buffers to CPU ownership, consume entropy, populate `AT_RANDOM`, implement
`getrandom`, or define reset and recovery.
