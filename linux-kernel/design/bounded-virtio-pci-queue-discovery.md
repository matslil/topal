# Bounded virtio-PCI queue discovery

This record resolves `TK-DEC-093` as the checked discovery boundary between
confirmed virtio feature negotiation and DMA-backed queue construction. It
observes the pinned virtio-rng queue without exposing an assigned BAR address,
allocating memory, or moving virtio policy into the provider.

## Decision

After the exact `TK-DEC-092` sequence has confirmed `FEATURES_OK`, checked
source performs five ordered accesses to the discovered common configuration
region: read `num_queues` as one; select queue zero; read its maximum
`queue_size` as eight; establish that `queue_enable` is zero; and read
`queue_notify_off` as zero. All accesses use the existing generic PCI
region-register operation and the two-byte widths required by the transport.

Queue selection, the meaning of the observed fields, and acceptance of the
pinned values remain source/library policy. The provider only realizes exact
two-byte accesses through the selected configuration gateway or an opaque BAR
mapping. A mismatch follows the existing fatal disposition.

The resulting facts prove that one disabled split virtqueue exists, that a
later policy may choose no more than eight descriptors, and that queue zero's
notification identity is zero. They contain no CPU address, device-visible
address, IOMMU mapping, cache policy, or memory ownership grant.

## Architecture pressure test

Queue facts are expressed as transport-relative register observations over a
previously validated PCI region. An AArch64 or RISC-V provider may realize the
same accesses through ECAM plus a configuration gateway or through an opaque
Device mapping with its required ordering. Exact two-byte semantics and
source-owned field interpretation remain invariant; x86 ports and physical BAR
addresses do not enter the source model.

The same checked queue facts can feed coherent x86 DMA, noncoherent DMA with
explicit cache maintenance, an IOMMU mapping, or a bounce-buffer provider.
None of those mechanisms is selected by discovery.

## Deliberate bounds

This increment recognizes only queue zero and the exact QEMU 10.2 virtio-rng
facts `num_queues=1`, `queue_size=8`, `queue_enable=0`, and
`queue_notify_off=0`. It does not resize or enable the queue, configure MSI-X,
enable PCI bus mastering, allocate or map DMA memory, publish descriptors,
notify or poll the device, consume entropy, or set `DRIVER_OK`.

The external transport reference is OASIS Virtio 1.4 Committee Specification
01, sections 3.1 and 4.1.4 through 4.1.5:
<https://docs.oasis-open.org/virtio/virtio/v1.4/cs01/virtio-v1.4-cs01.html>.
