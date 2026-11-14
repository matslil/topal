# Bounded CRNG extraction

This record resolves `TK-DEC-110` as the first output-producing transition of
the ready random generator. It maps Linux fast key erasure to an affine Topal
lifecycle without exposing keys, cipher state, output bytes, or instructions.

## Decision

After `TK-DEC-109`, one generic generator-extraction operation consumes ready
generator identity one at generation one. The provider constructs the standard
20-round ChaCha20 state from constants, the complete 32-byte key interpreted as
eight little-endian words, a zero 32-bit block counter, and a zero 96-bit nonce.
It computes exactly one 64-byte block.

The provider replaces the persistent generator key with bytes 0 through 31 of
that block before publishing bytes 32 through 63 as opaque output identity one.
It verifies both complete 32-byte publications, wipes and verifies the complete
temporary ChaCha input and output state, and returns ready generator identity
one at generation two together with the fresh affine output object. Any partial
key replacement, output publication, or wipe fails closed without making an
output object available.

Source names generator identity one, algorithm identity one, prior generation
one, resulting generation two, output identity one, block extent 64, output
offset 32, output extent 32, counter zero, and nonce identity zero. It observes
no key byte, output byte, address, word layout, intermediate state, register, or
target instruction. Extraction produces no userspace or kernel-visible byte;
later checked delivery must consume the opaque output object.

This is the same security boundary used by the current upstream Linux fast key
erasure helper: the first half of the initial ChaCha20 block replaces the live
key and at most the second half is returned. Topal fixes the full 32-byte second
half in this bounded profile so later `AT_RANDOM` and `getrandom` refinements can
consume explicitly partitioned material without rerunning or retaining the old
key.

## Architecture pressure test

ChaCha20 arithmetic and its little-endian word encoding have identical meaning
on x86-64, AArch64, and RISC-V. A provider may use scalar or vector instructions
and any private register allocation only when qualification proves the exact
20-round result, complete key replacement before output publication, and full
temporary-state destruction. A big-endian provider would perform the specified
little-endian loads and stores; host endianness never changes source meaning.

Generator identity, generation, readiness, affine ownership, output identity,
and publication order remain portable source semantics. Persistent and
temporary storage, constant materialization, rotations, instruction selection,
and wipe mechanics remain provider-private.

## Alternatives rejected

Returning the existing key was rejected because it is not random output and
destroys generator secrecy. Retaining the prior key after producing output was
rejected because it loses the fast-erasure forward-secrecy boundary. Exposing a
generic cipher primitive or source byte arrays was rejected because kernel
policy needs opaque affine state and mandatory cleanup rather than application
cryptography. Generating more than one block was rejected because the first
bounded consumers need at most 32 bytes and larger requests require separate
counter, interruption, and partial-delivery rules. Delivering directly to
`AT_RANDOM` or `getrandom` was rejected because extraction and ABI copy faults
have distinct failure and ownership semantics.

## Deliberate bounds

This increment admits one generation-one to generation-two extraction for
generator identity one, algorithm identity one, output identity one, one
64-byte block, a 32-byte split, and zero counter and nonce. It rejects another
identity, algorithm, generation, output, extent, split, counter, nonce,
readiness state, omission, duplication, reordering, partial publication,
incomplete wipe, or output reuse.

It does not deliver bytes to `AT_RANDOM`, `getrandom`, `/dev/random`, or a
kernel consumer. It does not implement multi-block generation, reseeding,
per-CPU generators, batching, queue recycling, asynchronous interrupt routing,
reset, or recovery.
