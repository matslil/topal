# Ordinary physical-frame metadata policy

This implements the policy part of the approved
[physical-frame contract](physical-frame-allocation.md), using the existing
[owned natural-result traversal](tail-metadata-query.md). It adds no language
syntax, source address or authority constructor. `TK-KERNEL-FRAME-POLICY-001`
identifies the ordinary policy and its complete-state native oracle. Integration
with the actual kernel allocator and physical transitions remains required.

The exclusive metadata owner contains a 128-byte control prefix followed by
24-byte cells, all fields encoded as unsigned little-endian eight-byte values.
Each cell contains an allocation identity, provenance tag and physical frame
number. Identity zero means free; provenance zero excludes the cell from
allocation. Sorted cells may describe sparse physical memory: neither adjacent
indexes nor equal provenance alone establishes physical contiguity. Capacity
comes from the normalized memory description, rather than application slots.
An allocator instance owns its own table and identity namespace.

| Prefix offset | Meaning |
| --- | --- |
| 0 | Number of cells |
| 8 | Requested whole-extent frame count |
| 16 | Alignment in frames minus one |
| 24 | Nonzero, non-all-ones extent identity |
| 32 | Expected provenance for release |
| 40 | Phase: search 0, allocate 1, validate release 2, release 3, done 4 |
| 48 | Current cell index |
| 56 | Candidate or release starting index |
| 64 | Candidate length or number of committed/validated cells |
| 72 | Candidate provenance |
| 80 | Previous physical frame number |
| 88 | Result: selected starting index plus one, or zero for exhaustion |
| 96–127 | Reserved, unchanged |

Before entry the caller admits the complete table, its exclusively owned
lifetime and normalized immutable topology, a positive request and a valid
power-of-two alignment. Allocation starts with phase/cursor/run/result zero.
Release starts in phase 2 with cursor/start set to the live extent's index,
run length/result zero and its identity/provenance/count. The actual caller
must reject foreign, stale, duplicate, partial and still-mapped extents before
entry. These numeric control fields do not replace affine extent authority.

Source selects the first aligned free span with equal nonzero provenance and
successive physical frame numbers. An occupied cell, reservation, provenance
boundary or physical gap breaks the span. A new candidate may start at the
same cell which broke the preceding span. No allocation-cell write occurs
before the entire span is selected. Exhaustion completes with result zero and
all allocation cells and the caller's issuance counter unchanged; invocation
history and ephemeral control fields may advance.

Release first checks every cell's identity/provenance and the entire extent's
contiguity. It then clears only allocation identities. Allocation and release
both recheck each cell before its commit. Failed writes or protocol checks
produce the all-ones result, retaining all performed writes. The actual kernel
caller must poison/fail fatally rather than publish an extent, resume ordinary
allocation or roll back metadata after such a result. No partial result is a
normal exhausted allocation.

The ordinary countdown is admitted as `2 * cell-count + 3`, after checked word
arithmetic. It covers search plus allocation, or whole release validation plus
release. Each recursive edge decreases it by one. The second carried natural
is a stop threshold: a helper returns zero while running, or the current
remaining measure after completion/failure. Argument evaluation reads that
threshold before evaluating the next step, so at most one additional step
observes the already-completed phase. The extra ordered terminal rule then
returns the checked result. Completed phases perform no further cell scans or
writes. The full countdown remains the admitted worst-case bound; early
completion does not reduce counter admission. Exhausted countdown before done
produces a protocol failure.
The result is a natural fact, checked in full before native word extraction.
It grants no physical frames. The provider must independently validate the
selected complete span and reservations, establish the live affine identity,
and zero physical frames before any ownership publication or reuse.

The native process oracle exercises the real source and metadata bytes. It
does not establish privileged caller stacks, interrupt/preemption exclusion,
kernel lifetime/retirement, physical zeroing or actual allocation from workload
requests. Those remain necessary for kernel publication and distribution,
container and VM-host qualification. AArch64 and RISC-V keep the same opaque
frame contract while choosing their own frame geometry and memory-map input.
