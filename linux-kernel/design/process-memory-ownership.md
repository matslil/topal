# Process memory ownership across serialized execution

`TK-DEC-284`, adopted under the standing chat authorization for systems-only
design changes. Implements the direction of `TOPAL-REQ-SYSTEMS-PROCESS-MM-001`
and `TOPAL-SYSTEMS-PROCESS-MM-001`; changes no core-language syntax, values,
pointer construction, tasks, exceptions or effects.

## Problem and retained obligations

The [runtime frame allocator](dynamic-frame-memory.md) gives the active process
a workload-sized heap. The [child](serialized-static-process-children.md) and
[resumable pair](resumable-static-process-pair.md) still save/exchange 34 heap
pages and complete low/high leaf tables. A single active MM state/token bank
cannot by itself represent two independent heaps across resize or exec.

The initial zero heap pointer is not the problem: the MM transfer path updates
context word 15 with the committed physical base. Existing bounded child and
pipe workloads are qualified and must remain supported. Blanket fork refusal,
larger fixed snapshots, restoring stale heap PTE bytes and fabricated success
would not implement the required behavior.

[fork(2)](https://man7.org/linux/man-pages/man2/fork.2.html) specifies initially
equal separate private memory with independent subsequent writes/mappings.
[execve(2)](https://man7.org/linux/man-pages/man2/execve.2.html) replaces the
calling process's program and initializes its heap anew.
[brk(2)](https://man7.org/linux/man-pages/man2/brk.2.html) distinguishes the raw
Linux return convention: the resulting break on success, current break on
failure. These are interface facts, not a replacement for the pinned baseline
or the complete mapping inheritance rules required by the original objective.

## Portable owners and transitions

The logical process owns an MM identity, image generation, program break and
mapping/backing owners independently of its scheduling position. A selected
active translation window is an execution resource, not shared-mm semantics.
The first implementation may keep the qualified image/stack parking while
giving each heap a distinct real frame extent. Eager copying is selected for
this stage; copy-on-write needs separately qualified faults and sharing counts.

| Transition | Required ownership behavior |
| --- | --- |
| Fork-like creation | Prepare child metadata and actual-size zeroed backing, copy the complete parent heap, pin it, then commit the relationship/PID and select the child; retain the parent's exact heap/break |
| Cooperative handoff | End the current syscall/copy lifetime, retain both heaps, publish the selected heap and invalidate stale translations, then resume its own native continuation |
| Resize | Use only the active process's break/handle; preserve retained mapped prefix and the inactive process; follow existing replacement publication/retirement ordering |
| Successful exec | Retire only the active heap, advance its image generation and initialize its new heap state; preserve the inactive process's identity, contents and pending continuation |
| Child exit | Restore the parent mappings with complete invalidation, then erase/release child backing; retain child status until existing wait/reap rules consume it |
| Outer-owner termination | Revoke active mappings; retire inactive child and active parent exactly once; preserve existing description/shared-service cleanup ordering |

The fork snapshot covers every currently mapped private heap byte, including
the allocated page containing a partial final break. A same-page break change
does not promise to zero retained page bytes; newly allocated pages are zeroed.
Zero-page heaps own no frame extent, but still have distinct MM/process identity.
No application contents, executable digest or test transcript selects behavior.

Prepublication resource unavailability leaves the existing processes, mappings,
breaks, bytes and child relationship unchanged and consumes no PID. Allocator
issuance already performed during preparation remains consumed; abandoning
prepared backing requires actual retirement. PID history and extent issuance
are separate domains. Failure during publication or another incomplete trusted
mutation poisons the owner, retains all held identities/performed effects and
enters nonreturning cleanup. A later preparation miss after a completed allocator
operation is instead handled by retiring that prepared owner as described above.
Comparison snapshots cannot implement rollback.

Every syscall invocation binds actual active process/MM/generation, entry epoch,
context and caller. Handoff, resize and exec exclude live user-copy loans.
No loan or checked user pointer can be carried into another process/generation.
Existing asynchronous, AP, DMA, fault-recovery and translation-stability
obligations remain; serializing the selected user alone is insufficient proof.

## Initial x86-64 realization and mapping from the current provider

| Current implementation | Required replacement |
| --- | --- |
| `kernel_frame_memory.ll` single active token/state banks | Separate active/inactive process MM owners plus preparation storage; exact live relationship and entry binding |
| `systems_process_children.rs` 34-page copy into `HEAP` | Allocate a distinct actual-size child heap through the qualified source allocator; copy/pin before child publication |
| `systems_process_pair.rs` fixed heap byte exchange | Keep each physical heap in place and publish the selected owner's heap translations |
| Full low/high leaf-page copies on create/restore/exchange | Preserve only separately owned nonheap image/stack state; heap entries are edited exclusively through the MM publisher |
| Exec/terminal cleanup of the global active heap | Retire the selected process only; child return and parent cancellation select/retire the correct other owner |
| Context word 15 physical heap pointer | A derived checked alias of the selected live MM owner, updated at each binding; it cannot confer ownership |

The existing heap has 519 possible 4096-byte leaves: low-table slots 503..511
and high-table slots 0..509; the high table's 510..511 remain stack leaves.
These are private initial-backend geometry, not portable API constants or a
new application quota. Nonheap save/restore must not overwrite those heap slots.
Image/stack byte/permission preservation and full root/table ownership still
need their own validation. Every producer that writes an overlapping table must
be migrated or excluded by its actual qualified selection, not by a test flag.

An inactive heap remains pinned to preserve backing lifetime, even though the
active translation window does not currently map it. Handoff alone neither
allocates nor frees heap frames. Before the old active heap can be released,
required invalidations must make its translations unusable. Physical aliases,
stale translations on other CPUs and asynchronous access cannot be excluded by
numeric record equality or a local invalidation observation.

Native operation transport and private offsets must be derived after complete
caller/resource analysis. This design does not assign new source-visible
operation numbers or permit callers to construct MM handles. Existing operation
8 for owned extents must retain its separate contract.

In the experimental MM selection, an initialized descriptor/process owner stays
eligible for child services after shell startup progresses beyond the legacy
11..15 dispatch window. The imported descriptor state, live process relationship,
actual context/root and backing owners remain mandatory; a later shell phase
neither revokes them nor supplies their authority. Predecessor selections retain
their original admission and error order.

AArch64 implementations need their own translation granule, ASID/TTBR context,
TLBI/barrier and extended-state rules. RISC-V needs its own page geometry,
satp/ASID/SFENCE and extension-state rules. Neither backend should expose x86
GS offsets, CR3, fixed snapshot bytes or leaf-slot encodings to ordinary Topal.
The portable obligations are ownership, selected context, access exclusion,
publication and retirement ordering. General shared-MM cloning, separate
hierarchies, mapping types and preemptive scheduling remain required later.

## Stages and acceptance

All stages stay in the initial Step 4 PR, with stage commits and human merge.

1. Design and portable model: specify identity/ownership transitions, compare
   them with an independent byte oracle and reject wrong saved event histories.
   This supplies no executable kernel or native authority.
2. Native MM owner binding: implement actual active/inactive/prepared owners,
   caller/context exclusion, allocator/pin/zero/retirement and publication paths.
   Bind complete resource bounds and fatal partial-effect cleanup.
3. Process integration: migrate creation, every switch/wait/exit/exec/cancellation
   and every overlapping mapping producer. Compile ordinary syscall fixtures
   for independent heaps below/at/above 34 pages, partial pages, target maximum,
   different break values, repeated yields, child exec, wait output retry and
   inactive-child cancellation. Preserve pinned binaries unchanged.
4. Production qualification: rerun affected native/whole-application tests and
   prior child/pipe/51-workload regressions with exact source/load/boot binding
   and cold public reconstruction before activation. Preserve unresolved
   concurrency/mode/fault gaps as blockers rather than claiming completion.

Risk is high: process isolation, affine frame ownership, publication, stale
translations, shared descriptions and native continuations interact. Stage 1
uses single-agent authority/source/diff review, a separate application-byte
oracle and a trace checker with saved-data negative controls; no independent
human or native security review is claimed. Later stages require actual
ordinary user execution and physical retirement evidence in the authorized
bounded VM. No host-security or AI-safeguard change is part of this design.

The [stage report](../step-4-process-memory-ownership.md) separates completed
evidence from native blockers. The full original distribution/container/VM-host
objective and [owned-extent qualification](owned-user-extents.md) remain open.
