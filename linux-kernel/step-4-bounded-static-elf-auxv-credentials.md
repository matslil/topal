# Step 4 bounded static ELF credential facts

## Outcome

The qualified x86-64 path now derives one coherent credential group from the
sealed PID-one process profile, adds `AT_UID=(11,0)`, `AT_EUID=(12,0)`,
`AT_GID=(13,0)`, and `AT_EGID=(14,0)` after `AT_EXECFN` and before `AT_NULL`,
publishes `TOPAL_KERNEL_STATIC_ELF_AUXV_CREDENTIALS_OK`, restores the exact
executable-name-qualified block, and publishes
`TOPAL_KERNEL_STATIC_ELF_AUXV_CREDENTIALS_REVOKED_OK`. Checked source sees only
affine owners, an opaque initial-process-credentials fact, and sealed policies;
it receives no numeric credential or auxiliary-vector representation.

The provider validates the complete prior block and inactive address space
before and after each transition. It clears all 512 qwords, reconstructs the
complete aligned stack block with the atomic real/effective user/group group,
completely reads the result back, and performs the inverse whole-frame
rollback.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-AUXV-CREDENTIALS-001` adds no core-language
element. Negative tests reject substituted ownership, process-profile
identity, rollback policy, and markers. Affine semantic state rejects partial,
duplicate, or out-of-order lifecycle operations. Structural tests prove native
tags 11–14, four sealed zero values, exact order, complete resident-metadata
comparison, 512-qword clear/reconstruction, one fence, exact entry positions,
and no active-address invalidation.

Artifact `/130`, provider and provider-object `/120`, root object `/119`, gate
`/132`, boot adapter `/69`, and boot-resource `/2` identify this contract. The
checked source contains 328 operations; the provider plan contains 114; the
root contains 176 fatal edges, a 389-event semantic trace, 4,643 typed
relocations, 4,195 UART calls, 76 required text symbols, and 117 observations.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions completed all
117 observations in order and produced identical hashes:

- static image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `7c277a82e51a3354a1ab0af656f498ac43fa655068d632d1706bd9af803723c3`;
- boot image: `32f551f3e6987e545aba5e32a6747cf28b7a6cfdcb3fdf1dec417e16528d9642`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `8bd95595c439afaef9ebce286e3614b8e12fbd2669c3fca0dd3b4424d01cf836`;
- boot provenance: `c4e0d6eeb4956fa2911892efdf2e98c3bdfea33037e504e2a4b5e86bce4d3486`;
- serial: `a6de86e918c050cf8ca1bf6b25ba41ce76ea506e4496a73decf45d6e01758763`;
- evidence: `cb654aa7d68cc34e8810a3493763248db9e2c03614d1e0b4593df0eaa9f6d7f8`.

## Risk and remaining boundary

Risk is high because inconsistent credential facts could misrepresent process
identity to future userspace. Review covered atomic group identity, exact
profile agreement, whole-frame coverage, retained executable-name alias,
inactive translation state, affine rollback, structural machine code,
negative cases, and duplicate physical execution.

This increment removes the group before returning. It does not add platform,
clock, secure-mode, or random facts, initialize TLS, construct a user context,
enter ring 3, execute BusyBox, or add a syscall.
