# Step 4 bounded virtio payload completion

## Outcome

The Topal kernel now returns the completed virtio-rng payload buffer to CPU
ownership after validating the used entry. Checked source performs a second
generic DMA completion transition with payload identity two, device identity
one, translation-domain identity one, device-to-memory direction,
provider-managed visibility, and the `EntropyPayload` role, then emits
`TOPAL_KERNEL_VIRTIO_PAYLOAD_CPU_OWNED`.

The transition does not read or interpret the payload. The prior descriptor
binding and used-ring observations establish that descriptor zero names payload
offset zero and completed exactly 32 bytes; the completion operation establishes
only ownership and device-to-CPU visibility.

## Validation boundary

Rule `TOPAL-SYSTEMS-VIRTIO-PCI-PAYLOAD-COMPLETE-001` requires the payload
transition after all four used-ring reads. Negative tests reject omission,
duplication, wrong metadata, and completion before the final used-ring field.
The control and payload buffers are both CPU-owned afterward, but neither has
been recycled and no entropy byte has been exposed to source.

The checked root contains 136 fatal edges, an exact 286-event semantic trace,
2,452 relocations, and 2,101 UART calls. The sealed provider plan remains 77
operations because the existing generic completion operation serves both
buffers. Artifact schema `/87`, unchanged provider and provider-object schemas
`/82`, artifact-root schema `/85`, boot-adapter schema `/47`, and gate schema
`/89` identify the contract.

## Physical evidence

The gate retains the pinned `pc-q35-10.2` machine, `qemu64-v1` CPU, SeaBIOS,
ISA serial console, and one `virtio-rng-pci-non-transitional` function at
`00:05.0`. The new marker is reachable only after the real device completes the
32-byte descriptor, the ISR signal is acknowledged, used metadata is checked,
and the payload-specific provider completion accepts its exact metadata.

Two successful independent pinned-QEMU runs were byte-identical and reached
all 65 ordered observations. Hashes are:

- linked kernel: `bfb77809c3ac7038a8b7a0362c7eac749cfb12647462080119a8085dce2eb159`;
- boot image: `daf7f9ed3dc42851d4c281a0ba03a45669e9ffe9fac8fa0f1308f1dd8f99a56c`;
- boot resource: `4e40627cbf586de35544acdf166cd1e84476deb603773b856119285f8e1813a5`;
- generated image: `a5378022c142d7040e45b9ef8036b05338b8d28efd2c92ca5b547cb4a9fafc01`;
- supplied image: `085b9584104febbc97035fe96f1732d6b10fc266a45778f3d11f5220be50904e`;
- artifact provenance: `94ebe6ea270060f6e4b6aeae5a1ee085e5fc2823eb1493711b663735c53add83`;
- boot provenance: `066a69418af857d08fac21a24d6e87247f53793b3aa77f41e99f8262687acfd4`;
- serial output: `29202d99d71f4ea1fe976cc5530a62ee045805de7c8d0a1ff577de7f45b92722`;
- evidence: `e86268d398b8eafa4d95bc9d4ee004d58a73fa2df0dcadab0f79d4b557010a0e`.

## Risk and remaining boundary

Risk is high because the increment grants CPU authority over memory written by
a real device. Review covered completion-evidence ordering, exact affine
metadata, noncoherent and bounce-buffer obligations, distinct control/payload
symbols, provider instruction reuse, root relocations, negative conformance
cases, and duplicate physical execution.

Payload inspection, entropy estimation and credit, entropy-pool state,
`AT_RANDOM`, `getrandom`, queue recycling, asynchronous interrupt routing,
reset, and recovery remain unavailable.
