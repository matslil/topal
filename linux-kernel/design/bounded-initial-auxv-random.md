# Bounded initial `AT_RANDOM` delivery

This record resolves `TK-DEC-111` as the first Linux userspace delivery of
cryptographically generated material. It binds opaque output to the initial
process ABI while retaining the extraction and interface-copy boundary.

## Decision

The qualification root shall complete the entropy pipeline through
`TK-DEC-110` after preparing the initial address space but before entering its
user context. Entry is unavailable until one generic delivery operation
consumes opaque output identity one and refines that prepared address space to
`RandomizedEntryReady`.

For the initial x86-64 profile, the provider adds native auxiliary-vector tag
25 (`AT_RANDOM`) before `AT_NULL`. Its value points to a distinct, aligned,
fully initialized 16-byte object inside the bounded initial stack. The object
does not overlap the vector, platform string, executable-name string, or the
guard boundary. The provider copies output bytes 0 through 15 into that object,
verifies the complete copy, wipes and verifies all 32 bytes of consumed
provider output, and records delivery before permitting context entry. Bytes
16 through 31 are deliberately discarded and never become another output.

Source names output identity one, source extent 32, selected offset zero,
delivery extent 16, discarded offset 16 and extent 16, address-space identity
one, Linux interface identity one, auxiliary tag 25, and single-use delivery.
It observes no output byte, user address, backing address, stack layout,
temporary value, or instruction. The constructed user probe validates the
ordered tag, exact provider-selected pointer, complete bounded readable object,
unchanged string objects, terminating `AT_NULL`, and exact enlarged-block
consumption without comparing random material to a source constant.

Current upstream Linux likewise defines `AT_RANDOM` as the address of 16 random
bytes and generates and copies those bytes while constructing ELF process
tables. Topal makes readiness, output ownership, publication verification, and
entry ordering explicit instead of treating the stack copy as an untyped byte
side effect.

## Architecture pressure test

The tag and 16-byte object are Linux UAPI semantics shared by native x86-64,
AArch64, and RISC-V processes. Each provider chooses a conforming in-stack
address and native pointer encoding, proves complete writable and later
readable bounds, and enforces delivery before entry. Pointer layout and page-
table mechanisms remain provider-private; output identity, affine consumption,
object extent, and entry-readiness refinement remain portable.

## Alternatives rejected

A fixed seed was rejected because it is not random. Copying raw virtio-rng
payload bytes was rejected because it bypasses conditioning, credit, and fast
key erasure. Mutating the stack after entry was rejected because Linux presents
the auxiliary vector atomically at process start and concurrent mutation would
be a data race. Returning the unused half as another output was rejected for
this bounded profile because it complicates lifetime and consumer ordering;
`getrandom` will use a fresh affine extraction. Exposing an address or ordinary
byte array to source was rejected because the ABI provider already owns layout
and secret-copy mechanics.

## Deliberate bounds

This increment admits one output-to-stack delivery for output identity one,
address-space identity one, interface identity one, tag 25, a 32-byte source,
the first 16 delivered bytes, and the final 16 discarded bytes. It rejects
another identity, interface, tag, extent, partition, address-space state,
omission, duplication, reordering, partial copy, incomplete wipe, output reuse,
or user entry before delivery.

It does not implement `getrandom`, `/dev/random`, kernel-consumer delivery,
multi-block generation, output buffering, reseeding, per-CPU generators, queue
recycling, asynchronous interrupt routing, reset, or recovery.
