# Step 4: process memory ownership

`TK-DEC-284`, `TOPAL-REQ-SYSTEMS-PROCESS-MM-001`,
`TOPAL-SYSTEMS-PROCESS-MM-001`, `TOPAL-COMP-SYSTEMS-PROCESS-MM-001`.

## Stage 1: design and portable reference foundation

The [design](design/process-memory-ownership.md) preserves the qualified child,
resumable-pair and pipe behavior while replacing the assumption of a fixed
34-page private heap with independent live frame ownership. Runtime MM already
updates the active heap pointer after allocation; its initial zero value is not
evidence that all child creation must be refused. The remaining integration gap
is the fixed snapshot/leaf-table exchange and single active MM state across
independent resize, handoff and exec.

The [reference model](tools/process-memory/README.md) separates process/PID,
address-space identity, image generation, entry epoch and extent issuance.
Parent/child heaps keep distinct backing while inactive; handoff remaps without
heap byte exchange. Child exit restores the parent before releasing the child.
Outer cancellation consumes both held heaps once. Failed creation consumes no
PID/MM identity. If an extent was already issued before a later preparation
miss, its issuance stays consumed while that extent is retired.

Seventeen bounded Python tests pass, including 3,200 generated application
transitions checked against a separate process-byte/break/generation oracle.
Explicit cases include zero, partial, 34-page and larger heaps through the
current 519-page geometry; 4/16/64-KiB page examples; independent writes/resize
and either process's exec; occupied/exhausted/fragmented/prepared-failure cases;
wait blocking/WNOHANG/output retry/recreation; stale entry/loan refusal and
inactive-child cancellation. A separate event checker rejects nineteen wrong
histories and state validation rejects seven inconsistent snapshots. All such
controls change reference data only.

The [qualification record](tools/process-memory/results/stage-1/report.json)
binds the selected exact inputs, full unit output, local links/stable IDs and
observed bounded-run resource controls. No Rust implementation, linked kernel,
provider, third-party binary, boot resource, canonical selection or host
protection changes in this stage. No VM or production regression is newly run.
Model identity/zeroing/invalidation facts are not native authority or physical
observations. The models omit actual image/stack/register/descriptor transfer,
table/permission checks, width exhaustion, partial-mutation fatal cleanup and
asynchronous/native lifetime proofs; those remain required.

Risk is high for isolation, ownership, translation and evidence errors. Review
uses single-agent authority/source/diff review, separate application-state and
event-history checks, plus model-only negative controls. No independent human
or native security review is claimed. Existing qualified native evidence remains
historical and is not extended by these results.

## Stage 2: first native preparation/cancellation component

The unpublished [native adapter](../src/topal-compiler/src/systems_process_mm.rs)
and [runtime primitive](../src/topal-compiler/src/runtime/kernel_process_mm.ll)
add an allocator-owned prepared token and an entry-bound receipt. The existing
owning gateway admits the exact additional private token cell; source values
cannot construct it. The experimental component executes preparation and
cancellation inside actual captured `brk` invocations. It does not implement
fork, switch an MM, publish the prepared heap or activate a public kernel path.
The Rust selection is compiled only for tests.

This adapter is a validation witness: it deliberately adds preparation/cancellation
to `brk`, including query/within-page invocations, to exercise the component under
an already bound real entry. It is not a publishable replacement `brk` policy.
The production allocation-free query/within-page contract remains unchanged.
Integration must replace this witness hook with actual process-creation callers
and qualify their entry/return/loan/resource binding before publication.

Preparation derives the complete current heap size, resolves its real pinned
extent, allocates disjoint zeroed backing, copies and reads back every heap word,
then pins the prepared extent. Its receipt binds the actual root, process context
and syscall epoch. Cancellation revalidates that binding, resolves the retained
extent, unpins it, erases and reads back every byte, releases the whole handle,
and clears/readbacks the receipt and token. Issuance remains consumed. A real
allocation miss returns with an empty receipt; that path and fatal partial-effect
cleanup still need their own native qualification.

One complete integration test passes in **530.78 seconds**, including construction
and the full ordinary VM/retirement. The guest retains its **600-second deadline**,
two-CPU TCG configuration and original filesystem/network/host boundaries.
Read-only [GDB observations](labs/qemu/x86_64/process_mm_preparation_observer.py)
record **32 preparations**: eight empty, eight one-page, twelve 34-page and four
67-page heaps. All four large heaps contain nonzero bytes beyond page 34.
The native component copies and subsequently erases **2,801,664 bytes** in total.
Whole active heap contents, token/break/context/epoch and all six recorded tables
remain unchanged, except permitted monotonic hardware A/D sets in descriptors.
Observed prepared identities are distinct; actual pin counts, zeroed backing
before release, consumed registry records and released cells are checked.

The [saved-record reader](tools/process-memory/readback_preparation.py) replays
every native record and rejects fourteen corrupted saved-record controls,
including incomplete copies beyond page 34, aliasing, missing pins, stale
receipts and rolled-back issuance. Controls never modify the guest. Its
[archive](tools/process-memory/results/stage-2-preparation/report.json) binds
selected source snapshots, generated LLVM, native command identities, complete
observations, read-only debugger script/logs and artifact hashes. Frozen replay
repeats these saved-record checks from archived Python inputs; it is not a
native rebuild or production reconstruction.

All seven generated LLVM modules pass explicit verification. The complete native
dispatch bound is **2,160 bytes**; the existing syscall adapter totals **2,240**,
and the full owned-transfer caller path totals **2,312 of 4,096 bytes**. Its
existing **4,088-byte** complete physical retirement layout remains unchanged.
Workspace/all-target locked checking, formatting, the 2,000-line Rust ceiling,
all **196 existing user-memory tests** and **17 portable process-memory tests**
pass. The ordinary metadata readback still independently verifies **136 syscalls,
124 modeled transitions, 44 stat outputs and 44 clock returns**.

Fifteen predecessor object/application/resource artifacts and the entire
**7,606-byte** console are byte-identical to the preceding native metadata build.
The intermediate source ELF has identical loadable bytes, entry and every other
section; its three nonallocated debug sections differ. No debug-equivalence
proof is claimed. An initial reader incorrectly required whole intermediate-ELF
identity and failed after the native VM had passed. Its exact reader and failure
record are retained; the revised reader explicitly records the debug scope.
Neither this reader failure nor the pre-existing GDB debug-section warning is
attributed to an AI safeguard.

The final sampled command-tree cgroup peak is **2,220,826,624 bytes**, below its
actual **2,947,301,376-byte** limit, with zero swap and zero sampled memory events.
The sampler records its last live observation and subsequent cgroup retirement;
it does not claim to observe an event occurring after that last sample. These
figures include the compiler, debugger and emulator, not just guest kernel RAM.

Risk remains high. Review combines authority/source/diff review, exact generated
target/dispatch deltas, native closure/stack analysis, real frame/byte observations,
separate metadata reconstruction and offline controls. Observer and readback
share a model; no independent human security review or complete native lifetime
proof is claimed. Original kernel/UAPI/permission checks are retained. Canonical
production selection, core-language meaning and pinned third-party binaries
remain unchanged; the 51-workload production corpus is not newly rerun here.

## Remaining stages and blockers

2. Complete active/inactive MM owner binding and prepared-to-active publication,
   complete caller/loan exclusion, identity-width and actual allocation-miss
   behavior, and native fatal partial-effect retirement. The preparation witness
   alone does not complete this stage. Replace its `brk` validation hook with
   real process callers and retain allocation-free query/within-page behavior.
3. Migrate creation, all switches, exec/exit/wait/cancellation and overlapping
   leaf-table writers. Run source-built ordinary syscall witnesses for independent
   heaps/partial breaks/resize/exec/yields/wait retry and physical cancellation.
   Preserve existing child, pipe and unchanged pinned application workloads.
4. Complete affected production qualification and immutable source/native/load/
   boot reconstruction before activation. Preserve mode, concurrent-access and
   fault gaps as acceptance blockers, not successful model evidence.

The [owned-extent work](step-4-owned-user-extents.md) retains its own unfinished
caller, lifetime and production gates. This foundation does not complete that
work, initial Step 4 or the original distribution, Docker/Podman and VM-host goal.
