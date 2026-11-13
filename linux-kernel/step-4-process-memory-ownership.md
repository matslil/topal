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
rechecked with the current readers, including **58 existing controls**: thirty
child-exec controls, twelve for one reader case and sixteen for one writer case.
The earlier summary's count of 142 incorrectly counted all cancellation cases;
the frozen archive actually retains one case of each wait kind.
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

## Successful maximum copy and empty/one-page handoff

The next [ordinary C program](labs/qemu/x86_64/user_extent_process_mm_geometry_cases.c)
execs a fresh first-party image, fills every byte of a partial **519-page heap**,
then creates an independently mapped child. Separate builds select actual
`fork` (57) and fork-like `clone` (56, SIGCHLD, zero child stack). The compile
choice affects userspace only. Both use the normal **640-MiB, two-CPU QEMU TCG**
guest, the same linked kernel and unchanged host protections.

The complete **2,125,824-byte** prepared copy matches the parent, including the
last mapped page's thirteen bytes beyond the byte break. Actual frame records,
pins and complete table observations bind disjoint backing. The child changes
its break within the last page without allocating, checks every copied byte,
inverts its complete heap, then shrinks to an empty heap. The parent retains
the original full pattern. Four ordinary yields hand off between the parent
and a child with first zero, then one mapped page. The new child page is wholly
zero before it is marked; those marks survive handoff independently of the
parent's retained **518-page** pattern. Child exit restores the parent; invalid
wait-output buffers preserve the status for a valid retry. Parent exit retires
its remaining heap and completes descriptor/PID-policy/MM readback.

Both native cases pass in **149.54 seconds**, including their builds. The
[reader](tools/process-memory/readback_geometry.py) checks **28 MM boundaries,
six retired extents and 6,520,832 erased bytes per case**. Its **22 saved-data
controls per case** reject incomplete maximum copies, final-page corruption,
lost/aliased empty owners, nonzero new pages, mapping/identity errors and missing
retirement or terminal erasure. These controls modify saved records only.

The [archive](tools/process-memory/results/stage-3-maximum-geometry/report.json)
retains both native cases, exact generated/debugger inputs, commands and bounds.
Frozen replay repeats their checks from selected archived inputs. Identified
allocation-refusal, child-exec and all four reader plus all four writer
cancellation records are rechecked with current readers on the byte-identical kernel, including their
**163 existing controls**; those older VM runs are not newly executed here.
All seven LLVM modules verify for both builds. Fresh workspace/all-target
checking, 196 user-memory tests, 27 process-memory tests, default ABI-fixture
hash, formatting and the Rust file ceiling pass. Thirteen predecessor
contributions remain byte-identical per build. Kernel SHA-256 remains
`b8212e5e9aca715d8db0d863cbc9655ed989c3884b938b57055c3e22e04d0a7c`.
Complete native stack/physical bounds remain **2,280 / 4,096** and
**4,088 / 4,096 bytes**. The swap-free cgroup spanning both native builds/runs
has a last sampled peak of **2,251,784,192 / 2,986,774,528 bytes**, with all
sampled event counters zero. Its last live sample and subsequent retirement do
not measure an unseen final instant.

Risk is high for process isolation and false acceptance of whole-heap copying,
empty-owner identity, handoff or erasure. Review uses the approved authority
chain, source/diff review, ordinary native requests, complete boundary bytes,
exact predecessor/kernel comparisons and saved-data controls. No independent
human security review is claimed. This qualifies the selected maximal eager
copy and zero/one-page handoffs; it does not qualify general address spaces,
every mapping producer/loan, continuous exclusion, identity-width exhaustion,
fatal partial effects or public activation. No core-language meaning changes.

## Runnable and zombie child relationships at parent termination

The next [source-built program](labs/qemu/x86_64/user_extent_process_mm_relationship_cases.c)
qualifies parent termination with a runnable child or an unreaped zombie.
Eight cases combine those relationships with ordinary `exit`/`exit_group` and
late-shell/early replacement startup. The child dirties a **101-page heap** and
an unlinked anonymous file with **256 nontrivial bytes**, then either yields
without parking a wait or exits with full status `0x217`. The parent retains
two writer aliases, closes its reader and independently resizes to **35 pages**.
An occupied-slot fork still returns EAGAIN. A runnable child's WNOHANG returns
zero without validating an unused output; a zombie's invalid wait-output buffer
returns EFAULT and preserves its unreaped status. The parent then terminates
without resuming or reaping that child.

Read-only observation captures the actual child's ordinary userspace departure
and complete process storage before either yield or exit. The runnable child
retains its heap, last reader, file description and complete dirty file page
until parent cancellation. The zombie retains status **0x17** after its heap,
file object/page, private continuation and reader have already been retired.
Parent termination performs no duplicate child retirement. Both paths preserve
the independent parent heap until detached cleanup, consume the remaining
descriptions/FIFO and read back whole descriptor, PID-policy and MM storage.
These are the existing bounded outer-owner lifetime rules, not general Linux
orphan reparenting or system-init termination semantics.

All eight native cases pass in **323.87 seconds**, including their builds,
using the normal **640-MiB, two-CPU QEMU TCG** guest. The
[reader](tools/process-memory/readback_relationship.py) checks **120 MM
boundaries, 40 retired extents and 7,798,784 erased bytes**. All **176 saved-data
controls** reject wrong relationships, altered departure/status, missing owners,
incomplete erasure, invented waits and lost/retained file bytes. These controls
change saved records only; no debugger memory/register writes manufacture a
relationship, failure or successful return.

The [archive](tools/process-memory/results/stage-3-relationship-termination/report.json)
binds all eight source configurations, actual guest commands, exact debugger
program, complete bounds and predecessor comparisons. It verifies the preceding
maximum-copy archive and rechecks its twelve identified native runs with current
readers: both maximum copies, allocation refusal, child exec and all eight
reader/writer cancellation cases. Their **207 controls** pass; those older VMs
are not newly executed here. The current linked kernels are byte-identical to
the retained maximum-copy kernel. Frozen replay repeats the new and retained
record comparisons from the selected archived inputs; it is not a new VM run
or full historical rebuild.

All seven LLVM modules verify for each of the eight builds. Fresh workspace/
all-target checking, 196 user-memory tests, 27 process-memory tests, default
ABI-fixture hash, formatting and the Rust file ceiling pass. Thirteen predecessor
contributions remain byte-identical per build. Kernel SHA-256 remains
`b8212e5e9aca715d8db0d863cbc9655ed989c3884b938b57055c3e22e04d0a7c`,
with complete native stack/physical bounds **2,280 / 4,096** and
**4,088 / 4,096 bytes**. The swap-free cgroup spanning all eight native builds/
runs has a last sampled peak of **2,056,458,240 / 3,333,943,296 bytes**, with
all sampled event counters zero. Its last live sample and retirement do not
measure an unseen final instant. Host protections and boundaries remain intact.

Risk is high for isolation, relationship/resource consistency and false claims
of exact retirement. Review uses the authority chain, source/diff review,
ordinary native requests, whole saved ownership/resource observations, exact
kernel/predecessor comparison and saved-data controls. No independent human
security review is claimed. This extends the selected runnable/zombie lifetime
evidence without changing kernel behavior or core-language meaning. Broader
mapping/loan/caller, identity/fatal, continuous lifetime/invalidation and public
activation obligations remain open.

## ELF symbol scopes in native audits

Preparing the ordinary-interface matrix exposed a host-side inventory failure
before VM execution: the linked process-MM kernel has two distinct local
functions named `p.copy`, while the small ELF reader required every definition's
name to be globally unique. The [ELF binding rules](https://gabi.xinuos.com/v42/elf/05-symtab.html#symbol-binding)
permit local names from separate input files to coexist. This is an artifact
reader issue, distinct from a guest, resource or model/provider safeguard failure.

The [reader](tools/user-memory/inspect_call_inventory.py) now keeps complete
defined-symbol enumeration separately from unambiguous name lookup. Process
roots, exact external-transfer addresses, dominance roots and paging-instruction
owners use the complete enumeration, including ambiguous local definitions.
Function lookup rejects ambiguous local names; conflicting external definitions
still fail. A local definition cannot satisfy an undefined global name. The
name-only relocation API rejects a defined target whose indexed identity that
name cannot represent; exact table indices remain available to indexed readers.
This is conservative validation of the compiler-owned artifacts, not a general
linker implementation or new kernel authority.

Seven [ELF scope controls](tools/user-memory/test_elf_symbol_scopes.py) cover
distinct local definitions, local/global coexistence, undefined declarations,
conflicting globals, identical definitions and indexed relocation identity.
The full user-memory suite passes **203 tests**. No native kernel, application,
core-language meaning or host protection changes. Risk is high for silent
omission or conflation in security evidence; review covers all consumers of
symbol enumeration and tests explicit ambiguity rejection. Ordinary-interface
matrix execution and frozen qualification remain subsequent work.

## Ordinary application interfaces on the process-MM kernel

The existing source-built transfer, exec-refusal, signal, poll, pipe, storage,
metadata, serial and stat fixtures now run on the experimental process-MM
selection. Their application sources and interface assertions remain unchanged.
The test factory selects kernel ownership independently of the application
cohort; default/predecessor selections retain their previous behavior. No public
kernel path or core-language meaning changes.

All **nine native cohorts** complete in the normal **640-MiB, two-CPU TCG VM**.
They cover **36 source invocations / 1,228 source-case syscalls**, with **2,032
recorded application-interface syscalls** when the transfer/startup/shell/file
and poll delegations are included. Complete process-MM observations cover
**702 boundaries**, **354 retired extents / 59,437,056 erased bytes**. Each
ordinary source invocation preserves its original 34-page heap while growing,
using and retiring its temporary transfer pages. The retained ABI witness
creates four children, replaces the parent image three times and completes
terminal cleanup per cohort. Its children exit directly; it supplies no new
cooperative-handoff observation.

The [observer](labs/qemu/x86_64/process_mm_regression_observer.py) reuses the
original process-MM capture/model in a separate namespace, preserving every
selected MM entry/return and register/ownership comparison. The common return
breakpoint is enabled only while a selected MM request is pending. Complete
events remain in memory; compact JSON is published atomically every 64 events,
on MM observer failure and at final completion. Four host-only transport and
checkpoint tests verify these mechanics; they do not supply native authority.

The [saved-record reader](tools/process-memory/readback_interfaces.py) joins
ordinary source records with complete MM banks, identities and heap bytes,
replays every MM event and rejects **139 altered-record controls**. Its separate
FIFO byte-order reconstruction checks wrapped and full-ring transfers. Metadata
reconstruction applies every captured filesystem/table delta to whole initial
storage and checks it against the metadata model with actual clock returns and
write payloads. That model is shared with the native metadata observer; neither
it nor the selected result checks independently replays every native observer
assertion. All controls change saved data only.

The [archive](tools/process-memory/results/stage-3-interface-regressions/report.json)
binds the nine builds, original C sources, exact observer programs, LLVM and
native artifacts, guest commands, resource samples and fresh validation. Every
linked kernel is byte-identical to the retained maximum-copy kernel:
`b8212e5e9aca715d8db0d863cbc9655ed989c3884b938b57055c3e22e04d0a7c`.
All seven LLVM modules verify per build; complete stack/physical bounds remain
**2,280/4,096** and **4,088/4,096 bytes**. Thirteen predecessor contributions
remain unchanged per build. Workspace/all-target locked checking, all **203
user-memory tests**, all **31 process-memory tests**, the default ABI-fixture
hash, formatting and the Rust file-size ceiling pass.

The archive verifies the preceding relationship archive and rechecks **twenty
identified previous native histories / 383 retained controls** on this same
kernel. Those older VMs and full historical rebuilds are not newly executed.
Frozen replay runs the selected archived Python inputs without a VM; cold
production reconstruction remains required.

Four earlier failed attempts remain recorded: the pre-VM ELF-name failure,
480- and 600-second observer deadlines, and a subsequent metadata
`SyscallEntered` assertion before any source invocation was recorded. The last
failure did not capture its assertion operands, so its cause remains unresolved.
A read-only diagnostic that added assertion-message context passed all four
metadata invocations and teardown; a subsequent fresh build/run with the
unchanged observer also passed. The diagnostic and retry cannot establish the
original failure's cause. Retain this observer reliability question for further
qualification.

The first efficient batch took **2,077.36 seconds**, passing six cohorts before
that metadata assertion; the explicitly selected metadata/serial/stat retry
passes in **992.98 seconds**, including guest construction. A saved-partial-data benchmark
measured the old JSON sink at **1.798 seconds / 2,945,859 Python writes**, versus
compact encoding at **0.641 seconds / one write**, with decoded equality.
This host-only benchmark does not establish the sole cause of either deadline.
The efficient observer keeps the existing finite **600-second** MM deadline;
predecessor deadlines, all interface assertions and host protections remain.
An initial offline reader also incorrectly expected handoffs from the ordinary
ABI witness; it now requires the actual direct-exit lifecycle and retains the
separate historical handoff evidence. None of these failures or successful
retries explains a model/provider safeguard decision.

The two successful-cohort batches have last sampled cgroup peaks of
**1,815,711,744 / 3,385,348,096 bytes** and
**1,842,712,576 / 3,227,897,856 bytes**, respectively. Both have zero swap and
zero sampled memory events. Last live samples and subsequent group retirement
do not measure an unseen final instant. The diagnostic has no retained cgroup
sample; its monitor attempted to attach after that group had retired.

Risk remains high for false acceptance of interface/ownership regressions.
Review combines the approved authority chain, full source/diff review, existing
ordinary native assertions, MM observations, FIFO byte-order reconstruction,
metadata reconstruction, exact artifact comparisons and altered saved-data
controls. No independent human security review is claimed. The observer
reliability question, remaining caller/loan/mapping and fatal/identity-width
proofs, continuous lifetime/invalidation, unchanged 51-workload application
corpus and public activation remain open. This stage does not complete initial
Step 4 or the original full Linux/distribution/container/VM-host objective.

## Unchanged application corpus on the process-MM kernel

All **51 unchanged pinned BusyBox workloads** pass on the experimental
process-MM kernel. The ignored `process_mm_preserves_pinned_busybox_workloads`
test selects their existing command/output/status contracts. Each workload uses
a separate normal **640-MiB, two-CPU TCG VM**, with the existing finite
600-second deadline. An explicit diagnostic subset cannot qualify the whole
corpus. This adds no public kernel selection or core-language meaning.

The [application observer](labs/qemu/x86_64/process_mm_application_inspect.py)
retains the predecessor's checks for operations 0..7. Separate namespaces retain
the existing owned-extent callback/copy/staging observations and process-MM
boundary checks. MM returns also check the original complete caller, private,
stack, facts, output and gateway-recovery retirement windows. Final read-only
observation captures whole frame-catalog and registry digests, the remaining
root-token bank and actual CR3. The shared console checker preserves the
existing prelude, output, exit-status, newline and successful-exec contracts.

The [saved-record reader](tools/process-memory/readback_applications.py) joins
legacy and process-MM observations, rechecks the selected MM transitions and
binds return-retirement addresses, sizes and zero digests. Kernel load spans
and reservations bind to the linked ELF and actual boot-resource length.
Complete final catalog and registry reconstruction uses the original firmware
frames and remaining pinned bootstrap allocation. These checks complement the
native observers; they do not independently replay every observer assertion or
prove continuous exclusion and every invalidation/release instruction.

The complete corpus records **1,054 MM boundaries**, including **102 successful
execs**, **27 child creations/commits/exits**, **four cooperative handoffs**,
and terminal cleanup in all 51 VMs. It observes **333 retired extents /
25,694,208 erased backing bytes**. The reader rejects **859 altered-record
controls**, which modify saved data only. Complete final catalog/registry
reconstruction binds all **152,008 firmware frames** per VM and the remaining
pinned five-page bootstrap hierarchy. The union of first-reach samples covers
**66 of 91 extent-helper sites** and matches the linked process graph; it does
not qualify every site, path or caller lifetime.

The [archive](tools/process-memory/results/stage-3-application-corpus/report.json)
binds the actual native command, source hashes before/after execution, exact
observer programs, pinned image and boot resource, generated artifacts and fresh
validation. Its kernel remains byte-identical to the preceding process-MM
cohorts: `b8212e5e9aca715d8db0d863cbc9655ed989c3884b938b57055c3e22e04d0a7c`.
All seven LLVM modules verify, and thirteen predecessor contributions remain
unchanged. Complete stack/physical bounds remain **2,280/4,096** and
**4,088/4,096 bytes**. Locked workspace/all-target checking, **203 user-memory
tests**, **37 process-memory tests**, the default ABI-fixture hash, formatting
and Rust file-size ceiling pass. Six new host-only console tests use the 51
retained real consoles and clean ABI console plus altered-output/status/exec/
prelude controls; these tests supply no fresh VM evidence.

The collector verifies the preceding interface archive and rechecks its
**nine interface and twenty process histories / 522 retained controls** on the
same kernel. Those are identified previous native executions, not new VMs or
complete historical rebuilds. The
[frozen replay](tools/process-memory/replay_applications.py) repeats the new and
retained saved-record comparisons and the frozen 203/37 Python suites without
executing a VM or rebuilding a kernel. Its overall success report is written
only after both model suites pass; an intermediate record-comparison result
cannot establish complete replay success.

The full native batch completes in **7,955.87 seconds**, with unchanged watched
source hashes and a last sampled cgroup peak of
**1,164,021,760 / 3,395,215,360 bytes**, zero swap and zero memory events. This
last live sample does not measure an unseen final instant. Host protections,
filesystem/network boundaries and the resource wrapper remain unchanged.

An earlier invocation failed before starting a VM because a relative pinned
image path resolved from Cargo's crate directory. Supplying the absolute path
fixed that invocation error. A preliminary native selection of `true`,
`child-echo` and `pipe-substitution-large` then passed in **481.30 seconds**,
with unchanged source hashes, a last sampled cgroup peak of
**695,607,296 / 3,384,328,192 bytes**, zero swap and zero memory events.
That preliminary run preceded final catalog/registry digest capture; it cannot
qualify the current complete corpus. These results provide no explanation of
an AI model or provider safeguard decision.

Risk remains high for accepting incorrect application behavior or memory
retirement. Review combines the authority chain, source/diff review, retained
native assertions, artifact bindings, complete final reconstruction and separate
saved-data controls; no independent human security review is claimed. The
earlier metadata observer assertion remains unexplained. Remaining caller/loan/
mapping, identity-width/resource/fatal, continuous lifetime and public activation
obligations still apply. This qualifies the unchanged application-corpus
component, not initial Step 4 or the full Linux/distribution/container/VM-host
objective.

## Process publication and retirement observations

`TOPAL-SYSTEMS-PROCESS-MM-001` and `TOPAL-SYSTEMS-USER-EXTENT-001` require
completed publication/invalidation before releasing an old heap. The optional
`TOPAL_TEST_PROCESS_MM_PUBLICATION=1` observer adds stops at the complete
149-byte publisher, actual unpin/erase/release boundaries and terminal root
handoff. Fresh owner/table metadata and raw per-CPU post-INVLPG states bind
equal-size and empty process transitions as well as resize/exec. This component
changes observation and saved-record validation; the native kernel remains
byte-identical to the preceding application corpus.

All **seven selected native tests / 21 VMs** and their complete source-bound
readers pass: owner handoff, child exec, physical allocation refusal, both
maximum fork/clone geometries, four blocked-reader cancellations, four
blocked-writer cancellations and eight runnable/zombie relationships. They
record **279 publications**, **26,191
invalidation states**, **45 equal-size
process publications** and **166 retired extents /
46,915,584 erased bytes**, with 21 terminal root
handoffs. All 252 new saved-data controls reject their changed records,
in addition to 403 original cohort controls. The raw traces retain
22 explicitly cancelled/retried CPU blocks; these are
validated trace records, not silently discarded missing states or TLB dumps.

The reader binds the complete observer program, linked publisher/caller bytes,
all six hierarchy pages, surrounding MM boundaries, every raw CPU-log header
and every observed active/inactive heap retirement. Twelve controls per VM
exercise missing CPU states, owner/span/root/call mismatches, barrier and mapping
changes, early/failed unpin, dirty release and retained authority. These
saved-data checks remain distinct from native execution and continuous lifetime.

The [archive](tools/process-memory/results/stage-3-process-publication/report.json)
freezes actual commands, inputs, kernel/boot artifacts, raw logs and historical
attempts. Seven LLVM modules verify per build; thirteen predecessor
contributions remain unchanged. Complete stack/physical bounds remain
**2,280/4,096** and **4,088/4,096 bytes**. Fresh locked workspace/all-target
checking, **203 user-memory tests**, **59 process-memory tests**, the default ABI
generator fixture, formatting and the Rust file-size ceiling pass. The native
matrix completes in **2,515.59 seconds**, with unchanged
hashes for all 649 watched inputs. Its
cumulative cgroup sample records **1,653,047,296 /
3,353,309,184 bytes**, zero swap and zero memory events. This
does not measure independent per-VM peaks or an unseen final instant.

The collector rechecks the identified previous **51 application, nine interface
and twenty process histories** on the byte-identical kernel using their original
boot-resource facts. They are historical native executions, not new VMs or full
historical rebuilds. The [frozen replay](tools/process-memory/replay_publication.py)
repeats all new and retained record comparisons and the frozen 203/59 model
suites without executing a VM or rebuilding a kernel. Its overall success marker
is written only after both model suites pass. A separate
[replay receipt](tools/process-memory/results/stage-3-process-publication-replay.json)
binds the archive manifest and complete bounded follow-up result.

The first observer pilot wrongly required every retained root-slot allocation
to have one pin. Actual root/low translation owners had one pin; a distinct
retained allocation legitimately had two. The corrected model binds the actual
translation-storage pins and preserves heap-owner checks; a separate pin-count
control covers that distinction. Later transport changes coalesce fresh metadata
reads and checkpoint complete histories, retaining every native assertion.
Historical method pilots do not qualify the revised observer; the fresh matrix
above supplies that evidence.

The first matrix's three native tests passed, then its allocation reader rejected
an intentionally retained source-case marker. That fixture terminates through
exec/exit instead of returning through its original marker. The corrected reader
distinguishes returning and terminal cases while requiring no pending memory
request or refusal; original terminal cleanup remains mandatory. Five completion
controls and the fresh allocation run pass. Original failure/pilot records and
exact or explicitly reconstructed historical source identities remain preserved.
An older manifest's hash-record format has an explicit SHA-only projection;
its original bytes and per-file metadata remain archived. These repository and
observer results establish no reason for an AI model or service safeguard.

This qualifies the experimental local publisher/retirement observation component.
It does not complete initial Step 4. Other callers, private loans, mapping writers,
continuous lifetime, AP/DMA/asynchronous access, fatal cleanup and public
activation retain the acceptance work below.

### Review of the publication evidence boundary

Risk is high because false acceptance could hide stale mappings or premature
frame release. Review uses the authority chain, the complete observer/model/
reader diff, the native call sites and saved-data controls. This is single-agent
review; it is not an independent human security review or a complete formal
proof. The native runtime and compiler outputs are unchanged by this observer
component.

| Required behavior | Evidence supplied by this component | Remaining acceptance work |
| --- | --- | --- |
| Publish the selected heap before return | Bound calls in `topal_frame_memory_request_v1`, `p.commit`, `p.switch` and `p.child_exit`; complete local publisher layout, table states and raw post-INVLPG loop records | All callers and mapping producers, continuous observation/access lifetime and asynchronous or other-CPU access |
| Revoke access before retiring active or inactive backing | Actual active/inactive token slots, completed demap history, unpin result, complete zero readback before release and consumed token/registry/cells | Other alias domains, all release paths and fatal partial effects |
| Retire an abandoned prepared owner | Earlier preparation-component evidence remains separately scoped | This observer hooks active and inactive token slots; it does not observe prepared-token cancellation in `pm.cancel` |
| Keep image/stack parking separate from heap translation | Source review confirms the process-MM child/pair branches copy only low slots 0..502 and high slots 510..511; heap publication is observed separately | Complete producer/caller binding and byte/permission ownership for every image/stack lifetime; selected table snapshots do not prove every intervening write |
| Finish user-copy loans before switching | Original admission/entry and cohort assertions remain active | Complete native caller/private-loan exclusion; a local interrupt or mode observation is insufficient |
| Preserve held identities after a trusted partial failure | Existing poisoned-owner requirement remains unchanged | Actual nonreturning fatal cleanup and identity-width/resource-edge qualification; successful ordinary cohorts do not exercise these paths |
| Activate a reconstructed public kernel | Exact experimental artifacts and retained application records remain available | Cold source/native/load/boot reconstruction and all activation gates, followed by the complete Linux application/distribution/container/VM-host objective |

The review maps current evidence to the existing requirements. It adds no new
language design or relaxation of the process/MM or owned-extent contracts.

## Source-built process policy failure diagnostics

The approved fatal contract preserves performed effects and held identities,
poisons the memory/allocator owner, retires invocation temporaries and never
resumes either process. Two diagnostic Topal policies now exercise this contract
through the existing ordinary fork/resize C program in QEMU. They use the normal
checked source binding and retain every controller/admission/delta guard. The
canonical allocator source and normal kernel behavior are unchanged. The
observer only reads guest state; it injects no register, data or instruction
changes. These deliberately incorrect policies are test inputs, not qualified
functional allocator implementations or public selections.

| Source failure | Actual prefix and terminal evidence |
| --- | --- |
| Allocate 101 pages with incorrect cell identities | Detected before replacement publication or issuance. The active child and inactive parent retain their complete 67-page heaps, handles and pins. All 101 performed source writes remain visible; the failed candidate is never issued. |
| Release the old 67-page child heap with incorrect cell identities | The copied 101-page candidate is pinned, published and invalidated before old-backing unpin/erasure. The invalid release leaves the old live unpinned record/token, the pinned candidate and the pinned parent accounted for. All 67 performed source writes and already-issued identities remain visible; release is explicitly incomplete. |

The full initial catalog, complete registry reconstruction, comparison snapshot,
process bytes and publication/CPU records bind those observations. Both terminal
paths preserve the original callee registers/stack, poison owners and erase the
declared private/source/output/recovery domains and **4,000 bytes** of direct-MM
gateway caller storage. That is the 3,952-byte inner window plus 48 saved-register
bytes. The conditional outer 88-byte user-extent adapter is a separate route;
these cases do not qualify it or every possible fatal path.

The [archive](tools/process-memory/results/stage-3-process-policy-failure/report.json)
retains **three fresh targeted VMs**: the complete unchanged ordinary cohort and
both diagnostics. The ordinary kernel reproduces SHA-256
`b8212e5e9aca715d8db0d863cbc9655ed989c3884b938b57055c3e22e04d0a7c`.
The diagnostic policies have distinct source/kernel identities. Their readers
check four and five completed prefix publications, respectively, and reject
**27 altered-record controls**. Those controls are host checks, not additional
guest executions.
The ordinary cohort retains its full original checks and completion obligations.

Fresh locked workspace/all-target checking, **383 compiler tests** (22 ignored),
**203 user-memory and 59 process-memory tests**, the default ABI generator fixture, formatting and
the Rust file ceiling pass. All seven LLVM modules verify for each build. The
complete source-derived caller bound remains **2,280/4,096 bytes**; the complete
physical profile remains **4,088/4,096 bytes**, with the direct-MM route distinguished
above. All **722 watched inputs** remained unchanged during the original
three-VM/readback batch. Its subsequent broader compiler validation failed with
explicit disk-quota errors in all 89 failed cases (294 passed, 22 ignored); the
original overall failure and logs remain preserved. No memory-limit or OOM
events occurred. A fresh seven-command qualification uses disk-backed task
scratch under `linux-kernel/.cache` and two Rust test threads, retaining host
protections, the original tmpfs quota and the bounded memory policy. It completes
in **3570.12 seconds**, with unchanged source hashes. Its cumulative cgroup sample
records **1,765,232,640 / 2,712,424,448 bytes**, zero swap and zero
memory events; this is not an independent per-VM peak measurement. The compiler
suite includes existing native/VM integration tests, separately logged rather
than added to the targeted three-VM count.

After the native phase, source review corrected the collector's comparison of
Python integer keys with JSON string keys. The archive explicitly preserves that
original collector, its original watched identity and the repaired collector's
identity. Every compiler, allocator, application, observer and reader input
remains unchanged. Collection requires the separate successful qualification;
it does not relabel the original failed batch as successful.

Frozen replay repeats all three saved-record readers and raw CPU-log checks; it
does not execute a VM, rebuild a kernel or rerun the model suites. The
[review receipt](tools/process-memory/results/stage-3-process-policy-failure-review.json)
adds complete archive/native-byte checks, 15 unchanged surrounding objects per
build and the linked terminal halt instructions. Review follows the existing
docs/se/spec/tool requirements. Risk remains high for false acceptance of owner
retention or cleanup. This is single-agent review, not independent human security
review or a complete formal proof. Partial source writes, failure inside the
publication loop, other failure/resource/identity-width shapes, all caller/
private-loan and mapping lifetimes, AP/DMA/asynchronous access, cold
public reconstruction and the complete Linux/distribution/container/VM-host
objective remain open. No AI safeguard explanation follows from these results.

## Partial source-status failures after performed stores

Two additional compiled Topal diagnostic policies return an invalid source
status after **13 actual metadata stores**, through the unchanged ordinary
fork/resize C fixture. They leave the canonical allocator, controller checks,
runtime implementation and core language unchanged. The producer returns false after
the thirteenth store, before advancing the progress header: the retained header
therefore records twelve processed cells and the actual allocation/release
phase. This diagnostic contract deliberately exercises an incomplete source
operation; it does not authorize partial success or a functional allocator.

| Diagnostic | Preserved effects and held owners |
| --- | --- |
| Allocation of 101 pages stops after 13 stores | Thirteen cells contain the pending identity, while the issuance counter remains unconsumed and no candidate is published. The child and parent retain their complete pinned 67-page heaps. |
| Release of the old 67-page child heap stops after 13 stores | The copied 101-page replacement is already pinned, published and invalidated; the old backing is unpinned and erased. Thirteen source identities are cleared, while the old whole token/live record, pinned candidate and pinned parent remain accounted for. Release is incomplete. |

The read-only observer now captures the real source-result decoder entry and
boxed `Nat` output. Both partial failures return `2^64 - 1`, decoded as invalid
protocol. ELF-bound native call sites distinguish that rejection in `f.invoke`
from the older completed-but-invalid-delta rejection in allocate/release. Full
initial catalog, whole registry reconstruction, comparison snapshot, retained
process bytes and issuance are checked through terminal cleanup. The decoder
output is erased on the fatal route. No guest registers, data or instructions
are injected by the debugger, and no third-party application is patched.

The [five-VM archive](tools/process-memory/results/stage-3-process-partial-source/report.json)
retains a fresh ordinary cohort and all four diagnostics. Ordinary, completed
allocation-failure and completed release-failure kernels reproduce their exact
previous SHA-256 identities; only the two new diagnostic source/kernel identities
differ. All **70 diagnostic saved-record controls** reject: 15 completed
allocation, 18 completed release, 17 partial allocation and 20 partial release.
They include changed returned values/native call sites, pretended completion
and lost performed stores. These are reader controls, not additional VMs.
The complete ordinary cohort retains its original checks and completion gates.

Fresh locked workspace/all-target checking, **203 user-memory tests**, **63
process-memory tests**, the default ABI fixture, formatting and the Rust file
ceiling pass. This increment changes only a private Rust diagnostic-test module
and observation/evidence tools. Its explicit observer validation profile binds
the previous full **383-passed / 22-ignored compiler suite** at commit
`9c84c21a305f502a29aaa0cce974ba85fc879c59`; it does **not** claim a new full-suite
run. The guard verifies 308 other compiler input hashes and rejects
production/dependency/higher-authority tracked changes. Four host-only controls
check successful reuse and refusal of changed source or failed reference tests.
The default full validation profile still runs the entire compiler suite.

All **724 watched inputs** remain unchanged throughout the matrix, which
completes in **904.11 seconds**. Its cumulative bounded cgroup sample is
**1,121,366,016 / 3,274,924,032 bytes**, with zero swap and zero memory events.
This is not a per-VM peak. Collection verifies all seven LLVM modules per build;
frozen replay repeats all five saved-record readers, CPU-log checks and controls
without a VM, kernel rebuild or model-suite rerun. The original three-VM archive,
its quota-failure history and successful recovery remain unchanged.

The [review receipt](tools/process-memory/results/stage-3-process-partial-source-review.json)
checks complete archive/native byte identity, 14 unchanged surrounding objects
per build, unchanged application sources and linked fatal halt instructions.
The first companion audit rejected a fifteenth object's equality: `boundary.o`
encodes the private history-pointer field at displacement 1,272 in the original
policies and 1,360 in the partial policies. The additional source helper changes
the recovery depth from 13 to 14 frames. The revised audit binds that operand to
the generated runtime's history field plus its 64-byte header, checks the linked
instruction and requires every other object byte to remain equal. No object or
guest bytes are modified. The failed audit remains recorded in the receipt;
the source-frozen native/archive/replay results remain unchanged.
Complete caller/physical bounds remain **2,280/4,096** and **4,088/4,096 bytes**.
These direct-MM fatal cases physically retire **4,000 bytes** (3,952 inner plus
48 saved-register bytes); they do not qualify the separate outer 88-byte adapter.

Risk remains high for false acceptance of partial effects, held identities or
cleanup. Review follows the existing docs/se/tool/spec authority chain and full
source/artifact checks; this is single-agent review, not independent human
security review or complete formal proof. Selected source-status and completed
delta failures are covered. Failure inside publication, other fatal/resource/
identity-width shapes, every caller/private loan and mapping writer, continuous
lifetime, AP/DMA/asynchronous access, public reconstruction and initial Step 4's
full Linux/distribution/container/VM-host acceptance remain unfinished. These
results establish no explanation of an earlier AI safeguard interruption.

## Remaining stages and blockers

2. Complete all caller/loan exclusion, identity-width and remaining allocation-miss
   behavior, and native fatal partial-effect retirement. The first integrated
   cohort does not complete native owner qualification. Queries and within-page
   changes in its real process path retain allocation-free behavior.
3. Broaden native cancellation beyond the selected blocked/runnable/zombie shapes,
   broader child exec/image cases, every remaining switch/exit and
   overlapping leaf-table writer. Qualify remaining creation/resize/resource
   failure shapes and full invalidation/pre-release ordering.
   The existing child/pipe and unchanged 51-workload application regressions
   pass on this experimental kernel; they do not discharge these remaining
   native ownership/lifetime obligations.
4. Complete affected production qualification and immutable source/native/load/
   boot reconstruction before activation. Preserve mode, concurrent-access and
   fault gaps as acceptance blockers, not successful model evidence.

The [owned-extent work](step-4-owned-user-extents.md) retains its own unfinished
caller, lifetime and production gates. This foundation does not complete that
work, initial Step 4 or the original distribution, Docker/Podman and VM-host goal.
