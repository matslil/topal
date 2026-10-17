# Step 4 bounded entropy-sample quality attribution

## Outcome

The Topal kernel now advances the captured 32-byte virtio-rng sample from raw
to assessed while preserving affine sample identity one. Checked source names
source identity one, trust-evidence identity one, byte count 32, quality 1024
parts per 1024, and exactly 256 credited entropy bits. It then emits
`TOPAL_KERNEL_ENTROPY_SAMPLE_CREDITED`.

The operation is source-owned kernel policy. It performs no provider call,
does not inspect or change the sample bytes, and does not infer trust from the
virtio device, transport, or successful completion. The initial Q35 profile
instead supplies an explicit deployment assumption that its configured host
entropy service is fully trusted.

## Validation boundary

Rules `TOPAL-SYSTEMS-ENTROPY-QUALITY-001` and
`TOPAL-SYSTEMS-VIRTIO-PCI-ENTROPY-QUALITY-001` require exactly one assessment
after raw-sample capture. The credited value is derived from sample bits and
quality with the specified floor calculation and cannot exceed the sample
extent. Negative tests reject omission, duplication, reordering, another
sample, source, evidence, extent, quality, or credit.

The checked root contains 137 fatal edges, an exact 290-event semantic trace,
2,528 relocations, and 2,173 UART calls. The sealed provider plan remains 78
operations because assessment is provider-free. Artifact schema `/89`,
provider and provider-object schemas `/83`, artifact-root schema `/87`,
boot-adapter schema `/49`, and gate schema `/91` identify the contract.

## Physical evidence

The gate retains the pinned `pc-q35-10.2` machine, `qemu64-v1` CPU, SeaBIOS,
ISA serial console, and one `virtio-rng-pci-non-transitional` function at
`00:05.0`. The new marker follows real device completion, used-ring
validation, payload ownership return, raw-sample capture, and the exact
source-owned quality transition.

Two successful independent pinned-QEMU runs were byte-identical and reached
all 67 ordered observations. Hashes are:

- linked kernel: `2799607805231e788991a1e39b66e22f11be0c75077f310515dac9da96836158`;
- boot image: `8631e161a29a5e81bea56ad46325b84bfb38010941d5272d54a344df6919d3e4`;
- boot resource: `4e40627cbf586de35544acdf166cd1e84476deb603773b856119285f8e1813a5`;
- generated image: `a5378022c142d7040e45b9ef8036b05338b8d28efd2c92ca5b547cb4a9fafc01`;
- supplied image: `085b9584104febbc97035fe96f1732d6b10fc266a45778f3d11f5220be50904e`;
- artifact provenance: `687dbea1c6c76d2ae60af5a82b883d4f528599434b66e78a34c1f3ab99cd4aa4`;
- boot provenance: `38abbaa7abdd32f547199034cb4c7e5a54dd5ab255b3f324f1193705b4d2dd07`;
- serial output: `2fefd682a284988c6e4d1a2a0d026b53055bc278b79afec13e0dc9c7d94c28a2`;
- evidence: `5fca5c0cded9952437867f872ebe31e397939c284b682eca222d62fa5b7d83d2`.

## Risk and remaining boundary

Risk is high because an incorrect quality decision can overstate entropy and
later weaken random output. Review covered explicit deployment evidence,
separation from device and provider mechanics, exact affine ordering,
arithmetic bounds, zero byte access, provider-plan stability, malformed and
reordered rejection, structural artifact checks, and duplicate physical
execution.

Conditioning, health tests, pool mixing and accounting, CRNG state,
`AT_RANDOM`, `getrandom`, queue recycling and renotification, asynchronous
interrupt routing, reset, and recovery remain unavailable.
