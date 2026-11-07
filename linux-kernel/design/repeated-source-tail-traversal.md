# Repeated ordinary source traversal

`TK-DEC-282`; systems-only qualification under standing authorization.

The kernel needs allocation and metadata algorithms whose amount of work follows
owned memory and live workload inputs. The current acyclic source profile and
monotonic temporary arena make such traversal impractical: expanding one helper
per slot neither provides general allocation nor bounds live storage correctly.
Ordinary Topal already defines checked terminating recursion. Qualify that
existing meaning in freestanding execution, with explicit reuse evidence.

## Direction and alternatives

Retain ordinary shared source analysis, functions, arbitrary-precision arithmetic,
effect order and checked termination. First identify a direct tail-recursive
entry with a natural countdown and a closed acyclic helper graph. Its return
branches may finish early. Every back edge passes the same borrowed region, if
present, and decreases the checked natural measure by a positive literal. An
inclusive nonnegative base bound must prevent decrement below zero. Secondary
scalar state may change; its range and storage proofs remain separate obligations.
This is an initial systems qualification, not a restriction on ordinary recursion.

Use a compiler-owned explicit traversal plan, followed by a separately qualified
runtime checkpoint. The checkpoint transfers every next-state scalar into a
disjoint owned bank, proves all retiring references dead, erases the old temporary
extent and installs the complete successor before the next source step. Input
evaluation and persistent byte effects keep their ordinary order. A source tail
call is not permission to reset storage containing a still-live value.

Unrolling grows code and total allocation with the workload. Native allocation
policy would move the subsystem algorithm outside ordinary Topal. A garbage
collector or general persistent graph allocator would introduce considerably
broader lifetime obligations. Raising arena/stack limits would retain the same
unbounded-growth defect. Checked tail traversal with explicit state transfer
provides a reusable foundation for allocation, VFS and process-memory algorithms.

## Source plan and proof boundary

The first stage inspects the shared checked program; it does not emit code or
admit a new executable profile. Bind the complete source text, every dependency's
identity/path/bytes, selected entry, parameter classifiers, countdown guard,
every tail edge, exact positive decrement, ordinary helper graph and inferred
owner reads/writes. Reject unknown forms, hidden captures/foreign calls,
missing/duplicate/unreachable functions, helper cycles, re-entry through helpers,
base-branch recursion, non-tail recursion and changed region arguments.

The source-only region selector supplies existing symbolic region/Nat parameter
types to the ordinary frontend. It performs no owner borrowing or native profile
admission. The plan also fingerprints every checked function, including literal
values, ordered expressions and arguments, using a versioned compiler-internal
encoding. This binds analysis inputs without making that encoding a public ABI.
Ordinary source still has its existing syntactic operand limit. Additional
state uses an ordinary package; the source-only selection gives initial and
recursive lookup the same ordered flattened signature. It reuses the existing
termination checker and checked Nat adaptation, with no new trusted proof or
change to ordinary package meaning. Qualified/defaulted/nested entry fields
remain unsupported by this selection.
The guard and back edges must refer to the original parameter storage, rather
than a shadowing local or decision payload with the same display spelling.

The initial guard is the ordinary ordered decision `measure <= bound`, where
`measure` is one Nat parameter and `bound` a nonnegative integer literal. Every
recursive edge uses `measure - step`, with `0 < step <= bound + 1`, retaining
the ordinary checked Int-to-Nat adaptation. Other branches may return without
recursion. The guard and all arguments execute exactly once in source order.
Termination follows from strict natural descent, not a trusted annotation or a
runtime instruction counter. For initial measure `m` and minimum step `d`, a
safe maximum back-edge count is zero when `m <= bound`, otherwise
`ceil((m - bound) / d)`; early return may reduce it.

A shape plan proves only that relationship and closed structural eligibility.
It is not an allocation bound, native stack certificate, authority grant,
source-origin attestation for an externally constructed IR, or permission to
emit a recursive freestanding object. Publication must rebuild the plan from
its exact shared frontend inputs and separately establish every missing proof.
Existing source profiles retain their exact schemas and refusal behavior.

## Runtime and lifetime obligations

The subsequent executable qualification needs an invariant for every carried
scalar, exact arbitrary-precision representations and evaluation costs, complete
helper stack bounds and a bound on the simultaneous live storage. Transfer
banks, pinned inputs/control, source work area and checkpoint stack must all
fit within the existing 65536-byte temporary and 4096-byte stack reserves.
Total work may depend on runtime inputs; live storage cannot grow per step.

Transfer roots are compiler selected; source gains no allocator or numeric-address
constructor. An outer suspended invocation's storage, borrowed owner and code
binding remain live. A bank cannot be erased while any argument, helper result,
view, destructor or cleanup obligation still depends on it. Transfer is complete
before retirement, including zero/negative/multi-limb values and aliases. Refusal
before source entry preserves the owner and histories. Failure after persistent
stores preserves those stores, reclaims all temporary domains and takes the
existing fatal path before any owner resumption. No rollback snapshot is used.

The initial view-bearing traversal has at most one invariant input view and no
view construction/window operation. It does not suspend, callback, publish a
view or return a resource. Logical helper/tail lifetimes and nonwrapping identity
history must remain valid despite physical storage reuse; a reused address is
not a fresh capability. Required total identity issuance must be admitted before
mutation, independently of the simultaneous live-view/frame bounds. Broader
view transport, mutually recursive groups, non-tail recursion, persistent source
graphs, destructors, concurrency and preemption require separate qualification.

Native lowering must represent the ordinary checked operations and the selected
checkpoint explicitly. LLVM tail-call optimization alone proves neither
termination nor reclamation. Debug evidence must identify optimized tail steps
and unavailable retired values accurately. Closed linked code/data, checkpoint
and runtime versions, source/native bindings, source effects, complete resource
bounds, physical erasure and trusted fatal recovery remain publication duties.

## Architecture pressure test

X86-64 implements the first native path. AArch64 and RISC-V constrain the common
contract: target-owned calling convention/alignment, pointer width and granule,
callee-save state, code identity and instruction/data ordering must be qualified
independently. The source plan expresses natural descent, owned values and
effects; it contains no register, instruction, stack address or pointer layout.
No other architecture becomes executable through this design.

## Four implementation stages

1. Specify the qualification and inspect actual checked source graphs, with
   positive and adversarial plan tests. This stage grants no executable profile.
2. Prove carried-value bounds and implement ordinary lowering/state transfer,
   checkpoint/runtime binding and independent guarded native components.
3. Bind a real owned allocator/metadata traversal to the kernel, preserving
   provenance, reservation, lifetime, refusal and fatal-cleanup laws.
4. Qualify exact committed source/native/VM/application inputs and publish
   complete evidence in the existing initial Step 4 PR.

The end state remains general allocation and persistent kernel subsystems,
followed by full pinned userspace/UAPI, Device Tree/drivers, networking and
isolation, Docker/Podman and VM hosting. A source-shape plan alone satisfies
none of those machine-execution acceptance requirements.
