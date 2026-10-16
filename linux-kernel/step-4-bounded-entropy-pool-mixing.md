# Step 4 bounded entropy-pool mixing

## Outcome

The Topal kernel now consumes assessed entropy sample identity one into empty
pool identity one through the fixed `topal.systems.entropy.pool.mix-sample/1`
transition. The transition returns the same pool in its mixed-but-uncredited
state, creates affine contribution identity one with a 256-bit untransferred
claim, consumes the sample, and emits `TOPAL_KERNEL_ENTROPY_POOL_MIXED`.

The sealed x86-64 provider computes BLAKE2s-256 over the exact 32 sample bytes
with personalization `TopalRNG`, publishes all 32 opaque pool-state bytes,
wipes all 32 sample bytes, verifies the wipe, and reports failure before a
partial result can be accepted. Source observes no byte, address, digest, or
instruction and the pool's credited-bit count remains zero.

## Validation boundary

Rules `TOPAL-SYSTEMS-ENTROPY-MIX-001` and
`TOPAL-SYSTEMS-VIRTIO-PCI-ENTROPY-MIX-001` require the exact first-generation
mix after assessment and empty-pool creation. Negative tests reject omission,
duplication, reordering, substituted identities, another byte count or credit,
and an incompatible pool or sample state.

The generated scalar provider function was executed independently on the
fixed input bytes `00` through `1f`. Python's BLAKE2s implementation and the
checked vector
`dd8ab1645ca92dd40c801f4f9ac8a4852159709249e4cdb4cf7cab58e899e24e`
agreed with all 32 produced bytes, and the test also observed all 32 input
bytes wiped. Structural tests cover all ten compression rounds, the four
rotation constants, personalization parameters, state publication, wipe
stores, and wipe verification.

The checked root contains 138 fatal edges, an exact 294-event semantic trace,
2,596 relocations, and 2,237 UART calls. The sealed provider plan contains 79
operations. Artifact schema `/91`, provider and provider-object schemas `/84`,
artifact-root schema `/89`, boot-adapter schema `/51`, and gate schema `/93`
identify the contract.

## Physical evidence

The gate retains the pinned `pc-q35-10.2` machine, `qemu64-v1` CPU, SeaBIOS,
ISA serial console, and one `virtio-rng-pci-non-transitional` function at
`00:05.0`. The new marker follows completion acknowledgement, used-ring
validation, payload ownership return, raw-sample capture, quality attribution,
empty-pool creation, provider mixing, state publication, and sample wiping.

Two successful independent pinned-QEMU runs were byte-identical and reached
all 69 ordered observations. Hashes for the complete runs are:

- linked kernel: `c30fa65c28c88c782f98d9fd4a444be114baad4a12b0f36a8e18d62832bbef62`;
- boot image: `f31b497aef9f77e9dcd9234b4ab1053be1b342d729ecb85ad38c6ea754f83f6d`;
- boot resource: `4e40627cbf586de35544acdf166cd1e84476deb603773b856119285f8e1813a5`;
- generated image: `a5378022c142d7040e45b9ef8036b05338b8d28efd2c92ca5b547cb4a9fafc01`;
- supplied image: `085b9584104febbc97035fe96f1732d6b10fc266a45778f3d11f5220be50904e`;
- artifact provenance: `9fbe1d45934d1c0edbe886da5bd9a96b25dfd195e054937cfbcf0c8af5630c18`;
- boot provenance: `adacdbd6a508daab11eb86aa26853ae551f3d4fdbcfeecc8a427c91cdcee377b`;
- serial output: `6cd48cf9095e138aeabc64a9e2558219746efa8e8b03cde01ce5e1b726091160`;
- evidence: `9ef92218793a88de898e1a6124cd6d8dcf46657e6c5c266c0506cc04059c9154`.

## Risk and remaining boundary

Risk is high because this increment adds the first cryptographic processing of
secret kernel state. Review covered exact affine ordering, algorithm and
personalization stability, portable byte-level results, separation of mixing
from credit, complete state publication, input destruction, executable
known-vector comparison, malformed and reordered rejection, structural
artifact inspection, and duplicate complete physical execution.

Health tests, credit transfer, readiness, reseeding, CRNG extraction,
`AT_RANDOM`, `getrandom`, a second sample, queue recycling and renotification,
asynchronous interrupt routing, reset, and recovery remain unavailable.
