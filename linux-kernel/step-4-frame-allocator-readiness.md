# General allocation readiness: frame-owner model

This conformance repair prepares the existing reference allocator for later
general allocation work. It implements the already approved
[physical-frame contract](design/physical-frame-allocation.md),
`TOPAL-SYSTEMS-BOOT-MEMORY-001`, `TOPAL-SYSTEMS-FRAMES-001` and
`TOPAL-SYSTEMS-MAPPING-001`. It changes no language design, requirement,
executable allocation shape, native provider or public Linux interface.

## Reproduced failures and repairs

Four regression tests failed before the repair. Their retained
[diagnostic log](tools/allocator-readiness/results/before-fix.log.gz) records
one passing oracle test and four failures, including a checked-build overflow.
This preliminary run was not a complete workspace qualification and did not
freeze its complete test-source inputs. The final validation below separately
binds the exact repaired sources and complete command logs.

The [frame model](../src/topal-semantics/src/systems_frames.rs) now:

- Establishes a private allocator-instance identity when an allocator is
  created. Equal handoff labels, allocator labels, extent indexes and frame
  facts cannot authorize release, mapping or unmapping in a different owner.
  Model snapshot clones retain their existing identity; they represent
  alternative model states and do not establish native affine enforcement.
- Checks identity-counter exhaustion before changing free ranges. Refusal
  preserves the complete allocator, including free runs, live extents,
  provenance and history.
- Checks extent metadata with nonoverflowing endpoint arithmetic on release,
  mapping and unmapping. Altered count, alignment or provenance cannot retire
  a valid mapping or change allocator state.
- Validates supplied normalized descriptions before granting frame authority:
  every range is nonempty, nonoverlapping and carries provenance; allocatable
  ranges contain complete frames and only allocatable source claims. Input
  order and sub-page reserved edge fragments remain accepted. Native boot
  discovery and reservation production remain separate obligations.

These checks repair model conformance to existing owner, complete-page,
reservation and explicit-failure laws. The private model identity has no
source spelling and grants no numeric-address authority.

## Completed validation

The [seven new tests](../src/topal-semantics/src/systems_frames_validation_tests.rs)
cover exact refusal-state preservation, equal-label foreign owners, altered
extent facts, malformed normalized descriptions, mapped/stale lifetime checks,
identity exhaustion and mathematical endpoint overflow. Valid normalized
memory is also checked at 4096-, 16384- and 65536-byte frame geometries; these
are portable model cases, not ARM64 or RISC-V execution evidence.

An independent 64-cell oracle executes **768 deterministic allocation/release
steps** with dynamic sizes and alignments, fragmentation, a retained reservation,
two provider provenances, capacity exhaustion and duplicate releases. Its
per-cell availability and provider boundaries are independent of the allocator's
free-run representation. Final release restores the exact original free ranges
without merging distinct provenance.

The [final validation record](tools/allocator-readiness/results/qualification.json)
binds the repaired model, tests and unchanged resource wrapper by SHA-256. All
**239 semantics unit tests**, locked workspace all-target checking, all four
coverage tests, formatting and the existing Rust source-size check pass.
Complete command logs are retained beside that record. Validation used the
normal adaptive wrapper with a 2943 MiB limit, 1569 MiB host reserve and zero
cgroup swap. No host memory or security setting changed.

Logs use lossless gzip storage. The published record binds both compressed
and original hashes, the original producer record and the exact validation
runner. Compression preserves every diagnostic byte and command outcome.

Risk is high because allocator provenance and mapping lifetime interact.
Self-review compared the complete admission/mutation order with the existing
design and specification, exercised every repaired refusal path, and checked
valid producer output, alternate frame geometries and independent allocation
observations. No independent review or new machine-execution evidence is
claimed. Compiler/native source is unchanged; the earlier TK281 qualification
retains its own exact commit and scope.

## Remaining implementation

The [readiness inventory](knowledge/common/allocator-readiness.md) still applies.
The native profile does not yet expose general dynamic frame requests or a
source allocator over the complete usable memory map. Ordinary repeated source
execution still needs checked termination, allocation/stack bounds and safe
temporary reclamation. General persistent object storage, synchronization and
VFS/process/virtual-memory integration remain required, followed by the full
distribution, Docker/Podman and VM-host acceptance work. This repair strengthens
the existing reference oracle without advancing those unexecuted capabilities.
