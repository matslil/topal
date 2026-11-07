# Bounded static ELF entropy-transport negotiation

This record resolves `TK-DEC-161` as the first write-capable hardware
qualification performed while the static process remains behind its entropy-
acquisition owner. It is a systems-profile refinement only and adds no core-
language element.

## Decision

After `TK-DEC-160`, `static image construction qualify entropy negotiation`
shall consume and return the exact acquisition owner. It shall accept only
transport identity `initial-virtio-rng-pci-transport` and policy
`bounded-static-elf-entropy-transport-negotiation`.

The x86-64 provider shall completely revalidate the retained process, execute
the existing exact 16-access Q35 virtio-rng feature-negotiation sequence, and
then roll the device back with an exact status-zero write and readback. The
qualification shall select only `VIRTIO_F_VERSION_1`, prove retained
`FEATURES_OK`, prove the final reset state, and restore and verify the
configuration-access gateway's discovered zero BAR, offset, and length scratch
fields after every access. It shall leave the later ordinary transport
sequence unconsumed. Success returns the same owner and permits
`TOPAL_KERNEL_STATIC_ELF_ENTROPY_NEGOTIATION_OK`.

## Architecture pressure test

The portable contract names an opaque transport and a negotiation policy, not
PCI regions, BARs, register offsets, access widths, feature bit positions, or
an MMIO mechanism. X86-64 binds the contract to the pinned Q35 PCI transport;
AArch64 and RISC-V providers may use PCI or MMIO transports while preserving
the same complete-process ownership, mandatory-feature, verified-reset, and
no-exposed-representation obligations.

## Alternatives rejected

Leaving the device in `FEATURES_OK` was rejected because it would make the
qualification affect the later ordinary transport lifecycle. Omitting reset
readback was rejected because a write alone does not prove rollback. Moving
the register protocol or feature words into source was rejected because it
would expose the target transport representation. Combining negotiation with
queue discovery or DMA was rejected because those steps have distinct
resource and rollback obligations.

## Deliberate bounds

This increment admits one exact negotiation and one exact reset/readback while
returning the same process owner. Any other owner, transport, policy, access,
feature selection, order, value, omission, duplication, final status, gateway
scratch restoration, process mutation, or recoverable disposition is invalid.
It does not discover or configure a queue, authorize bus mastering, allocate
DMA, notify the device, consume entropy, publish `AT_RANDOM`, enter the static
process, or add a syscall.
