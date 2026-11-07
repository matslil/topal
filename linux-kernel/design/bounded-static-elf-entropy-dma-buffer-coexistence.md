# Bounded static ELF entropy DMA-buffer coexistence

This record resolves `TK-DEC-164` as a reversible qualification transaction
for the two sealed virtio-rng DMA buffers while the static process remains
behind its entropy-acquisition owner. It is a systems-profile refinement only
and adds no core-language element.

## Decision

After `TK-DEC-163`, `static image construction qualify entropy dma buffers`
shall consume and return the exact acquisition owner. It shall accept only
device identity `initial-virtio-rng-device`, translation-domain identity
`initial-dma-domain`, and policy
`bounded-static-elf-entropy-dma-buffer-coexistence`.

The x86-64 provider shall completely revalidate the retained process, replay
the exact negotiation and queue-discovery sequences, enable bus mastering with
the exact masked transition, and qualify the existing exact two-buffer DMA
creation sequence. The transaction shall prove the sealed 4,096-byte aligned,
disjoint, initially zero control and payload backing objects and their exact
device, domain, direction, visibility, and role facts. No buffer identity shall
escape the transaction. It shall then perform the inverse masked bus-master
transition and verified device/gateway rollback. The later ordinary sequence
shall remain unconsumed. Success returns the same owner and permits
`TOPAL_KERNEL_STATIC_ELF_ENTROPY_DMA_BUFFERS_OK`.

## Architecture pressure test

The portable contract names opaque device and translation-domain identities,
not physical, virtual, bus, or I/O-virtual addresses. X86-64 may bind the
transaction to coherent identity-addressed backing; AArch64 and RISC-V
providers may use an IOMMU, noncoherent cache maintenance, or bounce storage.
All must preserve alignment, disjointness, initial zero state, exact roles,
transactional reclamation, and complete process ownership.

## Alternatives rejected

Returning the buffers beside the acquisition owner was rejected because later
ring construction needs its own explicit owner refinement. Reusing ordinary
frames was rejected because it omits device, domain, direction, and visibility
authority. Leaving bus mastering or candidate buffers live was rejected because
it would affect the later ordinary lifecycle.

## Deliberate bounds

This increment admits exactly the sealed control and payload candidates inside
one transaction and returns no DMA resource. It rejects any other owner,
device, domain, policy, request, order, omission, duplication, overlap,
alignment, initial contents, final state, process mutation, or recoverable
disposition. It does not construct or enable a queue, grant device ownership,
notify the device, consume entropy, publish `AT_RANDOM`, enter the static
process, or add a syscall.
