# Tail-transfer implementation inventory

This source-backed inventory records Stage 2 of
[TK-DEC-282](../../design/repeated-source-tail-traversal.md). It is not a runtime
certificate or native admission. The source-plan inspector and
[clear-prefix source](../../kernel/algorithms/clear-prefix.t) establish an actual
ordinary traversal. Experimental lowering/components are recorded below;
complete transfer, stack, allocation and cleanup qualification remains required.

## Actual carried state

The selected source has one invariant borrowed region and three natural facts:
remaining bytes, current offset and accumulated success status. The caller
supplies status one. Each nonterminal step evaluates the decremented remaining
value, incremented next offset and `next-status` in ordinary argument order.
`next-status` performs the actual store at the old offset only while status is
one; any store refusal produces zero. Zero status remains zero without further
stores. The terminal step returns whether status is one. No source resource or
numeric storage escapes.

If the caller proves `offset + remaining <= owned byte count` mathematically,
then a step with positive remaining has a contained offset. Its successor
preserves that sum and decreases remaining by one. After exactly the original
remaining count, the offset reaches the admitted endpoint and the terminal
step performs no store. The model's back-edge bound does not replace that
interval invariant or charge the final guard/base action automatically.

With admitted native counts/offsets at most `u64::MAX`, that interval invariant
keeps actual offsets and remaining values within the same bound. A conservative
interval-only proof may charge 65-bit increment intermediates instead. Either
approach must retain arbitrary-precision source meaning; it cannot introduce
wrapping integer arithmetic or assume a relation it has not established.
Status transport must cover the actual initial input and its subsequent 0/1
results. The source-only selector currently supplies unconstrained native-width
Nat facts; it does not establish any of these caller invariants.

## Existing representation and refusal boundaries

The shared [numeric runtime](../../../src/topal-compiler/src/runtime/value_core.ll)
uses an immutable sign/length header with 32-bit limbs; zero may refer to a
read-only constant. The current x86-64 layout has a 16-byte header.
`int.from.u64` requests 20 or 24 bytes for a nonzero native word; the qualified
eight-byte-aligned arena charges 24 bytes in either case. A transfer operation
must handle constants, zero, negative Int values, multi-limb values and equal or
aliased arguments according to their semantic values. Raw address equality is
not an owning root or a valid lifetime proof. These are current runtime facts,
not a portable source representation or a complete bound for arithmetic calls.

The [region runtime](../../../src/topal-compiler/src/runtime/kernel_regions.ll)
currently uses 64-byte private view/control records. It checks invocation owner,
code, generation, rights, contained spans and issued-view facts. Its count cap
is 65536 bytes and its issued-view cap is 256. The
[allocation analysis](../../../src/topal-compiler/src/systems_region_bounds.rs)
also assumes that region count cap. Neither cap may silently be removed from
an existing profile. A later broader owned-region profile must bind actual
extent authority, representable target pointers and complete new evidence.

Existing executable numeric allocation analysis still rejects recursive closures. Native stack
inspection also rejects cyclic call graphs. The shared emitter disables tail
calls for qualified runtime functions. A plan therefore cannot become executable
by bypassing one of those checks or depending on LLVM to remove recursion.

## Required transfer construction

The initial selected recursive component is the source entry, with acyclic
ordinary helpers. Its source caller is the private bridge rather than an
arbitrary suspended source parent. Keep the borrowed root/control and immutable
input frame live throughout the invocation. Before any reset, transfer every
next-state scalar into disjoint owned storage and prove that no remaining live
numeric object, view, helper frame or cleanup obligation depends on the retiring
extent. The old and next state may coexist during transfer; both costs count.

The implementation must charge both transfer banks, pinned state, source work,
adapter/runtime allocations, alignment and checkpoint stack. Erase the complete
retired extent, including older high-water contents, before reuse. Retain source
evaluation and persistent-store order. On normal completion and runtime abort,
reclaim all domains before the owning caller resumes or reaches its fatal
boundary. The existing 65536/4096 reserves remain unchanged.

The source creates no derived Window geometry. This does not remove fresh loan
identities: the existing [call model](../../../src/topal-semantics/src/systems_region_library.rs)
issues a fresh callee view and frame when entering a helper. Logical tail/helper
loans still need correct fresh nonwrapping identities and admission of total
issuance before mutation; one-step helper depth is not that proof. Physical
descriptor reuse cannot reset that history or redefine the existing 256-view
native profile. Debug information must identify optimized traversal steps and retired
unavailable values accurately, while preserving all applicable native debugging
obligations. Ordinary values in an active outer caller cannot be reclaimed as
part of a later nested extension without separate liveness evidence.

Independent native oracles need guard pages, complete value/root comparisons,
real source stores and exact effect order, reused-bank poison/zero checks,
exhaustion at each transfer boundary, aliases/constants/multi-limb inputs,
unchanged live caller/control state, and normal/fatal register/stack cleanup.
The source-only plan tests provide none of that physical evidence. Subsequent
real allocator/metadata integration must act on the actual owned bytes and
extents, preserving reservations, provenance, disjointness and release lifetime.

## Experimental return-boundary components

The [private step design](../../design/tail-step-component.md) now has shared
ordinary lowering and [exact native C evidence](../../tools/tail-traversal/results/stage-2-step-components/qualification.json).
Source/helper frames return before the test bridge erases work and the retired
bank. Every next scalar is copied first, with readonly-root extents bound from
actual generated constants. The next step reloads its roots. Private helper
hooks preserve scoped views and fresh frame/loan history rather than treating
physical descriptor reuse as a fresh capability.

The new component uses a distinct tail-region runtime for caller-proven larger
mapped extents, with its own hash and per-operation scoped-view validation.
Derived-window accounting remains separate from helper-loan history; Window
construction remains refused. Existing executable runtimes/caps are unchanged.

Fifteen native clients per cohort cover 70000-byte clear/read traversals,
signed/Boolean/Unit values, aliases, readonly multilimb constants, seven
defensive IR control-flow graphs and injected transfer/allocation failures.
All 327 compiler unit and 261 CLI integration tests pass. This is component
evidence only: the C host stack and fault observer do not qualify the actual
kernel's private stack/register/debug/erasure/fatal provider. The test context
layout and source-work bound must be bound to that actual provider, with a
complete model and native cleanup proof, before a real owner invokes it.

The subsequent [private-stack component](../../design/tail-stack-boundary.md)
has [separate exact evidence](../../tools/tail-traversal/results/stage-2-stack-boundary/qualification.json):
all 330 compiler unit and 261 CLI tests pass. Actual emitted x86-64 objects run
on guarded 4096-byte stacks with 65528-byte temporaries plus the eight-byte
recovery cell. Normal/fatal cleanup, geometry/busy refusal, persistent effects,
full successor values and caller/register/backtrace observations are checked.
This advances the target component only. Complete session/model and kernel
context/hook/layout binding, between-step failure cleanup and real owner
invocation remain required; sampled CFI evidence needs an abort-prologue review.

The [focused follow-up](../../tools/tail-traversal/results/stage-2-stack-boundary-cfi/qualification.json)
fixes entry-pivot and abort-prologue unwind transitions. Independent GDB walks
68 admitted-entry instructions and seven abort-recovery instructions, checking
saved caller registers and unavailable clobbered registers. All thirteen
component tests and 24 native clients pass with checking, formatting and size
checks. The complete session/kernel binding and between-step cleanup obligations
remain open.

The [controller follow-up](../../tools/tail-traversal/results/stage-2-controller-boundary/qualification.json)
implements a separate emitted between-step failure/cancellation entry. It uses
the controller's still-owned extents rather than inactive recovery/context
fields. Five admitted faults/cancellations preserve exact owner bytes/history
while reclaiming both temporary domains; eight invalid protocols reach fatal
handling without erasing unproven addresses. All fourteen component tests and
25 native clients pass with checking, four coverage tests, formatting and size
checks. GDB observes restored controller registers/stack/backtrace. Complete
session/model, kernel context/hooks/layout and actual owner integration still
need qualification; the C observers grant none of that authority.

The [owned session contract](../../design/tail-session-model.md) now has a
reference implementation over actual bootstrap storage, persistent per-allocation
identity history, complete scalar banks/work/result values and explicit returned/
retirement/fatal phases. Normal byte refusal remains distinct from invalid
authority, including natural values above a target word. The compiler rebuilds
source/range constraints before model construction; actual clear-prefix roots
retain the analyzed 80-byte bank.

[Exact session evidence](../../tools/tail-traversal/results/stage-2-owned-session/qualification.json)
passes all 253 semantics, 333 compiler unit and 261 CLI tests, plus checking,
coverage, formatting and source size. It retains 57 source snapshots and the
separate native cohorts. Fourteen new semantic cases include 70000 reference
steps over actual owner bytes, full scalar/alias retirement and 2601 independent
counter-admission cases. Reference buffers and logical reclamation do not
qualify native metadata/stack size, kernel hooks/context or physical erasure.
The next implementation must close those provider bindings and connect a real
kernel allocator/metadata owner; Stage 2 is still unfinished.

The [provider hook component](../../design/tail-provider-hooks.md) now supplies
actual freestanding context, allocation, complete value-transfer and scoped
helper mechanisms. Native tests exclude the C source-hook definitions. The
clearer uses a 1248-byte actual context and 1320-byte minimum temporary extent
including recovery metadata/cell; the reader needs 1352 bytes. Actual linked
source/hook call paths in this corpus use 408–504 source-stack bytes. Import,
instruction, object-frame and guarded/GDB observations are retained in
[the exact record](../../tools/tail-traversal/results/stage-2-provider-hooks/qualification.json).

All 253 semantics, 337 compiler unit and 261 CLI tests pass. Eighteen actual-hook
clients per cohort cover eight source variants, controller/invalid-outcome
recovery and eight defensive pointer/budget/copy fixtures. Pinned history is
independent of mutable context; performed stores/history persist through fatal
cleanup. This does not qualify kernel initialization, controller checkpoint/
completion or whole-invocation retirement. The next provider implementation
must close those bindings before real kernel owner invocation.

The [native checkpoint follow-up](../../design/tail-native-checkpoint.md)
implements complete returned-state/progress/budget validation and actual bank/
work retirement with fresh persistent identities. Source and checkpoint use the
same stack sequentially, with distinct exact roles and caller-supplied history.
A preserved register retains history through header erasure so late cleanup
failure still reaches trusted fatal handling. Two injected target readback
branches check that path after the header is entirely zero.

[Final checkpoint evidence](../../tools/tail-traversal/results/stage-2-native-checkpoint/qualification.json)
passes all 253 semantics, 340 compiler unit and 261 CLI tests plus checking,
coverage, formatting and size. Twenty-one component tests run 48 native clients
per cohort, including 23 actual-provider clients. Added coverage includes 26
controller failures/cancellations, fourteen checkpoint admission refusals per
native case, nonunit/large unreachable descent and seven-argument helper calls.
Reached source paths use 424–584 stack bytes and the checkpoint uses 312 bytes;
all actual contexts/minimum temporary costs fit the existing reserves. Seventeen
GDB runs per cohort observe caller state, retirement and instruction-level CFI.
Normal whole-invocation completion and actual kernel owning-caller admission/
context/refinement still need implementation and qualification before Stage 3.

The [native completion mechanism](../../design/tail-native-completion.md) now
validates terminal retirement and copies/reads back the actual scalar/Unit value
to a separately owned, explicitly costed 56-byte caller result frame. Its target
erases the entire invocation temporary/stack before clearing history activity
and returning. Running-body and late cleanup failures independently retain
history/output pins, erase partial output and preserve earlier owner stores and
issued identities. Kernel owning-caller resource accounting remains separate.

[Exact completion evidence](../../tools/tail-traversal/results/stage-2-native-completion/qualification.json)
passes all 253 semantics, 341 compiler unit and 261 CLI tests plus all checks.
Twenty-two component tests run 62 clients per cohort, including 37 native-provider
clients, 194 normal completions, fourteen completion failure fixtures, 23
completion admission refusals per native case and 31 GDB runs. All three roles
reuse the same private stack sequentially; completion paths use 184–248 bytes.
The record preserves 72 source snapshots and 124 native artifact records.
Actual kernel admission/interrupt context, complete caller-frame and retirement
proofs, owner refinement and real allocator/metadata integration remain unfinished.

The [provider admission/controller](../../design/tail-owner-admission.md) now
constructs native source context, current view and banks from immutable word
facts and actual history after complete nonwrapping admission. Reuse retains
actual counters and advances generation; refusal preserves complete state. It
then executes the three actual roles, retaining independent failure pins. Its
initial interface requires one nonempty exclusive read-write owner and three
word-bounded Nat facts, with no geometry-based ownership or permission grant.

[Exact admission evidence](../../tools/tail-traversal/results/stage-2-owner-admission/qualification.json)
passes all 253 semantics, 343 compiler unit and 261 CLI tests plus all checks.
Each controller cohort executes 501 normal invocations, 2252 refusals and two
explicit admitted machine failures, including 70000-step reuse and 2601 counter
fixtures. Native controller/weighted normal paths use 240/688 stack bytes.
The record preserves 78 sources, six controller artifacts and the separate 124
step artifacts. Actual typed kernel caller ownership, privileged/interrupt
context, total caller/fatal bounds and physical caller retirement remain required
before Stage 2 publication; no new kernel or VM execution is claimed here.

The [kernel root experiment](../../design/tail-kernel-root.md) now invokes the
actual source/controller roles from a typed exclusive 64-byte bootstrap owner,
with whole Nat facts `(64, 0, 1)` and an actual true-result gate. Its independent
outer recovery record retires controller frames/spills, private/output domains
and recovery before resumption or nonreturning failure. Source/checkpoint/
completion paths use 488/312/184 bytes; the complete initial normal path is
824 bytes including the outer frame. The controller caller retirement area is
320 bytes plus 48 bytes of saved-register copies. All three inner roles still
use the existing private stack sequentially.

[Exact privileged evidence](../../tools/tail-traversal/results/stage-2-kernel-experiment/qualification.json)
passes all 636 frontend, 253 semantics, 345 compiler unit and 261 CLI tests plus
the complete checks. Two cohorts execute ten VMs each: normal caller/final
retirement, admitted source/false-result failures and seven pre-entry/admission
refusals. Independent GDB observations cover actual CPL0/IF0, ABI/owner geometry,
performed stores/history, complete cleanup and entry/return/fatal CFI. The GNU
stack-discontinuity augmentation handles the actual high-private/low-kernel stack
placement; ordinary bridges retain their CIE bytes. Records retain 96 sources,
two kernel, six controller and 124 step artifact sets. All limits/OOM counters
remain zero under the unchanged wrapper.

The experiment uses a separate full typed boot fixture with other optional
source slots unselected. It is not canonical kernel/application qualification.
Public tail-root publication remains refused until complete source/module/native/
caller/ownership/context/boot binding is verified, without using an old acyclic
certificate. Real allocator/metadata ownership integration and the original
general-allocation/Linux acceptance remain required.

The [reconstructed publication path](../../design/tail-publication-binding.md)
completes Stage 2 for the initial 64-byte typed owner. It retains actual root and
ordinary source/module inputs, rebuilds the complete checked/native/caller path,
and compares seven fresh objects with the complete linked and loaded kernel
contributions. Source-only metadata and serialized native facts grant no authority.
The compiler pipeline selects this source-bearing route; program-only publication
continues to refuse the tail interface. Boot checks actual entry, permissions,
zero storage, source/native/target/trace/digest identity and mixed/orphan inputs.

[Final evidence](../../tools/tail-traversal/results/stage-2-publication-binding/qualification.json)
passes 636 frontend, 253 semantics, 349 compiler unit and 261 CLI tests, checking,
four coverage tests, formatting and size. Two cohorts produce isolated and fully
selected canonical artifacts, with 89/105 checked native input contributions and
22 privileged canonical-path VM observations; 20 separate raw-experiment VM
observations retain their own scope. The independent reader also implements its
own ELF relocation/load comparison. Canonical controller/combined/outer costs
are 248/688/824 bytes, with 328 controller plus 48 saved-register bytes retired.
The raw experiment's earlier 240/320 facts remain historical.

These VM observations use the clean-room static-ELF ABI fixture. The
[scope correction](../../tools/tail-traversal/results/abi-fixture-scope-correction/correction.json)
repairs an earlier BusyBox label without changing machine evidence. Real
application acceptance is still Stage 4. Actual allocator/metadata ownership
integration is Stage 3; the production root does not yet select this traversal.
Before qualifying larger owner extents, remove the legacy 65536 Count inference
cap for the new source selection and its isolated modules, and derive sound
caller-specific bounds. The existing initial 64-byte owner is within that cap.
