# Bounded initial CRNG seeding

This record resolves `TK-DEC-109` as the first ready random-generator state.
It maps the current Linux threshold-triggered reseed boundary to an affine
Topal lifecycle without exposing keys, pool bytes, or target instructions.

## Decision

After `TK-DEC-108`, one generic generator-initialization operation consumes
credited pool identity one into fresh generator identity one. The pool must
hold exactly 256 credited bits at its 256-bit capacity and a complete 32-byte
opaque conditioned state. The operation transfers those 32 bytes as the
ChaCha20 key for generator generation one, completely wipes the consumed pool
storage, and returns the generator in the `Ready` state.

The fixed initial profile treats the conditioned BLAKE2s-256 pool state as the
full-width ChaCha20 key without truncation, padding, byte reversal, or
source-visible derivation. The state is already a domain-separated
cryptographic digest of the accepted sample. Moving it rather than retaining
two live copies shortens secret lifetime and makes pool consumption explicit.

Source names pool identity one, generator identity one, algorithm identity
one, generation one, key extent 32, required credit 256, and observed pool
credit 256. It observes no key, pool-state byte, address, representation,
register, or target instruction. The provider must copy all 32 bytes into
distinct generator storage, verify the complete copy, wipe and verify all 32
pool bytes, and fail closed before publishing partial or duplicated state.

Initialization establishes readiness but produces no output. A later affine
generator operation must define ChaCha20 state construction, output limits,
fast key erasure, generation advancement, and the selected Linux interface.

The current Linux random subsystem similarly treats the 256-bit ready
threshold as the point that extracts a 32-byte key into the base CRNG and
marks it ready. Topal keeps the preceding mixing and credit transitions
separate so failures and trust policy remain explicit, but preserves this
readiness boundary rather than inventing a pool-only ready state.

## Architecture pressure test

ChaCha20's 32-byte key and the byte-level transfer have identical meaning on
x86-64, AArch64, and RISC-V. Providers may use different scalar, vector, or
copy instructions only when qualification proves the same complete transfer,
distinct storage, and destruction behavior. Endianness matters only when a
later ChaCha state is constructed; it does not change these key bytes.

Generator identity, generation, readiness, and affine ownership remain
portable source semantics. Storage placement, copy instructions, wipe
mechanics, and verification instructions remain provider-private.

## Alternatives rejected

Marking the credited pool itself ready was rejected because Linux readiness is
a CRNG property and users consume generator output, not pool state. Retaining
the credited pool after copying was rejected because it duplicates the live
seed. Exposing an ordinary source byte array or generic hash/cipher API was
rejected because keys require opaque affine ownership and mandatory wiping.
Provider-selected algorithms were rejected because readiness and output
compatibility must not vary by target. Producing `AT_RANDOM` or satisfying
`getrandom` during initialization was rejected because seeding and extraction
have different failure, consumption, and output contracts.

## Deliberate bounds

This increment admits one pool-to-generator transition with pool identity one,
generator identity one, algorithm identity one, generation one, a 32-byte key,
and exactly 256 credited bits at capacity. It rejects another identity,
algorithm, generation, extent, credit, state, omission, duplication,
reordering, partial copy, incomplete wipe, or initialization before credit.

It does not implement ChaCha20 block construction, output, fast key erasure,
reseed scheduling, per-CPU generators, `AT_RANDOM`, `getrandom`, queue
recycling, asynchronous interrupt routing, reset, or recovery.
