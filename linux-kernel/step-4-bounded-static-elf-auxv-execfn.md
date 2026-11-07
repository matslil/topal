# Step 4 bounded static ELF `AT_EXECFN` fact

## Outcome

The qualified x86-64 path now derives one executable-name fact from the
existing sealed `argv[0]` string, adds `AT_EXECFN=(31,0x5fffe0)` after the
coherent program-header group and before `AT_NULL`, publishes
`TOPAL_KERNEL_STATIC_ELF_AUXV_EXECFN_OK`, restores the exact program-header-
qualified block, and publishes
`TOPAL_KERNEL_STATIC_ELF_AUXV_EXECFN_REVOKED_OK`. Checked source sees only
affine owners, an opaque initial-executable-name fact, and sealed policies; it
receives neither the pointer nor the auxiliary-vector representation.

The provider validates the complete resident ELF metadata and inactive stack
leaf before and after each transition. It clears all 512 qwords, reconstructs
the complete aligned stack block, aliases `AT_EXECFN` to the existing
`busybox\0` `argv[0]` object, completely reads the result back, and performs
the inverse whole-frame rollback.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-AUXV-EXECFN-001` adds no core-language element.
Negative tests reject substituted ownership, executable-name identity,
rollback policy, and markers. Affine semantic state rejects partial,
duplicate, or out-of-order lifecycle operations. Structural tests prove tag
31, the exact `0x5fffe0` alias, complete resident-metadata comparison,
512-qword clear/reconstruction, one fence, exact entry positions, and no
active-address invalidation.

Artifact `/129`, provider and provider-object `/119`, root object `/118`, gate
`/131`, boot adapter `/69`, and boot-resource `/2` identify this contract. The
checked source contains 324 operations; the provider plan contains 112; the
root contains 174 fatal edges, a 385-event semantic trace, 4,545 typed
relocations, 4,101 UART calls, 74 required text symbols, and 115 observations.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions completed all
115 observations in order and produced identical hashes:

- static image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `e53c129d2576ddada032d9b041f7f511badf05bfc3575b1f412e0a4de7b4b7af`;
- boot image: `373750789a45afbabe618845a320b3f53a60c9a4a7d4c56dc91cf97014bc3a39`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `d70510a4199bbb31cd1ea0c3bb7497659407a49991f8678e7bddd0cfc7608648`;
- boot provenance: `6ee8871f9e82dc340057da6cd8aecc6af6316209528f41f4ed36cb0cf22b35c1`;
- serial: `b34e11111c1689a615c8a34e7b73e985d213da86c1999cfa2660c050dbe16893`;
- evidence: `96f9a36f5b136a703b7376b76e106d73baa53e378c5b441a43a5feb0faaf15dc`.

## Risk and remaining boundary

Risk is high because an incorrect pointer alias or lifetime could expose an
invalid executable name to a future process image. Review covered exact alias
identity and bounds, whole-frame coverage, retained resident metadata,
inactive translation state, affine rollback, structural machine code,
negative cases, and duplicate physical execution.

This increment removes the fact before returning. It does not add credential,
platform, clock, secure-mode, or random facts, initialize TLS, construct a user
context, enter ring 3, execute BusyBox, or add a syscall.
