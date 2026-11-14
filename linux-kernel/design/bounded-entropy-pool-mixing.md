# Bounded entropy-pool mixing

This record resolves `TK-DEC-107` as the first operation that processes opaque
entropy bytes. It consumes the assessed sample into the empty pool using a
fixed portable cryptographic construction while keeping mixing and credit
accounting separate.

## Decision

After `TK-DEC-106`, one generic entropy-pool mixing operation consumes assessed
sample identity one and the empty state of pool identity one. It returns the
same pool identity in a mixed-but-uncredited state and produces affine
contribution identity one carrying the sample's 256-bit credit claim. Sample
identity one is consumed and its provider storage is wiped before success.

The initial profile uses BLAKE2s-256 over the exact 32 sample bytes with the
eight-byte BLAKE2s personalization `TopalRNG`. The resulting 32-byte digest is
the pool's first opaque state. Source names conditioner identity one, pool
identity one, sample identity one, contribution identity one, byte count 32,
and contribution credit 256. It observes no address, input byte, digest byte,
or target instruction.

Mixing does not alter the pool's credited-bit counter: it remains zero. A
later source-owned operation must consume the contribution token to transfer
credit, subject to the pool capacity. This follows the Linux random-subsystem
boundary in which byte mixing and initialization-bit credit are distinct, but
does not require a line-for-line Linux implementation.

The provider must implement the fixed construction exactly, write all 32 pool
bytes, verify the produced state against an independent checked reference in
qualification, wipe all 32 sample bytes, and fail closed before returning a
partially mixed or incompletely wiped state. The provider cannot select the
algorithm, credit the pool, retain a sample copy, or expose either storage
object.

## Architecture pressure test

BLAKE2s operates on little-endian 32-bit words but its byte-level result is
architecture-independent. x86-64, AArch64, and RISC-V providers may use scalar
instructions, vector instructions, or shared compiler-generated code only
when their qualified implementation produces the same digest and wiping
behavior. Endianness conversion, register selection, vector width, and memory
layout remain provider-private.

The semantic operation names a qualified conditioner rather than an
instruction family. This prevents x86 features from entering portable source
and permits a future constant-time shared lowering without changing the pool,
sample, or contribution lifecycles.

## Alternatives rejected

XORing the sample into an empty array was rejected because it supplies no
cryptographic compression and leaves future combination unsafe. Exposing a
generic source-level hash API was rejected because the kernel needs opaque
secret-state handling, mandatory wiping, and affine credit evidence rather
than ordinary byte values. Crediting inside the mix was rejected because
successful byte processing does not authorize an entropy estimate. Retaining
the assessed sample after mixing was rejected because it permits accidental
reuse and extends secret lifetime. A provider-selected algorithm was rejected
because cryptographic policy must be stable across architectures.

## Deliberate bounds

This increment admits one first-generation mix with pool identity one, sample
identity one, conditioner identity one, contribution identity one, exactly 32
input bytes, and a 256-bit untransferred contribution. It rejects another
identity, extent, credit, pool state, sample state, omission, duplication, or
reordering.

It does not transfer entropy credit, declare the pool ready, reseed or expose
a CRNG, extract bytes, populate `AT_RANDOM`, implement `getrandom`, mix a
second sample, recycle the virtqueue, configure asynchronous interrupts, or
define reset and recovery.
