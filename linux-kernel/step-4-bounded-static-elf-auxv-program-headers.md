# Step 4 bounded static ELF program-header facts

## Outcome

The qualified x86-64 path now derives one coherent resident program-header
fact group, adds `AT_PHDR`, `AT_PHENT`, and `AT_PHNUM` after `AT_ENTRY` and
before `AT_NULL`, publishes
`TOPAL_KERNEL_STATIC_ELF_AUXV_PROGRAM_HEADERS_OK`, restores the exact entry-
qualified block, and publishes
`TOPAL_KERNEL_STATIC_ELF_AUXV_PROGRAM_HEADERS_REVOKED_OK`. Checked source sees
only affine owners, opaque resident metadata authority, and sealed policies;
it receives no ELF or auxiliary-vector representation.

The provider validates all 680 resident ELF header and program-header-table
bytes before and after each transition, the exact immutable header leaf, and
the inactive address space. It reconstructs a 16-byte-aligned block containing
`AT_PHDR=(3,0x400040)`, `AT_PHENT=(4,56)`, `AT_PHNUM=(5,11)`, every prior fact,
and `AT_NULL`; completely reads it back; and performs the inverse whole-frame
rollback.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-AUXV-PHDR-001` adds no core-language element.
Negative tests reject substituted ownership, metadata identity, policies, and
markers. Affine semantic state rejects partial, duplicate, or out-of-order
lifecycle operations. Structural tests prove the exact coherent group,
complete resident metadata comparison, 512-qword clear/reconstruction, one
fence, aligned entry positions, and no active-address invalidation.

Artifact `/128`, provider and provider-object `/118`, root object `/117`, gate
`/130`, boot adapter `/69`, and boot-resource `/2` identify this contract. The
checked source contains 320 operations; the provider plan contains 110; the
root contains 172 fatal edges, a 381-event semantic trace, 4,457 typed
relocations, 4,017 UART calls, 72 required text symbols, and 113 observations.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions completed all
113 observations in order and produced identical hashes:

- static image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `8c85a899fe9c9320a56e758c968e87f6dc780c2072430e523309fae414a00aee`;
- boot image: `bfbcf77047c964176a65c27dcb8ad66937a30c5d47c574a37db9f9820c14b128`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `45897dffe881fb77086875116cd3daa5dab0f74e639fe7f4faedda01e7e711f7`;
- boot provenance: `8d18de6ac94006e46621c9ecc25da05fb7d33bcf8f74e2b309643a368ee307a1`;
- serial: `d267b62d1a7454383f5181521c397f27d40107fd00a0a8007fa145a950f22af8`;
- evidence: `55ce27d2e864016a7dd5626210c913234165f399d30c53ffa365f8a6168d56f6`.

## Risk and remaining boundary

Risk is high because inconsistent resident-table facts could make a loader
parse unauthorized or malformed metadata. Review covered complete byte
identity, table bounds and shape, exact order, whole-frame coverage, inactive
translation state, affine rollback, structural machine code, negative cases,
and duplicate physical execution.

This increment removes the group before returning. It does not add credential,
platform, clock, secure-mode, or random facts, initialize TLS, construct a user
context, enter ring 3, execute BusyBox, or add a syscall.
