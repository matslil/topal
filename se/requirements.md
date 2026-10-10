# Core language requirements

The keywords **shall**, **should**, and **may** express mandatory behavior,
recommended behavior, and permitted variation respectively.

## TOPAL-REQ-MODEL-001 — Unified object model

The language shall define values, types, functions, constraints, capabilities,
effects, interfaces, modules, patterns, and protocols within one explicit object
taxonomy and recursive construction model.

## TOPAL-REQ-SAFE-001 — Defined safe behavior

Every accepted safe program shall have defined behavior for every permitted
execution. Operations that cannot satisfy their contracts shall be rejected or
produce an explicitly typed result.

## TOPAL-REQ-TOTAL-001 — Explicit termination and failure

Functions shall be total by default. Non-success outcomes shall be represented
explicitly; productive infinite computation shall use declared generator or
external-suspension semantics.

## TOPAL-REQ-CONC-001 — Race and deadlock prevention

The language shall reject safe programs whose declared resources, tasks, and
protocols cannot establish freedom from data races and internal deadlocks.

## TOPAL-REQ-DETERMINISM-001 — Scheduling independence

All executions permitted for a program shall produce the same semantic result,
apart from observations explicitly declared as permitted nondeterminism.

## TOPAL-REQ-EFFECT-001 — Observable-effect accounting

Observable interactions and affected resource identities shall be represented
in contracts sufficiently to validate ordering, independence, and containment.

## TOPAL-REQ-RESOURCE-001 — Resource and memory safety

Safe code shall access storage only through valid layouts, locations, address
ranges, lifetimes, and access capabilities, with ordering consistent with the
declared hardware and memory semantics.

## TOPAL-REQ-GENERIC-001 — Preserved generic meaning

Exported generic functions shall retain the type relationships, capability
evidence, effects, and other contracts required for an importing tool to
instantiate them without source access or semantic weakening.

## TOPAL-REQ-SERIAL-001 — Canonical native interchange

Topal's native serialization shall define versioned type descriptions,
canonical encodings where required, streaming behavior, validation, and
deterministic rejection of malformed input.

## TOPAL-REQ-TOOLS-001 — Tool conformance

The compiler, interpreter, linter, and other language tools shall implement the
applicable formal specification rules and shall identify unsupported language
revisions rather than silently changing meaning.

## TOPAL-REQ-INTEROP-001 — Execution interoperability

For the same valid program, inputs, and declared external observations, the
interpreter and compiled result shall have equivalent semantic behavior.

## TOPAL-REQ-TRACE-001 — Verifiable traceability

Normative rules and functional tests shall reference stable specification IDs;
specification rules shall trace to the design goals and requirements they
realize.

## TOPAL-REQ-SHARED-001 — Reusable toolchain layers

Language tools shall consume reusable source, lossless syntax, semantic, and
diagnostic layers rather than derive incompatible private representations.
Shared source and syntax data shall retain stable byte ranges, trivia, malformed
input, and incomplete input needed by batch tools, editor services, custom lint
rules, and static debugging. Runtime tools shall correlate execution decisions
with the same stable source identities without making application semantics
depend on observation.

## TOPAL-REQ-BEST-PRACTICE-001 — Shared programming guidance

The repository shall maintain a versioned best-practice database from which
human guidance, agent decision information, and optional lint rules are
traceably derived. Entries shall have stable owned identities, explicit status,
classification, applicability, defaults, tags, provenance, and license.
Generated projections shall remain version controlled and shall be verified
against their authoritative inputs.

## TOPAL-REQ-LINT-001 — Contained configurable linting

The Topal linter shall consume shared versioned syntax and semantic views,
produce diagnostics compatible with the interpreter and compiler, support
configuration and scoped suppression by stable best-practice identity, and
apply only explicitly selected safe rectifications. External databases and
library-supplied rules shall be supported without granting ambient authority or
automatic execution merely because a package is installed.

## TOPAL-REQ-TRANSFER-001 — Protocol-governed external interaction

External and inter-component data transfer shall use capability-authorized
endpoints whose typed operations, completions, failures, effects, lifetime, and
legal protocol transitions remain explicit. Local calls, messages, stores,
networks, and devices shall share this foundation without erasing their
distinct ordering, atomicity, reliability, or addressing semantics.

## TOPAL-REQ-DATA-VIEW-001 — Safe layered data access

Messages, packets, frames, sequences, and addressed regions shall retain their
semantic boundaries and support recursively nested representations. Validated
views over shared data shall permit bounded-copy inspection and transfer while
tracking ownership, span dependencies, mutation invalidation, resource limits,
and device-memory obligations.

## TOPAL-REQ-STORE-001 — Explicit store guarantees

File, relational, graph, document, key-value, and object stores shall share
identity, authority, transaction, snapshot, change, consistency, durability,
replication, and failure concepts where applicable without being forced through
one query or byte-stream interface.

## TOPAL-REQ-TRANSPORT-BINDING-001 — Replaceable faithful bindings

An application service may be realized over multiple local, network, or device
transports. A binding shall preserve the service protocol or reject unmet
requirements, and shall expose transport-specific correctness properties such
as addressing, scope, transaction boundaries, retry safety, security,
completion ordering, and resource constraints.

## TOPAL-REQ-CONTRACT-001 — Relational contracts

Revisioned function and state contracts shall express pure total preconditions,
postconditions, effect bounds, implementation requirements, and invariants with
unambiguous scope. Calls and public transitions shall prove their obligations;
protected safety obligations shall not be discharged by unverified trust.

## TOPAL-REQ-EVIDENCE-001 — Typed evidence provenance

Semantic and implementation evidence shall retain property, subject, static
parameters, status, producer, assumptions, language revision, and optional
architecture-model identity. Ordinary or generated source shall not mint proof
status, and implementation evidence shall not change semantic results.

## TOPAL-REQ-RESOURCE-BOUND-001 — Composable portable bounds

The language shall represent and conservatively compose portable work, span,
allocation, peak-live, retained, stack, queue, transfer, code-size, and progress
requirements. Unknown evidence shall fail a hard requirement, while an explicit
preference may use a semantically correct fallback.

## TOPAL-REQ-EXCLUSIVE-001 — Safe reuse and scoped allocation

The checker shall infer invocation-local exclusivity and enforce explicit
consumption without exposing mutable references. Named allocation regions shall
prevent dependent values from escaping except through checked move, promotion,
copy, or independence evidence and shall clean up deterministically.

## TOPAL-REQ-CONC-SYNTH-001 — Synthesized concurrency mechanisms

The language shall expose bounded interaction policies and immutable published
snapshots while keeping atomics, locks, fences, memory orders, and reclamation
mechanisms out of portable source. The selected systems profile may expose only
the checked foundations governed by `TOPAL-REQ-SYSTEMS-SYNC-001`. Only a
verified selected implementation may publish nonblocking progress evidence.

## TOPAL-REQ-TRANSACTION-001 — Structured atomic state

Transaction domains shall isolate candidate state, publish one complete commit
or none, expose conflict and abort outcomes, define cancellation winners and
same-domain nesting, and reject un-staged observable effects. Cross-domain
atomicity and durability shall require exact checked provider evidence.

## TOPAL-REQ-TIME-001 — Explicit temporal observations

Clock identity, instants, absolute deadlines, periodic release, lateness, and
timeout races shall be explicit semantic values and trace observations.
Relative timeouts shall create and propagate one deadline. Physical timing
claims shall fail closed without approved implementation evidence.

## TOPAL-REQ-STATIC-FLOW-001 — Checked static-rate flow

Closed static-rate flow graphs shall have checked balance and causality,
deterministically derived schedules and finite buffers, and identical logical
traces under interpreted and optimized execution.

## TOPAL-REQ-EFFECT-HANDLER-001 — Typed lexical handling

Effect protocols shall resolve handlers lexically, require complete typed
implementations, subtract handled effects, and use affine one-shot resumptions
with deterministic cleanup. Multiple resumption shall require verified safety
evidence.

## TOPAL-REQ-SPECIALIZE-001 — Verifiable specialization

A hard specialization requirement shall be satisfied only by compiler-owned or
checked-artifact code-shape evidence. Implementation plans shall be typed,
reproducible, read-only outside the compiler, and unable to change program
meaning.

## TOPAL-REQ-LAYOUT-COMPOSE-001 — Compositional representation

Semantic shapes and multidimensional, blocked, component-organized, sparse,
view, and conversion layouts shall have checked coverage, arithmetic,
lifetimes, access, and canonicalization. Foreign layout descriptions shall
receive no unchecked ABI or address authority.

## TOPAL-REQ-INFOFLOW-001 — Confidentiality and integrity flow

Information policies shall verify their label lattice, propagate explicit and
implicit control flow through values, effects, and messages, and permit
declassification or endorsement only with exact unforgeable scoped authority.

## TOPAL-REQ-ARCH-EVIDENCE-001 — Opaque architecture-evidence seam

Hard target-dependent requirements shall consume typed provider evidence
without exposing target facts to ordinary semantic computation. Architecture
models, schedulers, devices, fault domains, and foreign ABIs shall use this
boundary without being inferred from it, and favorable cost evidence shall not
establish semantic or hardware legality.

## TOPAL-REQ-ARCH-MODEL-001 — Layered compositional architecture model

A versioned architecture model shall distinguish execution architecture,
microarchitecture, ABI/platform, compute elements, memory and translation,
cache/coherence, transfer and communication mechanisms, interconnect topology,
board composition, and calibration. Reusable component models and board
overlays shall compose by stable identity without treating vendor, ISA,
processor, operating system, and deployment board as interchangeable.

## TOPAL-REQ-ARCH-COST-001 — Conditional multidimensional costs

Architecture costs shall retain their units, conditions, provenance,
applicability, uncertainty, and independent latency, throughput, code-size,
memory, transfer, resource-occupancy, energy, and predictability dimensions.
Unknown shall remain distinct from zero, unbounded, and unsupported. Hard
compatibility, capacity, safety, and timing constraints shall be checked before
any preference or weighted comparison.

## TOPAL-REQ-ARCH-MEMORY-001 — Explicit memory and communication topology

The model shall describe heterogeneous address spaces and memories, access
sizes and alignment, banks and controllers, virtual translation and IOMMUs,
cache organization and coherence scope, barriers and maintenance, DMA and
hardware channels, and shared interconnect routes and bottleneck resources.
Cache utilization, false sharing, contention, and path cost shall be derived
from model and program-plan facts rather than asserted as context-free hardware
constants.

## TOPAL-REQ-ARCH-COMPUTE-001 — Heterogeneous compute description

The model shall represent CPU, GPU, NPU, DSP, and fixed-function compute
elements without requiring one common ISA. It shall distinguish instruction
legality from conditional latency, throughput, encoding size, register and
pipeline resource use, and shall describe scalar, SIMD, scalable-vector, SIMT,
tensor, matrix, synchronization, launch, and completion facilities where
applicable.

## TOPAL-REQ-ARCH-TARGET-001 — Explicit generic, specific, and cross targets

A compiler after it admits multiple architecture profiles shall resolve its
implicit target to a recorded generic baseline for the compilation host's
architecture family and platform without silently enabling that host
processor's optional features. Explicit selection shall support a more specific
compatible target and a foreign cross target. Native detection and runtime
multiversioning shall be explicit qualified modes with a baseline fallback,
recorded assumptions, and no implicit foreign runtime or dynamic-loader
dependency. An incremental compiler shall continue to reject an unqualified
target rather than approximating this requirement.

## TOPAL-REQ-ARCH-PROVENANCE-001 — Validated Topal model identity

Architecture packages shall be declarative, total, deterministic,
authority-free Topal descriptions validated through shared target-independent
syntax and semantics before backend selection. Canonical identity shall cover
schema and language revisions, effective facts, imports, overlays, provenance,
and calibrations. Qualification shall separately report schema,
implementation, cost, and platform coverage; application source shall neither
mint physical evidence nor edit a diagnostic projection into proof input.

## TOPAL-REQ-ARCH-DIAGNOSTIC-001 — Explainable missing target information

When missing or uncertain program, target, board, calibration, or workload
information changes plan selection or prevents a hard guarantee, the compiler
shall identify the affected transformation, alternatives, conservative result,
model and policy identities, and the authorized source capable of supplying the
fact. It shall not suggest that ordinary source assert hardware truth or
evidence status.

## TOPAL-REQ-SYSTEMS-PROFILE-001 — Isolated freestanding systems profile

The optional `systems` feature shall add a closed privileged vocabulary without
changing portable Topal meaning or adding new grammar. It shall be valid only
for a qualified freestanding root artifact; selection alone shall grant no
runtime authority. Non-systems dependencies shall neither acquire systems
vocabulary nor machine authority through composition.

## TOPAL-REQ-SYSTEMS-ENTRY-001 — Typed special entry and disposition

Boot, secondary-processor, exception, interrupt, syscall, machine-critical,
and resumed-thread entry shall use typed static entry declarations rather than
ordinary calling conventions or source assembly. The compiler/backend shall
own physical frames, stacks, prologues, epilogues, security state, unwind data,
and return mechanisms. Source handlers shall receive only context-legal
capabilities and shall return one checked disposition.

The initial external-interrupt increment shall consume one processor context
into an affine local-notification send/wait session, admit only the matching
provider-created entry event, require consuming completion authority before
resume, restore the exact prior local maskable-interrupt state, and bind the
send, observation, entry, completion, resumption, and wait result to one
monotonic event identity. Vectors, controllers, frames, acknowledgement state,
wait instructions, and machine return mechanisms shall not be source values.

## TOPAL-REQ-SYSTEMS-OBSERVATION-001 — Declared systems nondeterminism

External events and concurrent winner selection shall be observable only
through declared source capabilities or invariant-preserving protocols with
typed results, ordering constraints, and trace identities. Portable code shall
remain deterministic absent such an input or effect. Compilers shall not
invent, discard, merge, or speculate a required observation.

The initial systems time increment shall borrow one qualified provider-created
monotonic clock through an admitted processor context. Each `now` shall return
an immutable instant retaining that exact clock identity and shall record a
distinct ordered observation. Accepted observations from one clock shall not
decrease. Counter representation, address, register, instruction, frequency,
calibration, enablement, and wrap state shall remain provider-private; wall
clock, periodic release, suspend, migration, SMP, and physical timing
guarantees remain unavailable.

The initial deadline increment shall construct one same-clock absolute
deadline from an instant and exact duration, consume it into one affine armed
event, and return the temporarily consumed processor context only after the
matching typed entry completes and resumes. Delivery shall record scheduled and
observed same-clock instants and shall never precede the deadline; late and
already-expired delivery shall preserve the original absolute deadline.
Comparator, route, vector, controller, acknowledgement, and wait mechanics
shall remain provider-private. Cancellation, periodic release, scheduler
integration, bounded latency, SMP delivery, suspend/migration behavior, and
userspace timer ABI remain unavailable.

## TOPAL-REQ-SYSTEMS-MACHINE-001 — Sealed semantic machine providers

Privileged operations shall be closed provider functions whose semantic state
transition, authority, resource identity, context, fault, ordering, target
evidence, and model behavior are explicit. Common operations shall state
architecture-independent meaning; facilities without one honest common
contract shall remain target-qualified. No source facility shall admit an
arbitrary instruction, register, intrinsic, or privileged operation.

## TOPAL-REQ-SYSTEMS-MEMORY-001 — Address, mapping, and fault isolation

Physical, kernel virtual, user virtual, device, DMA, and firmware-source
addresses shall retain distinct resource-qualified identities. Mapping and
location authority shall arise only through checked construction and shall
retain bounds, layout, rights, cache/order policy, owner, and lifetime. User
transfer and other admitted recovery shall use generated closed fault scopes;
source shall not dereference a user candidate or name a recovery instruction.

An entered bootstrap context shall refine through one exhaustive, affine
boot-memory transition. A qualified provider shall validate its opaque native
handoff; conservatively normalize overlapping source ranges; retain source
classification, ownership, reservation, and provenance; subtract every live
bootstrap object; and produce physical-frame authority only for complete
allocatable pages. Failure shall retain only authority needed for fatal boot
termination. Portable source shall observe neither raw firmware layouts nor
numeric addresses, and unknown or unsupported input shall not become
allocatable memory.

A memory-described bootstrap context shall refine through one exhaustive,
affine frame-allocator transition. The resulting allocator shall exclusively
own normalized allocatable-frame authority. Each successful allocation shall
produce a nonoverlapping opaque extent carrying allocator identity, frame
count, alignment, and provenance but no numeric address or access authority.
Release shall consume an extent back into its originating allocator. Invalid,
exhausted, cross-allocator, double-release, escaping, or disposition-with-live-
extent behavior shall fail closed.

Mapping an owned physical-frame extent for kernel access shall consume that
extent into one affine mapping with opaque provider-selected kernel-virtual
identity, explicit rights, execution policy, normal-memory kind, owner,
lifetime, and translation evidence. Only a live mapping shall authorize
bounds-checked plain byte access. Unmapping shall consume the mapping and
return exactly its original extent only after access authority is revoked;
frame release while mapped, use after unmap, cross-provider use, writable
execution, or disposition with a live mapping shall fail closed. No source
operation shall expose or equate its physical and virtual representations.

Bootstrap translation replacement shall use an exclusive affine update rather
than source-visible page-table records. Beginning an update shall borrow the
live allocator, reserve provider-selected backing, and snapshot an exact
semantic template. Commit shall consume a validated update into one inactive
translation space. Activation shall consume that space and current translation
authority, perform the provider publication and completion protocol, and
return a refined context owning the replacement. The initial template shall
preserve the qualified bootstrap coverage and permissions without widening
authority. Builder escape or duplication, use after commit, activation before
commit, wrong-provider activation, backing reuse while owned, or recoverable
continuation after indeterminate activation shall fail closed. Source shall
observe no table level, descriptor, physical backing address, activation
register, or target maintenance instruction.

An active translation change shall consume the active context into one
exclusive affine edit. Mapping shall consume frame ownership into a provisional
mapping that grants no access until commit publishes and completes the
target-qualified update. Unmapping shall consume the live mapping into a
provisional returned extent that cannot be reused until commit removes the
translation and completes invalidation. Successful commit shall consume the
edit and return a refined active context; failed or indeterminate commit shall
be fatal-only in the initial slice. Source shall not select or observe virtual
addresses, table structure, descriptor state, invalidation addresses,
processor masks, registers, or instructions. The initial policy shall admit
only one provider-selected, normal, read-write, non-executable kernel mapping.

An ordinary kernel-owned region shall provide bounds-checked plain byte access
without exposing a machine address. Load and store shall borrow a live region,
release shall consume it, and no access shall silently acquire volatile,
atomic, device, DMA, firmware, or user-memory behavior. Unsupported dynamic
bounds shall fail closed rather than become unchecked access.

## TOPAL-REQ-SYSTEMS-SYNC-001 — Checked shared-state synchronization

Systems shared state shall use typed atomic locations, affine critical scopes,
or verified higher-level protocols. Atomic orders shall define language
relations independently of target instructions and shall distinguish CPU,
device, DMA, translation, cache, and instruction domains. Plain conflicting
access shall remain a rejected race; masking one producer shall not imply
exclusion of another.

The first atomic-location increment shall consume one exclusively owned
ordinary region into one aligned unsigned machine-word location in the
`cpu-shared` domain. It shall admit compare/exchange with explicit
`AcquireRelease` success and `Acquire` failure orders followed by an `Acquire`
load, and shall require consuming `atomic end` before returning the original
region for plain use or release. The word width and implementation strategy
shall come from qualified target evidence. Source shall not observe an
address, register, opcode, exclusive-monitor state, or retry loop, and the
increment shall not claim MMIO, device, DMA, firmware, or user-memory
atomicity.

The initial local-maskable-interrupt scope shall consume and refine the
current processor context, retain the exact prior interrupt state in opaque
affine restoration authority, and require matching restoration before escape,
suspension, blocking, processor transfer, or disposition. Nested scopes, when
admitted, shall restore in last-in-first-out order. Source shall not observe
target flags, registers, masks, or instructions.

## TOPAL-REQ-SYSTEMS-CONTEXT-001 — Opaque scheduler context transfer

Running and suspended execution contexts shall be opaque linear resources
owning their stack and target state. Only a qualified context-transfer
operation may consume one running context and resume one validated suspended
context. Register slots and machine continuation addresses shall not be source
values, and transfer shall not be modeled as an ordinary returning call.

The initial context-transfer increment shall consume one checked exclusive
kernel-owned region into one suspended context for a static typed kernel-thread
entry on `InitialProcessor`. One transfer shall suspend the bootstrap caller,
run that entry on the new stack, permit terminal retirement only at a
statically verified point with no live obligations, resume exactly the caller,
and require consuming reclamation of the retired context and stack before
bootstrap disposition. The active address space shall remain unchanged and
local maskable interrupts shall be disabled across transfer. Registers, stack
pointers, frame layouts, continuation addresses, and save/restore instructions
shall remain provider-private. Preemption, scheduler policy, multiple threads,
migration, SMP, user contexts, general cancellation, extended-state switching,
TLS/per-CPU switching, stack growth, and cross-transfer unwind remain
unavailable.

## TOPAL-REQ-SYSTEMS-CONTEXT-002 — Cooperative context handoff

Context transfer shall support a symmetric cooperative handoff in which a
later-resumed continuation receives exactly one affine suspended or retired
peer outcome. A closed entry protocol may statically refine that outcome only
when the checker proves the selected continuation's behavior. Retirement shall
consume one running context and one matching suspended target, resume that
target, and require consuming reclamation of the retired stack.

The first executable extension shall construct two contexts from disjoint
checked stacks and enforce this source-selected FIFO sequence: the cooperative
worker hands back once, the terminal worker retires and is reclaimed, then the
cooperative worker resumes, retires, and is reclaimed. Runnable selection
shall remain ordinary source policy rather than provider behavior. The
extension shall retain one processor and address space with local interrupts
disabled and shall not imply preemption, blocking, priorities, dynamic run
queues, cancellation, migration, SMP, user contexts, TLS/per-CPU switching,
extended-state switching, stack growth, or cross-transfer unwind.

## TOPAL-REQ-SYSTEMS-DEVICE-001 — Device and DMA protocol ownership

Device locations shall bind layouts to legal access widths, side effects,
reserved-bit policy, ordering, and a live device session. DMA shall transfer
buffer ownership through explicit CPU, prepared, device, completion/failure,
and reclamation states with mapping, cache, notification, and completion
evidence. Coherent hardware shall not erase the protocol or evidence boundary.

## TOPAL-REQ-SYSTEMS-ARTIFACT-001 — Host-independent kernel artifact

A systems artifact shall use a distinct qualified target profile, generated
special roots, semantic placement, explicit relocation/code-model policy, and
atomic validated publication. It shall have no implicit host syscall,
allocator, process startup, libc, dynamic loader, foreign runtime, or ordinary
process-termination path. Boot packaging shall be a separately qualified
artifact adapter.

## TOPAL-REQ-SYSTEMS-QUALIFY-001 — Cross-architecture and target evidence

Every systems element shall have architecture-independent semantic and
negative tests, an abstract model transition or explicit model limitation,
target/provider provenance, artifact inspection, and emulator or hardware
evidence. Common semantics shall be reviewed against x86-64, AArch64, and
RISC-V; executable qualification may remain target-specific and shall fail
closed elsewhere.

## TOPAL-REQ-OPT-FEASIBILITY-001 — Constraints before optimization preference

An optimizer shall prove semantic preconditions and target legality and shall
enforce language guarantees, safety, capacity, timing, security, and explicit
user limits before comparing performance preferences. No optimization level,
cost estimate, or pass override shall admit an infeasible plan.

## TOPAL-REQ-OPT-POLICY-001 — Multidimensional deterministic selection

Optimization policy shall preserve cost units, conditions, provenance,
uncertainty, and unknown values. It shall eliminate proved dominated plans,
compare remaining plans by explicit ordered goals, and use a stable final
tie-break without collapsing unlike dimensions into an unversioned scalar
score. Identical recorded inputs shall produce the same plan.

## TOPAL-REQ-OPT-PROFILE-001 — Standard optimization profiles

The compiler shall expose versioned `O0`, `O1`, `O2`, `O3`, `Os`, and `Oz`
profiles with documented ordered intent. `O0` shall perform no optional Topal
rewrite or specialization. A selected profile shall not silently enable build-
host processor features, and artifacts shall record the effective profile and
plan revision.

## TOPAL-REQ-OPT-CONTROL-001 — Explicit target, goal, and pass controls

Compiler controls shall distinguish target selection from optimization intent,
support generic, specific, native, board, model, and foreign selections when
qualified, and provide ordered goals, hard limits, stable per-pass enable and
disable controls, isolated-pass selection, and optimization listing. Unknown,
incompatible, contradictory, and unqualified selections shall fail explicitly.
Mandatory correctness work shall not be disableable.

## TOPAL-REQ-OPT-EXPLAIN-001 — Explainable conservative decisions

The compiler shall be able to report its effective target and policy, enabled
passes, relevant candidates, preconditions, evidence, missing facts,
rejections, and decisions. A missing fact shall provide no favorable
preference. Diagnostics shall distinguish source proof from trusted target-
model input and identify a legitimate provider of the missing information.

## TOPAL-REQ-COMPILER-001 — Correct native compilation

At its unoptimized reference level, the compiler shall preserve the observable
semantics of every accepted source program and shall reject every feature
outside its explicitly versioned implementation subset. Optional optimization
shall not repair, weaken, or change source meaning.

## TOPAL-REQ-NATIVE-PLATFORM-001 — Freestanding platform boundary

A native Topal executable shall depend only on the interfaces explicitly
selected for its target. It shall not implicitly require a C or C++ standard
library, foreign-language runtime, startup object, or dynamic loader. Direct
operating-system mechanisms shall be isolated in a versioned platform layer
with typed inputs, results, failures, effects, and target identity.

## TOPAL-REQ-NATIVE-ABI-001 — Target-qualified ABI lowering

Every machine-code artifact shall record its target triple, data layout,
object format, CPU baseline, platform ABI, and Topal private-ABI revision.
Target-specific calling and layout decisions shall be derived only from a
qualified target implementation. A foreign boundary shall use an explicit
adapter and shall not expose a private Topal aggregate representation.

## TOPAL-REQ-NATIVE-ARTIFACT-001 — Validated library code-generation metadata

A compiled library shall retain the semantic GEIR and public interface
information needed for source-independent checking and instantiation, plus
target-qualified native slices and reproducible dependency, toolchain,
provenance, and debug mappings. LLVM IR or bitcode shall not become the stable
library compatibility boundary.

## TOPAL-REQ-NATIVE-LIBRARY-001 — Sealed standard-library dynamic boundary

A qualified standard-library native slice shall use `topal-library/1` wrappers
and expose one versioned entry table rather than the compiler-private ABI or a
foreign ABI. Linux and FreeBSD slices shall use the ELF SONAME
`libtopal-std.so.1`; Windows slices shall use `topal-std-1.dll`. An application
that selects `std` shall have at most one direct Topal dynamic dependency. The
standard library shall internalize its implementation modules and shall not
propagate their links as application dynamic dependencies.

The compiler shall validate the selected slice's source-interface identity,
library major version, ABI revision, target/object-format/layout identity,
export map, and digest before native linking. It shall record the one selected
slice in artifact metadata and reject a slice which adds a C/C++ runtime or
another Topal dynamic-library dependency.

## TOPAL-REQ-C-ABI-001 — Explicit checked C boundary

The first foreign-language boundary shall be a separately selected,
target-qualified C ABI access library. A generated access library shall bind
human-readable Topal ABI declarations, canonical machine metadata, the parsed
C interface, and the exact linked binary by reproducible identity. Checked
adapters shall prevent the private Topal value or calling representation from
crossing the boundary, and unsupported declarations shall fail closed.

## TOPAL-REQ-NATIVE-DEBUG-001 — Source-level native debugging

An unoptimized native artifact shall retain sufficient source, scope, function,
parameter, local-value, type, and unwind information for deterministic GDB
breakpoints, stepping, backtraces, and value inspection. Optimized artifacts
shall state their reduced variable-availability guarantees.

## TOPAL-REQ-LLVM-001 — Qualified LLVM use

The compiler shall version-check LLVM, verify every generated module, delegate
target-independent and target-dependent transformations only where LLVM's
semantics match Topal's proved facts, and record the disposition of applicable
LLVM facilities. A deferred or deliberately unused facility shall have a
documented reason.
