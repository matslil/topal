# Bounded entropy-pool creation

This record resolves `TK-DEC-106` as the first entropy-pool lifecycle
operation. It creates a source-owned affine pool resource before assessed
sample material is mixed, without choosing a cryptographic construction or
exposing storage.

## Decision

After `TK-DEC-105`, one generic entropy-pool creation operation produces pool
identity one in an empty state. The request names a credit capacity of 256
bits and exactly zero initial credited bits. Creation is source-owned kernel
policy and has no provider call, address, byte representation, target
instruction, or implicit allocation.

The pool identity is affine. It must be created exactly once before any later
mix operation and cannot be duplicated, substituted, or recreated. Empty-pool
creation neither borrows nor consumes the assessed sample, and it does not
make random output available. A later operation must define how opaque sample
material changes the pool state and how credit transfers into it.

## Architecture pressure test

The same lifecycle applies on x86-64, AArch64, and RISC-V because it defines a
kernel-policy resource rather than an implementation buffer. A later mixing
design may use portable compiler-generated code or a qualified provider
primitive, but neither choice changes creation semantics. Providers cannot
select pool capacity, assign initial credit, or infer readiness.

The 256-bit capacity is the bound for the initial QEMU profile, not a universal
word size, cache-line size, digest layout, or claim about Linux internals.
Other deployment profiles may choose another capacity through a separately
qualified request shape.

## Alternatives rejected

Creating and mixing the pool in one operation was rejected because it would
hide lifecycle and credit-transfer failures behind a cryptographic step.
Defining the pool as a 32-byte source-visible array was rejected because it
would expose representation and encourage ordinary memory access to secret
state. Letting the x86 provider create the pool was rejected because pool
policy is architecture-independent. Treating the assessed sample itself as
the pool was rejected because input ownership and persistent generator state
have different lifetimes.

## Deliberate bounds

This increment admits one empty pool with identity one, credit capacity 256,
and initial credit zero after the exact sample-quality transition. It rejects
another identity, capacity, initial credit, omission, duplication, or
reordering.

It does not allocate or expose pool bytes; mix, condition, hash, or destroy a
sample; transfer entropy credit; perform health tests; seed or reseed a CRNG;
populate `AT_RANDOM`; implement `getrandom`; recycle the virtqueue; or define
reset and recovery.
