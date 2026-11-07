# Bounded static ELF stack-backing zeroing

This record resolves `TK-DEC-141` as the initialization boundary between an
opaque reserved stack frame and later leaf publication. It is a systems-profile
refinement only: checked source observes a refined affine construction owner,
while providers own frame addresses, clearing mechanics, and readback.

## Decision

`static image construction zero stack backing` shall require the exact live
`StackSlotAuthorizedStaticImageConstruction` and nested one-frame backing from
`TK-DEC-140`. It shall accept only that owner, that backing identity, and
`extent` equal to `complete`. The backing extent remains live and borrowed by
the transition; the operation shall produce one affine
`ZeroedStackBackingStaticImageConstruction` owning the unchanged entry and
slot authorities plus a complete-zero invariant for the backing frame.

Before success, the provider shall revalidate a nonzero, page-aligned,
bounded frame identity, clear every byte of the 4096-byte extent, establish all
target-required visibility ordering, and read back every byte as zero. Source
shall emit exactly one `TOPAL_KERNEL_STATIC_ELF_STACK_BACKING_ZEROED_OK`
observation, release the exact whole backing extent, and only then revoke the
stack-slot authority using the zeroed construction owner.

Zeroing shall not install a leaf, expose the frame or slot address, write stack
metadata, derive a stack pointer, or change the inactive hierarchy. Whole
backing release consumes the zeroed frame and restores the slot-authorized
construction; release before complete zeroing is invalid.

## Exact x86-64 realization

The checked root shall pass only the opaque backing identity retained by
`TK-DEC-140`. The provider shall require 4096-byte alignment and the qualified
physical-address ceiling, execute a forward complete qword clear over 512
words, issue one store fence, and execute a forward complete qword comparison
against zero. It shall return success only after exact readback.

The hierarchy identity remains separately retained and shall not be passed to
or modified by the zeroing provider. Whole release clears the backing identity,
restores the hierarchy identity, and removes the zeroed-backing state.

## Affine lifecycle and rollback

Only the exact slot-authorized owner with one live backing extent may zero.
While the zeroed extent remains live, duplicate zeroing, slot or entry
revocation, activation, detach, construction-resource return, or recoverable
disposition is invalid. Only whole backing release restores the state admitted
by stack-slot revocation. Any identity, alignment, extent, clear, ordering, or
readback failure is fatal.

## Architecture pressure test

Complete zero initialization before user visibility is portable policy.
AArch64 and RISC-V providers may require different cache, barrier, or alias
handling. Non-coherent or tagged-memory targets may additionally clear tags or
perform cache maintenance. Capability systems may clear through a bounded
frame capability. None of those mechanisms or addresses become source-visible.

## Alternatives rejected

Relying on allocator-zeroed memory was rejected because allocation and
initialization are distinct contracts and recycled frames may contain data.
Sampled readback was rejected because any unchecked byte could disclose prior
kernel state. Clearing after mapping was rejected because a user-visible leaf
must never reference uninitialized memory. Combining zeroing with process-entry
construction was rejected because raw confidentiality initialization and
Linux stack layout need separate rollback boundaries.

## Deliberate bounds

This increment completely zeros, witnesses, and releases one backing frame. It
does not retain or map the frame, install a stack leaf, initialize
`argc`/`argv`/`envp` or auxiliary-vector contents, initialize TLS, construct a
user context, enter user mode, execute BusyBox, or add a syscall.
