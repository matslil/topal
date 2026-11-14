# Step 4 bounded static ELF `AT_PLATFORM` fact

## Outcome

The qualified x86-64 path now derives the native userspace platform from a
sealed provider fact, adds `AT_PLATFORM=(15,pointer-to-"x86_64\0")` after
`AT_CLKTCK` and before `AT_NULL`, publishes
`TOPAL_KERNEL_STATIC_ELF_AUXV_PLATFORM_OK`, restores the exact clock-rate-
qualified block, and publishes
`TOPAL_KERNEL_STATIC_ELF_AUXV_PLATFORM_REVOKED_OK`. Checked source sees only
affine owners, an opaque `native-userspace-platform` fact, and sealed policies;
it receives no platform string, pointer, or auxiliary-vector representation.

The provider validates the complete prior block and inactive address space
before and after each transition. It clears all 512 qwords, reconstructs the
complete aligned stack block with the distinct bounded platform string,
completely reads the result back, and performs the inverse whole-frame
rollback.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-AUXV-PLATFORM-001` adds no core-language
element. Negative tests reject substituted ownership, platform identity,
rollback policy, and markers. Affine semantic state rejects duplicate or out-
of-order lifecycle operations. Structural tests prove native tag 15, the exact
bounded pointer and `x86_64\0` bytes, separation from the `busybox` executable-
name object, complete resident-metadata comparison, 512-qword clear and
reconstruction, one fence, exact entry positions, and no active-address
invalidation.

Artifact `/133`, provider and provider-object `/123`, root object `/122`, gate
`/135`, boot adapter `/69`, and boot-resource `/2` identify this contract. The
checked source contains 340 operations; the provider plan contains 120; the
root contains 182 fatal edges, a 401-event semantic trace, 4,911 typed
relocations, 4,451 UART calls, 82 required text symbols, and 123 observations.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions completed all
123 observations in order and produced identical hashes:

- static image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `6e0f8eab2139d51112b7af19640289bf1eb6bd0004cbf259978dcf8a13093afc`;
- boot image: `1a04f05a47cabf234fba14a7b77c7bc012ffad14b646188dc27438f0c09ebcbe`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `445881e6d27bde1b0416704503fa919490b21993d3e970d385ac4ffe1afcd972`;
- boot provenance: `739770fd212db304e6065f44e670ddb75a9f3e221723b6a1b80b8a7b64345fd0`;
- serial: `24bbc156911ad1481741c638f831f6f2eb852cc25880d50be257396ce7f927e1`;
- evidence: `275c9128b7850fd5e43b422b789bc3b11584a2a9d81000c222f6235f1b260803`.

## Risk and remaining boundary

Risk is high because a forged pointer, unterminated string, or accidental alias
could expose invalid ABI data or conflate executable and platform identity.
Review covered the sealed provider identity, complete pointer containment and
string bytes, alias separation, whole-frame coverage, retained prior facts,
inactive translation state, affine rollback, structural machine code, negative
cases, and duplicate physical execution.

This increment removes the fact before returning. It does not add random or
hardware-capability facts, initialize TLS, construct a user context, enter ring
3, execute BusyBox, or add a syscall.
