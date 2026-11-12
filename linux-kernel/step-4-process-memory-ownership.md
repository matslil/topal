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

## Child exec and replacement startup

The next experimental cohort retains the initialized descriptor/process owner
through replacement startup. Child, identity and registration dispatch no longer
use the image-stage counter as eligibility. Ordinary registrations retain their
opaque values and return the actual thread identity; matching legacy startup
observations may still progress. The MM entry now checks actual descriptor
readiness before binding its process identity, including phase-zero exec startup.
The existing predecessor selections retain their original emitted paths.

The [child-exec source](labs/qemu/x86_64/user_extent_process_mm_exec_cases.c)
uses normal fork, exec, identity/registration, FS-base, break, yield, exit and
wait calls. A child grows its copied heap to 101 pages, sets nondefault FS and
opaque registration values, then execs. The replacement observes a fresh empty
heap, cleared registrations and zero actual FS base before allocating and
checking all 101 new pages for zero. It chooses its own distinct FS base and
heap bytes, yields to the retained 67-page parent, resumes after the parent's
independent shrink, and exits. The parent keeps its own identity, original FS,
heap bytes and wait-retry behavior. Four parent image generations exercise four
child execs; the complete ABI witness's four existing child/clone cases also run.

The freshly constructed first-party image includes an ordinary Linux-entry to
System V adapter for the C startup routine. Userspace alone selects its routine
from argv. The adapter reserves disjoint scratch below the retained initial
payloads and contributes 144 bytes plus that 1,024-byte separation to the user
stack bound. It does not modify a completed application or any pinned third-party
binary. This is a userspace component for the existing qualified image shape,
not qualification of a general ELF loader or an arbitrary distribution binary.

The complete native test passes in **193.26 seconds** under the unchanged
two-CPU VM and 600-second deadline. Its [saved-record reader](tools/process-memory/readback_exec.py)
checks **101 boundaries**, four child replacement entries, seven execs in total,
eight creations/exits, eight handoffs and final cleanup. Twenty existing controls
and ten child-exec controls reject altered saved data. The checks bind reset
observations to nonzero pre-exec FS/registration values and retain the complete
parent allocation and bytes across child exec. They record **36 consumed/zeroed
extents / 7,766,016 bytes**, with **1,654,784 fork-copy bytes** and 28 allocation-free
queries plus four within-page changes. These remain boundary and saved-data
observations, not independent continuous exclusion or pre-release-order proof.

The [child-exec archive](tools/process-memory/results/stage-3-child-exec/report.json)
binds kernel SHA-256
`fd61f94f31f24ca3209b8b291446da2bb09f2e398680e32c2c81e8fb314995a0`,
source-built image, native inputs and debugger program. The compiled complete
stack bound remains **2,280 / 4,096 bytes**, and physical retirement remains
**4,088 / 4,096**. The swap-free native cgroup sample peaks at **687,190,016 bytes**
under its **2,969,485,312-byte** limit, with all sampled event counters zero.
Its last live sample and subsequent group retirement do not measure an unseen
final instant.

The archive retains the initial adapter failure: its first stack placement
overlapped auxiliary data and the existing credential check rejected startup.
Moving userspace scratch preserved that check and its required data. An earlier
successful child-exec run with less reset observation remains separate history;
only the expanded run above supports these reset claims. No repository failure
or successful retry establishes the reason for any AI safeguard decision.

Risk remains high for ownership, exec/reset, stack layout and restored native
state. Review uses the authority chain, source/diff review, native ordinary
requests, saved-data controls and predecessor comparisons; no independent human
security review is claimed. This does not activate the component in production.

The previous fork/handoff cohort also passes on the updated experimental kernel
in **176.73 seconds**. Its provider, gateway, target and dispatch objects match
the child-exec run byte-for-byte. Thirteen predecessor contributions remain
byte-identical, including unselected owned provider/gateway artifacts. All seven
generated LLVM modules verify. Locked workspace/all-target checking, 196 existing
user-memory tests, 17 process-memory tests, the unchanged default ABI-fixture
hash test, formatting and Rust source-size validation pass. The frozen replay
rechecks both cohorts from their selected saved inputs. These checks preserve
the earlier archives as historical evidence and do not newly run the complete
51-workload production corpus.

## Parent termination with a live inactive child

The process-MM selection now routes imported-parent `exit` and `exit_group`
through one terminal owner path, independently of the image-stage counter.
Actual syscall entry, parent identity, live table and relationship remain
mandatory. A parked pipe wait resolves its saved native FD through the retained
description bank and the existing actual FIFO-generation validator before its
condition/object record is cleared and read back. Inactive heap retirement
precedes inactive-slot destruction; active descriptions, pipes, RAM files and
input follow their existing destructors. Entropy/registration cleanup,
detachment, active-MM retirement and whole private-byte readback remain intact.
The terminal path retires diagnostic transcript bytes after destructors instead
of forging a stage value. Predecessor selections retain their original paths.

The [ordinary C program](labs/qemu/x86_64/user_extent_process_mm_cancel_cases.c)
has four separately built userspace cases: syscall 60 and 231, each at initial
shell stage 32 and at early replacement stage 1. The latter enters at stage zero;
its ordinary break query advances the diagnostic stage to one. Early creation
starts with an empty parent heap; late creation copies 67 pages. Each child
grows to 101 pages, retains dirty bytes beyond the former 34-page limit, creates
an unlinked anonymous file, writes 256 nontrivial bytes into its resident page,
and blocks on an empty pipe read. The parent keeps two writer aliases, closes
its reader, selects its independent 35-page heap and exits. The child's reader
and anonymous file then have their final references in inactive slots. Actual
low-eight-bit exit statuses are `3a`, `00`, `c5` and `01`; userspace compile
choices never select kernel dispatch behavior.

All four final native cases pass in **161.80 seconds**, including their builds,
under the unchanged two-CPU VM and bounded wrapper. The complete selected stack
bound remains **2,280 / 4,096 bytes**, and physical retirement remains
**4,088 / 4,096**. The swap-free group sample peaks at **1,867,710,464 bytes**
under **2,951,454,720 bytes**, including compiler activity, with every sampled
event counter zero. The last live sample and subsequent retirement do not
measure an unseen final instant.

The [saved cancellation reader](tools/process-memory/readback_cancel.py) binds
the actual wait and FIFO generation, endpoint references and aliases, child-only
anonymous description/object/page and dirty bytes to complete destructor storage
readback. It separately binds inactive retirement to detached active cleanup.
Its controls mutate saved data only. The
[archive](tools/process-memory/results/stage-3-parent-cancellation/report.json)
retains the kernel/image identities, exact compile choices and debugger
programs, selected sources, regression evidence and earlier failed attempts.
Frozen replay supplies no independent native load or continuous lifetime proof.

The four records cover **60 MM boundaries**, **20 retired extents** and
**3,899,392 erased/released bytes**. All **48 saved-data controls** reject their
altered records. The complete child-exec/ABI regression also passes in
**189.76 seconds**, retaining its 101 boundaries, four child execs, eight
creations/exits/handoffs, 36 consumed extents and thirty controls. Thirteen
predecessor object contributions remain byte-identical; the selected provider,
gateway, target and dispatch objects match across all five VM runs. The five
linked kernels are also byte-identical, with SHA-256
`b8212e5e9aca715d8db0d863cbc9655ed989c3884b938b57055c3e22e04d0a7c`.
All seven LLVM modules verify in each build. Locked workspace/all-target checking,
196 user-memory tests, 17 process-memory tests, the default ABI fixture hash,
formatting and the Rust file ceiling pass. Frozen replay repeats both cohorts
from sixty selected inputs; it does not newly execute the full 51-workload corpus.

The earlier failures remain separate history. First, inactive endpoint
destruction received the user's preserved R12 rather than the trusted table;
the terminal caller now initializes that destructor argument. Next, the existing
final zero readback caught an unretired fstat/startup transcript; its diagnostic
counter is now retired after owners, while transfer staging still must already
be empty. A later observer expected stage zero at exit despite the ordinary
query advancing it; it now observes replacement entry separately and checks the
actual exit stage. The expanded wait-owner validation then received a fresh
four-case run. These are guest/observer failures and do not explain an earlier
model/provider safeguard decision.

Risk remains high for trusted ownership, wait cancellation, descriptor aliases
and teardown ordering. Review uses the authority chain, full source/diff review,
actual VM requests, saved-data controls and predecessor comparisons. No
independent human security review is claimed. Blocked-reader cancellation here
does not qualify every wait kind, inconsistent internal-state fatal retirement,
every invalidation/pre-release instruction, continuous exclusion, general Linux
orphan/init semantics or public activation.

## Blocked-writer cancellation and dirty FIFO retirement

The next qualification stage changes the source-built application and evidence
tools, without changing native kernel behavior. Its four cases use both parent
exit operations at shell and early replacement startup. The child retains the
last writer description and its dirty 101-page heap/anonymous file. Ordinary
256-byte writes fill all 4,096 FIFO bytes with a repeated nontrivial pattern;
the next one-byte write parks. The parent retains two reader aliases, closes
its writer, selects its independent 35-page heap and exits. The existing
wait-owner validator now executes its writer branch against that retained FD
and actual FIFO generation. Cancellation precedes endpoint destruction, and
the whole dirty FIFO is erased/read back with the shared store.

All four native cases pass in **162.47 seconds**, including builds. They retain
the preceding 60 MM boundaries, 20 retired heap extents and 3,899,392 erased
heap bytes. The [reader](tools/process-memory/readback_cancel.py) now checks both
wait kinds, corresponding endpoint reference counts, parent aliases and all
4,096 pre-cancellation FIFO payload bytes. Its **64 saved-data controls** reject
the altered writer records, including lost occupancy, wrong reference count,
wait-kind mismatch and a changed final FIFO byte. The
[archive](tools/process-memory/results/stage-3-writer-cancellation/report.json)
retains these records and the exact userspace compile choice; this choice never
selects kernel behavior.

All four linked kernels are byte-identical to the preceding parent-cancellation
and child-exec kernel, SHA-256
`b8212e5e9aca715d8db0d863cbc9655ed989c3884b938b57055c3e22e04d0a7c`.
The child-exec execution is retained from the preceding stage rather than newly
run here; its thirty saved-data controls run again. Thirteen predecessor object
contributions remain unchanged. All seven LLVM modules verify in each new
build. Fresh locked workspace/all-target checking, 196 user-memory tests,
17 process-memory tests, default ABI fixture hash, formatting and source-size
validation pass. The existing reader-cancellation archive replays from its own
frozen inputs with the extended replay command, retaining all 48 of its controls.

Complete stack and physical-retirement bounds remain 2,280 and 4,088 bytes.
The swap-free four-case group peaks at **1,561,300,992 / 3,102,629,888 bytes**,
including compiler activity, with every sampled event counter zero. The last
live sample plus group retirement does not measure an unseen final instant.

Risk is high for validation completeness and false acceptance of ownership or
FIFO reclamation. Review uses the authority chain, full source/diff review,
ordinary native requests, unchanged-kernel comparison and saved-record controls.
No independent human review is claimed. This adds blocked-writer boundary
evidence, not continuous loan/exclusion or complete invalidation/release-order
proof. Ready/zombie relationship variants, native allocation misses, identity
exhaustion, fatal partial effects, the full application corpus and cold public
activation remain required.

## Physical allocation refusal and ordinary retry

The next source-built program qualifies the existing prepublication refusal
law through actual physical shortage. The linked kernel remains byte-identical
to the preceding native cohorts. The guest uses **52 MiB RAM**, two CPUs and
the original firmware map; host protections and the bounded wrapper remain
unchanged. Read-only observation retains the original E820 bytes and kernel/
boot-resource reservations. Independent normalization reconstructs all **1,480
eligible frames** and matches every physical catalog entry, with complete
registry, live-owner and free-span checks. This is a constrained guest, not a
host memory-limit change or an injected allocator failure.

The [ordinary C program](labs/qemu/x86_64/user_extent_process_mm_allocation_cases.c)
execs a fresh first-party image, then grows its initially empty heap to a
partial 519-page break and the exact mapped limit. It fills and checks every
mapped byte. Fork cannot obtain another 519-page extent: **445 frames** remain
free, in a largest run of 445, while registry slots and issuance remain
available. The gateway returns ordinary unavailability without a prepared
owner, changed mapping, modified bytes or consumed identity. A separate actual
userspace return observation verifies `ENOMEM`, stack/callee preservation,
unchanged complete descriptor storage and the unchanged **4,096-byte retained
PID-policy store** after the fork frontend has cleared transient fields.

Shrinking the parent to a partial one-page break permits successful creation
of **PID 2**, confirming that the failed attempt did not consume it. The child
retains its copied page independently. Its 519-page resize encounters a second
real refusal: **962 frames** are free, but the largest eligible run has only
**518**. The raw syscall preserves its old break and backing. A 518-page retry
succeeds, preserving the old page and zeroing the complete new tail. The child
marks its first/last bytes and exits; the parent retains its original page and
wait-output retry behavior. A later empty-heap fork creates **PID 3** without
issuing backing. Parent exit completes descriptor, PID-policy and MM retirement.

The final native test passes in **62.85 seconds**, including its guest build.
The [saved reader](tools/process-memory/readback_allocation.py) checks **27 MM
boundaries**, both physical refusal reasons, the ordinary userspace return,
one/zero-page creation, full heap contents and **six retired extents /
4,399,104 erased bytes**. All **21 saved-data controls** reject altered evidence.
Ten independent catalog/firmware cases extend the existing seventeen portable
tests to **27 process-memory tests**. These tests do not substitute for native
ownership or construct allocation authority.

The [archive](tools/process-memory/results/stage-3-allocation-refusal/report.json)
binds the actual guest command, application, firmware observations, debugger
program and selected sources. The earlier two successful runs retain their own
records with narrower observation scopes; they are not evidence for the final
firmware/identity/return observations. Existing child-exec and reader/writer
cancellation native records are retained on the byte-identical kernel and
rechecked with the current readers, including all **142 existing controls**.
Those VM runs are not newly executed here. Frozen replay repeats the new and
retained record checks from the archived inputs.

All seven LLVM modules verify. Fresh workspace/all-target checking, 196
user-memory tests, 27 process-memory tests, the default ABI-fixture hash,
formatting and the Rust file ceiling pass. Thirteen predecessor contributions
remain byte-identical. The kernel SHA-256 remains
`b8212e5e9aca715d8db0d863cbc9655ed989c3884b938b57055c3e22e04d0a7c`,
with complete stack/physical bounds **2,280 / 4,096** and **4,088 / 4,096 bytes**.
The final swap-free cgroup sample peaks at **508,801,024 / 3,062,714,368 bytes**
with all sampled event counters zero. Its last live sample plus retirement does
not measure an unseen final instant. Guest allocator refusal and host resource
limits are distinct; neither explains any model/provider safeguard decision.

Risk is high for false absence proofs, refusal side effects, identity history
and comparison completeness. Review uses the authority chain, source/diff
review, actual ordinary requests, independent complete-catalog reconstruction,
predecessor comparisons and saved-data controls. No independent human security
review is claimed. This qualifies the selected fork/resize physical misses and
retries, not every resource failure, successful maximal-heap fork copy, identity
exhaustion, fatal partial effects, continuous exclusion or public activation.

## Remaining stages and blockers

2. Complete all caller/loan exclusion, identity-width and remaining allocation-miss
   behavior, and native fatal partial-effect retirement. The first integrated
   cohort does not complete native owner qualification. Queries and within-page
   changes in its real process path retain allocation-free behavior.
3. Broaden native cancellation to every remaining wait/relationship shape,
   broader child exec/image cases, every remaining switch/exit and
   overlapping leaf-table writer. Qualify remaining maximal-copy and resource
   failure shapes and full invalidation/pre-release ordering.
   Existing child cases pass in this component; full pipe and 51-workload
   application regressions remain required.
4. Complete affected production qualification and immutable source/native/load/
   boot reconstruction before activation. Preserve mode, concurrent-access and
   fault gaps as acceptance blockers, not successful model evidence.

The [owned-extent work](step-4-owned-user-extents.md) retains its own unfinished
caller, lifetime and production gates. This foundation does not complete that
work, initial Step 4 or the original distribution, Docker/Podman and VM-host goal.
