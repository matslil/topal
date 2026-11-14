# Step 4 bounded static ELF `AT_SECURE` fact

## Outcome

The qualified x86-64 path now derives initial execution security from the
sealed PID-one process profile, adds `AT_SECURE=(23,0)` after the credential
group and before `AT_NULL`, publishes
`TOPAL_KERNEL_STATIC_ELF_AUXV_SECURE_OK`, restores the exact
credential-qualified block, and publishes
`TOPAL_KERNEL_STATIC_ELF_AUXV_SECURE_REVOKED_OK`. Checked source sees only
affine owners, an opaque initial-execution-security fact, and sealed policies;
it receives no numeric security-mode or auxiliary-vector representation.

The provider validates the complete prior block and inactive address space
before and after each transition. It clears all 512 qwords, reconstructs the
complete aligned stack block with the non-secure-mode fact, completely reads
the result back, and performs the inverse whole-frame rollback.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-AUXV-SECURE-001` adds no core-language element.
Negative tests reject substituted ownership, security identity, rollback
policy, and markers. Affine semantic state rejects duplicate or out-of-order
lifecycle operations. Structural tests prove native tag 23, a sealed zero
value, exact order, complete resident-metadata comparison, 512-qword
clear/reconstruction, one fence, exact entry positions, and no active-address
invalidation.

Artifact `/131`, provider and provider-object `/121`, root object `/120`, gate
`/133`, boot adapter `/69`, and boot-resource `/2` identify this contract. The
checked source contains 332 operations; the provider plan contains 116; the
root contains 178 fatal edges, a 393-event semantic trace, 4,731 typed
relocations, 4,279 UART calls, 78 required text symbols, and 119 observations.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions completed all
119 observations in order and produced identical hashes:

- static image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `edc1d66cbdd0c3a2440813843e9d5a08f1c806f67f7f1dca9690a23a6041db80`;
- boot image: `57083085258d122ecf4174f9591718e67ea3aa3d43580ea465cb23cbb5bf9396`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `73e2c6e1e6e7074d222671ca941b2c97dd3aaa46e025f151e7fed7e9d78f8430`;
- boot provenance: `85fb64fefb94520f2391c099fc221a7889b6c6da1f58646d824311f8f66898c0`;
- serial: `af216d56ccda38b5c91b68c9cf39962c91aa2bdcf17eaeb019221be3152695d8`;
- evidence: `3c319d13bbe602f266526f2ce98b3f15be6e2334efc398ca65eee16e6d220510`.

## Risk and remaining boundary

Risk is high because an incorrect secure-execution fact could change libc
loader behavior and process trust decisions. Review covered derivation from
the sealed initial-process profile, exact zero-value agreement, whole-frame
coverage, retained credential group, inactive translation state, affine
rollback, structural machine code, negative cases, and duplicate physical
execution.

This increment removes the fact before returning. It does not add platform,
clock, or random facts, initialize TLS, construct a user context, enter ring 3,
execute BusyBox, or add a syscall.
