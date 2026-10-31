# Bounded entropy-sample quality attribution

This record resolves `TK-DEC-105` as the first explicit trust decision for the
captured virtio-rng sample. It attributes quality in source-owned kernel policy
without exposing sample bytes or moving deployment trust into an architecture
provider.

## Decision

After `TK-DEC-104`, one generic entropy-sample quality operation consumes the
raw state of sample identity one and returns the same affine identity in an
assessed state. The request names source identity one, trust-evidence identity
one, the exact 32-byte sample extent, quality 1024 parts per 1024, and 256
credited entropy bits. The initial Q35 profile supplies that evidence as an
explicit assumption that its configured host entropy service is fully trusted.

The credited value must equal
`floor(sample-bits * quality-parts-per-1024 / 1024)` and cannot exceed the
sample extent. Assessment neither reads nor changes sample bytes. It records a
policy fact needed by a later pool-mixing protocol; it does not itself create
an entropy pool, condition data, seed a generator, or make output available.

The trust evidence belongs to the board/deployment profile, not to the
virtio-rng device class and not to the target provider. A different VM backend,
bare-metal RNG, or deployment may supply a lower-quality evidence resource or
none at all. Device discovery and successful transport completion never imply
quality.

## Architecture and deployment pressure test

Quality assessment is target-independent and requires no machine instruction,
cache operation, address, or provider call. x86-64, AArch64, and RISC-V use the
same affine state transition. Their providers remain responsible only for the
earlier DMA visibility and opaque byte-copy boundaries.

The initial QEMU profile is deliberately deployment-specific: the harness
configures the virtio-rng backend from the host operating system and records
full-quality trust as a qualification assumption. Nested virtualization,
untrusted hosts, migration, confidential guests, physical entropy devices, and
adversarial backends require their own evidence and are not covered by this
decision.

## Alternatives rejected

Implicitly crediting every captured `EntropyPayload` was rejected because a
semantic role is not trust evidence. Letting the x86 provider select quality was
rejected because providers realize target mechanics rather than deployment
security policy. Mixing and crediting in one operation was rejected because
successful cryptographic mixing does not prove input entropy. Leaving the
sample permanently at zero credit was rejected because it would prevent the
qualified profile from ever establishing random readiness despite its explicit
trusted-host assumption.

## Deliberate bounds

This increment admits one transition of sample identity one from raw and
zero-credit to assessed and 256-credit, using source identity one, evidence
identity one, byte count 32, and quality 1024/1024. It rejects another identity,
extent, evidence, quality, credit, omission, duplication, reordering, or
assessment before capture.

It does not inspect, condition, mix, serialize, expose, or destroy the sample;
create or seed a pool or CRNG; populate `AT_RANDOM`; implement `getrandom`;
recycle the virtqueue; configure asynchronous interrupts; or define reset and
recovery.
