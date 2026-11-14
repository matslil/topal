# Step 4 bounded DMA device preparation

## Outcome

The Topal kernel now transfers its constructed virtio-rng DMA objects from
exclusive CPU ownership to exclusive device ownership. Checked source prepares
entropy-payload buffer two first, then virtqueue-control buffer one, and emits
`TOPAL_KERNEL_DMA_DEVICE_OWNED` only after both transitions succeed.

The transition retains opaque buffer, device, and translation-domain
identities plus exact direction, visibility, and role facts. Source never
observes an address, cache operation, mapping command, or target barrier, and it
cannot access either buffer again until a future completion returns ownership.

## Validation boundary

Rule `TOPAL-SYSTEMS-DMA-PREPARE-001` requires complete split-ring construction,
exact payload-before-control ordering, exclusive CPU ownership on entry, and
provider completion of all mapping, coherency, and ordering work before
device-owned state is returned. Negative tests reject malformed, missing,
duplicated, reordered, substituted, or premature requests and incomplete live
state.

The initial x86-64 provider checks both provider-private page symbols, their
4 KiB alignment, the sealed identities, device/domain, direction, visibility,
and roles. Its coherent identity-domain realization needs no cache instruction,
but the checked call boundary and semantic transition remain. Object and root
inspection prove both data-symbol relocations, two preparation calls, no
address return, and no hidden queue access. The checked root contains 121 fatal
edges, an exact 265-event semantic trace, 2,203 relocations, and 1,891 UART
calls. The sealed provider plan contains 73 operations.

Artifact schema `/81`, provider and provider-object schemas `/77`, artifact-root
schema `/79`, boot-adapter schema `/41`, and gate schema `/83` identify the
contract. Focused compiler, language, semantics, provider-byte, and gate-fixture
suites, formatting, and the complete workspace suite pass.

## Physical evidence

The gate retains the pinned `pc-q35-10.2` machine, `qemu64-v1` CPU, SeaBIOS,
ISA serial console, and one `virtio-rng-pci-non-transitional` function at
`00:05.0`. QEMU executed the payload preparation and then the control
preparation before the new marker.

Two independent pinned-QEMU runs were byte-identical and reached all 59 ordered
observations. Hashes are:

- linked kernel: `cf85fe4b98e9d80003dfd2142691cbcb6b57d24668d9c73564682bf6e69cae6b`;
- boot image: `4d1fb54536491b9139f8e0dbe62bc74fddc8e791636ec4139d75524d0b6e738c`;
- boot resource: `4e40627cbf586de35544acdf166cd1e84476deb603773b856119285f8e1813a5`;
- generated image: `a5378022c142d7040e45b9ef8036b05338b8d28efd2c92ca5b547cb4a9fafc01`;
- supplied image: `085b9584104febbc97035fe96f1732d6b10fc266a45778f3d11f5220be50904e`;
- artifact provenance: `ed877eb702b6827b0ab1f7fe6f9531677863b11bdc3182694fe7dfeb3b673e8b`;
- boot provenance: `90faa842704ce8ad8db843e9b2a9e56bd20f46d993ddd8f96a3623f214b439f1`;
- serial output: `a6ea0a45f58834c7a91f1b840f6d62904f69616789f2fb593d4b349648b0114f`;
- evidence: `5118ead36f712be692b9c033dfba735b7fb21f9adb1a0e144b302a84b328f477`.

## Risk and remaining boundary

Risk is high because a missing coherency action, wrong ownership transition,
or reversed preparation order could expose stale metadata or an inaccessible
payload to a DMA-capable device. Review covered affine ownership, exact
metadata, cross-architecture noncoherent/IOMMU/bounce pressure, payload-before-
control visibility, provider symbol isolation, negative semantic sequences,
structural encodings, and duplicate physical execution.

Neither queue address has been published to the transport. Queue enablement,
`DRIVER_OK`, notification, completion, CPU ownership return, entropy delivery,
`AT_RANDOM`, and `getrandom` remain unavailable.
