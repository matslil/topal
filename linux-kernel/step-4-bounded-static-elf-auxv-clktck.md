# Step 4 bounded static ELF `AT_CLKTCK` fact

## Outcome

The qualified x86-64 path now derives Linux's userspace accounting rate from a
sealed ABI fact, adds `AT_CLKTCK=(17,100)` after `AT_SECURE` and before
`AT_NULL`, publishes `TOPAL_KERNEL_STATIC_ELF_AUXV_CLKTCK_OK`, restores the
exact secure-mode-qualified block, and publishes
`TOPAL_KERNEL_STATIC_ELF_AUXV_CLKTCK_REVOKED_OK`. Checked source sees only
affine owners, an opaque `linux-user-hz` fact, and sealed policies; it receives
no numeric rate or auxiliary-vector representation.

The provider validates the complete prior block and inactive address space
before and after each transition. It clears all 512 qwords, reconstructs the
complete aligned stack block with the accounting-rate fact, completely reads
the result back, and performs the inverse whole-frame rollback.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-AUXV-CLKTCK-001` adds no core-language element.
Negative tests reject substituted ownership, clock identity, rollback policy,
and markers. Affine semantic state rejects duplicate or out-of-order lifecycle
operations. Structural tests prove native tag 17, sealed value 100, separation
from hardware and scheduler clocks, exact order, complete resident-metadata
comparison, 512-qword clear/reconstruction, one fence, exact entry positions,
and no active-address invalidation.

Artifact `/132`, provider and provider-object `/122`, root object `/121`, gate
`/134`, boot adapter `/69`, and boot-resource `/2` identify this contract. The
checked source contains 336 operations; the provider plan contains 118; the
root contains 180 fatal edges, a 397-event semantic trace, 4,819 typed
relocations, 4,363 UART calls, 80 required text symbols, and 121 observations.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions completed all
121 observations in order and produced identical hashes:

- static image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `4ad31345764d1ee57487669c21da5bb24dcd46cfcd2183fe94fae0a38ac7ee6b`;
- boot image: `dbaaa769fb0133cbb83aa46dfc0da238762ba033a2d1782627d2a696f04d81a5`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `04b1ddcd485a28e66525746e328d906d28f0cd3525f4b20d65f027fa47585bc2`;
- boot provenance: `cf2fee813da18c3452e46e76c5ce9986f3e936c6620af541aac3886aca451ecc`;
- serial: `41e5f03657acfa65235fef09063fb082bbd2d57c22fa070349d320e05f479b2f`;
- evidence: `e576bd99bcb5a2bf6175cbf0103b217cc154364eb328bd95803cd04e76835b47`.

## Risk and remaining boundary

Risk is moderate-high because conflating `USER_HZ` with a hardware or scheduler
frequency would expose an incorrect libc ABI. Review covered the sealed Linux
rate, clock-identity separation, whole-frame coverage, retained security and
credential facts, inactive translation state, affine rollback, structural
machine code, negative cases, and duplicate physical execution.

This increment removes the fact before returning. It does not add platform or
random facts, initialize TLS, construct a user context, enter ring 3, execute
BusyBox, or add a syscall.
