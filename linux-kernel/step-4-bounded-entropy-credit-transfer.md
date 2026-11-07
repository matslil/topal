# Step 4 bounded entropy-credit transfer

## Outcome

The Topal kernel now consumes affine contribution identity one into mixed pool
identity one through `topal.systems.entropy.pool.credit-contribution/1`. The
checked transition moves the pool from zero to exactly 256 credited bits at
capacity 256, consumes the contribution so it cannot be reused, and emits
`TOPAL_KERNEL_ENTROPY_POOL_CREDITED`.

Credit transfer is source-owned trust policy. It emits no provider call,
preserves the complete provider-owned opaque BLAKE2s pool state, and exposes no
byte, digest, sample material, address, representation, or target instruction.
A full credit count still does not imply generator readiness or random output.

## Validation boundary

Rules `TOPAL-SYSTEMS-ENTROPY-CREDIT-001` and
`TOPAL-SYSTEMS-VIRTIO-PCI-ENTROPY-CREDIT-001` require exactly one transfer after
the first mix. Negative tests reject omission, duplication, reordering,
substituted pool, contribution, or conditioner-origin identities, another
prior value, contribution, capacity, result, overflow, reuse, or incompatible
pool state.

The checked root contains 138 fatal edges, an exact 296-event semantic trace,
2,630 relocations, and 2,271 UART calls. The sealed provider plan remains 79
operations, proving the accounting transition gained no provider lowering.
Artifact schema `/92`, provider and provider-object schemas `/84`, artifact-root
schema `/90`, boot-adapter schema `/52`, and gate schema `/94` identify the
contract.

## Physical evidence

The gate retains the pinned `pc-q35-10.2` machine, `qemu64-v1` CPU, SeaBIOS,
ISA serial console, and one `virtio-rng-pci-non-transitional` function at
`00:05.0`. The new marker follows completion acknowledgement, used-ring
validation, payload ownership return, sample capture, explicit quality
attribution, pool creation, BLAKE2s mixing, state publication, and sample wipe.

Two successful independent pinned-QEMU runs were byte-identical and reached
all 70 ordered observations. Hashes for the complete runs are:

- linked kernel: `a4e877f2f9618bf39bb08c87fcd717c0d61dafdbad9f26193fb334bf3d96ac88`;
- boot image: `28f472122d38ab5cbcb5b1fd34527556b4b35d1675e7c457086e0787ee4a9e5a`;
- boot resource: `4e40627cbf586de35544acdf166cd1e84476deb603773b856119285f8e1813a5`;
- generated image: `a5378022c142d7040e45b9ef8036b05338b8d28efd2c92ca5b547cb4a9fafc01`;
- supplied image: `085b9584104febbc97035fe96f1732d6b10fc266a45778f3d11f5220be50904e`;
- artifact provenance: `f7334a6b7a002b74659d880f031091d2409df8414e8a9e8f97521974a79bb864`;
- boot provenance: `cb47e1ffc1f5541cde4ce570586cf8e01e28a63c140fa436ea62e8afceadccf9`;
- serial output: `e3c6810b1d457f6c370da6c212c5d2a40d410e0fe26dcf1b50066a6875fbc453`;
- evidence: `5fa212e707ed28c3320add7440360bbe380e78e844b93148c14b1e217be3c000`.

## Risk and remaining boundary

Risk is medium because this increment changes security-sensitive trust
accounting but neither accesses secret bytes nor adds provider instructions.
Review covered exact checked addition, bounded capacity, affine token
consumption, double-credit prevention, origin and pool binding, opaque-state
preservation, provider-plan stability, malformed and reordered rejection,
structural artifact inspection, and duplicate complete physical execution.

Health tests, readiness, CRNG initialization and reseeding, extraction,
`AT_RANDOM`, `getrandom`, multiple contributions, queue recycling and
renotification, asynchronous interrupt routing, reset, and recovery remain
unavailable.
