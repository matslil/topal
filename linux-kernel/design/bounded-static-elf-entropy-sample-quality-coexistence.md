# Bounded static ELF entropy-sample quality coexistence

This record resolves `TK-DEC-171` as one explicit quality attribution followed
by complete assessed-sample destruction while the static process remains
behind its entropy-acquisition owner. It is a systems-profile refinement only
and adds no core-language element.

## Decision

After `TK-DEC-170`, `static image construction qualify entropy sample quality`
shall consume and return the exact acquisition owner. It shall accept only the
sealed initial virtio-rng device, DMA domain, queue and transport, sample
identity one, source identity one, trust-evidence identity one, and policy
`bounded-static-elf-entropy-sample-quality-coexistence`.

The provider shall completely revalidate the retained process and replay the
exact ordinary device sequence through zero-credit opaque sample capture. The
target-independent kernel policy shall then perform the exact assessment from
`TK-DEC-105`: 32 bytes, quality 1024 parts per 1024, and 256 credited bits under
the initial Q35 deployment evidence. Assessment shall neither inspect nor
change the bytes and shall invoke no architecture-specific trust decision.

Before returning, the transaction shall reset and verify the transport,
overwrite and read back the complete payload, ring, and assessed-sample
storage, reverse bus mastering, and restore gateway state. No assessed sample,
credit authority, byte, or DMA identity shall escape. The ordinary later
sequence shall remain structurally unconsumed. Success returns the same owner
and permits `TOPAL_KERNEL_STATIC_ELF_ENTROPY_SAMPLE_QUALITY_OK`.

## Architecture and deployment pressure test

Quality attribution is a target-independent affine policy transition and
requires no machine instruction. X86-64, AArch64, and RISC-V providers remain
responsible only for capture visibility and verified destruction. The full-
quality evidence is specific to the pinned Q35 deployment; another backend,
host trust model, migration regime, or physical source requires different
evidence and may admit less or no credit.

## Alternatives rejected

Implicit credit from successful virtio completion was rejected because device
mechanics are not trust evidence. Letting the target provider choose quality
was rejected because it would absorb deployment security policy into an
architecture adapter. Mixing the assessed sample into a pool was rejected
because attribution and conditioning are separate boundaries. Returning live
credit or trusting destruction without readback was rejected because either
would weaken affine ownership or confidentiality.

## Deliberate bounds

This increment admits one exact raw-to-assessed transition for sample identity
one, exact full-quality Q35 evidence, complete destruction and same-owner
return. It rejects another identity, source, evidence, extent, quality, credit,
policy, ordering, duplicate, escaped authority or byte, partial erase, process
mutation, or recoverable disposition. It does not create or mix a pool, seed or
extract a generator, publish `AT_RANDOM`, enter the static process, or add a
syscall.
