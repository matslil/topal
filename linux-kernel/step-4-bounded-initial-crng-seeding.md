# Step 4 bounded initial CRNG seeding

## Outcome

The Topal kernel now consumes fully credited entropy-pool identity one through
`topal.systems.entropy.generator.initialize/1` and creates ready entropy-
generator identity one. The checked transition requires exactly 256 credited
bits, records ChaCha20 algorithm identity one and generation one, consumes the
pool, and emits `TOPAL_KERNEL_ENTROPY_GENERATOR_READY` without producing random
output.

The x86-64 provider moves the complete 32-byte pool state into distinct
generator-key storage, verifies the copy, wipes and verifies the source pool,
and only then returns success. Source code cannot observe either storage
address, the key bytes, or target instructions. The operation is generic; the
provider owns this concrete storage and transfer realization.

## Validation boundary

Rules `TOPAL-SYSTEMS-ENTROPY-GENERATOR-001` and
`TOPAL-SYSTEMS-VIRTIO-PCI-ENTROPY-GENERATOR-001` require the sole initialization
after the first credit transfer. Negative tests reject omission, duplication,
reordering, substituted identities, algorithm, generation, key length,
required or observed credit, insufficient credit, or incompatible pool state.
The consumed pool cannot subsequently be credited or initialized again.

The checked root contains 139 fatal edges, an exact 298-event semantic trace,
2,670 relocations, and 2,307 UART calls. The sealed provider plan contains 80
operations. Artifact schema `/93`, provider and provider-object schemas `/85`,
artifact-root schema `/91`, boot-adapter schema `/53`, and gate schema `/95`
identify the contract.

## Physical evidence

The gate retains the pinned `pc-q35-10.2` machine, `qemu64-v1` CPU, SeaBIOS,
ISA serial console, and one `virtio-rng-pci-non-transitional` function at
`00:05.0`. The new marker follows payload completion, sample capture, explicit
quality attribution, pool creation, BLAKE2s mixing, contribution credit, the
verified key transfer, and verified pool wipe.

Two successful independent pinned-QEMU runs reached all 71 ordered
observations. Hashes for the recorded run are:

- linked kernel: `9a9c99ef1b490712d82b5a867e7f7106fbaf709fc07e6bd28c2b4190a09eed19`;
- boot image: `c7a3cae8ee394146b127c894aa12b62707c3b9bc7739453d579e0bb633345e1b`;
- boot resource: `4e40627cbf586de35544acdf166cd1e84476deb603773b856119285f8e1813a5`;
- generated image: `a5378022c142d7040e45b9ef8036b05338b8d28efd2c92ca5b547cb4a9fafc01`;
- supplied image: `085b9584104febbc97035fe96f1732d6b10fc266a45778f3d11f5220be50904e`;
- artifact provenance: `11d7bb47c57c14547365da28d730d7876098612f511652a51f8233ab9f5db2a8`;
- boot provenance: `2f9e6745abf67ed54bf3294d2569f6a98c7d3bababc0a112690f5db27b8d9e47`;
- serial output: `51548caa9c01e4271a0e26ab62171488a87311bca75d7d7e641e67a045a450b1`;
- evidence: `1b5bd01c5ca03ee6ae6203c748322e639f68dcf1dfebdc0efaf23177d05980d6`.

## Risk and remaining boundary

Risk is high because this increment establishes the secret generator key and
the readiness boundary used by later userspace randomness. Review covered the
exact credit threshold, affine pool consumption, distinct storage, complete
copy verification, mandatory source wipe and verification, readiness only
after provider success, malformed and reordered rejection, structural artifact
inspection, and duplicate complete physical execution.

Health tests, reseeding, fast key erasure, extraction, `AT_RANDOM`, `getrandom`,
multiple contributions, queue recycling and renotification, asynchronous
interrupt routing, reset, and recovery remain unavailable.
