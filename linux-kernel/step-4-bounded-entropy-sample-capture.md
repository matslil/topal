# Step 4 bounded entropy-sample capture

## Outcome

The Topal kernel now captures the completed 32-byte virtio-rng payload into a
fresh opaque raw-sample resource after the payload returns to CPU ownership.
Checked source names sample identity one, payload buffer identity two, offset
zero, byte count 32, device identity one, translation-domain identity one,
device-to-memory direction, provider-managed visibility, the
`EntropyPayload` role, and explicitly zero credited entropy bits. It then emits
`TOPAL_KERNEL_ENTROPY_SAMPLE_CAPTURED`.

The source never observes the payload or sample address or bytes. The x86-64
provider resolves both private views, loads each source qword once, copies it
to a distinct 32-byte kernel-owned sample section, verifies the destination,
and fails closed on a partial or mismatched transfer. The payload buffer
remains CPU-owned for a later queue-recycling design.

## Validation boundary

Rules `TOPAL-SYSTEMS-ENTROPY-SAMPLE-001` and
`TOPAL-SYSTEMS-VIRTIO-PCI-ENTROPY-CAPTURE-001` require exactly one capture
after payload completion. Negative tests reject omission, duplication,
reordering, another identity or range, and any nonzero entropy credit. The
sample is affine, raw, and uncredited; capture performs no conditioning,
mixing, health testing, pool update, or userspace delivery.

The checked root contains 137 fatal edges, an exact 288-event semantic trace,
2,492 relocations, and 2,137 UART calls. The sealed provider plan contains 78
operations. Artifact schema `/88`, provider and provider-object schemas `/83`,
artifact-root schema `/86`, boot-adapter schema `/48`, and gate schema `/90`
identify the contract.

## Physical evidence

The gate retains the pinned `pc-q35-10.2` machine, `qemu64-v1` CPU, SeaBIOS,
ISA serial console, and one `virtio-rng-pci-non-transitional` function at
`00:05.0`. The new marker is reachable only after real device completion,
used-ring validation, payload ownership return, exact provider-side copying,
and destination verification.

Two successful independent pinned-QEMU runs were byte-identical and reached
all 66 ordered observations. Hashes are:

- linked kernel: `ddabd0d36510df42d749ed95724bf9340aaeba439d8255db2ebbfb89529102df`;
- boot image: `0f8810b25b6034ac5b3523e3426dbde95334bd7229af02dd550cfd89276ce136`;
- boot resource: `4e40627cbf586de35544acdf166cd1e84476deb603773b856119285f8e1813a5`;
- generated image: `a5378022c142d7040e45b9ef8036b05338b8d28efd2c92ca5b547cb4a9fafc01`;
- supplied image: `085b9584104febbc97035fe96f1732d6b10fc266a45778f3d11f5220be50904e`;
- artifact provenance: `d161f9f49dd60c8d0abd0038c63c94853061fedeae155eb9557a7fa133a4792b`;
- boot provenance: `2456d56b30c2bb13677dc20672a2f8db7e3dae7dd9f83b4e45440cb06d8a238e`;
- serial output: `308293f4433a6490dfe4762e696fa92708c8742673713780ad55f9acaab00eaf`;
- evidence: `939bec9944dec6b0aa27536b4d18b65160d1feb5f43b4a0bb654195245831bce`.

## Risk and remaining boundary

Risk is high because the increment creates kernel-owned state from bytes
written by a real device. Review covered ownership ordering, exact affine
metadata, raw-sample opacity, zero-credit preservation, noncoherent and
bounce-buffer obligations, distinct provider storage, generated copy and
verification instructions, root relocations, negative conformance cases, and
duplicate physical execution.

Entropy conditioning, health tests, trust policy, pool accounting, CRNG state,
`AT_RANDOM`, `getrandom`, queue recycling and renotification, asynchronous
interrupt routing, reset, and recovery remain unavailable.
