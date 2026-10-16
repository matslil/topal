# Step 4 bounded virtio split ring

## Outcome

The Topal kernel now constructs the first CPU-owned split virtqueue in the
previously created shared-control DMA page. Checked source binds descriptor
zero to the opaque entropy-payload buffer, writes its 32-byte length and
device-write flag, places descriptor zero in available slot zero, publishes
available index one last, and emits `TOPAL_KERNEL_VIRTIO_SPLIT_RING_OK`.

The source never observes a physical, virtual, bus, or IOVA address. A generic
DMA-reference operation names the two buffer identities and lets the provider
materialize the device-visible reference. Generic bounded integer stores keep
the queue-size-eight layout and publication order in source-owned policy.

## Validation boundary

Rule `TOPAL-SYSTEMS-DMA-CONSTRUCT-001` requires exclusive CPU ownership,
matching device and translation-domain identities, compatible roles and
directions, in-bounds naturally aligned stores, and exact transition order.
Negative tests reject malformed, missing, duplicated, reordered, or
substituted construction requests.

The initial x86-64 provider privately resolves the two DMA symbols, checks the
sealed offsets, widths, values, and identities, and emits the reference and
integer stores. Object inspection proves the reference-binding and store
entry points, their data relocations, and the structural encodings. The checked
root contains the exact five construction calls, 119 fatal edges, an exact
262-event semantic trace, 2,168 relocations, and 1,862 UART calls. The sealed
provider plan contains 72 operations.

Artifact schema `/80`, provider and provider-object schemas `/76`, artifact-root
schema `/78`, boot-adapter schema `/40`, and gate schema `/82` identify the
contract. Focused compiler, language, semantics, provider-byte, and gate-fixture
suites, formatting, and the complete workspace suite pass.

## Physical evidence

The gate retains the pinned `pc-q35-10.2` machine, `qemu64-v1` CPU, SeaBIOS,
ISA serial console, and one `virtio-rng-pci-non-transitional` function at
`00:05.0`. QEMU executed the checked opaque-reference binding followed by all
four integer stores before the new marker.

Two independent pinned-QEMU runs were byte-identical and reached all 58 ordered
observations. Hashes are:

- linked kernel: `1f6642f5322a9e36fa29c5018946fb504d8499f9675c567bb5e2d835948a6551`;
- boot image: `61c987797ca5ad2790b695c48b6203e7eaa71dce202c46a0b3c25a637f962875`;
- boot resource: `4e40627cbf586de35544acdf166cd1e84476deb603773b856119285f8e1813a5`;
- generated image: `a5378022c142d7040e45b9ef8036b05338b8d28efd2c92ca5b547cb4a9fafc01`;
- supplied image: `085b9584104febbc97035fe96f1732d6b10fc266a45778f3d11f5220be50904e`;
- artifact provenance: `36a30e029076702e41e9e11d2857eda47ff037f9e76ccbba3f524fc0d087d74e`;
- boot provenance: `0ecd0a7625276ffdb74f6473f42e4bab6347dec8418fa805c24e2301449cbb64`;
- serial output: `cf5f3ae04544ca98f22dd0f231461626e05579984f178f2270a79c4957d070ea`;
- evidence: `ab8bb64332df5b1c11333671c699faa8be31ec326a83678060d1aeca49c21c44`.

## Risk and remaining boundary

Risk is high because a wrong reference, width, offset, or publication order
could let the device corrupt memory after ownership transfer. Review covered
opaque address handling, same-domain qualification, role and direction
compatibility, exact split-ring offsets and values, available-index-last
ordering, zero-initialized remainder, provider data relocations, negative
semantic sequences, and duplicate physical execution.

Both pages remain CPU-owned. Cache/coherency preparation, device ownership,
queue-address publication, queue enablement, `DRIVER_OK`, notification,
completion, entropy delivery, `AT_RANDOM`, and `getrandom` remain unavailable.
