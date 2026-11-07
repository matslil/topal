# Bounded user-image execution residency

This record resolves `TK-DEC-047` as a systems-profile refinement. It retains
the two approved finalized image mappings while the existing sealed user probe
runs, then tears them down after the probe returns and before address-space
completion. It changes no core-language rule and does not execute the
constructed image.

## Decision

After publishing `TOPAL_KERNEL_USER_IMAGE_READY`, source prepares and enters
the existing qualification probe without surrendering the finalized
`ExecutableCodeMapping` or `ImmutableDataMapping`. The probe completes its
fixed syscall sequence and returns to the exact kernel continuation. Source
then emits `TOPAL_KERNEL_USER_IMAGE_RETAINED` while both mappings are still
resident, revokes and releases data before code, and only then completes and
reclaims the probe address space.

```text
finalize code + data -> image-ready witness
  -> prepare/enter sealed probe -> exact kernel return
  -> image-retained witness
  -> unmap/release data -> unmap/release code
  -> complete/reclaim address space
```

Probe preparation and execution borrow the resident mapping authority without
consuming, duplicating, transferring, or widening it. Only the returned kernel
continuation may resume source teardown. Address-space completion requires both
construction mappings revoked and both returned extents released.

## Isolation and failure contract

The provider must preserve each mapping's identity, backing, placement,
contents, and RO/X or RO/NX authority across probe preparation, entry, syscall
handling, fault recovery, exit, and kernel-continuation restoration. The sealed
probe must neither fetch from nor read or write the constructed mappings.

Construction placements must be distinct from the probe code, stack, transfer
candidates, and deliberate fault-guard range. All existing copy fault and W^X
contracts remain unchanged. On x86-64 the provider therefore relocates its
private `probe-code` and `probe-data` placements away from the page beginning
at `0x602000`; these addresses remain target-private evidence, not source
semantics.

Early teardown, entry without both roles, role or backing substitution,
mutation during execution, missing or early retained-image witness,
address-space completion while either mapping remains, and recoverable exit
with live authority are invalid. Fatal-only completion remains in force after
permission publication begins.

## Alternatives and cross-architecture pressure

Tearing mappings down before the probe proves construction residency only.
Executing the constructed code would additionally require entry selection,
loaded instruction semantics, image ownership transfer, and a return contract.
Folding the mappings into probe ownership would obscure their affine teardown
and prematurely define a loader object.

The portable contract is one borrowed execution interval over two opaque
role-specific mappings followed by exact source-owned revocation. x86-64,
AArch64, and RISC-V providers privately choose non-conflicting placements,
page-table representation, permission encoding, synchronization, and
translation invalidation.

This increment does not execute constructed contents, retain mappings after
address-space completion, define arbitrary segments or aliases, relocate an
image, parse ELF, provide demand paging, or create a general process model.
