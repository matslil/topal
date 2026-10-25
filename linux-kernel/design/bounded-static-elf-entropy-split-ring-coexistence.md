# Bounded static ELF entropy split-ring coexistence

This record resolves `TK-DEC-165` as a reversible qualification transaction
for the first virtio-rng split ring while the static process remains behind its
entropy-acquisition owner. It is a systems-profile refinement only and adds no
core-language element.

## Decision

After `TK-DEC-164`, `static image construction qualify entropy split ring`
shall consume and return the exact acquisition owner. It shall accept only
device identity `initial-virtio-rng-device`, translation-domain identity
`initial-dma-domain`, queue identity `initial-virtio-rng-queue-zero`, and
policy `bounded-static-elf-entropy-split-ring-coexistence`.

The x86-64 provider shall completely revalidate the retained process; replay
the exact negotiation, queue-discovery, bus-master, and two-buffer creation
sequences; and construct the existing exact queue-size-eight descriptor and
available-ring state. Construction shall bind the payload reference without
exposing its device-visible address, store descriptor length and flags, store
available slot zero, and publish available index one last.

Before returning, the transaction shall erase the constructed fields in
reverse publication order, including the opaque 64-bit reference, prove no DMA
identity escaped, reverse bus-master authority, and restore verified device and
configuration-gateway state. The later ordinary entropy sequence shall remain
unconsumed. Success returns the same owner and permits
`TOPAL_KERNEL_STATIC_ELF_ENTROPY_SPLIT_RING_OK`.

## Architecture pressure test

The portable contract names opaque device, domain, and queue identities rather
than physical, virtual, bus, or I/O-virtual addresses. Identity-translated
x86-64 can bind its payload backing directly; AArch64 and RISC-V providers may
bind an IOVA or bounce object and may require noncoherent cache maintenance at
the later ownership transition. Every provider must preserve the same exact
layout facts, available-index-last ordering, reference opacity, reverse-order
erasure, and complete owner/process state.

Completing the provider's already-modeled naturally aligned 8-byte integer
store supports erasing a typed reference without returning its value to source.
This remains provider lowering for an existing systems operation, not a new
language scalar or address facility.

## Alternatives rejected

Leaving the ring live was rejected because it would affect the later ordinary
lifecycle. Rebinding the reference to a synthetic null buffer was rejected
because it would retain a false typed reference. Exposing the stored address so
source could clear it was rejected because it would break DMA opacity.
Introducing a virtio-specific clear intrinsic was rejected because exact-width
zeroing of CPU-owned storage is already represented by the generic bounded
integer-store operation.

## Deliberate bounds

This increment admits exactly one sealed reference binding, the four existing
split-ring stores, and their inverse zero stores inside one transaction. It
rejects any other owner, device, domain, queue, policy, layout, value, order,
omission, duplication, exposed address, final byte state, process mutation, or
recoverable disposition. It does not prepare or publish DMA, enable or notify
the queue, consume entropy, publish `AT_RANDOM`, enter the static process, or
add a syscall.
