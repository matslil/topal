# Bounded static ELF entropy queue-activation coexistence

This record resolves `TK-DEC-168` as a reversible empty-queue activation
transaction while the static process remains behind its entropy-acquisition
owner. It is a systems-profile refinement only and adds no core-language
element.

## Decision

After `TK-DEC-167`, `static image construction qualify entropy queue activation`
shall consume and return the exact acquisition owner. It shall accept only
device identity `initial-virtio-rng-device`, translation-domain identity
`initial-dma-domain`, queue identity `initial-virtio-rng-queue-zero`, transport
identity `initial-virtio-rng-pci-transport`, and policy
`bounded-static-elf-entropy-empty-queue-activation-coexistence`.

The x86-64 provider shall completely revalidate the retained process and replay
the exact prerequisite sequence through split-ring construction. Before either
buffer leaves CPU ownership, it shall withdraw the available index from one to
zero and read it back, retaining the exact descriptor and available-slot
contents but exposing no available work. It shall then prepare both buffers,
publish all three queue references, enable queue zero with readback, and set
and verify `DRIVER_OK` through the existing generic PCI-region operations.

Without notifying the queue, the transaction shall reset device status to zero
and prove status, queue enable, and all three logical 64-bit queue-address
registers are zero. Only then may it cancel control and payload preparation.
It shall erase and read back every ring field, reverse bus-master authority,
and restore verified configuration-gateway state. No DMA identity shall escape
and the ordinary later sequence shall remain unconsumed. Success returns the
same owner and permits
`TOPAL_KERNEL_STATIC_ELF_ENTROPY_QUEUE_ACTIVATION_OK`.

## Architecture pressure test

The contract names queue availability, activation, reset, and ownership rather
than register or instruction mechanisms. A transport that may autonomously
poll an enabled queue sees an empty available ring. Providers may realize reset
and register verification through MMIO, configuration gateways, or another
qualified transport, but must prove device-visible references are revoked
before returning CPU ownership. Noncoherent, IOMMU, and bounce-buffer providers
retain their existing preparation and cancellation duties.

## Alternatives rejected

Activating the already available descriptor without notification was rejected
because notification suppression does not prove a device will not poll.
Skipping `DRIVER_OK` was rejected because it would not qualify the complete
activation boundary. Trusting reset without reading back queue state was
rejected because it would leave publication rollback unproved. Keeping the
queue active for the later ordinary lifecycle was rejected because it would
couple two independently qualified transactions.

## Deliberate bounds

This increment admits exactly one CPU-owned available-index withdrawal, exact
queue enablement and `DRIVER_OK`, no notification, exact reset, complete
transport zero readback, and full rollback. It rejects another owner, device,
domain, queue, transport, policy, nonempty activation, missing readback,
notification, escaped identity, nonzero final state, process mutation, or
recoverable disposition. It does not submit work, observe completion, consume
entropy, publish `AT_RANDOM`, enter the static process, or add a syscall.
