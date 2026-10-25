# Bounded static ELF entropy-pool lifecycle coexistence

This record resolves `TK-DEC-172` as one complete create, mix, credit, and
verified-destruction lifecycle for the initial entropy pool while the static
process remains behind its acquisition owner. It is a systems-profile
refinement only and adds no core-language element.

## Decision

After `TK-DEC-171`, `static image construction qualify entropy pool lifecycle`
shall consume and return the exact acquisition owner. It shall accept only the
sealed initial transport resources and Q35 quality evidence, sample identity
one, pool identity one, BLAKE2s conditioner identity one, contribution identity
one, and policy `bounded-static-elf-entropy-pool-lifecycle-coexistence`.

The provider shall completely revalidate the retained process and replay the
exact ordinary device path through the assessed 32-byte, 256-credit sample. It
shall create the exact empty 256-bit-capacity pool from `TK-DEC-106`, perform
the exact first-generation BLAKE2s mixing transition from `TK-DEC-107`, and
transfer the exact live 256-bit contribution into pool credit as required by
`TK-DEC-108`. Creation and credit are target-independent policy transitions;
only the opaque conditioner receives target lowering.

Before returning, the transaction shall reset and verify the transport,
overwrite and read back the payload and ring, then overwrite and read back all
provider-private sample and pool storage. It shall reverse bus mastering and
restore gateway state. No sample, pool, contribution, credit authority, byte,
or DMA identity shall escape, and the ordinary later sequence shall remain
structurally unconsumed. Success returns the same owner and permits
`TOPAL_KERNEL_STATIC_ELF_ENTROPY_POOL_LIFECYCLE_OK`.

## Architecture pressure test

The pool and sample are opaque semantic resources, and conditioning is an
algorithm contract rather than an instruction sequence. X86-64 may call its
qualified BLAKE2s realization; AArch64 and RISC-V may use distinct scalar or
accelerated code only with identical complete-input, domain-separation,
contribution, credit, and destruction semantics. Providers must erase every
private copy created for conditioning.

## Alternatives rejected

Treating assessment as an already credited pool was rejected because trust,
conditioning, and accounting are distinct. Crediting before complete mixing
was rejected because it could make unconditioned input available. Combining
generator initialization was rejected because pool readiness and generator
key ownership are separate boundaries. Returning a live pool beside the same
acquisition owner or trusting destruction without readback was rejected
because either would weaken affine ownership or confidentiality.

## Deliberate bounds

This increment admits exactly one empty pool, one assessed sample, one BLAKE2s
first contribution, exact zero-to-256 credit transfer, complete destruction,
and same-owner return. It rejects another identity, capacity, conditioner,
contribution, credit arithmetic, order, duplicate, escaped authority or byte,
partial erase, process mutation, or recoverable disposition. It does not seed
or extract a generator, publish `AT_RANDOM`, enter the static process, or add a
syscall.
