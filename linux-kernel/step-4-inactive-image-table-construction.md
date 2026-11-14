# Owned inactive image-table construction

`TK-DEC-288` implements inactive table construction under the approved systems
authorization. `kernel image prepare tables` returns one affine
`InactiveKernelImageTables`, consumed by `kernel image discard tables` before
ordinary boot continues. Image and page policy remain provider selected; source
exposes no counts, addresses, native formats or permission bits. This resource
cannot substitute for `ZeroedKernelImageBacking`. Core semantics are unchanged.

`TOPAL-SYSTEMS-KERNEL-IMAGE-TABLES-001` connects the systems requirement and
compiler requirement `TOPAL-REQ-SYSTEMS-KERNEL-IMAGE-001` /
`TOPAL-COMP-SYSTEMS-KERNEL-IMAGE-001` to source checking, distinct checked
operations, transitions, resource-state modeling, root lowering, complete
image reconstruction, native construction and independent byte replay.

## Complete inactive storage and construction

The x86 producer allocates and pins two whole image-sized banks plus a separate
whole low USER leaf. The process bank's high USER leaf remains in its reserved
slot; kernel-only subdivision pages do not become USER-admissible. The low leaf
is independently owned and stays zero while the inactive process bank points
to it. A numeric address or spare bank subextent cannot supply that owner.

Normal owning-gateway queries retain every existing context/lease/admission
check. The existing complete whole-owner validator is reused unchanged. Both
banks and the low leaf must have exact issuer/identity/count/alignment/pins,
contiguous RAM provenance, image/boot/active-root exclusions and pairwise
disjointness before writes. Inactive construction initializes owned data,
zeroes/verifies all storage, fills both table banks and verifies every word.
Discard checks the same owner identities and topology, erases/readbacks all
three extents, then unpins/releases whole owners with issued history retained.
Partial effects remain fatal/poisoned without an ordinary success witness.

The constructor and runtime verifier share the provider's bound template for
consistency. The independent Python oracle derives full words separately from
the actual image partition and a portable permission/subdivision model. It
checks every captured byte, including unused entries, parent links, process
low/high USER links and the separate zero leaf. It rejects permission/pointer
changes even when an offline altered record claims a matching new digest.
These controls never alter a guest or the retained native files.

The provider's existing draft non-image RW/NX and MMIO bytes remain explicitly
unqualified for active coverage or hardware memory attributes. Constructed
bytes grant no activation, protected context, publication or lifetime witness.
The active legacy five-page producers and USER validators remain unchanged.

## Exact final-image feedback

The initializer's constants and code contribute to the image it describes.
A private compiler proposal retains the complete partition/template, not only
its page count. Every candidate is freshly reconstructed from source/native
objects, linked allocations, loads and reservation; loading requires the exact
final partition/template and whole storage to match the emitted constructor.
The native run's first table candidate retained the same eight-page count but
failed that stronger match and was not loaded. The next candidate converged.
External final image identity avoids an embedded self-hash. Saved JSON supplies
observational descriptions, never allocator or construction authority.

## Validation and review

The [committed result](tools/user-memory/results/stage-4-inactive-image-table-construction.json)
records **all fifteen commands passing** over **974 stable inputs** in
**866.79 seconds**. **654 frontend**, **257 semantics**, four compiler
component tests, five canonical layout tests, four legacy owner tests, the
default ABI fixture, workspace/all-target checks, formatting/file ceiling,
**275 user-memory** and **83 process-memory** models pass. Both native protocols
complete ordinary initial execution and three exec replacements, each with
eight checked console-copy lines. Independent readers reject **22** table/
low-leaf/rehash controls and **12** backing-only record controls.

Fresh table construction reconstructs **24 objects and 885 allocated
contributions**. The **408-byte** root token bank binds preparation slots **1,
2 and 3** to whole counts **8, 8 and 1**, each alignment/pins **1**. It constructs
**65,536 bytes** of table banks while retaining the **4,096-byte** zero low leaf,
and zeroes/discards all **69,632 bytes**. IDs **2, 3 and 4** are released without
resetting next-ID **5**. Every captured preparation control remains unchanged.

The new kernel SHA-256 is
`6f935bd1fe60b2c87b9c040169896d6ebc1d568ffb73167dcdab2bb403a1c4fa`.
The backing-only regression retains
`064058b5c8fb800137cf055b971dbc1cf258ab3729ba93cfda005bc4ea805b19`;
the separate original-profile rebuild retains
`28057d5606b6c59f704e861b2e22d0b6dd04e79f9d07ff4933546695214204c1`.
The final actual bootstrap CR3 is **46,321,664**,
with token **[1, 5, 0]**, count/alignment/pins **5/1/1**. Its full five-page
capture has SHA-256
`bd804e4df254f769d43a9332d4ca2c44efc19dba8fc66b1049bacfae02ae8a78`.

The helper's conservative stack sum is **1,032 bytes**;
with the normal gateway window it is **5,128 / 8,192 bytes**,
bound to the actual boot stack by native observation. Import and acyclic call
closure pass. The only privileged instruction in the new component reads CR3;
it writes no CPU control register and never activates the prepared banks.
Peak/limit are **2,668,883,968 / 3,356,004,352 bytes**, with
zero swap and all recorded memory-event counters zero.

The full local matrix is `.cache/x86_64/inactive-image-tables-stage-2/run.json`;
the native build is `.cache/x86_64/process-mm-interface-regression-2656457-1794643009169853614`. The compact result binds all
44 changed authority/code files to their tested hashes and retains native
artifact hashes. The descriptive status update is recorded separately after
validation. Choose a fresh output directory:

```sh
rtk scripts/run_bounded.py -- python3 -B linux-kernel/tools/user-memory/run_image_backing.py linux-kernel/.cache/x86_64/NEW-OUTPUT --tables
```

Read-only hardware breakpoints observe five native phases: patterned owned
storage, zeroed storage, both constructed banks and zero low leaf, erased owned
storage, and complete release. The observer independently reads actual tokens,
registry records, every page cell, firmware/exclusion ranges, controls and all
bank bytes. It also captures the actual final bootstrap CR3 and its whole live
five-page owner after ordinary application/exec completion. These captures are
scoped to the new image and do not relabel older missing endpoint evidence.

The source/semantics suite includes new wrong-resource/marker/interleaving and
live-disposition checks. Compiler controls reject a backing-only plan, a changed
partition with the same count, a missing/aliased/wrong-purpose low leaf and
changed count/pins. The independent oracle adds four word/geometry/permission/
aliasing tests. The matrix also reruns the previous backing-only native protocol
and its reader, and separately rebuilds the original profile.

Risk is high for ownership, memory validation and compatibility. A separate
single-agent diff review checks higher-level intent, resource-kind separation,
complete final template binding, all three whole owners, normal gateway entry,
call/import/stack closure, readbacks, erasure, release and exact predecessor
kernel identities. No existing memory-validation mechanism is removed or
widened. Host protections, resource bounds and VM boundaries are retained.

The first full matrix failed before native work because a new test reused a
model state after its consuming completion check. A separate live state now
tests that obligation; production semantics were unchanged. That failed record
is retained separately. It provides no explanation of an AI safeguard action.

The historical full compiler suite, its host-RWX/injected-state cases and the
51-workload third-party corpus are not rerun for this source snapshot. This
compact receipt is not a closed source archive or production publisher proof.

## Remaining work

The new resource contains real verified inactive primary-view tables. All
bootstrap/process/saved-root and alias producers, effective permission controls,
non-image/device roles, asynchronous writers, activation/restoration, continuous
lifetime/fatal handling and cold production qualification remain required.
Full Linux UAPI, DTS, distribution, Docker/Podman and VM-host acceptance remain
unfinished. Step 4 and the original plan remain in progress.
