# Bounded static ELF retained-process disjoint work

This record resolves `TK-DEC-156` as the first resource-bearing work performed
while a complete static initial process remains behind its opaque affine owner.
It is a systems-profile refinement only and adds no core-language element.

## Decision

`static image construction qualify retained disjoint work` shall consume the
exact `RetainedStaticElfInitialProcess` from `TK-DEC-155`, use the same sealed
frame allocator to acquire one aligned physical frame disjoint from every
retained process resource, completely validate that exclusion, release only
the fresh frame, and return the unchanged retained-process owner. Source names
the owner, one-frame request, and policy
`bounded-static-elf-retained-disjoint-work`; it observes no address, frame
number, allocator cursor, process resource, or representation.

The operation shall publish
`TOPAL_KERNEL_STATIC_ELF_RETAINED_DISJOINT_WORK_OK` only after allocation,
exclusion proof, release, and complete revalidation of the retained process.
The existing qualification restoration then consumes the returned owner and
must reproduce the exact random-slot-authorized construction and nested stack
backing.

For the qualified x86-64 realization, compiler-controlled root state retains
the process resource tuple while the existing provider allocator selects one
fresh frame. The provider proves the fresh extent does not overlap the payload,
leaf metadata, upper hierarchy, or stack backing, releases only that extent,
and revalidates all retained mappings, metadata, entry facts, strings, zero
regions, and random-slot zeros. It performs no retained-process store, fence,
invalidation, activation, or release.

## Architecture pressure test

The portable contract is affine exclusion between a retained compound resource
and fresh allocator output. x86-64 may prove it through E820-backed physical
intervals; AArch64 or RISC-V may use firmware memory descriptors and different
translation resources. Physical addresses, descriptor encodings, page-table
levels, and allocator representation remain provider-private.

## Alternatives rejected

A console-only call was rejected because it exercises no competing resource.
Exposing the retained tuple to the generic allocator was rejected because it
would leak provider representation and permit source-selected alias checks.
Running the complete entropy pipeline in this increment was rejected because
it would combine lifetime proof with DMA, generator, secret-delivery, and
rollback boundaries before resource coexistence is qualified.

## Deliberate bounds

This increment admits one one-frame, one-frame-aligned qualification request,
one complete exclusion proof, immediate release, and exact owner return. It
rejects another owner, request, policy, overlapping selection, partial release,
retained mutation, retained release, missing revalidation, recoverable failure,
or restoration before the work completes.

It does not retain the fresh frame, perform device or DMA work, consume entropy,
publish `AT_RANDOM`, initialize TLS, construct a user context, enter ring 3,
execute BusyBox, or add a syscall.
