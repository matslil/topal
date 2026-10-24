# Step 4 bounded static ELF random-slot authority

## Outcome

The qualified x86-64 path now validates the complete load-metadata-qualified
static ELF stack, proves one aligned 16-byte zero destination and space for one
future native `AT_RANDOM` entry, publishes
`TOPAL_KERNEL_STATIC_ELF_RANDOM_SLOT_AUTHORIZED_OK`, preserves the entry block
byte-for-byte, revokes the authority, and publishes
`TOPAL_KERNEL_STATIC_ELF_RANDOM_SLOT_REVOKED_OK`. Checked source sees only an
affine owner, the semantic random-object identity, extent 16, and sealed
policies; it receives no tag, byte, pointer, address, offset, or encoding.

The provider completely validates the inactive hierarchy, exact RW/NX stack
leaf, guard, image leaves, resident headers, deterministic entry block, fixed
strings, and otherwise-zero frame on both transitions. The future object's two
qwords are proved zero. Neither transition stores a byte, fences, nor
invalidates an active address.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-RANDOM-SLOT-001` adds no core-language element.
Negative tests reject substituted ownership, value, extent, policies, and
markers. Affine semantic state rejects duplicate, premature, or out-of-order
lifecycle operations. Structural tests prove complete unchanged-frame
readback, both zero object qwords, absent stores and store fences, preserved
translation state, and exact revocation.

Artifact `/136`, provider and provider-object `/126`, root object `/125`, gate
`/138`, boot adapter `/69`, and boot-resource `/2` identify this contract. The
checked source contains 352 operations; the provider plan contains 126; the
root contains 188 fatal edges, a 413-event semantic trace, 5,198 typed
relocations, 4,726 UART calls, 88 required text symbols, and 129 observations.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions completed all
129 observations in order within the supported 60-second allowance and
produced identical hashes.

- static image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `81890d6b21f7aa3ad1e24647107c6fde57463ed9d23127d5487dc5b969acb92a`;
- boot image: `c8645153c736a3f7595584fed3eaa5b446ed936066aa3878961deea7551ca329`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `f504fd82242d6b80a1d7ad3130b83787dfda3122b525f0f4ea72421a29ef1a0f`;
- boot provenance: `17fa58bd83536d64dab31e90dd2a06244a0614bf2ee880ae51613f27736553f0`;
- serial: `687aa9ed8a88e0653b241d04b912eaacd6efb772f16e67baf1648a9d2c388346`;
- evidence: `bd68f00f1c3b115eae414674d501522f11f940591d1e91b03996d16d6b51cde2`.

## Risk and remaining boundary

Risk is high because publishing a pointer before secret initialization would
create false Linux ABI state, while an overlapping or source-forgeable slot
would corrupt process-entry data or leak representation. Review covered exact
owner and policy agreement, complete bounds and alignment, object identity,
zero-state, non-overlap with every current object and guard, unchanged stack
content, inactive translation state, affine rollback, structural machine code,
negative cases, and duplicate successful physical execution.

This increment revokes the authority before returning. It does not retain the
static construction across the entropy pipeline, consume or expose random
material, publish `AT_RANDOM`, initialize TLS, construct a user context, enter
ring 3, execute BusyBox, or add a syscall.
