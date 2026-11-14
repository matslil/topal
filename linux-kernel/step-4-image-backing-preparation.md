# Image-sized inactive backing preparation

`TK-DEC-287` realizes the reservation phase of the approved kernel-image
protection design. Boot source requests the current image's backing with a
provider-selected page policy and receives one affine preparation resource.
It observes successful reservation, consumes that resource through discard,
then continues the existing boot protocol. Source supplies no counts, native
addresses or table layout. Core language semantics are unchanged.

## Source and native binding

`TOPAL-SYSTEMS-KERNEL-IMAGE-PREPARE-001` connects
`TOPAL-REQ-SYSTEMS-KERNEL-IMAGE-001` and
`TOPAL-COMP-SYSTEMS-KERNEL-IMAGE-001` through the frontend, checked reserve/
discard operations, transition model, root lowering and private provider.
The checker refuses changed image/policy, numeric geometry, duplicate fields,
wrong owner/context, interleaved work, missing observations and live disposition.
Unbound native lowering refuses preparation.

The private producer derives a sizing proposal from a freshly reconstructed
image, regenerates the full root/helper/object/link/load image, and requires
that final image to reproduce the exact whole-bank requirement before loading.
No saved JSON grants allocation authority. The final typed layout, root-object
identity, compiler-derived owner slots and helper constants come from the same
fresh producer. Non-convergent candidates are never loaded by that producer.

The x86 component allocates and pins two distinct whole eight-page banks
through the existing Topal allocator. It enters the existing owning gateway
normally for its queries, preserving admission/context/lease checks. It reuses
the complete native whole-owner validator without changing USER admission.
Exact issuer/generation/count/alignment/pins, contiguous RAM provenance,
disjointness, image/boot exclusions and active-root exclusion remain required.

Before exposing the zeroed resource, the private qualification helper initializes
every owned word with a nonzero deterministic pattern, then zeroes and reads
back every byte. This is normal initialization of owned inactive RAM, before
any success witness. Discard revalidates owners and inactivity, erases and
verifies all bytes, unpins and releases the whole extents. Issuer/history remain
advanced. Failure remains fatal; a partial effect cannot produce success.

The separate component inspection closes its eight imports, direct calls and
acyclic stack graph. The conservative helper sum is 624 bytes; including the
unchanged gateway window gives 4,720 bytes within an 8,192-byte boot-stack
allowance. Native entry observations bind that allowance to the actual stack.
The only privileged instruction in the component reads CR3; it writes no CPU
control register. The original active five-page producers and validators remain
unchanged. Prepared banks never become active tables.

## Validation and review

The [committed result](tools/user-memory/results/stage-4-image-backing-preparation.json)
records **all thirteen commands passing** over **964 stable inputs** in
**567.36 seconds**: **653 frontend**, **256 semantics**, two new compiler
component tests, the native VM case, twelve rejected replay controls, five
canonical image-layout tests, four legacy owner tests, the default ABI fixture,
workspace/all-target checking, formatting/file ceiling, **271 user-memory** and
**83 process-memory** model tests.

Fresh reconstruction closes **24 raw input objects and 880 allocated
contributions**. The exact **384-byte** root token bank binds preparation slots
**1 and 2**, each eight pages with alignment one and one pin. Four native
snapshots demonstrate initialization, zeroing, erasure and release of both
banks: **65,536 bytes** are zeroed and the same complete backing is discarded.
Issued IDs **2 and 3** are released without resetting next-ID **4**. The
ordinary guest completes its initial program and three replacements, including
eight checked console copies and the unchanged serial output/status contract.

The new kernel SHA-256 is
`064058b5c8fb800137cf055b971dbc1cf258ab3729ba93cfda005bc4ea805b19`.
The separately rebuilt original profile remains
`28057d5606b6c59f704e861b2e22d0b6dd04e79f9d07ff4933546695214204c1`.
The final actual bootstrap root is **46,284,800**, owned by issuer **1**, ID
**4**, registry slot **0**, whole count **5**, alignment **1**, pins **1**.
Its native five-page capture has SHA-256
`b30814ef4137f8d0a2a8557179951584d8897f5a1b64f8640c7bdb6116db934d`.
Preparation preserves CR0 **2147483665**, CR3 **1052672**, CR4 **32** across all
four checkpoints. CR0.WP is clear during this inactive phase and set at the
final ordinary bootstrap capture; this stage grants no effective protection.

Peak/limit are **2,472,022,016 / 3,390,791,680 bytes**; swap and every recorded
memory-event counter are zero. The full local record is
`.cache/x86_64/image-backing-preparation-stage-1/run.json`; the fresh VM build is
`.cache/x86_64/process-mm-interface-regression-2647473-1794640719385596840`.
The committed result binds all 42 changed authority/code inputs to their tested
hashes and retains native artifact hashes. The descriptive status update was
written afterward and is recorded separately from the frozen matrix inputs.
Choose a fresh output directory:

```sh
rtk scripts/run_bounded.py -- python3 -B linux-kernel/tools/user-memory/run_image_backing.py linux-kernel/.cache/x86_64/NEW-OUTPUT
```

Read-only hardware breakpoints capture full banks at initialization, zeroing,
discard erasure and completed release. Snapshots retain real tokens, registry
records, all page cells, firmware/resource exclusions, issuer history, control
registers and bank bytes. After ordinary shell and three exec replacements,
the observer captures the actual final bootstrap CR3, its whole live five-page
owner and all five table pages. The replay checks current bytes/owners/control
records and twelve altered-record controls without running another guest.
These new-image captures do not relabel the older image's 32 missing endpoints.

The full compiler suite's historical host-RWX/injected-state tests and the
51-workload third-party corpus are not rerun for this source snapshot. Their
remaining coverage cannot be inferred from selected tests or previous kernel
identities. This compact result is not a complete source archive or production
publication proof.

Ownership, compatibility and native memory validation are high risk. The
single-agent security and compatibility review checks higher-level authority,
the affine source/model protocol, final sizing feedback, exact live-owner
checks, normal gateway entry, whole erase/release, component call/stack bounds
and native/replay evidence. No existing USER validation is replaced. Guest
work stays in the authorized bounded VM with host protections unchanged.

Development failures remain separately recorded: an initial direct admission
call lacked the required gateway lease and failed before preparation; it was
replaced by normal query calls without changing admission. A later observer
read a compiler-reused register instead of the boot pointer; capture now occurs
at allocator initialization. The transfer fixture needed its existing
600-second process-memory budget and accounting for its eight console-copy
lines. A proposed recursive negative fixture did not produce the expected
rejected call graph after compilation; that unestablished test expectation
was removed rather than counted as a passing control.
No outcome establishes why an earlier AI safeguard intervened.

## Remaining work

Both actual inactive backing allocations now exist. Complete image-derived
table construction, all bootstrap/process/saved-root and alias producers,
effective controls, competing writers, activation/restoration, publication and
continuous lifetime/fatal behavior still need qualification. Preparing and
discarding owned RAM does not establish image protection. Full Linux UAPI,
DTS, distribution, Docker/Podman and VM-host acceptance remain unfinished.
