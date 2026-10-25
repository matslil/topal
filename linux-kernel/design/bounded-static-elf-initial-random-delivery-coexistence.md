# Bounded static ELF initial-random delivery coexistence

This record resolves `TK-DEC-174` as the first persistent Linux userspace
entropy result for the retained static ELF process. It is a systems-profile
refinement only and adds no core-language element.

## Decision

After `TK-DEC-173`, `static image construction deliver entropy initial random`
shall consume the exact acquisition owner and produce one affine randomized
static-process owner. It shall accept only the sealed preceding entropy
resources, generator and output identity one, Linux interface identity one,
auxiliary tag 25, exact 32-byte source partitioned into a delivered 16-byte
prefix and discarded 16-byte suffix, single-use delivery, and policy
`bounded-static-elf-initial-random-delivery-coexistence`.

The x86-64 provider shall completely revalidate the retained process, replay
the exact entropy pipeline, atomically publish one native `AT_RANDOM` pair and
one distinct aligned 16-byte object in the authorized stack slot, verify the
copy without exposing bytes, and wipe and verify the complete 32-byte provider
output. It shall then destroy and verify every remaining sample, pool, key,
DMA, ring, and transport resource before returning the randomized owner. The
stack result remains live after all entropy resources are gone.

For reversible qualification, `static image construction revoke entropy
initial random` shall consume that exact randomized owner, verify and erase the
native random object and auxiliary-vector pair, restore the prior acquisition
owner, and permit the existing nested rollback. Production entry will consume
the forward result instead of invoking the qualification-only inverse.

## Architecture pressure test

The portable contract names Linux tag 25, the source/output partition, affine
ownership, and atomic delivery. X86-64, AArch64, and RISC-V providers choose
their native stack address and pointer encoding, prove destination bounds and
non-overlap, and retain secret bytes only in the process-owned object. The
source does not observe an address, byte, stack offset, or instruction.

## Alternatives rejected

Returning the acquisition owner after silently erasing the stack result was
rejected because it would prove only another transient lifecycle. Keeping the
generator or output alive beside the process was rejected because delivery is
the ownership transfer and all provider copies must then be destroyed. Direct
raw-device delivery was rejected because it bypasses assessment, conditioning,
credit, and fast-key-erasure boundaries. A non-reversible gate-only mutation
was rejected because it would prevent exact qualification of the existing
construction rollback before process entry is added.

## Deliberate bounds

This increment admits one exact output-one delivery and one exact inverse for
qualification. It rejects another owner, identity, tag, extent, partition,
policy, duplicate, premature delivery, overlap, partial copy, incomplete wipe,
or escaped entropy resource. It does not construct a user context, enter the
static process, execute BusyBox, add a syscall, or provide `getrandom`.
