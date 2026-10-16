# Step 4 bounded virtio used-ring validation

## Outcome

The Topal kernel now returns only the virtqueue-control DMA buffer to CPU
ownership after the acknowledged virtio-rng completion signal and validates
the device-written used-ring metadata. Checked source reads used flags zero,
used index one, descriptor identity zero, and completed length 32 before
emitting `TOPAL_KERNEL_VIRTIO_USED_RING_OK`.

The operation remains architecture-neutral. Source names the opaque buffer,
device, translation domain, direction, visibility policy, role, field offsets,
widths, and expected values. The x86-64 provider realizes the ownership return
as coherent checked ordering; a noncoherent provider would perform its cache or
DMA synchronization before returning CPU authority.

## Validation boundary

Rules `TOPAL-SYSTEMS-DMA-COMPLETE-001` and
`TOPAL-SYSTEMS-VIRTIO-PCI-USED-RING-001` require one exact affine ownership
transition and four exact ordered reads. The used ring begins at absolute
control-buffer offset 148, so flags, index, element identity, and element length
are read at absolute offsets 148, 150, 152, and 156 (used-ring-relative offsets
0, 2, 4, and 8). Negative tests reject malformed, missing, duplicated,
substituted, reordered, or premature operations.

The checked root contains 135 fatal edges, an exact 284-event semantic trace,
2,412 relocations, and 2,064 UART calls. The sealed provider plan contains 77
operations. Artifact schema `/86`, provider and provider-object schemas `/82`,
artifact-root schema `/84`, boot-adapter schema `/46`, and gate schema `/88`
identify the contract.

## Physical evidence

The gate retains the pinned `pc-q35-10.2` machine, `qemu64-v1` CPU, SeaBIOS,
ISA serial console, and one `virtio-rng-pci-non-transitional` function at
`00:05.0`. A preliminary physical run exposed a coordinate error in the draft
design: offsets 20, 22, 24, and 28 were relative to the wrong split-ring base.
The design, specification, and implementation were corrected to the absolute
control-buffer offsets above before qualification.

Two subsequent independent pinned-QEMU runs were byte-identical and reached
all 64 ordered observations. Hashes are:

- linked kernel: `f05cfa3acb79230ee3ea6d58b6cd9ef79f0df018944c7061d3fd6f5eb16b261b`;
- boot image: `3e762996470a768d7615572b4adc81e687e2aafeaae91400aa1f9cfd3e0c26f8`;
- boot resource: `4e40627cbf586de35544acdf166cd1e84476deb603773b856119285f8e1813a5`;
- generated image: `a5378022c142d7040e45b9ef8036b05338b8d28efd2c92ca5b547cb4a9fafc01`;
- supplied image: `085b9584104febbc97035fe96f1732d6b10fc266a45778f3d11f5220be50904e`;
- artifact provenance: `697f91f03e2f65ab5808cd83dbbed4b758845e522c53c54c358ba07277492d61`;
- boot provenance: `b3e20df3b05980145584f961deb11df5c3beb04fbdf9d2280ae0375b2fd83054`;
- serial output: `bb3bd8320399aa4743494bb100aba1bb2fe63e24e6dfe231dbaa19a2c5689330`;
- evidence: `8e0a086e6ff7ff35e3857db3c74d6da616989cb21a9a3796192c0bb123c994a6`.

## Risk and remaining boundary

Risk is high because the increment changes DMA ownership after real device
activity and trusts device-written queue metadata only after provider-managed
visibility. Review covered affine ownership, exact split-ring coordinates,
alignment and bounds, coherent realization, provider instruction encodings,
root relocations, negative conformance cases, and duplicate physical
execution.

The payload buffer remains device-owned. Payload visibility and byte
validation, entropy delivery, `AT_RANDOM`, `getrandom`, queue recycling,
asynchronous interrupt routing, reset, and recovery remain unavailable.
