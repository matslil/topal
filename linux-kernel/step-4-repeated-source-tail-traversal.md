# Step 4: repeated ordinary source traversal

`TK-DEC-282`; follows the
[systems direction](design/repeated-source-tail-traversal.md),
`TOPAL-REQ-SYSTEMS-TAIL-TRAVERSAL-001`, `TOPAL-COMP-SYSTEMS-TAIL-PLAN-001`,
`TOPAL-SYSTEMS-TAIL-PLAN-001` and `TOPAL-SYSTEMS-TAIL-REUSE-001`.

Stages 1, 2 and 3 are complete for the qualified traversal and owned frame
allocator binding. Committed production-kernel and application qualification
remains Stage 4. The progress sections retain the scope of their historical
checkpoints. The current publication result is recorded in
[the final Stage 3 section](#stage-3-complete-source-bearing-frame-publication).

## Stage 1: source structure and progress

The [source-only selector](../src/topal-language/src/compiler_model/kernel_regions.rs)
provides symbolic region/Nat parameter types to the existing ordinary frontend.
For this selection, initial and recursive entry lookup use the same ordered
flattened package signature. Existing checked `Decreases` and Nat adaptation
remain required. The source grammar, operand limit, ordinary package meaning
and existing native qualification paths are unchanged.

The [tail inspector](../src/topal-compiler/src/systems_tail_plan.rs) identifies
the actual natural countdown guard and checks every direct tail edge, including
all argument evaluations and unselected branches. Decrements are positive,
literal and nonnegative under the guard. The optional borrowed region remains
the same input. Helpers must be resolved, reachable and acyclic, without hidden
captures, foreign dependencies, view construction or re-entry. Inferred reads/
writes propagate through helper calls and cannot exceed an explicit empty
effect row. Existing Nat-to-Int widening remains legal; the inspector creates
no narrowing proof.

The result binds exact source/dependency identity/path/bytes and fingerprints
every checked function, including ordered expressions, literals and arguments.
The checked-function fingerprint uses a versioned compiler-internal encoding;
it is no source or native ABI. Its one-step helper-depth fact is not a recursive
stack or logical-identity bound. Descriptive public metadata cannot attest
externally constructed IR or authorize emission.

The actual [clear-prefix source](kernel/algorithms/clear-prefix.t) carries
remaining bytes, offset and success status through ordinary recursion. Helpers
perform byte stores in source order and retain a failed status without rolling
back earlier effects. A separate ordinary read-probe variant tests Optional
loads and transitive read effects. Neither source is selected by the current
kernel or made executable by this source-plan stage.

The [plan tests](../src/topal-compiler/src/systems_tail_plan_tests.rs) exercise
actual shared source/module analysis, packaged recursion and legacy refusal;
independent natural-countdown oracles; all-branch progress; base/non-tail/
argument recursion refusals; guard/step/arity/classifier changes; changed region
transport; helper cycles and missing/duplicate/unreachable functions; source/
checked-graph/dependency binding; hidden/static/resource-result refusal; effect
containment; Nat transport and function/parameter/expression/depth budgets.
Bindings and Optional payloads cannot shadow measure/region storage; distinct
storage with the same display name remains distinct.
IR mutation tests establish defensive inspection only and do not claim source
origin. The eighteen targeted tests and complete affected frontend/compiler
validation pass.

An earlier complete run passed 636 frontend tests and 293 compiler tests before
review added the explicit shadow-storage refusal and two regressions. Its
[historical scope](tools/tail-traversal/results/stage-1-before-shadow-guard/scope.json)
and exact command logs remain separate. That preliminary run bound input hashes
without saving all source bytes and does not qualify the final guard. Fresh final
validation retains exact compressed source snapshots as well as command logs.

The [final record](tools/tail-traversal/results/stage-1/qualification.json)
passes all **636 frontend unit tests** and **295 compiler unit tests** with
no failures or ignored tests; the eighteen tail-plan tests are included in
the compiler total. Locked workspace all-target checking, all four coverage
tests, workspace and included-file formatting, and the unchanged Rust size
ceiling also pass. At that checkpoint, the systems ledger contained **312 stable rules**.
Independent readback verifies every source snapshot, original/compressed log
hash and command outcome. The unchanged wrapper used a 2998 MiB adaptive limit,
1569 MiB host reserve and zero cgroup swap, with disk-backed scratch behind a
short owned temporary alias. No host safeguard changed.

The record preserves the verification-input snapshots used by the tests.
Subsequent report/traceability edits record their actual outcomes; they do not
change the qualified compiler/source/normative inputs. This qualification is
source-inspector and compiler regression evidence, not new kernel execution.

Risk is high at termination, recursive signature identity, capability transport,
effects and eventual storage reuse. Review follows ordinary execution, resource,
effect and optimization meaning, then the systems-only requirements/specification
and full diff. It distinguishes structural eligibility from native resource and
ownership evidence. No independent review or new native qualification is claimed.

## Stage 2 progress: conditional storage analysis

The [carried-state proof](design/tail-state-bounds.md) and
[implementation](../src/topal-compiler/src/systems_tail_bounds.rs) rebuild the
source plan, resolve linear relations through immutable storage and closed
helpers, and derive upper/lower potentials from the measure. Constant resets
expand the baseline; other resets must fit that baseline rather than the
wider global envelope. The analysis retains the existing 256-bit arithmetic
ceiling and explicit budgets for symbolic and allocation proof work.

The [step walker](../src/topal-compiler/src/systems_tail_step_bounds.rs)
charges ordinary prefix/guard/helper work, each tail argument once and terminal
actions without following the recursive edge. It uses the reviewed shared
numeric/region algorithms and their exact digests. Existing executable profiles
retain their subtraction and recursive-closure refusals. Both state banks,
the proposed pinned metadata layout, signed initial copies and immutable
caller-frame facts are accounted for separately.

For the actual clear-prefix source, the conditional layout needs 80 bytes per
bank, 72 initial bridge bytes, 160 private metadata bytes and at most 576 bytes
of source work per step: 968 private temporary bytes in this proposed layout.
The offset envelope is 65 bits. These numbers qualify the analysis only; the
actual native checkpoint must independently account for its complete layout,
stack, representation, lifetime and cleanup before any executable publication.

[Call counts](../src/topal-compiler/src/systems_tail_call_bounds.rs) include
source-call multiplicities rather than distinct functions or graph depth.
Fresh-invocation limits reserve root identity 1 and next counter 2; an existing
history requires admission using its actual counters. No view count cap,
logical lifetime or history is reset by this analysis.

The [22 targeted tests](../src/topal-compiler/src/systems_tail_bounds_tests.rs)
cover actual clear/read source, signed drift, nonunit descent, helper/binding
relations, all argument/prefix/terminal costs, constant and multilimb state,
equal-span edges, Optional payloads, invalid growth/reset/Nat/allocation cases,
legacy refusal and independent finite-counter exhaustion oracles. Compact DAG
mutation tests establish defensive analysis only, without source-origin claims.
The [current record](tools/tail-traversal/results/stage-2-bounds/qualification.json)
retains exact inputs and complete command logs. Stage 2 remains in progress.

The final bounds-component run passes all **317 compiler unit tests**, including
the **22 targeted tests**, locked workspace all-target checking, all **four
coverage tests**, formatting and the unchanged Rust source-size ceiling. The
systems ledger contained **313 stable rules** at that checkpoint. The unchanged wrapper used an
adaptive 3028 MiB limit, 1569 MiB host reserve and zero cgroup swap. Peak cgroup
usage was 1043 MiB, with zero limit hits, OOM events or OOM kills. Independent
readback verifies all 23 source snapshots, three original verification-input
snapshots, six complete logs and runner hashes. Factual report/trace updates
follow the run; its original verification inputs remain retained.

An earlier full command pass is retained with its
[historical scope](tools/tail-traversal/results/stage-2-bounds-before-identity-guard/scope.json).
Review subsequently corrected the fresh-identity ceiling to reserve a
representable next counter and added the independent counter oracle. That
earlier run does not qualify the corrected boundary. The final run retains
its own exact snapshots.

## Stage 2 progress: shared lowering and native C components

The [return-boundary design](design/tail-step-component.md) lowers one source
step into a synthetic private function. Ordinary statements, argument
evaluation (including capture propagation), scalar/region operations, helper
bodies and payloads use the shared emitter. Tail-position control flow returns
a private terminal/successor outcome after copying the actual result or every
next scalar. No recursive native call is emitted. These outcomes do not become
source return values; debug information names the synthetic tail step.

The [component builder](../src/topal-compiler/src/systems_tail_component.rs)
rebuilds the plan/ranges and binds function identities, readonly integer-root
extents, LLVM bytes and a distinct tail-region runtime. Helpers receive a
private context operand, issue fresh logical frames/loans, validate their
current scoped views and retire those scopes before returning. The distinct
runtime admits a caller-proven mapped owner extent beyond the older 65536-byte
cap; the old runtime, its caps and every existing executable profile remain
unchanged. No component record is a kernel publication certificate or an
attestation of source origin for externally constructed IR.

The independent [C hooks](labs/qemu/x86_64/tail_step_component_runtime.c) and
[client](labs/qemu/x86_64/tail_step_component_client.c) execute actual generated
objects with guarded owner/private mappings and protected readonly caller
frames. Read-probe owners are also mapped readonly. They check exact source
results, successor facts before retirement, complete bank/work erasure,
readonly/aliased/160-bit signed roots, source order, partial/refused byte
accesses, fresh helper history and pre-entry counter admission. The clearer
and reader each traverse 70000 bytes using reused storage. Thirteen transfer/
result-copy failures and two pre-store allocation failures preserve the
independently expected owner bytes and consume history without rollback.

The [exact component record](tools/tail-traversal/results/stage-2-step-components/qualification.json)
passes all **327 compiler unit tests**, all **261 CLI integration tests**,
locked workspace all-target checking, four coverage tests, formatting (including
both new included emitter files) and source size. The ten targeted step tests
and 22 bounds tests are included in the unit total. The ledger has **314 stable
rules**. Each targeted/full-unit cohort independently executes fifteen native
clients: eight actual-source variants and seven explicitly defensive IR graphs,
covering 306 cases, 151318 source-step entries, 151303 completed checkpoints and
fifteen injected fatal dispositions. The defensive graphs claim no source origin.

Exact 37 source snapshots, three original verification-input snapshots, nine
complete logs, actual LLVM/metadata/headers, native command traces and outputs
are retained. Thirty native artifact records distinguish the two cohorts;
binary hashes bind the original external cache objects/executables and archived
reconstruction inputs. Independent readback verifies all hashes, command
outcomes and original cached binaries; native compiler/runtime stderr is empty.
The unchanged wrapper used a 3028 MiB adaptive limit, 1569 MiB reserve and zero
cgroup swap, with no limit hits or OOM events.

This establishes experimental source lowering and C transfer/lifetime
components. The C bridge's ordinary host stack and `longjmp` fault observer do
not establish a kernel provider's 4096-byte stack, register reclamation or
nonreturning fatal cleanup. Its context occupies at most 1280 bytes in these
fixtures inside a guarded 65536-byte temporary reservation; that observation
does not promote the earlier proposed 968-byte layout into a native certificate.
The actual provider/model, stack/debug/recovery binding, real owner integration
and machine/application qualification remain required. Stage 2 is unfinished.

## Stage 2 progress: emitted private stack boundary

The [target boundary](../src/topal-compiler/src/systems_tail_boundary.rs)
implements the [private-stack contract](design/tail-stack-boundary.md) as an
x86-64 object. The independent client supplies a guarded 4096-byte source stack
and 65528 temporary bytes; the instance recovery cell consumes the remaining
eight bytes of the existing temporary reserve. Source/helper code and test
hooks use integer registers only. Normal return restores the caller stack,
erases and reads back the entire source stack and recovery header, clears the
instance cell and volatile registers, then returns the private outcome.
Admitted abort and invalid machine outcomes erase the complete temporary extent
and source stack before the bound fatal observer. Persistent stores/history and
the eight-byte canary outside the owned temporary extent remain intact.

The [exact boundary record](tools/tail-traversal/results/stage-2-stack-boundary/qualification.json)
passes **330 compiler unit tests**, **261 CLI tests**, workspace all-target
checking, four coverage tests, formatting and source size. It retains 45 source
snapshots, three verification-input snapshots, nine complete logs and 48 native
artifact records. Each targeted/full-unit cohort independently executes 24
clients covering 480 cases, 297380 source-step entries, 297349 completed
checkpoints and 31 fatal dispositions. Those totals include the earlier C
components and explicitly marked defensive IR/invalid-outcome fixtures.

The boundary clients refuse 1392 malformed/busy pre-entry attempts and three
active-session reentries per cohort without changing source history. Two GDB
runs per cohort observe the actual source-stack pivot, all six restored
callee-saved registers, cleared volatile registers, caller backtraces and
normal/abort/invalid-outcome reclamation. Full Boolean and wide signed values
are compared at each successor as well as at completion. Independent readback
verifies archive hashes, original cached objects/executables and all command
outcomes. GDB emits only its recorded `.debug_names` compatibility warning;
native compiler/runtime stderr is empty.

The unchanged wrapper used a 2942 MiB adaptive limit, 1569 MiB host reserve and
zero cgroup swap. Peak usage was 1953615872 bytes (about 1863 MiB), with no limit
hits or OOM events. No memory limit or host security setting was relaxed.

This establishes a user-mode target component, with mapped writable memory and
the fixed integer-only calling convention as preconditions. It does not yet
bind a complete session model, kernel hooks/layout/code/context, between-step
failure cleanup or a real kernel owner. GDB samples do not establish every
instruction's unwind state; review identified abort-prologue CFI as a remaining
refinement. Stage 2 remains unfinished.

The [focused CFI follow-up](tools/tail-traversal/results/stage-2-stack-boundary-cfi/qualification.json)
corrects that refinement without changing ordinary profiles. It selects the
saved-stack CFA before the first pivot and marks overwritten abort registers
unavailable until recovery establishes the saved owner frame. GDB checks caller
register recovery at all 68 instructions of the admitted entry path and the
seven abort-prologue instructions, in addition to the existing reclamation
observations. All thirteen component tests and 24 native clients pass again,
with workspace all-target checking, formatting and source size. This focused
run retains its own 45 source snapshots, three verification snapshots, four
logs and 24 native artifact records; the preceding full regression record
retains its original inputs. No limit/OOM events occurred.

The [controller-boundary follow-up](tools/tail-traversal/results/stage-2-controller-boundary/qualification.json)
adds the missing between-step failure/cancellation entry. The owning controller
supplies its still-owned extents directly; inactive recovery and source-context
fields supply no authority. The emitted object cleans both complete extents on
the live controller stack and enters bound fatal handling. Eight malformed/
busy protocol cases reach fatal handling without erasing unproven addresses.
The independent observer's escape remains a test facility, not kernel recovery.

Five admitted cases cover invalid phase, incomplete successor staging,
cancellation with a poisoned recovery header, cancellation after checkpoint
retirement, and a missing terminal result. Exact owner bytes and frame/loan
history are checked after fatal cleanup; no completed store is rolled back.
GDB additionally compares the controller caller's saved registers/stack and
backtrace after reclamation. All **fourteen component tests**, **25 native
clients**, workspace all-target checking, four coverage tests, formatting and
source size pass. The cohort covers 485 cases, 297385 source entries, 297350
completed checkpoints, 36 admitted fatal dispositions, 1432 source-entry
refusals, four active reentry probes and three GDB runs. The eight invalid
controller calls are separate from admitted fatal dispositions.

This focused record retains 45 source snapshots, three verification snapshots,
five logs, all 25 native input/output/command records and binary identities.
Independent readback verifies every retained hash and cached binary. Its normal
3003 MiB adaptive limit, 1569 MiB reserve and zero swap needed no adjustment;
peak usage was 1039486976 bytes, with no limit/OOM events. The preceding full
330-unit/261-CLI record and focused CFI record retain their exact earlier inputs;
the new fourteen-test run does not relabel them as a full 331-unit regression.
Complete session/model and kernel hook/layout/context binding remain required
before a real owner invokes these components.

## Stage 2 progress: owned reference sessions

The [session contract](design/tail-session-model.md) now has an
[owned reference implementation](../src/topal-semantics/src/systems_tail_session.rs).
It exclusively borrows actual existing storage, copies complete scalar values
through disjoint banks and distinguishes source execution, returned steps,
checkpoint retirement, terminal completion and fatal disposition. Opaque value
references bind a session, epoch and issued root; physical reuse cannot revive
old bank/work/constant references or scoped region views. Helpers retain the
existing closed DAG and current-frame loan rules. Whole work/old-bank erasure
follows complete successor validation and exact natural descent.

[Allocation-owned history](../src/topal-semantics/src/systems_storage_tail.rs)
persists across sessions. Admission checks actual remaining frame/loan counters
before generation or history changes. Every issued helper/root identity is
retained. Cancellation, abandonment, incomplete reclamation and invalid admitted
protocol preserve performed stores/history, reclaim modeled buffers and exclude
ordinary owner access until terminal release. Normal byte bounds/rights/value
refusal remains None/false with no state or storage observation; arbitrary natural
arguments are checked without word truncation. Invalid view provenance takes
precedence over a normal refusal. Review corrected that distinction before the
final qualification run.

The [compiler adapter](../src/topal-compiler/src/systems_tail_model.rs) rebuilds
the actual source plan and carried bounds before constructing reference
constraints. It retains the helper graph, invariant region, classifiers/ranges,
measure/decrements, issuance budgets and work/bank layout. The actual
clear-prefix source binds an 80-byte bank and its precise helper scopes. This
adapter neither authenticates external IR as source nor binds executable hooks,
readonly code/data, actual kernel context or physical erasure.

The [exact session record](tools/tail-traversal/results/stage-2-owned-session/qualification.json)
passes all **253 semantics tests**, **333 compiler unit tests**, **261 CLI tests**,
workspace all-target checking, four coverage tests, formatting and source size.
Fourteen new semantics cases and two source/model cases are included in those
totals. They cover a 70000-step reference traversal over actual owner bytes,
complete wide signed/Boolean/Unit and alias transfers, 65-bit carried values,
nonunit progress, stale authority, source-return phases and fatal/cancel/drop
cleanup. An independent 2601-case counter oracle checks exact admission and
complete refusal-state preservation; separate tests inspect actual retained
contents and fatal owner exclusion before terminal erasure.

Each targeted/full-unit cohort independently repeats the **25 native clients**:
485 cases, 297385 source entries, 297350 completed checkpoints, 36 admitted fatal
dispositions, 1432 pre-entry refusals, four reentry probes and three GDB runs.
Eight invalid controller protocols remain separate. These native observations
and the new reference transitions have independent scopes; their presence in
one regression run does not establish model/native/kernel refinement.

Exact 57 source snapshots, three original verification snapshots, eleven logs
and 50 native artifact records are retained. Independent readback verifies all
archive/current-source hashes, metadata/LLVM identities, command outcomes and
original cached binaries. Native compiler/runtime stderr is empty; GDB retains
its recorded `.debug_names` warning. The unchanged wrapper used a normal adaptive
3058 MiB limit, 1569 MiB host reserve and zero swap. Peak usage was 2737250304
bytes, with no limit hits or OOM events. The ledger contains 316 stable rules.

Risk remains high around ownership, reference retirement and failure ordering.
Self-review follows the authority hierarchy, whole-state refusal/counter oracles,
direct owned-byte checks and complete regression evidence. No independent human
review or new kernel/VM qualification is claimed. Stage 2 still requires the
provider-owned kernel-safe hooks, complete actual context/code/data/layout and
stack costs, interrupt/context binding and joined physical evidence. Real kernel
allocator/metadata integration remains Stage 3.

## Stage 2 progress: provider-owned source hooks

The [hook component](../src/topal-compiler/src/systems_tail_runtime.rs) now emits
[freestanding provider code](design/tail-provider-hooks.md) for context checks,
aligned allocation provenance, complete numeric transfer, private inputs/results
and scoped helper calls. Native-mode tests link those actual LLVM objects;
their C source-hook implementations are excluded. Independent C code initializes
test facts, drives calls/checkpoints and observes complete values/owner effects.

The actual clear-prefix context occupies 1248 bytes, including control, all
helper descriptors, both banks, source work, allocation-start bitmap, result,
budgets and padding. Its recovery header and instance cell bring the minimum
temporary cost to 1320 bytes. The reader needs 1352 bytes. Neither number is the
earlier proposed 968-byte layout. All fit the existing temporary reserve. The
40-byte persistent history lives outside the erasable extent and is independently
guarded in native tests. Production hooks contain no prototype fault counters.

The runtime-specific target boundary pins the admitted caller's history before
source entry. Context validation compares against that independent record before
dereferencing history. Admitted abort/invalid outcome marks the actual pinned
domain inactive/fatal before full temporary/source-stack reclamation. Older
boundary components preserve their contracts. Current-view checks and descriptor
reuse also rely on the closed source graph's no-escape/lifetime obligations;
pointer equality alone does not establish fresh authority.

[Actual object inspection](../src/topal-compiler/src/systems_tail_runtime_inspection.rs)
checks exact runtime imports and complete source-hook linkage, function-section
call relocations, stack metadata and acyclic reached call paths. Disassembly
inspection rejects implicit host-service, SIMD/FPU, privileged and indirect-call
dependencies. Across these compiled programs, source/hook stack paths use
408–504 bytes, including call return words, under the unchanged 4096-byte reserve.
This is reached source-stack evidence; owning-controller/kernel context and
complete invocation retirement remain separate obligations.

The [exact provider record](tools/tail-traversal/results/stage-2-provider-hooks/qualification.json)
passes all **253 semantics tests**, **337 compiler unit tests**, **261 CLI tests**,
workspace all-target checking, four coverage tests, formatting and source size.
The eighteen component tests include four new provider cases. Each targeted/
full-unit cohort executes **43 native clients**: 657 cases, 443423 source entries,
443375 checkpoints and 50 admitted fatal dispositions, with 2808 geometry/busy
refusals, four earlier C-mode reentry probes and fourteen GDB runs. Repeated
cohorts retain separate records rather than counting as distinct workloads.

Eighteen clients per cohort use actual provider hooks. Eight exercise actual
clear/read, signed, Boolean, Unit, wide readonly and aliased source values. A
controller fixture and invalid-outcome fixture cover recovery. Eight explicitly
defensive LLVM fixtures test forged history/numeric pointers, frame/loan budget
exhaustion, oversized allocation, duplicate staging, interior-work and wrong-bank
numeric roots. These fixtures claim no source correspondence. Complete owner
bytes and exact consumed history distinguish failures before and after a store;
actual history, private/source stack, canaries and caller state are observed.

Exact 66 source snapshots, three original verification snapshots, eleven logs
and 86 native artifact records are retained. Native records additionally retain
runtime LLVM/metadata, object closure/stack observations, disassembly and command
traces. Independent readback verifies original code/layout/command/binary hashes
and path arithmetic. Compiler/runtime stderr is empty; GDB records only its
`.debug_names` compatibility warning. The normal adaptive limit was 3025 MiB,
with 1569 MiB reserve and zero swap; peak usage was 1945735168 bytes, with no
limit/OOM events. The ledger contains 317 stable rules.

This advanced actual source hooks and their target recovery. At that checkpoint,
test initialization was not kernel admission, and checkpoint/completion still
used independent C. The next obligation was provider-owned controller return
bodies and complete context/owner/final-retirement binding under the same
reserves. Broader
argument/target ABI qualification, real allocator/metadata integration and exact
kernel/VM/application qualification remain required. Stage 2 is unfinished.

## Stage 2 progress: native checkpoint retirement

The [native checkpoint](design/tail-native-checkpoint.md) now validates and
retires actual returned source state. Its [LLVM body](../src/topal-compiler/src/runtime/kernel_tail_checkpoint.ll)
runs through a [distinct target entry](../src/topal-compiler/src/systems_tail_checkpoint_boundary.rs)
on the same private stack used by the completed source call. The caller supplies
the actual history independently of mutable context. Source and checkpoint role
checks stay distinct. Complete scalar/result representations, retired helpers,
root scope, exact natural decrement and per-step/invocation budgets are checked
before erasure or fresh identity publication. Whole work/provenance and old bank
are retired; terminal paths clear both banks and retain the actual result.

Review found a late-cleanup defect: header erasure could destroy the history pin
before an erasure-check failure used it. Returned source and checkpoint bodies
now leave that pin in a preserved register through target cleanup; a live-body
abort still uses its independently armed header. Two explicit target fixtures
invert the final header-readback branch after successful erasure. Both reach
nonreturning fatal handling with the original history poisoned, complete private
domains erased and exact earlier stores/issuance preserved. These are machine
failure fixtures, not altered-source correspondence claims.

The native controller fixture covers 26 admitted invalid-state/cancellation
cases. Added cases check forged history before dereference, invalid/noncanonical
numeric roots, wrong descent, retired helpers, global/per-step frame and loan
exhaustion, damaged terminal state and invalid outcomes. Fourteen checkpoint
geometry/busy/history refusals per native case preserve the entire mapped
private extent, stack, history and active cell. Positive programs also cover
nonunit descent, unreachable decrements above the native word range and a
seven-argument helper whose actual values and issuance are checked. Zero source
allocation still reserves an explicitly costed eight-byte physical work granule.

Actual object/import/call-path and source/runtime instruction checks pass.
Source paths use **424–584 bytes**, while the checkpoint path uses **312 bytes**;
the two roles use the unchanged **4096-byte stack sequentially**. Actual contexts
range from 400 to 3056 bytes, with minimum temporary costs of 472 to 3128 bytes
including recovery metadata/cell. Clear-prefix remains 1248/1320 bytes. GDB
checks restored caller registers, complete retired stack/header, role-specific
backtraces, 89 checkpoint-entry instructions, 77 runtime source-entry
instructions, the earlier 68-instruction entry and seven abort instructions.

The [final exact record](tools/tail-traversal/results/stage-2-native-checkpoint/qualification.json)
passes all **253 semantics**, **340 compiler unit** and **261 CLI integration
tests**, plus locked all-target checking, four coverage tests, formatting and
source size. The 21 component tests execute **48 native clients per targeted/
full-unit cohort**: 737 cases, 444583 source entries, 444512 completed checkpoint
returns, 73 admitted fatal dispositions, 6976 admission refusals, four earlier
C-mode reentry probes and seventeen GDB runs. Twenty-three clients use native
provider hooks/checkpoints, including fourteen GDB runs. The separate controller
fixtures contribute 31 admitted cases and sixteen invalid protocols in the
combined corpus; the earlier C fixture accounts for five/eight of those counts.

The record retains 69 source snapshots, three original verification inputs,
eleven command logs and 96 native artifact records across the two cohorts.
Independent readback verifies current/archive/source/runtime/boundary/command/
binary identities, complete layout costs and both native call-path bounds.
Compiler/runtime stderr is empty; only 34 known GDB `.debug_names` warnings
remain. The unchanged wrapper used a normal adaptive **3105 MiB** limit, **1569
MiB** host reserve and **zero swap**; peak usage was **1900933120 bytes**, with
no limit/OOM events. The ledger contains **318 stable rules**. The
[preliminary record](tools/tail-traversal/results/stage-2-native-checkpoint-preliminary/scope.json)
retains the earlier traceability failure and pre-fix implementation separately.

Risk remains high around caller preservation, retirement and failure ordering.
Self-review checked the authority hierarchy, complete value/history/byte oracles,
actual object/stack/CFI observations and the newly exercised late failure paths.
No independent human review or new kernel execution is claimed. Owning-caller
admission, normal whole-invocation/result/context reclamation, interrupt/context
binding and actual kernel owner refinement remain required before publication.
Real allocator/metadata integration remains Stage 3; Stage 2 is unfinished.

## Stage 2 progress: native invocation completion

The [completion body](design/tail-native-completion.md) now copies the actual
terminal scalar/Unit result through a distinct role on the same private stack.
It validates context/history, root and helper scope, step bounds and full bank/
work/provenance retirement, then validates the complete result and copies/reads
back its 56-byte private representation into an independently owned caller frame.
That frame has an explicit cost in runtime metadata; it belongs to the still-live
caller and grants no additional invocation temporary storage.

The [target completion boundary](../src/topal-compiler/src/systems_tail_completion_boundary.rs)
then restores the caller stack, erases/reads back the entire private stack and
temporary extent, revokes active recovery, and only then clears history activity.
Persistent frame/loan/generation history and owner stores survive. The C native
test finalizer now calls this actual provider implementation; it verifies the
returned value, whole caller/owner/history state and physical cleanup independently.

Completion pins its output and actual history through live-body abort and late
target cleanup. Fatal recovery erases even a partially copied output, attempts
every private domain and never returns to the owner. Fourteen new completion
fixtures cover damaged phase/history/banks/work/provenance/result/helper/step
facts, dirty output, partial copy, late cleanup, invalid machine return and a
source hook called under the wrong role. Deliberately changed LLVM/object paths
are defensive implementation fixtures, not source-correspondence grants.
Twenty-three completion geometry/busy/output/history cases preserve every
private byte, stack, output, history and active cell before entry; normal cases
also alternate history/output allocation order.

Object inspection now bounds all three sequential private-stack roots. Source
paths remain 424–584 bytes, checkpoint uses 312 bytes and completion uses
**184–248 bytes**, under the unchanged 4096-byte reserve. Actual invocation
context/minimum temporary costs remain unchanged. The caller's separate 56-byte
output and complete live caller frame/stack still require kernel accounting.
GDB walks all **107 completion-entry instructions**, as well as the prior entry
and abort paths, and observes exact role/caller registers, stack, full temporary
erasure and normal/fatal history disposition.

The [exact completion record](tools/tail-traversal/results/stage-2-native-completion/qualification.json)
passes all **253 semantics**, **341 compiler unit**, **261 CLI** and four coverage
tests, locked workspace all-target checking, formatting and source size. Each
targeted/full-unit cohort runs **62 native clients**, including 37 actual-provider
clients: 751 cases, 444653 source calls, 444582 completed checkpoint returns,
**194 normal completions**, 87 admitted fatal dispositions, 13402 pre-entry
refusals, four earlier C-mode reentry probes and 31 GDB runs. The two cohorts
retain separate exact records; repetitions are not distinct workloads.

The record retains 72 source snapshots, three original verification inputs,
eleven complete command logs and 124 native artifact records. Independent
readback verifies all source/archive/LLVM/runtime/target/command/binary identities,
the explicit output cost and all three native call-path bounds. Only 62 known
GDB `.debug_names` warnings remain; native compiler/runtime stderr is empty.
The unchanged wrapper used a **3064 MiB** adaptive limit, **1569 MiB** host
reserve and **zero swap**. Peak usage was **1895260160 bytes**, with no limit/OOM
events. The systems ledger contains **319 stable rules**.

Risk remains high at result transfer, private lifetime boundaries and fatal
ordering. Self-review checked the authority hierarchy, whole value/history/byte
oracles, exact objects and independent native/debug observations. No independent
human review or new kernel/VM execution is claimed. This completes the native
completion mechanism; actual kernel admission, interrupt context, total caller
resource/retirement bounds and owner refinement still precede Stage 2 completion
and publication. The existing root/namespace controllers supply the ownership and
caller-retirement patterns to refine; their current executable profiles remain
unchanged.

## Stage 2 progress: provider admission and invocation control

The [owning invocation controller](design/tail-owner-admission.md) now constructs
actual source context and executes all three native roles. Its first interface
accepts one independently owned nonempty read-write byte extent, three immutable
word-bounded Nat facts, actual persistent history and disjoint private/stack/
output domains. The [builder](../src/topal-compiler/src/systems_tail_owner.rs)
rebuilds source/range/layout constraints and rejects other caller signatures
without weakening existing component profiles.

[Actual provider code](../src/topal-compiler/src/runtime/kernel_tail_owner.ll)
checks complete geometry, idle recovery, zero private domains, canonical inactive
nonfatal history, generation overflow and whole-invocation identity budgets before
any mutation. It returns refusal without changing bytes, counters or generation.
A successful first invocation establishes counters two and generation one;
reuse retains actual next counters and advances actual generation. The root
frame/loan numbered one belongs to the new generation, matching the reference
model. Context, current view and canonical initial banks are built from actual
facts and selected code. C test code no longer constructs those fields for this
controller path.

The controller accepts only expected source/checkpoint/completion outcomes.
Admitted protocol failure uses its independently retained history/output facts,
retains earlier stores/issuance, erases output and enters trusted private cleanup
and nonreturning fatal handling. The source Boolean remains its actual result;
the kernel caller must still validate its own postcondition before resumption.
Geometry establishes neither mapped ownership nor write permission, and caller
word extraction must prove the whole semantic value fits without truncation.

The [independent guarded client](labs/qemu/x86_64/tail_owner_client.c) checks 91
repeated invocations, including a 70000-step traversal, with one continuing actual
history. All owner bytes, results, generation/issuance deltas, immutable protected
inputs, retired domains and canaries are checked. Another 2601 finite counter
fixtures independently cover budget exhaustion. Forty-nine null/wrap/alias/cell/
empty-owner cases and twelve malformed-state, dirty-buffer, maximum-count and
busy cases check whole refusal preservation. The normal client completes **501
invocations** and records **2252 refusals**. Two explicit source/controller machine
fixtures each observe one admitted fatal disposition after the first store, with
exact performed bytes/history preserved and private/output/stack domains erased.
These altered-code fixtures claim no source correspondence.

[Object inspection](../src/topal-compiler/src/systems_tail_owner_inspection.rs)
checks six exact imports, complete function stack metadata and acyclic calls.
The final controller uses **240 caller-stack bytes**; the weighted normal
controller/boundary/inner-body path uses **688 bytes**, within the existing limit.
This excludes the actual kernel caller and fatal handler, whose complete bounds
and physical frame/spill retirement still need proof. Existing 62 per-step native
clients and their guarded/GDB evidence remain separately retained.

The [final admission record](tools/tail-traversal/results/stage-2-owner-admission/qualification.json)
passes all **253 semantics**, **343 compiler unit**, **261 CLI** and four coverage
tests, locked all-target checking, formatting and source size. It retains 78
source snapshots, three original verification inputs and twelve command logs.
The two targeted/full-unit controller cohorts retain six actual controller
artifact records, alongside 124 existing step-component records. Independent
readback checks every source/archive/object/LLVM/command/binary identity, all
three inner role bounds, controller path arithmetic and exact per-client results.
Controller compiler/runtime stderr is empty. The separate step corpus retains
its 62 known GDB `.debug_names` warnings across both cohorts.

The unchanged wrapper used a **3075 MiB** adaptive limit, **1569 MiB** reserve and
**zero swap**. Peak usage was **2259890176 bytes**, with no limit/OOM events.
The systems ledger contains **320 stable rules**. The
[preliminary compile failure](tools/tail-traversal/results/stage-2-owner-admission-before-stack-type/scope.json)
is retained separately; the final stack checker has explicit checked word types.

Risk remains high at admission ordering, permissions, persistent identity and
failure recovery. Self-review checked the authority hierarchy, whole refusal/
counter/byte oracles and actual native object evidence; no independent human
review is claimed. This implements the controller mechanism. It does not qualify
privileged kernel context or establish actual caller ownership, total caller/
fatal-handler bounds, physical caller retirement or executable publication.
The next integration must bind the real typed kernel root callsite and its VM
evidence before Stage 2 is complete. General allocator/metadata work remains
Stage 3, with committed kernel/application qualification in Stage 4.

## Stage 2 progress: privileged kernel root experiment

The explicit systems-only `borrowed-byte-region-tail-action-v1` interface now
connects an actual typed 64-byte plain bootstrap owner to the three native roles
through a [kernel adapter](../src/topal-compiler/src/systems_tail_kernel.rs).
The [contract](design/tail-kernel-root.md) preserves ordinary source values,
termination, effects and ownership. The caller supplies whole Nat facts
`(64, 0, 1)` and requires the actual Boolean true before resuming. Opaque effects
remain unknown in the model, with an explicit runtime byte/result guard.

The adapter checks actual CPL0, masked interrupts, aligned root stack, bootstrap
storage extent and owner-offset key. Its independent recovery record survives an
interrupted LLVM controller. Normal return checks inner retirement, erases the
result and controller caller area, restores all six original callee registers,
erases their popped spill copies and revokes recovery. Source/result failure
preserves performed stores and issuance, poisons admitted history and retires
all owned invocation domains before halting without owner resumption. A refusal
before admission preserves prior history. Final root retirement erases history.

The actual source/checkpoint/completion paths use **488/312/184 bytes**. The
controller's own path remains **240 bytes**, its joined normal path **688**.
Including the outer return word, saved registers and argument/alignment area
gives **824 bytes**, below 4096. The erased controller caller area is **320 bytes**,
plus **48 bytes** of popped original-register copies. History/output/recovery/input
costs remain explicit at **40/56/32/24 bytes**, alongside the existing private
65528-byte extent, eight-byte active cell and sequential 4096-byte source stack.

Actual kernel stack placement revealed a debugger issue absent from the earlier
native process layout: the high private stack unwinds to a lower kernel stack.
Tail boundaries now emit the GNU discontinuous-stack CIE augmentation; ordinary
bridge CIE bytes remain unchanged. GDB presents these frames as signal
trampolines. Native tests verify their actual PC/symbol and recovered caller.
Kernel observations walk 90 entry instructions, 114 normal-return instructions
and 133 fatal-recovery instructions. Repeated string instructions are checked at
both ends with separate whole-domain readback. One full-unit run reported the
same PC twice during a step; its 115 debugger stops cover the same 114-instruction
return path. The independent reader checks identical consecutive-PC-normalized
sequences and identical kernel adapter objects across both cohorts. The first
reader's stricter stop-count assumption is [retained](tools/tail-traversal/results/stage-2-kernel-experiment/independent-readback-history/scope.json).

The [typed fixture](labs/qemu/x86_64/tail-kernel-root.t) preserves the full current
boot/handler shape, with other optional source-library slots unselected. It runs
the clean-room static-ELF ABI fixture and completes the root continuation and final
fatal disposition. The [independent observer](labs/qemu/x86_64/tail_kernel_inspect.py)
uses the existing lab QEMU/firmware/device configuration and checks ten cases:

- normal: 65 source entries, 65 checkpoints, one completion, preserved caller,
  whole owner clearing and final history erasure;
- source abort after the real first store: exact history `4/4/1/0/1`, preserved
  store, whole cleanup and no owner resumption;
- false result: full traversal/completion, then history `194/194/1/0/1`, cleanup
  and no owner resumption;
- invalid key, busy active cell, enabled interrupts and invalid stack: refusal
  before source execution, preserving the prior owner/history;
- generation exhaustion, frame exhaustion and dirty private storage: refusal
  before source execution, preserving prior history and reclaiming adapter-owned
  domains without resuming the owner.

Mutated register/state cases are defensive machine fixtures and make no claim
that those mutations correspond to the checked root source. The normal path
uses the exact source and actual owning caller. Its history is
`194/194/1/0/0` before finalization and entirely zero afterward.

The [exact experiment record](tools/tail-traversal/results/stage-2-kernel-experiment/qualification.json)
passes all **636 frontend**, **253 semantics**, **345 compiler unit**, **261 CLI**
and four coverage tests, locked all-target checking, workspace/included-file
formatting and source size. Fourteen command logs, 96 source snapshots and three
original verification snapshots are retained. Targeted/full-unit cohorts each
execute ten VMs; two kernel artifact sets retain exact LLVM/native/input/command/
observation evidence. Existing native regressions retain 124 step-component and
six controller artifact sets with unchanged case counts. Independent readback
checks current/archive/source/object/LLVM/command/binary identities and all VM
cases. Only known GDB `.debug_names` warnings remain.

The unchanged wrapper used **2847 MiB**, a **1569 MiB** host reserve and **zero
swap**. Peak usage was **2625748992 bytes**, with no limit/OOM events. The systems
ledger now contains **321 stable rules**. A development run exhausted `/tmp` disk
quota while saving images; its artifacts were preserved in the repository cache,
and final validation used disk-backed scratch without raising memory limits.

Risk is high for kernel context, recovery, identity and physical retirement.
Self-review and independent machine observations are recorded; independent human
review is still required through the existing PR. This is an unpublished kernel
component experiment. The public artifact publisher deliberately refuses the
new interface until complete source/module/native/caller/ownership/context/boot
binding is implemented. The test-only boot helper produces no provenance grant.
Canonical executable publication, Stage 2 completion, general allocation and
broader userspace/container/VM-host acceptance remain unfinished.

## Stage 2 complete: reconstructed kernel publication

The [source-bearing publisher](../src/topal-compiler/src/systems_tail_publication.rs)
now retains and reparses the actual checked root and exact selected ordinary
source/module closure. The normal compiler pipeline supplies this source. The
program-only API still refuses the tail interface. Canonical publication and boot
packaging rebuild the source plan, value/storage bounds, shared ordinary LLVM,
native source/runtime/controller and target adapters; supplied native hashes,
plans, addresses and bounds never replace reconstruction.

The [linked comparison](../src/topal-compiler/src/systems_tail_linked.rs)
replays fresh ELF relocations and compares every allocated contribution of all
seven objects. It covers anonymous constants, code, writable and zero storage,
symbol ownership, disjoint endpoints, alignment and actual load-segment bytes
and permissions. The actual ELF entry must select the reconstructed bootstrap.
Local symbols resolve within their own input; external definitions must be
unique. Linker processing flags may aggregate in shared output sections, while
exact input bytes and mapped permissions remain checked. The
[binding design](design/tail-publication-binding.md) records the closed static
LLVM options, unreachable hosted-runtime removal and packaging-only toolchain
dependency. Legacy publication profiles retain their existing paths.

The canonical path's measured controller owning-stack cost is **248 bytes**;
its weighted normal path remains **688**, or **824** with the outer allowance.
It erases **328 controller bytes plus 48 saved-register bytes**. These fresh
measurements differ from the earlier experimental object's 240/320-byte facts;
the regenerated adapter uses the actual new bounds. Source/checkpoint/completion
paths remain **488/312/184 bytes** for the selected clearer. The checked owner
is still exactly **64 bytes**, with facts `(64, 0, 1)` and an actual true-result
gate. Neither this caller nor the frontend's existing Count annotation qualifies
general larger allocator extents.

Four new tests cover complete reconstruction and changed code/constants, five
invalid source/root selections, canonical publication/boot and ten privileged
VM modes, and the normal compiler pipeline with all nine other real source slots
retained. That full-kernel test changes only the tail selection and its observed
byte guard in a test copy; the committed production root does not yet select it.
Fourteen provenance mutations, a present-null binding, an independently rebuilt
changed source and five altered ELF cases are refused. The ELF cases recompute
the outer kernel digest before checking source code, anonymous constants, entry,
load-offset projection and initialized bytes shadowing BSS.

The [final record](tools/tail-traversal/results/stage-2-publication-binding/qualification.json)
passes **636 frontend**, **253 semantics**, **349 compiler unit**, **261 CLI**
and four coverage tests, locked workspace all-target checking, workspace/included
formatting and the unchanged size ceiling. Fifteen command logs, 140 source and
three original verification snapshots are retained. Targeted/full-unit cohorts
produce eight reconstruction fixtures, including four canonical artifacts:
89 complete input contributions for the isolated root and 105 with all selected
libraries. They execute **22 canonical-path VM component observations**. The
separate raw experiment contributes **20** more observations. The record also
retains 124 native step and six controller artifact sets with their exact scope.

The independent reader verifies source/archive/tool/object/command identities,
all native and VM observations, and independently parses ELF and replays every
fresh input relocation without using the compiler's comparison implementation.
Canonical VM stderr is empty; only the existing raw/native GDB `.debug_names`
warnings remain. The unchanged wrapper used **2861 MiB**, a **1569 MiB** host
reserve, **zero validation swap**, and a peak of **2189172736 bytes**. Limit/OOM
counters stayed zero. The systems ledger contains **322 stable rules**.

The earlier experimental description incorrectly called the static-ELF fixture
BusyBox. These component tests use `qualified_x86_64_static_user_image_fixture`,
whose clean-room payload is an ABI fixture rather than an application. The
[bounded factual correction](tools/tail-traversal/results/abi-fixture-scope-correction/correction.json)
retains the original and corrected design hashes and exact fixture definition.
Original experiment and final-run snapshots/results are unchanged; 139 final
source inputs remain byte-identical and the one design-label correction is
verified as an exact text replacement. No real BusyBox or application pass is
claimed by these VM observations.

Risk remains high for arbitrary-precision values, identities, ownership,
physical retirement, native provenance and serialized bindings. Self-review
follows authority order and compares independent numeric/state oracles, native
clients, privileged GDB observations and the separate ELF readback. Independent
human review remains required through the existing initial Step 4 PR. This is
the Stage 2 commit; the complete Linux/distribution acceptance remains unfinished.

## Stage 3 progress: actual source extent facts

Preparation for larger allocator metadata found a conformance defect: tail
source and its isolated module checks inherited the legacy 65536-byte Count
annotation, although the separately qualified native tail family accepts a
word-sized extent. This unsupported bound could justify a Nat result for
`65536 - Count` on an actual larger owner. The tail selector now retains ordinary
Nat count meaning without that finite source bound. Its isolated module entry
uses the same meaning; native resource analysis independently retains its actual
x86-64 word-domain proof. Legacy selectors retain their existing bound and
qualification. No core type, arithmetic or termination rule changes.

Three [frontend regressions](../src/topal-language/src/compiler_model/kernel_region_count_tests.rs)
cover primary/imported/isolated Count facts, legacy controls, unproved Nat
narrowing, valid Int subtraction and private/initialized module refusal. The
[native count fixture](labs/qemu/x86_64/tail_count_client.c) executes direct and
imported ordinary helpers against guarded actual process-owned extents of 1,
65535, 65536, 65537, 70032, 131073 and 262144 bytes. Each variant executes 56
cases with correct and incorrect expected counts, persistent history, immutable
inputs, unchanged owner bytes, complete private/stack erasure and canaries.
This establishes native Count observation, not physical-frame or privileged
kernel allocator ownership. The kernel caller remains qualified for 64 bytes.

The [final prerequisite record](tools/tail-traversal/results/stage-3-count-inputs/qualification.json)
passes **639 frontend**, **253 semantics**, **350 compiler unit**, **261 CLI**
and four coverage tests, checking, formatting and size. Ten command logs and
145 exact source snapshots are retained. Targeted/full-compiler cohorts
each execute both Count variants, with 112 cases per cohort. Existing regressions
retain 62 step, three controller, one raw-kernel and eight reconstruction fixture
sets, including the four Count clients and two canonical artifacts. Eleven
canonical and ten raw VM component observations use the static-ELF ABI fixture;
they make no new application acceptance claim. Independent source/log/native
readback and separate canonical ELF relocation/load replay pass.

The unchanged wrapper used a **3064 MiB** limit, **1569 MiB** host reserve and
zero validation swap. Peak usage was **733036544 bytes**, with no limit/OOM
events. A [preliminary failed run](tools/tail-traversal/results/stage-3-count-before-unique-scratch/scope.json)
retains a concurrent test-directory collision: its compiler command passed 349
tests and failed one directory creation. The shared fixture now adds an atomic
per-process sequence to its clock-derived name; the final full suite passes.
The failed run and its refusal by the readback remain unchanged.

Risk is high because unsupported range evidence can affect source typing and
native resource analysis. Self-review checks the tail-only selection, module
isolation, unchanged legacy path, negative narrowing and actual native extent
observations. This is a Stage 3 prerequisite; the actual allocator policy,
metadata ownership and kernel integration still require implementation before
the planned Stage 3 commit. No separate increment PR is created for this part.

An earlier command/readback pass was not accepted as complete frozen-source
qualification: the audit found two changed included analyzer files missing from its
[source inventory](tools/tail-traversal/results/stage-3-count-before-complete-input-inventory/scope.json).
A fresh qualification passes with all 145 source inputs retained before
execution. Its producer
derives changed source/specification/design paths before freezing, including
new files. The original passing commands and their incomplete inventory remain
unchanged. The final reader verifies every retained source byte and the complete
modified/new source/specification/design inventory against this checkpoint.

## Stage 3 progress: natural metadata-query results

The separate [natural-result component](design/tail-metadata-query.md) supplies
ordinary `(KernelByteRegion, Nat, Nat, Nat) -> Nat` source selection and an owned
native constructor. Its first Nat remains the checked countdown. The Boolean
constructor/root interface retains its exact profile and true-result gate.
Neither a natural result nor the component creates an allocator, address or
physical-frame grant. Kernel owning-caller/context/cost and publication binding
remain separate work.

The initial [byte query](kernel/algorithms/find-zero.t) preserves the countdown
at the first zero. A caller with a validated request derives its position by
subtracting that fact from the original requested end. Zero denotes no match;
the all-ones word marks an invalid interval. Typed natural constants preserve
the ordinary branch classifier. A closed helper DAG and sticky word-bounded
result fit the existing reset invariant, while the cursor has its own affine
potential. This is a byte-traversal mechanism, not aligned contiguous allocation
or provenance policy.

The [guarded native clients](labs/qemu/x86_64/tail_nat_owner_client.c) check eight
search/invalid-interval cases over 70032 actual process-owned bytes, twenty
arithmetic cases including results wider than 64 bits, and five 256-bit readonly
constant transfers through actual checkpoints. All 48 result bytes, canonical
sign/length/limbs, unused zero padding, immutable inputs, unchanged owner data,
nonwrapping history and complete private/stack retirement are checked. The
result uses the existing numeric output kind and complete 56-byte frame, with
no word truncation. Constructor tests reject Boolean/Nat confusion and a measure
in the wrong input slot. A frontend regression checks the exact symbolic Nat
signature, publication/feature shape and recursive evidence.

The [exact query record](tools/tail-traversal/results/stage-3-natural-query/qualification.json)
passes **640 frontend**, **253 semantics**, **352 compiler unit**, **261 CLI**
and four coverage tests, workspace checking, workspace/included formatting and
the unchanged source-size ceiling. It retains 149 frozen source snapshots,
three original verification snapshots and eleven complete command logs. The
changed/new code and normative inventory was audited before final acceptance.
Targeted/full-compiler cohorts each execute all three native programs, with
33 cases per cohort and six exact natural-component artifact sets. Their source
paths use **600/456/456 stack bytes**; controller/weighted paths use **240/800**
for the byte query and **240/656** for arithmetic/constants. These do not include
a future kernel adapter's owning frame and retirement cost.

Independent readback verifies every source/native/runtime/command/binary identity,
all complete-value observations and constructor/profile metadata. Existing
regressions retain four Count clients, 62 step, three controller, one raw-kernel
and eight reconstruction fixture sets, including the two canonical artifacts.
Their eleven canonical and ten raw VM observations retain static-ELF ABI scope;
the separate ELF relocation/load comparison passes. Only the existing native/raw
GDB `.debug_names` warnings remain; natural-component tool/runtime stderr is empty.

The unchanged wrapper used **3107 MiB**, a **1569 MiB** host reserve and zero
validation swap. Peak usage was **3077210112 bytes**, below the actual limit,
with zero limit/OOM events. The systems ledger contains **323 stable rules**.
Factual report/trace updates follow the run; frozen inputs remain unchanged.

Risk is high for source result identity, arbitrary-precision representation,
borrowed metadata and physical retirement. Self-review compares exact source
classifiers, separate result constructors, the unchanged Boolean path, guarded
native value/owner oracles and independent artifact readback. No independent
human review or new kernel allocator authority is claimed. Actual allocation
policy, metadata ownership/kernel callers and complete committed application
qualification remain required before the planned Stage 3/4 commits.

## Stage 3 ordinary frame-policy component

The [frame metadata design](design/frame-metadata-policy.md) now has actual
[ordinary Topal source](kernel/memory/frame-metadata.t). Its owned table records
allocation identity, provenance and physical frame number for each cell. Source
chooses an aligned contiguous free span, marks the selected extent, validates a
whole release before clearing cells, and preserves partial writes on protocol
failure. The separate query prefix carries progress and request facts; it is
not source temporary-arena storage or an authority constructor.

The first expanded source exceeded the existing function budget. Checked helper
sharing now retains equivalent same-declaration bodies, their complete ordered
calls/arguments, source/module identity and the recursive entry/self edge. It
removes only analysis-only range annotations, checks original and retained
helper effects/closure, and keeps the existing 64-function limit. Three new
sharing regressions cover small-closure identity, foreign/capture/unresolved/
empty-effect/helper-cycle refusal, declaration separation and budget refusal.
The allocator qualifies with **32 checked functions**, without changing core
language meaning or legacy acyclic profiles.

The [final component record](tools/tail-traversal/results/stage-3-frame-policy/qualification.json)
passes all twelve commands: **643 frontend**, **253 semantics**, **353 compiler
unit**, **261 CLI** and **four coverage tests**, targeted Count/Nat/policy tests,
locked workspace all-target checking, workspace/included formatting and the
unchanged source-size ceiling. It retains **158 exact source snapshots**, three
original verification snapshots, complete command logs and two new native
policy artifact sets. The original verification snapshot retains the trace row
as pending; this factual report and the trace update follow the completed run.

The [independent C oracle](labs/qemu/x86_64/frame_policy_client.c) checks every
possible starting position and the complete candidate interval, separately from
the source's incremental state machine. Each targeted/full-unit cohort executes
**137 cases**: 54 successful allocations, 49 releases, 23 exhausted requests and
11 protocol failures. It compares the complete owner outside the explicit query
control fields, including immutable requests/topology, reservations and unused
bytes. Cases cover alignment, physical gaps and provenance boundaries, multiple
live extents, fragmentation, reuse, malformed/duplicate releases and retained
partial allocation commits. A 3000-cell, **72128-byte** owner exercises selection,
stores and release beyond the old 65536-byte extent. Guard pages, immutable
input facts, complete natural output, monotone history and whole private/stack
erasure remain checked.

The policy's source resource proof bounds intermediates at **69 bits**, one
recursive step at **18200 bytes** and private temporary use at **18592 bytes**.
The runtime requires **20088 bytes** including recovery overhead. Independent
native call-path replay checks a **1368-byte** source path and **1568-byte**
weighted source/controller path within the unchanged 4096-byte private stack;
the raw owner path is 240 bytes. These costs exclude a future actual kernel
allocator caller and its retirement.

[Independent readback](tools/tail-traversal/results/stage-3-frame-policy/independent-readback.json)
passes for all current/frozen source identities, native binaries/LLVM/commands,
both policy cohorts and the weighted call-path calculation. It also retains the
Count/Nat regressions, 62 step components, three owner components, one raw kernel
and eight reconstruction sets. The eleven canonical and ten raw VM observations
retain **static-ELF ABI fixture** scope; they add no real BusyBox/application
acceptance. Separate ELF relocation/load replay still checks 89/105 allocated
input contributions for the canonical regression artifacts.

The unchanged wrapper used **3120 MiB**, a **1569 MiB** host reserve and zero
validation swap. Peak usage was **2881191936 bytes**, with zero limit/OOM events.
The systems ledger now has **324 stable rules**. The
[preceding failed run](tools/tail-traversal/results/stage-3-frame-policy-before-format/qualification.json)
remains retained as failed: functional suites passed, but included formatting
found one Rust line wrap. The final run uses fresh source snapshots after that
formatting repair and a clarification of completed-phase prefix reads; it does
not relabel the earlier run.

Risk is high around checked-body sharing, arbitrary-precision bounds, metadata
mutation, extent identity and failure preservation. Self-review follows the
authority hierarchy, checks that only the diagnostic entry-self graph edge is
ignored, preserves every source argument/effect, and compares independent whole
metadata and native call-path oracles. No independent human review is claimed.
The actual privileged allocator, normalized map/reservations, physical zeroing,
opaque extent publication, dynamic workload requests and complete caller/context
retirement remain required. This is concrete policy implementation, not Stage 3
kernel integration or the planned Stage 4 application qualification.

## Stage 3 kernel allocator source binding

The [kernel binding design](design/frame-policy-kernel-binding.md) now has an
explicit `frame-allocator-library` root slot with
`physical-frame-policy-tail-v1`. The source checker and reference model retain
its exact interface/reserve obligation and require one described-memory
allocator creation. Removing the optional declaration leaves the legacy checked
operations unchanged. Wrong interfaces/slots, duplicates and a persistent-state
reserve masquerading as the temporary reserve are refused.

The [source binding checker](../src/topal-compiler/src/systems_frame_policy_binding.rs)
reconstructs the actual root, validates contained primary/module inputs, checks
the ordinary Nat interface/progress and isolates imported declarations before
building the existing standalone component. Its source obligation retains the
complete root/library snapshots and component identity. Independently changed
valid roots change the root identity while preserving the policy component;
changed valid policy bodies change the component identity while retaining the
root identity. Neither form receives native allocator authority.

Provider planning, program-only publication and the ordinary compiler pipeline
explicitly refuse the new profile until actual native ownership/zeroing/caller
reconstruction exists. The pipeline checks its selected source instead of
falling through to the old frame selector. Tests verify refusal before output
creation, including a deliberately unavailable tool directory, and reject
missing/extra/mismatched inputs, private exports and wrong result/measure/
progress/parameter profiles. These checks do not qualify a new kernel caller.

The [source-binding record](tools/tail-traversal/results/stage-3-frame-source-binding/qualification.json)
passes all **thirteen commands**, including **647 frontend**, **253 semantics**,
**356 compiler unit**, **261 CLI** and **four coverage tests**, targeted
Count/Nat/frame/source-binding tests, locked workspace all-target checking,
workspace/all-changed-Rust formatting and source-size checks. It freezes **163
source inputs**, three original verification inputs and complete command logs.
Four new frontend/model tests and three compiler tests cover the declaration,
context, replay and refusal paths. Six source-obligation artifact sets retain
original/root-changed/policy-changed cases across targeted/full-unit cohorts.

[Independent readback](tools/tail-traversal/results/stage-3-frame-source-binding/independent-readback.json)
checks exact current/frozen inputs and the full root/library/component links,
all source/runtime/controller/boundary identities, separate root/body changes
and explicit unqualified-caller diagnostics. Retained native cohorts still
pass 137 frame-policy cases and 33 Nat cases each, with the same 32-function,
1368/1568-byte source/weighted frame-policy stack paths. The retained Count and
raw/canonical kernel regressions still pass; their 21 VM observations remain
**static-ELF ABI fixture** scope, with no new BusyBox or allocator-guest claim.

The unchanged wrapper used **3084 MiB**, a **1569 MiB** host reserve and zero
validation swap. Peak usage was **2670063616 bytes**, with zero limit/OOM events.
The systems ledger now contains **325 stable rules**. Original verification
snapshots retain the pending trace row; factual report/trace changes follow the
completed run without changing any frozen source input.

Risk is high around binding an authority-owning subsystem to ordinary source.
Self-review checks authority order, exact context/reserve/interface admission,
all original inputs, independent replay, and refusal before legacy native
planning or output creation. No independent human review is claimed. The actual
native allocator is still required: the current old root's release lowering
clears private registers without updating an allocator table. The next binding
must connect real allocation/release/mapping lifetimes, normalized reservations,
opaque live identities, physical zeroing and actual kernel stack/context/
retirement. Source-only coupling does not complete that implementation or the
planned Stage 3 commit.

## Stage 3 native allocator and bootstrap physical target

The [native ownership controller](../src/topal-compiler/src/runtime/kernel_frame_controller.ll)
now wraps the ordinary frame policy with a live extent registry and private
issuer/identity/slot tokens. It checks full canonical Nat results, exact whole
metadata deltas, physical continuity and provenance. A backwards Boolean scan
independently rejects false exhaustion without choosing another allocation.
Whole release requires a live matching identity and no mapping pins; successful
release consumes its token and registry record. Physical zeroing finishes
before token/registry/issuance publication. Source and partial physical writes
survive terminal failures; they are not rolled back.

The [bootstrap target](../src/topal-compiler/src/runtime/kernel_frame_target.ll)
constructs its actual table from E820 boundaries and complete linked-image and
boot-resource reservations. It classifies intervals before page alignment,
preserving unaligned boundaries even between identical RAM claims. Capacity
derives from the normalized allocatable frame count. The backing covers the
existing 1 GiB bootstrap identity-map geometry; unsupported RAM claims are
refused, not truncated. Native identity budgets come from the reconstructed
source's frame/loan costs and include its terminal step.

The target checks actual bootstrap CPL/IF/stack geometry and exact private
domains. Its zeroing callback validates pending issuance and all selected cells
before writing the actual physical bytes. The terminal callback poisons the
allocator, retains source writes and pending issuance, retires source domains
and halts. Complete owning-caller stack/recovery retirement is still required.
The [binding design](design/frame-policy-kernel-binding.md) records these limits.
Source-obligation schema 3 binds the native controller and bootstrap template
identities as well as the original root/library/component snapshots. It grants
no native kernel publication authority; the compiler/provider gates stay closed.

The [bootstrap qualification](tools/tail-traversal/results/stage-3-bootstrap-frame-target/qualification.json)
passes all **fifteen commands**, including **647 frontend**, **253 semantics**,
**358 compiler unit**, **261 CLI** and **four coverage tests**, targeted native
controller/target and preceding source components, locked all-target workspace
checking, workspace/all-changed-Rust formatting and the unchanged source-size
ceiling. It freezes **174 source inputs**, three original verification inputs,
all logs, six controller artifact sets and two privileged target artifact sets.
The preceding 163-input source-binding record retains its historical scope.

Eleven new VM cases pass in each targeted/full-unit cohort: real-map creation,
controlled sparse/unaligned allocation, duplicate creation, interrupt/stack
refusal, a reserved-zero terminal fault and four malformed/unsupported/no-RAM
handoffs. The real E820 case creates **152525 allocatable frames** and compares
every table cell against reference normalization. Allocation workloads use the
controlled maps, with runtime request facts, simultaneous extents, mapping-pin
refusal and whole release/reuse. Physical observations check complete zeroing
before publication and actual dirty-page reuse; the forced reservation failure
publishes no handle and preserves source effects. These fixtures run no user
application and do not replace production root/workload integration.

[Independent readback](tools/tail-traversal/results/stage-3-bootstrap-frame-target/independent-readback.json)
passes on its first attempt. It replays all frozen/current inputs, source/native
identities, separate root/body changes and public refusal, and independently
reconstructs emitted controller/target templates. A per-frame oracle separately
replays the actual boot maps and complete linked-image reservation. Native
stack recurrences include the architecture observation's otherwise hidden
eight-byte push: allocation/release paths are **1840/1824 bytes**, creation is
208 bytes and pin/resolve paths are 200/216 bytes. These component bounds
exclude the owning gateway, entry fixture and complete fatal caller recovery.
The retained legacy ELF relocation/load comparisons and Count/Nat/policy native
cohorts also pass. Their 21 VM observations retain **static-ELF ABI fixture**
scope; the 22 new observations are allocator components, with no new BusyBox
or distribution claim.

The unchanged wrapper used **3115 MiB**, a **1569 MiB** host reserve and zero
validation swap. Peak usage was **1865277440 bytes**, with zero limit/OOM events.
Self-review follows authority order, compares the independent reference and
per-frame oracles, validates native calls/stack and reviews the complete diff.
Risk remains high around live identity, physical writes, ownership and failure
cleanup; no independent human review is claimed. The real owning gateway,
production allocation/release/mapping lifetimes, dynamic MM requests and full
source/native/load publication remain required before the planned Stage 3
commit and Stage 4 qualification.

## Stage 3 owning bootstrap gateway

The [owning gateway](../src/topal-compiler/src/systems_frame_gateway.rs)
now captures one actual boot invocation and requires CPL0, IF0, DF0, the
admitted stack alignment/window and a fresh private lease. The target variant
requires that lease and checks its current stack against the owning invocation.
Normal and fatal paths preserve all six callee-saved registers and physically
retire 4032 scratch bytes, 48 saved-register bytes and the separate 48-byte
recovery record. The caller-owned 32-byte provider result survives a normal
resolve; terminal paths retire it too. Syscall stack binding remains separate.

The [targeted gateway record](tools/tail-traversal/results/stage-3-frame-gateway/qualification.json)
passes **eight commands** with **178 frozen source inputs**: two gateway tests,
the original bootstrap test, three source-binding/refusal tests, locked
workspace all-target checking, four coverage tests, workspace/all-changed-Rust
formatting and source-size checks. This is targeted regression evidence; the
preceding 358-test full compiler run retains its historical 174-input scope.
The unchanged wrapper used 3118 MiB with a 1569 MiB host reserve and zero swap;
peak usage was 1102659584 bytes, with zero limit/OOM events.

Sixteen gateway VM scenarios pass, plus eleven original bootstrap scenarios.
In addition to constructor/allocation/refusal/zeroing cases, these cover source
abort after the first persistent allocation-cell write, invalid completion
after an identity was issued, busy/DF entry refusal, and a late fault after
caller and saved-register retirement. Terminal paths retain performed metadata
and issuance, poison the allocator and retire their actual caller storage.
The late test corrupts old saved-register copies after erasure and verifies
they are cleared again without being reloaded. Debugger observations unwind
through the gateway and recover the caller's registers while ordinary source,
checkpoint and completion bodies execute on their separate private stack.

[Independent readback](tools/tail-traversal/results/stage-3-frame-gateway/independent-readback.json)
passes on its first attempt, replaying exact sources, native/object/log
identities, call/stack recurrences, per-frame map oracles and all caller/fatal/
late/ABI observations. The conservative terminal-callback contribution is 96
bytes; the dispatch/body bound is **1952 bytes**, or **2024 bytes** including
the gateway, alignment, saved registers and return word, within 4096 bytes.
The original target keeps its distinct raw-component behavior and bounds.
An [earlier validator-scope failure](tools/tail-traversal/results/stage-3-frame-gateway-before-changed-rust-scope/historical-scope.json)
is preserved: its formatter unintentionally checked unchanged historical
include files. The fresh record checks every current changed/new Rust file;
no compiler source was changed to repair that scope error.

This remains an unpublished bootstrap component. The generator/target variant
is compiled under tests until actual root lifetime and publication binding
exist. The public frame-policy gates remain closed. Production allocation,
release and mapping must retain opaque identities and call this boundary;
current old release lowering still only clears registers. Dynamic workload/MM
requests and complete source/native/load reconstruction remain required before
the planned Stage 3 commit. No new userspace/container/VM-host acceptance or
independent human review is claimed.

## Full boot Root and early completion

The next unpublished binding executes the complete existing boot Root through
the source allocator. It retains opaque handles across actual construction,
prepared-process and mapping lifetimes, pins every live loan, and calls whole
release before clearing physical facts. The five-page active bootstrap hierarchy
and two-page map-edit hierarchy also use real allocations. Map-edit storage is
released after unmap commit and TLB invalidation; the active CR3 hierarchy remains
live and pinned at completion. Private provider facts are erased after use.

The policy now uses an additional ordinary ordered terminal rule. Its carried
stop threshold ends completed or failed traversals without burning the remaining
countdown. The complete worst-case countdown and identity admission remain
unchanged. Systems-only checking retains every original branch and derives
natural descent from the initial literal guard; the core parser and ordinary
language design are unchanged. Complete bounds include every extra tested
operand and terminal action. The source closure has 33 functions, with bounds
of 53 frame identities and 38 region loans per step; private temporary storage
uses 18632 bytes within the existing 65536-byte reserve.

The [full-root record](tools/tail-traversal/results/stage-3-frame-root/qualification.json)
passes eight commands for **188 frozen source inputs**: all **362 compiler**,
**648 language** and **253 semantics** unit tests, four coverage tests, locked
workspace all-target checking, workspace/all-changed-Rust formatting and size
checks. This supersedes the earlier checkpoints as current-source evidence.
The unchanged wrapper used 3116 MiB, zero swap and a 1569 MiB host reserve.
Final peak usage was **2452267008 bytes** (2.28 GiB), with zero limit/OOM events.

The full-root guest uses all **151977** normalized allocatable frames from its
unmodified firmware map, after reserving the actual larger linked image and
boot resource. Independent complete-map oracles reject omitted RAM. Its **67
gateway calls** issue 14 distinct extents, resolve each, change mapping pins and
consume 13 extents. The sole remaining extent is the five-page active CR3
hierarchy with one pin. Physical zeroing, actual caller/private retirement,
immutable topology, consumed tokens and registry state are observed. The
identified static-ELF ABI fixture completes through the final user-backing
marker. This is fixture execution, not real distribution-application acceptance.
The full compiler cohort also reruns sixteen gateway and eleven raw-target VM
scenarios and the native metadata/controller/refusal oracles.

[Independent readback](tools/tail-traversal/results/stage-3-frame-root/independent-readback.json)
checks exact source/log/object identities, source bindings, native stack
recurrences, a separate complete-map oracle, every issued/pinned/released handle
and the actual linked ELF reservation. Its first attempt rejected the full
root's retained optional `.debug_names` warning; that failure is preserved. The
reader now recognizes only that specific ignored-index warning in the root
fixture. Canonical publication already strips that index; this unpublished
fixture does not qualify its debug-name index. Gateway CFI observations retain
their separate scope.

An earlier development guest reached actual allocation/pinning/release but
timed out at 300 seconds while the old policy exhausted its countdown. The
early-completion source completes the full build/boot test in about 34 seconds.
No host security setting, memory limit or source reserve was relaxed.

The experimental root/gateway route remains test-built and public frame-slot
publication remains refused. Dynamic workload/MM requests, actual syscall-stack
leases and complete canonical source/native/load reconstruction remain required
before the planned Stage 3 commit. The original distribution, container and
VM-host acceptance remains open.

## Stage 3 progress: captured syscall requests and heap retirement

The test-built complete Root now selects a memory provider that obtains backing
from actual `brk` requests instead of its predecessor's fixed 34-page heap.
The [runtime design](design/dynamic-frame-memory.md) binds captured process
context, CR3, GS, entry identity, ABI request, exact continuation stack and
compiled return site. Source allocation and memory handling share one owning
gateway. An eight-byte zero alignment slot preserves the existing entry ABI;
the native body is 2080 bytes and the combined syscall bound is **2160 bytes**.
The original 4096-byte caller and 65536-byte source reserves are unchanged.

Replacement copies the retained prefix into zeroed new backing, pins it before
publication, invalidates affected translations, then unpins, erases and consumes
the old whole extent. Query/within-page changes allocate nothing. Exec and exit
revoke and release the complete live heap. The predecessor's startup witness
does not determine request legality or allocation counts. Its remaining legacy
user-copy validators and the current 519-leaf geometry still limit this target;
general virtual memory is unfinished.

The [runtime-memory record](tools/tail-traversal/results/stage-3-frame-memory/qualification.json)
passes eight commands: **363 compiler**, **648 language**, **253 semantics** unit
tests, four coverage tests, locked workspace all-target checking and formatting/
size checks. Its **201 frozen inputs** are supplemented by a
[772-input producer closure](tools/tail-traversal/results/stage-3-frame-memory/producer-inputs.json).
Changed/new sources were frozen before validation; additional unchanged
producers are checked against the immutable base commit already identified in
that record. Prior lab result archives are outputs and are excluded from that
producer inventory. No new CLI-suite qualification is claimed.

The normal userspace fixture executes **80 memory operations**: 76 `brk` calls,
three exec resets and one exit cleanup. Different actual counts include 3, 67
and 519 pages, plus shrinking to zero, reuse, preserved prefix bytes, zero newly
mapped pages, within-page updates, invalid/overflow-sized requests and user DF
preservation. CFI across the private source stack recovers the actual syscall
caller and all six preserved registers. Nine context/critical/busy refusal
cases claim no foreign retirement authority. Partial source writes and failure
after replacement publication retain actual effects and held identities,
poison the owner, retire the caller and never resume the process.

A separate **50 MiB VM** uses its complete unmodified firmware map. It has 970
allocatable frames after actual reservations; its largest free extent is 384
pages when a real 519-page user request arrives. Ordinary exhaustion preserves
the current heap's bytes, mappings, handle and issuance counter. The test edits
no memory map, register or return result to manufacture that outcome.

[Independent readback](tools/tail-traversal/results/stage-3-frame-memory/independent-readback.json)
checks all inputs/logs/objects, source bindings and stack recurrences. A separate
extent model replays the boot prefix and successful heap replacements against
the small VM's topology, reproduces its complete metadata and registry hashes,
and derives the same 384-page maximum free extent. The original full Root,
sixteen gateway and eleven raw-target scenarios also pass in this cohort.
Final peak memory is **1795899392 bytes** (1.67 GiB), with zero limit/OOM events,
zero swap and the unchanged host reserve and wrapper.

This supersedes earlier current-source checkpoints without granting canonical
frame-slot publication. The route remains test-built; complete canonical
source/native/load reconstruction, general MM/user-copy integration and original
distribution/container/VM-host acceptance remain required. Stage 3 is unfinished.

## Stage 3 progress: complete fresh native reconstruction

The source reconstruction path now rebuilds the actual frame Root/provider,
ordinary source/runtime/owner/boundary, controller, target, dispatch and gateway.
It also rebuilds all ten other declared libraries from their explicit immutable
snapshots. The frame declaration is retained in the checked program; a typed
private source check selects this provider. Program-only APIs still refuse it.

All twenty native objects receive exact contribution endpoints. Fresh replay
compares relocated payloads, symbols, permissions, alignment, zero storage and
load owners. Complete coverage rejects additional unowned code/data or storage.
Only minimal alignment and narrowly derived linker header/empty-GOT page-tail
extents are accepted. The actual kernel reservation equals the full allocated
image end. Static source closure avoids indirect global references; the exact
strong own-function GOTPCRELX address relaxation is reconstructed byte for byte.
An indexed symbol lookup preserves ambiguity checks while avoiding repeated
whole-table scans.

The [reconstruction record](tools/tail-traversal/results/stage-3-frame-reconstruction/qualification.json)
passes eight commands for **777 frozen complete source/producer inputs**:
**365 compiler**, **648 language**, **253 semantics** unit tests, four coverage
tests, workspace all-target checking and formatting/size checks. A failed initial
program-header type check is retained in its separate failed record. No source
reserve, host memory/security setting or wrapper limit was relaxed. The final
peak is **2468528128 bytes** (2.30 GiB), with zero limit/OOM events and zero swap.
Workspace checking passes with unused-code warnings for the production-private
helpers whose artifact integration remains pending.

The actual rebuilt image has **804 relocated contributions**. Its normal guest
executes 80 memory operations and all twelve context/refusal/fatal/exhaustion
scenarios pass. Complete small-VM topology now has 972 allocatable frames and a
386-page maximum free extent when a real 519-page request is refused; independent
replay reproduces metadata/registry hashes. Compilation derives **2032 bytes**
for the dispatch body and **2112 bytes** with its syscall caller. Source CFI and
whole normal/fatal retirement remain observed. Code mutation per native role,
RWX permissions, entry/padding changes, missing library/object and changed source/
toolchain/bounds/original-object data are refused.

[Independent readback](tools/tail-traversal/results/stage-3-frame-reconstruction/independent-readback.json)
passes on its first attempt. It checks all frozen inputs/logs/binaries, fresh
source determinism, native stack recurrences, the complete ELF allocation/load
coverage, linker geometry, actual reservation and independent extent replay for
both raw and freshly rebuilt guests. The preceding Root, raw-target, owning
gateway and syscall-memory cohorts also pass with this source version.

This is fresh source/native/load qualification, not yet canonical publication.
The artifact schema can retain observational frame binding data, but public
frame publication and boot remain refused, including orphan/native metadata.
Complete artifact/source/provenance/boot association is the next integration.
Stage 3, general MM and original distribution/container/VM-host acceptance remain
unfinished.

## Stage 3 complete: source-bearing frame publication

The normal compiler pipeline now accepts an explicit frame source declaration
only through the complete source-bearing artifact route. It retains that
declaration while rebuilding the actual Root/provider, frame policy and all ten
other selected libraries. Publication compares the metadata producers' original
object identities, links the twenty reconstructed inputs and verifies stable
geometry after binding the complete allocated image reservation.

Published provenance retains the immutable root/frame/other-library/module
snapshots, generated objects and LLVM, fresh bounds and load ownership. Boot
reconstructs them with the qualified toolchain, compares the entire binding,
target/entry/placement and unique artifact associations, then retains every
other selected owner validator. Each legacy library's source name/hash and
module metadata must also match its actual reconstructed snapshots. Program-only
frame selection and unsupported mixed tail owners remain refused.

The [artifact record](tools/tail-traversal/results/stage-3-frame-artifact/qualification.json)
passes nine commands for **780 frozen inputs**: **367 compiler**, **648 language**
and **253 semantics** unit tests, **261 CLI** tests, four coverage tests, workspace
all-target checking, formatting and the unchanged source-size ceiling. The public
image binds **804 contributions** from twenty objects and executes **80 memory
operations** plus twelve context/refusal/fatal/exhaustion scenarios. The actual
combined syscall bound remains **2112 bytes** within the unchanged 4096-byte
reserve. The complete 50 MiB guest still has 972 allocatable frames and a
386-page largest free span at a real 519-page exhaustion request.

Fifteen atomic source/publication/boot refusal cases include missing bindings,
undercharged bounds, changed snapshots, absent libraries, duplicate/extra
identities, orphan tail metadata, detached other-source metadata and modified
native code with a recomputed outer kernel hash. Null decoding and thirty
identity mutations across the ten legacy library records are also refused.

[Independent readback](tools/tail-traversal/results/stage-3-frame-artifact/independent-readback.json)
checks all source/log/object identities, native stack paths, ELF allocation/load
coverage, every published output, boot payload/resource association and actual
VM observations. A separate extent model reproduces the public image's small-map
metadata and registry hashes. Its original field-name failure is retained; the
corrected reader checks the public operation/request prefix against the fully
captured Root prefix before replaying its handle/alignment facts. No kernel or
test obligation was weakened. An earlier intentionally interrupted cohort
retains the review-discovered missing metadata association and its partial log;
that interruption was not a memory failure.

Final peak memory is **2511380480 bytes** (2.34 GiB), with zero limit/OOM events
and zero validation swap. The wrapper, requested limit, host reserve, source
reserves and host security settings are unchanged. Five checking warnings concern
test/component-only helpers; the public source/native/boot path is connected.

This completes Stage 3's owned allocator/source/native/publication integration.
The production root still needs to select it and qualify exact committed kernel
and real application inputs in Stage 4. These new guests execute the identified
static-ELF ABI fixture. They grant no new unchanged BusyBox, distribution,
container or VM-host acceptance. General virtual memory and replacement of the
legacy user-copy validators remain required.

## Subsequent stages and original acceptance

Stage 2 now binds the native roles to the initial actual kernel caller, complete
live-storage/caller-stack costs, retirement and owner/context refinement. Its
64-byte qualification does not establish an allocator's larger owned resources.
The [transfer inventory](knowledge/common/tail-transfer-readiness.md)
records actual current representations and refusal boundaries, with the existing
65536-byte temporary and 4096-byte stack reserves intact.

Stage 3 uses the new execution path on a real owned allocator/metadata resource,
preserving source effects, provenance, reservations, lifetime and fatal cleanup.
Stage 4 must qualify exact committed native/kernel/VM/application
inputs and publish complete evidence in the existing initial Step 4 PR.

General allocation, persistent VFS/process/virtual-memory state, full pinned UAPI
and distribution replacement, Device Tree/drivers, networking and isolation,
Docker/Podman and VM hosting remain required. Other architectures constrain the
abstraction; implementation remains x86-64. The preceding native checkpoint
retains its own [namespace qualification](step-4-shared-namespace-transactions.md).
