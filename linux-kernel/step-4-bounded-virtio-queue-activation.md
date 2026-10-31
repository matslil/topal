# Step 4 bounded virtio queue activation

## Outcome

The Topal kernel now enables virtio-rng queue zero and completes the virtio
device-status handshake. Checked source writes and verifies `queue_enable = 1`,
then writes and verifies status 15 (`ACKNOWLEDGE | DRIVER | FEATURES_OK |
DRIVER_OK`) before emitting `TOPAL_KERNEL_VIRTIO_QUEUE_ACTIVE`.

The activation remains expressed through generic PCI-region register accesses.
Queue and status meaning stay in source-owned policy; the x86-64 provider owns
only its previously qualified configuration-gateway realization.

## Validation boundary

Rule `TOPAL-SYSTEMS-VIRTIO-PCI-QUEUE-ACTIVATE-001` requires all three queue
addresses to be published before the exact four-operation activation sequence.
Negative tests reject missing, duplicated, reordered, substituted, premature,
or malformed operations. Source and semantic tests also preserve diagnostic
priority for every prerequisite DMA stage.

The checked root contains 128 fatal edges, an exact 274-event semantic trace,
2,290 relocations, and 1,961 UART calls. The sealed provider plan remains 74
operations because activation reuses the existing generic access primitive.
Artifact schema `/83`, provider and provider-object schemas `/79`, artifact-root
schema `/81`, boot-adapter schema `/43`, and gate schema `/85` identify the
contract.

## Physical evidence

The gate retains the pinned `pc-q35-10.2` machine, `qemu64-v1` CPU, SeaBIOS,
ISA serial console, and one `virtio-rng-pci-non-transitional` function at
`00:05.0`. The activation is the final checked action after the pre-existing
interrupt, scheduler, and userspace qualifications. This ordering prevents a
newly live PCI function from perturbing those independent tests while still
proving the real device accepted both writes and both readbacks.

Two independent pinned-QEMU runs were byte-identical and reached all 61 ordered
observations. Hashes are:

- linked kernel: `b3df8ba4dc33dfb079a02958940ceb613d578067027fe4baaaacd8e77bba8745`;
- boot image: `cd8a0ead7da581b57f55203c5993133e0e6ca69462a5fbcf8b92e27ac0c3464d`;
- boot resource: `4e40627cbf586de35544acdf166cd1e84476deb603773b856119285f8e1813a5`;
- generated image: `a5378022c142d7040e45b9ef8036b05338b8d28efd2c92ca5b547cb4a9fafc01`;
- supplied image: `085b9584104febbc97035fe96f1732d6b10fc266a45778f3d11f5220be50904e`;
- artifact provenance: `9f15b467c09cd57253a082ca1c609e263b01f3c3956a32d845182e3befb9969b`;
- boot provenance: `2922a21120dcdec9b2ed713d75b6c16c3f5cd367c591c6d17131cc09528ed456`;
- serial output: `a78e4081da1573fe7619af0952de28100e4a83e9afc5b53c3caf7b163718d9e5`;
- evidence: `da0fdb1942de6552a71f8858e50cc214bd18806f05497df9feda61f28259d018`.

## Risk and remaining boundary

Risk is high because an incorrect live transition can expose DMA memory or
change interrupt behavior. Review covered prerequisite ownership and address
publication, exact register widths and values, readback, ordering, the absence
of a new provider primitive, physical device behavior, and duplicate physical
execution.

The queue is enabled but not notified. Completion observation, interrupt
handling for this device, CPU ownership return, entropy delivery, `AT_RANDOM`,
and `getrandom` remain unavailable.
