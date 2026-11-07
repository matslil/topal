# Bounded virtio queue-address publication

This record resolves `TK-DEC-098` as the first transport publication of an
opaque DMA identity. It writes the split queue's descriptor, driver, and device
area references into the selected virtio common configuration without making
any device-visible address available to source.

## Decision

A generic PCI-region DMA-reference publication operation combines:

- a validated PCI function and BAR-relative region;
- one register offset and reference width;
- one `DeviceOwned` DMA buffer identity and in-buffer offset; and
- the matching device and translation-domain identities.

Source owns the register selection and queue layout. The provider resolves the
opaque buffer reference to the device-visible address required by the selected
translation domain and writes it through the qualified region-access
mechanism. The operation returns only success or failure, never the address.

The initial virtio-rng policy publishes three 64-bit references in order:
control offset zero to `queue_desc` at common-configuration offset 32, control
offset 128 to `queue_driver` at offset 40, and control offset 148 to
`queue_device` at offset 48. All use the already selected queue zero, common
region, device identity one, translation-domain identity one, and control
buffer identity one.

## Architecture pressure test

An identity-translated x86 provider writes the buffer symbol plus offset. An
IOMMU provider writes its mapped IOVA plus offset; a bounce provider writes the
device-side reference. A mapped-MMIO provider may perform one native 64-bit
store when the register protocol permits it, while the pinned x86 PCI
configuration gateway decomposes the value into ordered low and high dword
writes. Source observes neither representation nor decomposition.

The abstraction binds a typed DMA reference to a device register; it is not a
general address read, integer cast, raw MMIO store, or virtio provider
intrinsic. Register identity and ordering remain portable driver policy.

## Deliberate bounds

This increment admits exactly the three publications above after both buffers
are device-owned. It rejects alternate regions, registers, widths, identities,
offsets, order, omission, duplication, or publication before preparation.

It does not enable the queue, set `DRIVER_OK`, notify the device, observe
completion, return CPU ownership, consume entropy, or define recovery.
