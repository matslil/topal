# Step 4 bounded virtio DMA buffers

## Outcome

The Topal kernel now creates two real, address-free DMA-memory authorities for
the pinned Q35 virtio-rng function after queue discovery and PCI bus-master
authorization. Checked source creates a bidirectional shared-control page for
the split virtqueue and a distinct device-write payload page for entropy, then
emits `TOPAL_KERNEL_DMA_BUFFERS_OK`.

Both 4 KiB buffers begin in `CpuOwned` state with 4 KiB alignment. Their
requests carry opaque buffer, device, and translation-domain identities plus
direction, visibility, and role. No physical, virtual, bus, or IOVA address is
visible to source, and creation grants no device access.

## Validation boundary

Rules `TOPAL-SYSTEMS-DMA-001` and `TOPAL-SYSTEMS-DMA-BUFFER-001` require fresh
affine identities, exact metadata, disjoint placement, zeroed contents, and
linear ownership. Negative tests reject malformed, missing, duplicated,
reordered, or substituted requests.

The initial x86-64 provider reserves two distinct pages in a dedicated
8 KiB uninitialized-data section aligned to 4 KiB. ELF inspection proves both
symbol sizes, exact 4 KiB separation, section kind/alignment, a closed checked
creation provider, and absence of source-visible addresses. The checked root
contains two creation calls, 114 fatal edges, an exact 256-event semantic
trace, 2,119 relocations, and 1,829 UART calls. The sealed provider plan
contains 70 operations.

Artifact schema `/79`, provider and provider-object schemas `/75`, artifact-root
schema `/77`, boot-adapter schema `/39`, and gate schema `/81` identify the
contract. Focused compiler, language, semantics, provider-byte, and gate-fixture
suites, formatting, and the complete workspace suite pass.

## Physical evidence

The gate retains the pinned `pc-q35-10.2` machine, `qemu64-v1` CPU, SeaBIOS,
ISA serial console, and one `virtio-rng-pci-non-transitional` function at
`00:05.0`. The Linux boot adapter loaded and zeroed the enlarged provider BSS,
and QEMU executed both checked creation calls before the new marker.

Two independent pinned-QEMU runs were byte-identical and reached all 57 ordered
observations. Hashes are:

- linked kernel: `9920222ad5868b75800cb7221705a81501607d1d5c51235f3a4e547536e9f8e2`;
- boot image: `cfb996acf9898df58fb48414452945fafdb5be5904be9fb0f3ffdac54f54c2c6`;
- boot resource: `4e40627cbf586de35544acdf166cd1e84476deb603773b856119285f8e1813a5`;
- generated image: `a5378022c142d7040e45b9ef8036b05338b8d28efd2c92ca5b547cb4a9fafc01`;
- supplied image: `085b9584104febbc97035fe96f1732d6b10fc266a45778f3d11f5220be50904e`;
- artifact provenance: `f7e69f35e5309f21795d56a0ebd0b787548ce2542b974108ff99aac65102bbf5`;
- boot provenance: `c86cf56e374ebf6e9269dcb58bf0d104f77b626484039dccc258380b509a8e5f`;
- serial output: `d79048fb8fa77a673ffcdf7e4f04e32f26fd39b96bb9fe461f2e78ed06775bf8`;
- evidence: `b91b2372f226abeb09e1beb7e301e089b7080e00b1aeeea0094b03787cff29ae`.

## Risk and remaining boundary

Risk is high because aliasing, misalignment, or incorrect device/domain
qualification could permit memory corruption once ownership is transferred.
Review covered affine identity, exact roles and directions, disjoint aligned
ELF placement, zero initialization, cross-architecture coherency/IOMMU
pressure, closed provider arguments, negative semantic sequences, and duplicate
physical execution.

Both pages remain CPU-owned. DMA byte access, split-ring construction,
preparation and device ownership, queue-address publication, queue enablement,
`DRIVER_OK`, notification, completion, entropy delivery, `AT_RANDOM`, and
`getrandom` remain unavailable.
