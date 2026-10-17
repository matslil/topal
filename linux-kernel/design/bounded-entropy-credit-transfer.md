# Bounded entropy-credit transfer

This record resolves `TK-DEC-108` as the first explicit trust-accounting
transition for opaque mixed entropy. It consumes the contribution created by
`TK-DEC-107` without reprocessing secret bytes or conflating credited capacity
with generator readiness.

## Decision

After the first-generation mix, one generic entropy-credit operation consumes
mixed-but-uncredited pool identity one and contribution identity one. The
contribution must identify pool one, carry exactly 256 untransferred bits, and
originate from the already completed conditioner identity one transition. The
operation returns the same affine pool identity with 256 credited bits and no
live contribution token.

The pool has a fixed credit capacity of 256 bits, so the initial transition is
an exact non-saturating addition from zero to capacity. Source names the pool,
contribution, prior credit, transferred credit, capacity, and resulting credit.
It observes no pool bytes, digest, sample material, storage address, provider
mechanism, or target instruction.

Credit transfer is source-owned trust policy and has no provider operation.
The provider-owned opaque pool state remains byte-for-byte unchanged. A
successful transfer establishes only a fully credited mixed pool; it does not
declare a generator ready, extract output, or authorize userspace delivery.
Those require later distinct source-owned lifecycle transitions.

## Architecture pressure test

The transition is pure affine accounting over identities and bounded natural
values. x86-64, AArch64, and RISC-V use the same operation without lowering,
endianness, atomic instructions, or memory representation. A future concurrent
pool service may require synchronization around ownership transfer, but that
does not alter this single-owner semantic boundary.

Separating credit from the conditioner lets future providers change their
private scalar or vector realization without gaining authority over trust. It
also permits a future health-test operation to withhold or reduce a
contribution before transfer without exposing the underlying bytes.

## Alternatives rejected

Crediting automatically when mixing succeeds was rejected because processing
bytes proves neither their unpredictability nor their accepted trust evidence.
Leaving the contribution live after transfer was rejected because it permits
double credit. Letting the provider update credit was rejected because credit
is portable security policy, not target mechanics. Treating a full credit
counter as generator readiness was rejected because readiness also requires a
separate initialized generator state and output policy. Saturating arbitrary
input was rejected for the initial profile because it could hide excess or
misdirected credit.

## Deliberate bounds

This increment admits one transfer for pool identity one and contribution
identity one, from exactly zero credited bits to exactly 256 credited bits at
capacity 256. It rejects another identity, origin, capacity, prior value,
transfer value, result, pool state, reused contribution, omission, duplication,
or reordering.

It does not define health tests, partial or saturating transfer, revocation,
multiple contributions, readiness, reseeding, CRNG state, extraction,
`AT_RANDOM`, `getrandom`, queue recycling, asynchronous interrupt routing,
reset, or recovery.
