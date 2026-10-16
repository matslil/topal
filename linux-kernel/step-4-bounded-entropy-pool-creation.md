# Step 4 bounded entropy-pool creation

## Outcome

The Topal kernel now creates one opaque affine entropy-pool resource after the
captured sample receives its explicit quality attribution. Checked source
names pool identity one, a 256-bit credit capacity, and exactly zero initial
credited bits. It then emits `TOPAL_KERNEL_ENTROPY_POOL_CREATED`.

Creation is source-owned lifecycle policy. It emits no provider call, storage,
address, byte representation, or cryptographic algorithm and leaves the
assessed sample live and unchanged. A later distinct transition must define
mixing and entropy-credit transfer.

## Validation boundary

Rules `TOPAL-SYSTEMS-ENTROPY-POOL-001` and
`TOPAL-SYSTEMS-VIRTIO-PCI-ENTROPY-POOL-001` require exactly one creation after
sample assessment. Negative tests reject omission, duplication, reordering,
another identity or capacity, and nonzero initial credit.

The checked root contains 137 fatal edges, an exact 292-event semantic trace,
2,561 relocations, and 2,206 UART calls. The sealed provider plan remains 78
operations. Artifact schema `/90`, provider and provider-object schemas `/83`,
artifact-root schema `/88`, boot-adapter schema `/50`, and gate schema `/92`
identify the contract.

## Physical evidence

The gate retains the pinned `pc-q35-10.2` machine, `qemu64-v1` CPU, SeaBIOS,
ISA serial console, and one `virtio-rng-pci-non-transitional` function at
`00:05.0`. The new marker follows device completion, used-ring validation,
payload ownership return, raw-sample capture, quality assessment, and exact
empty-pool creation.

Two successful independent pinned-QEMU runs were byte-identical and reached
all 68 ordered observations. One preliminary invocation exited cleanly during
early serial emission before the entropy path; that behavior did not reproduce
in either complete run. Hashes for the complete runs are:

- linked kernel: `da7abb7d4d27b639ee2283720d6c494068752715440c427889324a8c62760c85`;
- boot image: `14dc924b3de13c90d316a7e7772e131d6029adb4fe13efc18f4d2ef44166b6d3`;
- boot resource: `4e40627cbf586de35544acdf166cd1e84476deb603773b856119285f8e1813a5`;
- generated image: `a5378022c142d7040e45b9ef8036b05338b8d28efd2c92ca5b547cb4a9fafc01`;
- supplied image: `085b9584104febbc97035fe96f1732d6b10fc266a45778f3d11f5220be50904e`;
- artifact provenance: `3bdf0e9898075e856610e10281b7bac96a63e0aa0bacfdcace058447ca452e1c`;
- boot provenance: `8a997ce82b03e8fcfc1e20a7df1e6278785a6bf4bcf378aa94c4ddd227d6cbb0`;
- serial output: `dbe8e68540419433dfd2acc85a627cd39783b40a8251876657f43bf688de711f`;
- evidence: `2a25221938a08d367e683557e47cb044080ef294bf9224e909383a2b901e6a37`.

## Risk and remaining boundary

Risk is medium because this increment adds security-sensitive lifecycle state
but neither manipulates secret bytes nor changes cryptographic readiness.
Review covered exact affine ordering, zero-credit initialization, separation
from sample ownership, source/provider responsibility, malformed and reordered
rejection, provider-plan stability, structural artifact checks, and duplicate
complete physical execution.

Mixing, conditioning, health tests, credit transfer, CRNG state, `AT_RANDOM`,
`getrandom`, queue recycling and renotification, asynchronous interrupt
routing, reset, and recovery remain unavailable.
