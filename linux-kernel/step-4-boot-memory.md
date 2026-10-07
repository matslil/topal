# Step 4 boot-memory refinement increment

## Outcome

The checked kernel root now consumes its entered `BootstrapContext` through
the approved `context boot describe memory` decision before performing any
ordinary bootstrap operation. The success action receives only its
`MemoryDescribedContext`; the error action receives only its fatal-capable
failure context. Reuse of the entered context, a nonfatal failure action,
missing result coverage, or a transition after another operation is rejected.

The architecture-independent semantic library models provider source ranges,
live reservations, provenance, overlap splitting, conservative effective
classification, page-edge reservation, absence of allocatable pages, and the
consuming success/failure context transition. Its mathematical range values do
not grant machine-address authority.

## Initial x86-64 provider

The sealed provider revision `topal.provider.x86_64-qemu-pc-q35/2` implements
the first physical handoff slice. Its generated validator:

- receives the Linux boot-protocol 2.15 `boot_params` address through the
  backend-owned entry convention;
- requires the handoff and complete zeropage to remain below the 16 MiB
  bootstrap reservation floor;
- rejects nonzero setup-data pointers until bounded extension-list handling is
  qualified;
- accepts one through 128 zeropage E820 entries;
- rejects arithmetic overflow and empty RAM entries;
- treats every non-RAM type as unavailable for allocation; and
- proves that at least one complete 4 KiB RAM page remains above the 16 MiB
  reservation floor.

Linked-artifact inspection rejects any generated load segment crossing that
floor. The floor therefore contains the boot adapter, transition structures,
initial stack, boot handoff, linked kernel, and bootstrap pool for this closed
image. The generated root calls the typed validator, tests its Boolean result,
and branches to the existing nonreturning fatal provider on failure. Topal
source names none of the offsets, registers, instructions, or E820 encodings.

This is a validation witness for the first memory-described phase, not yet a
general frame allocator. Later allocator work must materialize and consume the
normalized range capabilities modeled by the common semantics, and separately
qualify setup-data extension traversal and any additional boot path.

## Evidence before the physical gate

- semantic tests cover overlap precedence, provenance, reservations, page
  edges, malformed and overflowing ranges, unsupported handoffs, and empty
  allocatable results;
- source-checker tests cover exhaustive affine refinement and wrong-context or
  wrong-disposition rejection;
- provider-object tests inspect the closed generated validator symbol and
  deterministic provider provenance;
- root-object tests require the validator dependency and its fail-to-fatal
  branch; and
- linked-artifact tests enforce the 16 MiB reservation floor and updated
  semantic trace.

Pinned-QEMU execution and refreshed content-addressed evidence are the next
stage; this increment alone does not claim that physical result.
