# Step 4 bounded static ELF hardware-capability facts

## Outcome

The qualified x86-64 path now derives native userspace capabilities from one
sealed provider fact, atomically adds `AT_HWCAP=(16,0x178bfbfd)` and
`AT_HWCAP2=(26,0)` after `AT_PLATFORM` and before `AT_NULL`, publishes
`TOPAL_KERNEL_STATIC_ELF_AUXV_HWCAP_OK`, restores the exact platform-qualified
block, and publishes `TOPAL_KERNEL_STATIC_ELF_AUXV_HWCAP_REVOKED_OK`. Checked
source sees only affine owners, an opaque
`native-userspace-hardware-capabilities` fact, and sealed policies; it receives
no mask, CPUID leaf, feature bit, or auxiliary-vector representation.

The provider validates the complete prior block and inactive address space
before and after each transition. It clears all 512 qwords, enlarges the
aligned entry block downward by 32 bytes without moving or overlapping its
three strings, reconstructs the complete block with the pinned `qemu64-v1`
capability pair, completely reads the result back, and performs the inverse
whole-frame rollback.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-AUXV-HWCAP-001` adds no core-language element.
Negative tests reject substituted ownership, capability identity, rollback
policy, and markers. Affine semantic state rejects duplicate or out-of-order
lifecycle operations. Structural tests prove native tags 16 and 26, exact mask
`0x178bfbfd`, zero `AT_HWCAP2`, the downward-aligned extended block, complete
512-qword clearing and reconstruction, one fence, exact rollback, preserved
strings and prior facts, and no active-address invalidation.

Artifact `/134`, provider and provider-object `/124`, root object `/123`, gate
`/136`, boot adapter `/69`, and boot-resource `/2` identify this contract. The
checked source contains 344 operations; the provider plan contains 122; the
root contains 184 fatal edges, a 405-event semantic trace, 4,997 typed
relocations, 4,533 UART calls, 84 required text symbols, and 125 observations.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions completed all
125 observations in order and produced identical hashes:

- static image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `4570b6ca3c139f23150cc0f204580bec96a60f7e7ba08fa513f99676c4570bc7`;
- boot image: `dd7e0abfd7d56e95c2733a76033c7ed2cb0b15d992b2bbac6a17ed8c25191d37`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `4ca8694d843793c602cf9721cddfae429bf13af581a00aecec1f2a1b8baa9322`;
- boot provenance: `e53fb0b2115ddb8040c8a1bc72eae6144bfe5c7d9b91a19cd53cc384c44a9d44`;
- serial: `62d3fd0d4ac617854af3729c6b6cef67c250ab12d8e6f236c64fab9f3ba74020`;
- evidence: `9ea5066addb5939886965c71573ab0f3541c8a0b73af6064a7789675de37394e`.

## Risk and remaining boundary

Risk is high because an overstated capability can make libc execute an
unsupported instruction, while a misplaced zero-valued `AT_HWCAP2` can be
mistaken for termination or overlap a string. Review covered exact guest-
profile derivation, atomic pair ownership, tag/value ordering, the downward
layout extension, complete non-overlap and readback, retained prior facts,
inactive translation state, affine rollback, structural machine code, negative
cases, and duplicate physical execution.

This increment removes the facts before returning. It does not add random data
or a vDSO, initialize TLS, construct a user context, enter ring 3, execute
BusyBox, or add a syscall.
