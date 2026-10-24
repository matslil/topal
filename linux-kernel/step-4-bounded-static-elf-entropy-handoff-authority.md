# Step 4 bounded static ELF entropy-handoff authority

## Outcome

The qualified x86-64 path now transfers the complete retained static ELF
process into one opaque entropy-pending affine owner and can revoke that
handoff back to the exact retained-process owner. Authorization validates the
complete prepared process, including its private stack backing and authorized
random slot, publishes
`TOPAL_KERNEL_STATIC_ELF_ENTROPY_HANDOFF_AUTHORIZED_OK`, and closes the loader
owner. Qualification then validates the same unchanged state, publishes
`TOPAL_KERNEL_STATIC_ELF_ENTROPY_HANDOFF_REVOKED_OK`, and restores the retained
owner for the existing teardown path.

The checked source names only the retained process, its stable process
identity, and sealed authorization and rollback policies. It cannot observe
the random-slot address, stack-backing identity, mapping resources, or target
representation. The handoff therefore establishes a subsystem boundary
without exposing loader representation or granting more authority than the
future entropy operation needs.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-HANDOFF-001` adds no core-language
element. Negative tests reject substituted owners, process identities,
policies, and markers. The affine semantic model permits exactly one live
retained or entropy-pending process owner, requires the authorized random slot
and closed construction allocator channel, and rejects completion while either
owner remains live. Structural tests prove complete unchanged-state readback,
absence of stores, fences, invalidation, allocation, or entropy access, and
exact later restoration.

Artifact `/139`, provider and provider-object `/129`, root object `/128`, gate
`/141`, boot adapter `/69`, and boot-resource `/2` identify this contract. The
checked source contains 362 operations; the provider plan contains 131; the
root contains 194 fatal edges, a 423-event semantic trace, 5,466 typed
relocations, 4,982 UART calls, 93 required text symbols, and 134 observations.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions completed all
134 observations in order within the supported 60-second allowance and
produced identical hashes.

- static image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `33979da72994c00bccb70be400a409639e47818d5428368b4de138b086803de0`;
- boot image: `7c7c00dd6e62df4bb30b5bc3138db3a7a1d3d3bc3e7553df16087b1214afc530`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `80106a2b8d3645f5c0bbeb0fb1f2d54bdf64eb945cf179b1f1519108befb3686`;
- boot provenance: `e2f2a6f45ebca82f017a7a2f1b27a99500ca1c6e6f0d285fb889903cd1c01d23`;
- serial: `df6e702edaa899180cd20f8dfefea67661fa0f8be60cab349899971d88f1420e`;
- evidence: `f10a1a34a5e2d581a2328b19a8e98cdc31b4e171399d83a061ae326d0498b1f1`.

## Risk and remaining boundary

Risk is high because duplicating either affine owner or permitting mutation
during the handoff would break exclusive ownership of a fully prepared user
process. Review covered exact owner consumption and production, complete
process revalidation, preservation of the private backing identity, structural
machine code, negative cases, exact rollback, and duplicate successful
physical execution.

This increment establishes and qualifies only the ownership boundary. It does
not yet retain the entropy-pending owner across entropy acquisition, consume
entropy, write or publish `AT_RANDOM`, initialize TLS, construct a user
context, enter ring 3, execute BusyBox, or add a syscall.
