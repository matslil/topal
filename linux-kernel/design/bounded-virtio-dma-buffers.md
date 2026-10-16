# Bounded virtio DMA buffers

This record resolves `TK-DEC-095` as the first concrete DMA-memory authority
for the pinned virtio-rng function. It allocates opaque device-qualified memory
after queue discovery and PCI bus-master authorization, while retaining CPU
ownership. It does not publish an address or permit the device to access the
memory.

## Decision

The bootstrap platform authority may create exactly two page-sized DMA buffers
for one validated device and translation domain:

1. a 4 KiB, 4 KiB-aligned shared-control buffer for the split virtqueue's
   descriptor table and driver/device rings; and
2. a distinct 4 KiB, 4 KiB-aligned device-write payload buffer for entropy.

Creation returns fresh opaque affine identities in the `CpuOwned` state. The
request records device identity, translation-domain identity, byte count,
alignment, transfer direction, cache/visibility policy, and semantic role.
The buffers must be disjoint from each other and from every live ordinary,
user, firmware, and bootstrap allocation. No physical, virtual, bus, IOVA, or
provider address becomes source-visible.

The source/library layer owns the number, roles, sizes, directions, ordering,
and later layout of these buffers. The provider owns storage selection,
address translation, IOMMU or identity-domain realization, cache-coherency
mechanics, placement, and failure behavior. Creation alone grants CPU access;
only later checked preparation and ownership-transfer transitions can grant
device access.

## Architecture pressure test

The semantic request names a device and translation domain, not an x86
physical address. The initial x86 Q35 provider may use statically reserved,
cache-coherent, identity-translated pages. An AArch64 provider may instead
perform cache maintenance and create an IOMMU mapping; a RISC-V provider may
select coherent or noncoherent memory according to its platform description.
All preserve opaque identity, disjointness, direction, alignment, and linear
ownership.

The shared-control role is explicitly bidirectional because the driver writes
the descriptor and available rings while the device writes the used ring. The
payload role permits device-to-memory transfer only. Keeping the roles
separate avoids treating all DMA memory as universally bidirectional.

## Deliberate bounds

This increment admits only the two sealed initial buffers and rejects alternate
devices, domains, roles, directions, sizes, alignments, ordering, omission,
duplication, or overlap. It does not expose CPU byte access yet, construct a
split ring, prepare or transfer ownership, publish DMA addresses, enable the
queue, set `DRIVER_OK`, notify the device, consume entropy, or define recovery.

The external reference is the Linux DMA API ownership and direction model and
OASIS Virtio 1.4 split-virtqueue layout:
<https://docs.kernel.org/core-api/dma-api-howto.html>,
<https://docs.oasis-open.org/virtio/virtio/v1.4/cs01/virtio-v1.4-cs01.html>.
