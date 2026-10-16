# Bounded PCI bus-master authorization

This record resolves `TK-DEC-094` as the explicit authority transition that
permits the pinned virtio-rng function to initiate PCI memory transactions. It
does not allocate, map, publish, or transfer ownership of DMA memory.

## Decision

The bootstrap platform authority may derive one transaction-scoped masked PCI
configuration-word update after queue discovery. A request identifies the PCI
function, a two-byte-aligned configuration offset, mask, expected masked
pre-state, and desired masked post-state. The provider serially reads the word,
verifies the pre-state, writes a value that changes only masked bits, reads the
word back, and proves the requested post-state before returning authority.

Checked source requests exactly one transition for `0000:00:05.0` at command
offset `0x04`: mask `0x0004`, expected pre-state zero, desired post-state
`0x0004`. This sets the PCI Bus Master Enable bit while preserving the memory-
and I/O-space enables and every other command bit. Exact word access avoids
writing the adjacent status register, whose bits can have write-one-to-clear
semantics.

The source/library layer owns the function choice, register meaning, mask, and
transition policy. The provider owns mechanism selection, serialization,
privilege, access ordering, target fault behavior, and exact-width realization.

## Architecture pressure test

The operation contains PCI coordinates and a configuration-relative field, not
an x86 port, ECAM address, instruction, or host call. The x86 provider may use
configuration mechanism 1; AArch64 and RISC-V providers may use ECAM or a
firmware-mediated configuration service. All must preserve exact word access,
unmasked bits, pre-state validation, readback, and transaction ordering.

Bus-master permission and DMA memory authority remain distinct capabilities.
This transition is therefore equally applicable to coherent DMA, noncoherent
DMA with cache maintenance, IOMMU mappings, and bounce buffers.

## Deliberate bounds

This increment admits only one command-word update and rejects arbitrary
configuration writes, alternate masks or functions, already-set pre-state,
omission, duplication, and reordering. It does not map a BAR, allocate DMA
memory, install an IOMMU mapping, construct or enable a virtqueue, set
`DRIVER_OK`, notify the device, consume entropy, or define recovery.

The external references are PCI Express Base Specification command-register
semantics as represented by the Linux PCI definitions, and OASIS Virtio 1.4
device initialization section 3.1:
<https://github.com/torvalds/linux/blob/master/include/uapi/linux/pci_regs.h>,
<https://docs.oasis-open.org/virtio/virtio/v1.4/cs01/virtio-v1.4-cs01.html>.
