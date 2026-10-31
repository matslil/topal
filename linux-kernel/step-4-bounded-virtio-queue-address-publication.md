# Step 4 bounded virtio queue-address publication

## Outcome

The Topal kernel now publishes all three split-queue DMA references to the
pinned virtio-rng common configuration. Checked source publishes control-buffer
offsets zero, 128, and 148 to `queue_desc`, `queue_driver`, and `queue_device`
in that order, then emits `TOPAL_KERNEL_VIRTIO_QUEUE_ADDRESSES_OK`.

Source owns the transport register identities, queue layout, and publication
order but never observes the underlying address. A generic PCI-region DMA-
reference operation lets the provider resolve the opaque device-owned buffer
through its translation domain and perform the qualified register write.

## Validation boundary

Rule `TOPAL-SYSTEMS-DMA-PUBLISH-001` requires both DMA buffers to be
device-owned, matching device/domain identities, an in-bounds representable
buffer reference, a valid destination region, and the exact descriptor-driver-
device sequence. Negative tests reject malformed, missing, duplicated,
reordered, substituted, or premature publications.

The x86-64 provider privately resolves the control-page symbol and decomposes
each logical 64-bit publication into ordered low/high dword writes through the
pinned PCI configuration gateway. Object inspection proves the exact gateway
selector, BAR, three register/offset pairs, six data writes, high-dword shift,
and absence of an address return. Root inspection proves three control-symbol
relocations and three typed calls. The checked root contains 124 fatal edges,
an exact 269-event semantic trace, 2,250 relocations, and 1,929 UART calls. The
sealed provider plan contains 74 operations.

Artifact schema `/82`, provider and provider-object schemas `/78`, artifact-root
schema `/80`, boot-adapter schema `/42`, and gate schema `/84` identify the
contract. Focused compiler, language, semantics, provider-byte, and gate-fixture
suites, formatting, and the complete workspace suite pass.

## Physical evidence

The gate retains the pinned `pc-q35-10.2` machine, `qemu64-v1` CPU, SeaBIOS,
ISA serial console, and one `virtio-rng-pci-non-transitional` function at
`00:05.0`. QEMU accepted all six gateway data writes and executed the new
marker after the three logical publications.

Two independent pinned-QEMU runs were byte-identical and reached all 60 ordered
observations. Hashes are:

- linked kernel: `3faf73219eeffc9ee59f65e92d3d4d7a51e2a51b9dba59926baa48766f6e6c63`;
- boot image: `96c86fddbee71e620581bac5cf1c9619228a83a5c5bb30c56a5cf40246a75109`;
- boot resource: `4e40627cbf586de35544acdf166cd1e84476deb603773b856119285f8e1813a5`;
- generated image: `a5378022c142d7040e45b9ef8036b05338b8d28efd2c92ca5b547cb4a9fafc01`;
- supplied image: `085b9584104febbc97035fe96f1732d6b10fc266a45778f3d11f5220be50904e`;
- artifact provenance: `63dc9baa9055a4f795141997ee0932f6c12aed7efd5d0457d93fb0b9895eb378`;
- boot provenance: `a0a36723a95071da149e5c7a654e7d91a76526d5a63f4cb8b61d15fb2cfaef24`;
- serial output: `b31bb984ed521762649d94d9b6b619cb07037a99df2e6c97dea6a6334ec4ae74`;
- evidence: `b5c0a767f6e5b3970c931cb122a8524d5a59c9175898aee724110c68dfd22b0a`.

## Risk and remaining boundary

Risk is high because a wrong translated reference, register, half ordering, or
offset can grant a DMA-capable device access to unintended memory. Review
covered opaque address handling, exact device/domain and ownership state,
cross-architecture IOMMU/bounce/MMIO pressure, low-before-high gateway writes,
provider encodings, negative semantic sequences, and duplicate physical
execution.

The queue remains disabled. `DRIVER_OK`, notification, completion, CPU
ownership return, entropy delivery, `AT_RANDOM`, and `getrandom` remain
unavailable.
