# Bounded static ELF entropy queue-address coexistence

This record resolves `TK-DEC-167` as a reversible qualification transaction
for split-queue address publication while the static process remains behind
its entropy-acquisition owner. It is a systems-profile refinement only and
adds no core-language element.

## Decision

After `TK-DEC-166`, `static image construction qualify entropy queue addresses`
shall consume and return the exact acquisition owner. It shall accept only
device identity `initial-virtio-rng-device`, translation-domain identity
`initial-dma-domain`, queue identity `initial-virtio-rng-queue-zero`, transport
identity `initial-virtio-rng-pci-transport`, and policy
`bounded-static-elf-entropy-queue-address-coexistence`.

The x86-64 provider shall completely revalidate the retained process and replay
the exact prerequisite sequence through payload-then-control DMA preparation.
It shall publish the exact descriptor, driver, and device-area references in
that order through the existing generic PCI-region DMA-reference operation.

Because the queue remains disabled, the transaction shall then clear the
device, driver, and descriptor address registers in reverse order and read all
three back as zero before cancelling control and payload preparation. It shall
then erase and read back every ring field, reverse bus-master authority, and
restore verified device and configuration-gateway state. No DMA identity shall
escape and the ordinary later sequence shall remain unconsumed. Success returns
the same owner and permits
`TOPAL_KERNEL_STATIC_ELF_ENTROPY_QUEUE_ADDRESSES_OK`.

## Architecture pressure test

The contract publishes and revokes opaque device-visible references rather
than integers or physical addresses. An identity-mapped x86-64 provider may
resolve them to physical addresses; IOMMU and bounce-buffer providers may use
IOVAs or device-side copies. A mapped-MMIO provider may use native 64-bit
stores, while the pinned Q35 provider may decompose them through its private
configuration gateway. Every provider must prove zero readback before
returning either buffer to CPU ownership.

## Alternatives rejected

Leaving queue addresses published was rejected because it would affect the
later ordinary lifecycle. Clearing only device status was rejected because it
would not prove transport-register rollback. Returning a device-visible
address to source was rejected because it would break reference opacity.
Enabling the queue merely to exercise the published references was rejected
because activation is the next distinct portable-driver boundary.

## Deliberate bounds

This increment admits exactly descriptor-then-driver-then-device publication
and device-then-driver-then-descriptor zeroing inside one sealed transaction.
It rejects another owner, device, domain, queue, transport, policy, publication
or erasure order, escaped identity, nonzero final register, final ownership or
byte state, process mutation, or recoverable disposition. It does not enable
or notify the queue, permit device access, consume entropy, publish
`AT_RANDOM`, enter the static process, or add a syscall.
