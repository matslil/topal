# Bounded static ELF stack-entry block

This record resolves `TK-DEC-143` as the first Linux process-entry-content
boundary for the guarded static ELF stack. It is a systems-profile refinement
only: checked source selects a sealed semantic launch profile, while providers
own stack direction, word width, pointer representation, byte order,
alignment, concrete addresses, and write mechanics.

## Decision

`static image construction initialize stack entry` shall require the exact
live `PublishedStackLeafStaticImageConstruction` and nested one-frame backing
from `TK-DEC-142`. It shall accept only that owner, that backing identity,
profile `bounded-static-elf-busybox-echo`, and extent `complete`. The operation
shall borrow the live extent and produce one affine
`InitializedStackEntryStaticImageConstruction` retaining the unchanged entry,
slot, leaf, and backing authorities.

The sealed profile shall contain these semantic Linux process-entry facts:

1. argument count two;
2. immutable arguments `busybox` and `echo`, in that order;
3. an argument-vector terminator;
4. an empty environment with its terminator; and
5. exactly one terminating `AT_NULL` auxiliary-vector pair.

All pointer-bearing facts shall designate bytes within the same bounded stack
backing. Before success, the provider shall validate the inactive hierarchy,
the exact RW/NX stack leaf, the guard and image leaves, and completely zeroed
backing. It shall construct the complete aligned entry block, perform all
target-required visibility operations, and read back every word, string byte,
terminator, and otherwise-zero byte. The hierarchy shall remain inactive.

Source shall emit exactly one
`TOPAL_KERNEL_STATIC_ELF_STACK_ENTRY_INITIALIZED_OK` observation and then
invoke `static image construction clear stack entry` with the exact owner,
backing, and policy `bounded-stack-entry-qualification-rollback`. Clearing
shall zero and verify the complete frame, preserve the exact leaf and
hierarchy, and reproduce the published-stack-leaf construction admitted by
the existing leaf-revocation operation. Source shall emit exactly one
`TOPAL_KERNEL_STATIC_ELF_STACK_ENTRY_CLEARED_OK` observation before leaf
revocation.

The operations shall not expose a virtual or physical address, stack pointer,
pointer-sized word, byte order, layout offset, leaf ordinal, or write
mechanism. They shall not add nonterminating auxiliary-vector entries,
initialize TLS, activate the hierarchy, construct a user context, or enter the
image.

## Exact x86-64 realization

The provider shall use eight-byte little-endian words, downward-growing stack
placement, and a 16-byte-aligned entry pointer within the selected 4096-byte
leaf. It shall place the two null-terminated strings in the same leaf and use
canonical user pointers to those exact bytes. The entry words shall be, in
order, `argc`, two argument pointers, null `argv`, null `envp`, `AT_NULL`, and
zero. Padding and all bytes outside the sealed block and strings shall remain
zero.

Initialization and clearing shall each use one store fence before complete
readback. Because the address space remains inactive, no active-address
invalidation is permitted. The current root shall remain the qualified
bootstrap root and differ from the static hierarchy before and after both
operations.

## Affine lifecycle and rollback

Only the exact published-leaf owner may initialize. While initialized, leaf
revocation, backing release, stack-slot or entry revocation, activation,
detach, duplicate initialization, or recoverable disposition is invalid. Only
exact complete clearing may restore the published state admitted by leaf
revocation. Any content, pointer, alignment, owner, backing, leaf, hierarchy,
root, ordering, or readback failure is fatal.

## Architecture pressure test

The portable contract is an aligned, bounded Linux process-entry block with
semantic arguments, an empty environment, and a terminating auxiliary vector.
AArch64 and RISC-V providers use their Linux entry alignment, word, and pointer
conventions. Capability systems derive bounded in-stack string capabilities
without fabricating or widening authority. Those target representations stay
below the checked-source interface.

## Alternatives rejected

Exposing a numeric stack pointer was rejected because later context creation
needs affine authority rather than forgeable integers. Initializing through an
active user mapping was rejected because provider-private backing access can
qualify the content without granting execution authority. Adding the complete
BusyBox auxiliary vector in this increment was rejected because executable
metadata, credentials, randomness, platform facts, and TLS each require their
own dependency and rollback evidence. Retaining initialized contents after
qualification was rejected because the next refinement needs a distinct
consumption boundary.

## Deliberate bounds

This increment initializes, witnesses, completely clears, and verifies one
minimal static BusyBox `echo` entry block. It does not retain the block after
qualification, add nonterminating auxiliary-vector entries, initialize TLS,
derive a source-visible stack pointer, construct a user context, enter user
mode, execute BusyBox, or add a syscall.
