# Step 4 bounded static ELF load-metadata facts

## Outcome

The qualified x86-64 path now derives static-executable load metadata from the
validated image, atomically adds `AT_BASE=(7,0)` and `AT_FLAGS=(8,0)` after
`AT_HWCAP2` and before `AT_NULL`, publishes
`TOPAL_KERNEL_STATIC_ELF_AUXV_LOAD_METADATA_OK`, restores the exact capability-
qualified block, and publishes
`TOPAL_KERNEL_STATIC_ELF_AUXV_LOAD_METADATA_REVOKED_OK`. Checked source sees
only affine owners, an opaque `static-executable-load-metadata` fact, and sealed
policies; it receives no loader base, ELF flags, tag, or header representation.

The provider validates the complete prior block, the absent-interpreter fact,
accepted x86-64 ELF flags, and inactive address space before and after each
transition. It clears all 512 qwords, enlarges the aligned entry block downward
by 32 bytes without moving or overlapping its three strings, reconstructs the
complete block with both explicit zero-valued facts, completely reads the
result back, and performs the inverse whole-frame rollback.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-AUXV-LOAD-METADATA-001` adds no core-language
element. Negative tests reject substituted ownership, metadata identity,
rollback policy, and markers. Affine semantic state rejects duplicate or out-
of-order lifecycle operations. Structural tests prove native tags 7 and 8,
explicit zero values at their distinct entry positions, the downward-aligned
extended block, complete 512-qword clearing and reconstruction, one fence,
exact rollback, preserved strings and prior facts, and no active-address
invalidation.

Artifact `/135`, provider and provider-object `/125`, root object `/124`, gate
`/137`, boot adapter `/69`, and boot-resource `/2` identify this contract. The
checked source contains 348 operations; the provider plan contains 124; the
root contains 186 fatal edges, a 409-event semantic trace, 5,099 typed
relocations, 4,631 UART calls, 86 required text symbols, and 127 observations.

## Physical evidence

Two successful pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions completed all
127 observations in order and produced identical hashes. The independent run
used the supported 60-second allowance after an earlier attempt exhausted the
default 30 seconds under host load after producing the correct prefix.

- static image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `d71f375cf674b4c57b698e18b2e8c48021a673b7da9964255de4f32429ac1393`;
- boot image: `6735d66627a8fe89e90867aede1609b4eb0ee87071b3b932d0fe39fa861e13f7`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `7958816867ad567aa485000a2247cbc92a4471b6b1150953b0c68dbb1c11f0c8`;
- boot provenance: `82a022a2cc758745ec71fc5b908c6196b42b9bb49304809b7f01af94f0bce5cf`;
- serial: `662c05543bdb432d2cff9712e85dbce9436f634d684f54eacec2cdb2f5d433e8`;
- evidence: `343b3e3e1b15bc46e3a799f113b094ab0584cd2c037704863ec1fbcebddfc070`.

## Risk and remaining boundary

Risk is high because a zero-valued entry can be accidentally omitted or
mistaken for termination, and a false interpreter base can redirect dynamic-
loader behavior. Review covered exact agreement with the static `ET_EXEC`
plan, atomic pair ownership, tag/value ordering, the downward layout extension,
complete non-overlap and readback, retained prior facts, inactive translation
state, affine rollback, structural machine code, negative cases, and duplicate
successful physical execution.

This increment removes the facts before returning. It does not admit an
interpreter or dynamic loading, add random data or a vDSO, initialize TLS,
construct a user context, enter ring 3, execute BusyBox, or add a syscall.
