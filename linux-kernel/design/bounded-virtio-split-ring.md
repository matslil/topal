# Bounded virtio split-ring construction

This record resolves `TK-DEC-096` as the first CPU-owned construction of a DMA
protocol object. It initializes one descriptor and one available-ring entry in
the previously created virtqueue-control buffer without exposing either DMA
buffer's provider-selected address.

## Decision

Two generic operations refine a `CpuOwned` DMA buffer while preserving CPU
ownership:

- an integer store names a destination buffer identity, byte offset, exact
  width, and value; and
- a DMA-reference binding names destination and referenced buffer identities
  and offsets, then stores the referenced buffer's provider-selected
  device-visible address without returning that address to source.

Both operations require the same validated device and translation domain,
bounds, natural alignment, compatible direction/role authority, and exclusive
CPU ownership. The reference is a typed relocation-like fact, not an integer
available to ordinary source computation.

The initial virtio-rng policy constructs descriptor zero in control buffer one:
its address references offset zero of payload buffer two, length is 32 bytes,
flags contain only `VIRTQ_DESC_F_WRITE`, and next is zero. It places descriptor
zero in available-ring slot zero and publishes available index one last. All
other descriptor, available, and used-ring bytes remain their proven initial
zero. The split-ring layout uses queue size eight, descriptor offset zero,
available offset 128, and used offset 148.

## Architecture pressure test

The source operation binds opaque DMA identities, not physical or virtual
addresses. An identity-translated x86 provider may store the payload symbol's
address. An IOMMU provider stores its mapped IOVA; a bounce-buffer provider
stores the bounce address. Noncoherent providers retain the same CPU-owned
writes but perform required cache maintenance only during the later preparation
transition.

Widths, offsets, layout values, and publication order remain source/library
policy. Address resolution, relocation, target endianness realization, and
fault behavior remain provider responsibilities.

## Deliberate bounds

This increment admits one address binding and four integer stores: descriptor
length, descriptor flags, available slot zero, and available index. It rejects
alternate buffers, offsets, widths, values, order, omission, duplication, or
reference domains. It does not prepare cache visibility, transfer ownership,
publish queue addresses to the transport, enable the queue, set `DRIVER_OK`,
notify the device, observe completion, consume entropy, or define recovery.

The external reference is OASIS Virtio 1.4 split-virtqueue layout and available
buffer notification requirements:
<https://docs.oasis-open.org/virtio/virtio/v1.4/cs01/virtio-v1.4-cs01.html>.
