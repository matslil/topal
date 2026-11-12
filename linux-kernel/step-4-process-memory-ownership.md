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

## Stage 3: first independent process/MM integration

The test-only process selection now routes ordinary creation, cooperative
handoff and child exit through distinct active/inactive frame owners. It keeps
heap PTEs out of the legacy image/stack snapshots and uses the existing MM
publisher for heap changes. Prepared ownership binds the source PID/MM/image
generation and actual entry/context/root; allocator handles remain distinct
from these logical identities. Real process callers replace the preparation
stage's deliberate `brk` witness hook in this selection.

The [source-built application](labs/qemu/x86_64/user_extent_process_mm_cases.c)
executes normal `brk`, fork, yield, exit and wait syscalls. In each of four image
generations the parent grows to a partial 67-page heap, the child copies it and
grows independently to a partial 101-page heap, and two handoffs preserve their
different bytes and breaks while the parent shrinks to 35 pages. Child exit
restores the parent; read-only and unmapped wait outputs fail before a valid
retry reaps the same child. The parent's original 34-page prefix and break are
restored. Unsupported clone flags and occupied-slot errors retain their order.

The complete ordinary ABI witness also runs its existing four 34-page child/
clone cases, including TLS/SIMD and shared-description checks. The
[native observer](labs/qemu/x86_64/process_mm_process_inspect.py) records eight
preparations, eight child publications, eight handoffs, eight child exits, three
parent execs, one sole-parent cancellation boundary and final detached cleanup.
The sole-parent boundary does **not** test cancellation of a live inactive child.
The ignored compiler test passes in **177.55 seconds**. The pinned two-CPU QEMU
configuration, 600-second guest deadline and host boundaries are retained.

The [saved-record checks](tools/process-memory/readback_process.py) replay 97
boundaries and reject twenty altered saved-data controls. They compare actual
allocator records/pins, disjoint complete backing, all 519 selected heap leaves,
entry/mode observations, whole-heap copies, retained resize prefixes and zeroed
new pages. They require every disappearing extent to have a consumed registry
record, unowned cells and complete zero bytes: **32 extents / 6,111,232 bytes**.
Fork copies cover **1,654,784 bytes**. These are boundary observations, not
independent continuous translation or pre-release ordering observations.

The [archive](tools/process-memory/results/stage-3-process-integration/report.json)
binds kernel SHA-256
`abb82dd0548df5386aef8651f536a96a97b407a237d3e8fd16af2089306dac10`,
the actual application sources/image, generated native modules, caller bounds,
debugger program, saved observations and selected inputs. Thirteen predecessor
contributions compare byte-identically, including regenerated unselected owned
provider/gateway artifacts at the archived gateway parameter. All seven generated
LLVM modules verify. Locked workspace/all-target checking, 196 user-memory
models, 17 process-memory models, the unchanged default ABI-fixture hash test,
formatting and the Rust source-size ceiling pass.

Complete compiled stack use is **2,280 / 4,096 bytes**; the complete physical
retirement window remains **4,088 / 4,096 bytes**. The native cgroup's sampled
peak is **1,396,228,096 bytes**, below its **2,983,030,784-byte** limit, with swap
disabled and all sampled memory-event counters zero. The sampler records its
last live observation and subsequent cgroup retirement, not an unseen final
instant.

The archive retains failed attempts: dispatch initially excluded the normal
prompt's later startup state; a child's ordinary exit initially encountered an
outer-executable witness check; and the observer initially counted only the four
new cases instead of also requiring the four existing child/clone cases. The
experimental dispatch now retains initialized descriptor/process ownership at
later prompt states and routes live-child exits through relationship-checked
teardown. Predecessor selection bytes remain preserved. These concrete guest/
observer failures do not explain any earlier model/provider safeguard decision.

Risk remains high. Review is single-agent authority/source/diff review, native
ordinary execution, saved-data controls and predecessor byte comparison; no
independent human security review is claimed. The observer and boundary reader
share a model. The additional resize and whole-case reader checks do not replace
independent native instruction/lifetime qualification. This component remains
unpublished; original Step 4 acceptance is still open.

## Remaining stages and blockers

2. Complete all caller/loan exclusion, identity-width and actual allocation-miss
   behavior, and native fatal partial-effect retirement. The first integrated
   cohort does not complete native owner qualification. Queries and within-page
   changes in its real process path retain allocation-free behavior.
3. Complete active-child exec with an inactive parent, live inactive-child
   cancellation on every parent terminal path, every remaining switch/exit and
   overlapping leaf-table writer. Qualify native empty/one/maximum heaps,
   prepublication resource failure and full invalidation/pre-release ordering.
   Existing child cases pass in this component; full pipe and 51-workload
   application regressions remain required.
4. Complete affected production qualification and immutable source/native/load/
   boot reconstruction before activation. Preserve mode, concurrent-access and
   fault gaps as acceptance blockers, not successful model evidence.

The [owned-extent work](step-4-owned-user-extents.md) retains its own unfinished
caller, lifetime and production gates. This foundation does not complete that
work, initial Step 4 or the original distribution, Docker/Podman and VM-host goal.
