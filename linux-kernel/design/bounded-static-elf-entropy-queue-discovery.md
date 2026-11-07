# Bounded static ELF entropy-queue discovery

This record resolves `TK-DEC-162` as reversible virtio-rng queue discovery
while the static process remains behind its entropy-acquisition owner. It is a
systems-profile refinement only and adds no core-language element.

## Decision

After `TK-DEC-161`, `static image construction qualify entropy queue` shall
consume and return the exact acquisition owner. It shall accept only transport
identity `initial-virtio-rng-pci-transport`, queue identity
`initial-virtio-rng-queue-zero`, and policy
`bounded-static-elf-entropy-queue-discovery`.

The x86-64 provider shall completely revalidate the retained process, replay
the exact 16-access negotiation, execute the existing exact five-access queue-
discovery sequence, and prove queue zero exists with size eight, disabled
state, and notification identity zero. It shall then write and read back device
status zero. Every region access shall preserve the verified configuration-
gateway scratch rollback from `TK-DEC-161`, and the later ordinary sequence
shall remain unconsumed. Success returns the same owner and permits
`TOPAL_KERNEL_STATIC_ELF_ENTROPY_QUEUE_OK`.

## Architecture pressure test

The portable contract names an opaque transport and queue identity, not PCI
regions, BARs, queue registers, sizes, notification offsets, or an MMIO
mechanism. X86-64 binds it to the pinned Q35 PCI transport; AArch64 and RISC-V
providers may use PCI or MMIO while preserving complete-process ownership,
verified queue facts, complete rollback, and no exposed representation.

## Alternatives rejected

Leaving the device negotiated was rejected because it would affect the later
ordinary lifecycle. Treating a hard-coded queue size as provider truth was
rejected because it would not exercise the device. Combining discovery with
DMA allocation, queue enablement, or notification was rejected because those
steps introduce distinct ownership and rollback obligations.

## Deliberate bounds

This increment admits one exact negotiation, five exact queue observations,
and complete device/gateway rollback. It rejects any other owner, transport,
queue, policy, access, order, value, omission, duplication, final state,
process mutation, or recoverable disposition. It does not authorize bus
mastering, allocate DMA, construct or enable a queue, notify the device,
consume entropy, publish `AT_RANDOM`, enter the static process, or add a
syscall.
