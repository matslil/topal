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

The first executable user-transfer refinement shall remain exclusive to the
systems feature. It shall derive one affine scope from the exact live syscall
entry, thread, address space, and ABI; consume and return that scope through
every transfer outcome; and forbid syscall disposition while the scope is
live. A bounded `copy_to_user` shall return complete extent or structured fault
with copied-prefix progress without converting its untrusted ABI candidate to
a Topal reference.

The initial x86-64 qualification shall implement native Linux `uname` number
63 with a source-owned `new_utsname`, verify a successful copy, and force a
copy across an unmapped guard page. Only generated copy sites shall recover
from the matching page fault and the syscall shall return `-EFAULT`; other
faults shall remain fatal.

The input-direction refinement shall keep destination staging inaccessible
until a selected layout has been copied completely and validated. A bounded
`copy_from_user` shall return a complete value and extent or a structured fault
with copied-prefix progress and no partial value. The initial x86-64
qualification shall implement native Linux `nanosleep` number 35 for one
zero-duration native `timespec`, verify an immediate successful result, and
force a second read across the same unmapped guard page. Only the exact
generated read site shall recover and return `-EFAULT`; other faults shall
remain fatal. Nonzero sleep, remainder output, interruption, restart,
arbitrary layouts and extents, concurrent mapping changes, general UTS
namespaces, and core-language exception handling remain unavailable.

The initial opaque user-address-space refinement shall consume one qualified
three-frame physical extent into a prepared user address space with fresh
identity. User-state preparation shall consume that address space into the
suspended user context. The provider shall use only its extent for the
executable page, non-executable stack, and user leaf-translation backing; fixed
hidden physical user pages are forbidden. Terminal completion shall revoke and
invalidate every derived user mapping and return a revoked address space.
Reclaim shall consume only that revoked state and return the exact indivisible
extent before source release. Source shall observe no physical or virtual
address, PTE layout, or target return state.

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

The bounded multi-frame qualification shall remain exclusive to the systems
feature and shall add no address or pointer conversion. The initial x86-64
profile shall admit only static requests for one, two, three, or exactly 503
contiguous base frames with unit alignment and return every request as one
indivisible affine extent. The exact 503-frame payload request shall occur only
after the earlier extent is returned and shall be selected wholly within one
normalized E820 RAM run while excluding reservations. Its mapping-resource
refinement shall admit exactly one nested unit metadata frame, prove the two
complete intervals disjoint, observe the pair once, release metadata first,
and then release the payload. Other counts, non-unit alignment, dynamic
requests, extent splitting, other simultaneous source-owned shapes, reversed
release, and disposition with either extent live shall fail closed.

The bounded static-ELF leaf-publication refinement shall consume the exact live
payload and metadata extents into one opaque construction owner. It shall
publish all 503 ordered page-to-frame pairs in initial RW/NX state inside an
inactive translation object, keep the remaining target-table capacity absent,
and expose no target representation. Success shall be observed once before
complete revocation. Revocation shall return metadata only after clearing the
complete object and completing target ordering; payload reclaim shall follow
metadata release and return the original indivisible extent. Partial
publication, wrong order or identity, duplicate transitions, early release,
and disposition with live construction ownership shall fail closed.

The bounded disjoint-extent refinement shall admit exactly one additional
static one-frame, unit-aligned source-owned extent while one prepared initial
user address space owns its three-frame backing. The allocator and provider
shall prove complete-interval non-overlap without exposing an address. Source
shall release the additional extent before user-context binding. A third live
extent, another request shape, overlap, backing release, or disposition with
the additional extent live shall fail closed.

The initial prepared user address space shall admit one sealed `probe-guard`
mapping round trip over that disjoint extent. The provider shall install RW/NX
user authority, confirm the fixed byte write/read observation, revoke the
mapping, complete target-required invalidation, and return the exact extent
before reporting success. Wrong provenance, overlap, partial access, stale
translation, duplicate verification, or user-context binding before release
shall fail closed. No numeric address shall become source-observable.

The bounded affine user-mapping refinement shall separate that transaction
into map, fixed-byte store, fixed-byte load/verification, and unmap operations.
Each operation shall consume the preceding mapping refinement. Map shall
consume the exact disjoint extent; unmap shall return that same extent only
after revocation and target completion. Release, user-context binding,
duplicate use, reordering, or recoverable disposition while a mapping is live
shall fail closed. Layout, rights, byte value, addresses, and target
translation mechanics shall remain sealed.

The sequential mapping-generation refinement shall admit exactly two complete
lifecycles over the same returned extent. Each map shall create a fresh opaque
identity. The second map shall begin only after first-generation revocation and
target completion; stale or cross-generation refinement use, identity reuse,
release between generations, a third generation, or user-context binding
before the second release shall fail closed. Simultaneous mappings and
persistent image policy remain excluded.

The affine user-mapping permission refinement shall require each stored RW/NX
generation to pass through a distinct read-only/NX refinement before load and
verification. Protection shall preserve mapping, address-space, and backing
identity and contents, remove only write authority, and complete target stale-
translation revocation before success. Store after protection, load from stale
writable authority, permission widening, skipped protection, or recoverable
failure after update publication shall fail closed.

The affine user-mapping executable refinement shall require each verified
RO/NX generation to pass through a distinct executable RO/X refinement before
unmap. Promotion shall preserve mapping, address-space, backing, placement,
and content identity, shall never restore write authority, and shall complete
target translation and instruction-observation requirements before success.
Promotion from writable or unverified authority, W+X authority, skipped or
duplicate promotion, stale refinement use, or recoverable failure after
publication begins shall fail closed.

The sealed user-mapping role refinement shall classify the first verified
generation as immutable executable code through the approved RO/NX-to-RO/X
promotion and the second as immutable non-executable data through an explicit
RO/NX-preserving seal. Cross-role substitution, executable promotion of the
data generation, data sealing of the code generation, unclassified unmap,
unauthorized write-authority restoration, or W+X authority shall fail closed. Both roles
shall remain opaque and shall be revoked before frame return.

The writable user-data refinement shall alternatively consume the exact second
verified RO/NX generation into an opaque RW/NX writable-data role. It shall
preserve mapping, address-space, backing, placement, content, and user-access
identity; restore only write authority; prove executable authority remains
absent; and complete target translation observation before success. Use on the
code generation, use before verification, identity substitution, W+X,
source-visible permissions, or unmap through stale authority shall fail closed.

The bounded resident-image refinement shall allocate a distinct two-frame code
extent and one-frame data extent, retain finalized executable-code ownership
while constructing immutable data, and prove both mappings coexist with
pairwise-disjoint backing and sealed placements. Data unmap and release shall
precede whole-code-extent unmap and release; user preparation shall require
both mappings absent. A third mapping,
backing reuse, swapped placement or role, cross-role unmap, code-first teardown,
release while mapped, or recoverable disposition with either mapping live shall
fail closed. Provider-private placement and revocation mechanics shall remain
unobservable.

The bounded multipage-executable refinement shall map both frames of the code
extent at adjacent virtual base pages, copy and verify the complete external
segment, remove write authority from both leaves before granting either execute
authority, and revoke and return the extent only as a whole. The initial x86-64
entry and successful terminal path shall execute from the second page. Exact
return shall precede one
`TOPAL_KERNEL_USER_IMAGE_MULTIPAGE_EXEC_OK` marker. Partial publication,
single-leaf permission change or invalidation, W+X authority, entry outside the
two-page extent, partial unmap, or split return shall fail closed.

The bounded static-ELF planning refinement shall parse the complete ELF64 and
program-header table before exposing any load fact to artifact construction.
It shall reject wrong class, data encoding, version, type, machine, header
sizes, table bounds, program-header count, interpreter presence, interval or
arithmetic overflow, file-out-of-bounds data, incongruent or invalid
alignment, overlapping rounded load pages, W+X, wrong four-role ordering,
entry outside R/X, executable stack, TLS outside R/W, RELRO outside R/W, and
an x86-64 span exceeding 512 pages or one aligned 2 MiB leaf-table window.
Success shall retain only immutable checked interval, permission, entry, TLS,
RELRO, and stack facts. It shall grant no frame, mapping, copy, or entry
authority and shall not make the planned image runtime-selectable.

The bounded static-ELF page-realization refinement shall consume only a
successful immutable load plan. It shall reject nonadjacent load-page roles and
derive one future affine frame request equal to the complete page count, one
ordered virtual-page/frame-ordinal record per page, initial RW/NX construction,
and exact entry-stage RO/NX, RO/X, or RW/NX permission. It shall retain RELRO
pages RW/NX inside the writable role until the exact checked process protection
transition seals them RO/NX, require complete-span zeroing before the four exact
file copies, prove complete page coverage and no W+X state, and retain no
allocation, mapping, copy, permission-publication, or entry authority.

The bounded external static-process qualification shall accept only the exact
approved executable digest and coherent resident-header variant. X86-64 entry
shall use the complete final Linux initial stack, establish a fresh x87/SSE
context, preserve all non-result syscall registers and provider callee-saved
registers, and restore or scrub every private register, stack, translation-
status, and process-context value before rollback. Qualification shall execute
the pinned BusyBox `echo` applet through its one-byte newline and zero exit
without committing its bytes and shall not generalize acceptance to another
executable, stack shape, argument vector, or syscall sequence merely from that
evidence. Separately specified systems refinements may extend those bounds
only with their own checked contracts, error cases and physical qualification.

The reusable static-process refinement shall accept one through eight copied
arguments occupying at most 256 bytes including terminators for the sealed
executable. It shall preserve runtime payload bytes, reject inaccessible and
oversized inputs before image consumption, erase staging on rejection, and
return the original image owner and its state unchanged. Committed replacement
shall construct an ABI-aligned stack from the checked aggregate, reset image
and thread state, preserve process identity and eligible descriptions, and
reclaim all resources after termination. Startup outputs shall be validated by
mapping identity and complete extents rather than argument-dependent numeric
addresses. Console writes shall stage zero through 256 request-selected bytes
for standard output/error, return declared descriptor/range/size errors without
partial publication, and preserve the syscall register contract. Termination
shall retain the low eight bits of the native status and perform the same
cleanup for successful and nonzero application results. Qualification shall
cover positive and negative transfers, multiple applets and payload values,
observable status, and complete rollback. Native vectors, registers and stack
mechanics shall remain provider-private.

A separately qualified reusable file-description refinement shall distinguish
descriptor slots/flags, shared descriptions with rights/status/position, and
immutable backing identities. It shall use the declared finite descriptor
capacity, lowest-slot allocation, checked user transfers and truthful native
limit queries. Aliases shall share position, invalid-source replacement shall
preserve its target, and a last close shall destroy one description exactly
once. Immutable file data shall come from a retained validated boot-resource
view rather than process-modified loaded bytes. Exec failure shall preserve
table state; successful exec shall close only eligible close-on-exec slots;
terminal cleanup shall release every remaining reference and erase private
table storage. Qualification shall cover error atomicity, aliasing and reuse,
metadata, read/seek/EOF, exec filtering and complete external file consumption.

A separately qualified process-signal registration refinement shall retain
shared action and blocked-mask state, normalize native flags and unmaskable
signals, preserve input/output overlap semantics and distinguish rejection
before commit from output failure after commit. Registration shall retain
handler/restorer fields only as opaque ABI values. Exec shall preserve ignored
handlers and the blocked set while clearing action metadata and other handlers;
terminal reclamation shall erase the complete state. Positive and negative
clean cases and actual external setup calls shall qualify these services;
asynchronous delivery, pending queues, frames and return/restart behavior shall
remain separately tracked requirements.

A separately qualified read-only namespace refinement shall retain immutable
objects, process cwd and independently opened descriptions. It shall copy
native names through checked live user mappings, resolve components and
relative directory authority, distinguish missing/non-directory/invalid cases,
preserve allocation and lookup error order, and erase private copies on all
paths. Opens, metadata, directory records and file reads shall agree on object
identity and backing. Cwd shall survive exec. Qualification shall cover error
atomicity, independent opens and aliases, native directory-record boundaries,
cwd/exec lifetime and external file/directory consumers. Writable storage,
links, mounts and broader VFS behavior remain separately required.

A separately qualified realtime-seconds refinement shall obtain actual epoch
seconds from a declared platform clock, validate coherent calendar acquisition
and conversion bounds, and publish the same returned/native-output value only
after complete writable user checks. Null output shall be a pure query; copy
faults shall not partially publish. Private staging shall be erased. Calendar
and physical tests shall cover epoch/leap/century boundaries, actual platform
dates, query/copy behavior and rollback. No monotonic-uptime substitution or
application-specific timestamp shall implement realtime observation.

A reusable prepared-root identity refinement shall observe retained real and
effective user/group identity independently of ordinary call order and pathname
operations. Idempotent root selection shall preserve the same identity and all
other process resources. Authority shall come from the prepared process owner,
not mutable user memory. Tests shall cover repeated/reordered calls, native
integer width, exec inheritance, failure independence and terminal erasure.
Other identity changes remain separately required and unqualified.

A reusable exec-environment refinement shall transport caller-owned byte
aggregates without chosen key/value or argv-zero pathname matching. It shall
admit empty/null environments, preserve duplicates/empty entries, enforce its
declared vector/byte bounds and native copy/error ordering, erase failed
prefixes, and preserve all process resources before commit. Fresh stack vectors
and payloads shall be derived from copied offsets, with payloads isolated from
stack growth (above entry RSP in the initial x86 binding) and an execution name
independent of argv[0]. Qualification shall cover
copy/boundary errors, maximum transport, downward stack growth, state on failed
exec, changed cwd/exported environments and complete reclamation.

A repeated static-exec refinement shall admit successive image replacements
under the same retained process/executable owner after each qualified startup,
without depending on the original shell command phase. Each transition shall
validate current backing, copy before commit, preserve process resources,
filter/reset image-local state, construct fresh startup/random identity and
erase staging. Failed requests shall preserve the current generation; terminal
reclamation shall release retained authority exactly once. Qualification shall
execute multiple replacements, later-generation failed-prefix retries,
repeated descriptor/signal/cwd inheritance and complete rollback. Broader
mapping and executable selection remain separately required.

A reusable prepared-process observation refinement shall expose stable
PID/TID/parent and immutable system-information facts repeatedly and independently
of application call order after qualified startup. It shall import private
prepared identity, ignore unused native arguments, preserve non-result registers
and process state, check complete writable native records before publication,
retain no caller pointer and preserve identity across exec. Tests shall cover
repetition/reordering, native record bytes and faults, replacement shells and
complete reclamation. Broader process and UTS namespace behavior remains open.

The immutable boot-root open refinement shall decode ordinary native access
intents with its qualified modifiers while retaining the read-only filesystem
view. Complete name copying and free-slot checks shall precede walking; lookup,
relative authority and object-type errors shall precede denial of write access
to existing objects. Rejection shall publish no handle, preserve process state
and erase private name copies. Tests shall cover access/error ordering,
exhaustion, preserved inherited resources and an interactive replacement shell
using ordinary serial prompt/read/command cycles. Broader filesystem mutation,
process creation and terminal/job-control authority remain separately required.

The bounded static-process identity adapter shall expose the existing sealed
PID-one fact for one ordered `getpid` observation. It shall require the exact
live process, executable continuation, active checked address space, retained
stack, and header-selected predecessor; return one without retaining native
arguments or mutable identity state; and preserve complete terminal rollback.

The bounded static-process child-disposition adapter shall accept one complete
readable request after the sealed process-identity observation. It shall
require executable handler, return-trampoline, and continuation facts, the
approved child-status policy and blocked set, active checked address space,
retained stack, empty prior disposition, and exact predecessor phase. It shall
copy and validate the complete request before retaining only private process
state, return success, and validate and clear that state at terminal rollback.
It shall expose no native layout in portable source and shall grant no signal
generation, delivery, frame, handler-execution, restart, or general signal
authority.

The bounded static-process parent-identity adapter shall expose the sealed
absence of a parent for one ordered observation after child-disposition
installation. It shall require the exact live process, executable
continuation, active checked address space, retained stack, live disposition,
and predecessor phase; return zero without process lookup or retaining native
arguments or mutable identity state; and preserve the disposition lifecycle
and complete terminal rollback.

The bounded static-process empty-input shell adapter shall admit one ordered
publication of the established system identity, one sealed root current-
directory publication, one terminal-capability miss, default observations for
interrupt, quit, and termination dispositions, installation of one ignored
quit disposition, and one sealed end-of-input read. It shall validate complete
writable outputs, copy and retain no user request pointer, preserve process-
owned dispositions until terminal clearing, mutate no input destination on the
zero result, and require the exact live process, predecessor phase, executable
continuation, active hierarchy, retained mappings, and complete rollback.

The bounded initial-configuration adapter shall recognize exactly the sealed
absolute path `/etc/busybox.conf` for the live initial process and report that
the optional object is absent. Dispatch shall require an executable return
continuation and a complete writable native metadata destination, shall publish
no output on absence, and shall retain no pathname, destination, or lookup
state. The provider shall own the native directory-relative selector, pathname
encoding validation, metadata layout, syscall number, and register convention.
Another path, directory selector, flag, incomplete destination, lifecycle
phase, duplicate request, or success result shall fail closed. This requirement
does not introduce a VFS, mount, inode, descriptor-allocation, pathname-
resolution, or general metadata-query service.

The bounded startup-credential-tail adapter shall expose sealed real GID,
effective UID, and effective GID observations after the optional configuration
lookup. Each pointer-free query shall require the live initial process and an
executable continuation, return the matching immutable zero-valued credential
fact, retain no argument or mutable identity state, and advance only the exact
private startup phase. Results shall remain coherent with `AT_GID`, `AT_EUID`,
and `AT_EGID`. A missing, duplicate, reordered, inconsistent, or post-terminal
query shall fail closed. Native syscall selection and result placement shall
remain provider-owned. This adds no credential mutation, supplementary groups,
namespaces, capabilities, process table, or general identity service.

For the initial writable ELF qualification, `WritableDataMapping` shall replace
the immutable data role while preserving data-first teardown and all resident
ownership invariants. The ELF data program header shall be `PF_R | PF_W`, and
constructed code shall write and read one memory-only BSS byte before its
terminal syscall. The exact
`TOPAL_KERNEL_USER_IMAGE_WRITABLE_DATA_OK` marker shall occur after successful
return and before retained-image teardown; missing, duplicate, renamed, early,
or late emission shall fail closed.

The resident-image qualification shall emit the exact
`TOPAL_KERNEL_USER_IMAGE_READY` marker after both role-specific mappings are
finalized and before either is unmapped. Emission shall preserve both affine
mapping owners and grant no additional authority. Missing, duplicate, renamed,
early, or late emission shall fail closed. The marker shall expose no mapping
identity, address, permission representation, or target mechanism and shall
not imply that the image can enter user execution.

The bounded execution-residency qualification shall preserve the finalized
RO/X code and RO/NX data mappings across preparation, entry, syscall and fault
handling, exit, and exact kernel-continuation restoration of the sealed user
probe. The probe shall not access or execute either constructed mapping.
Source shall emit exactly one `TOPAL_KERNEL_USER_IMAGE_RETAINED` marker after
return and before data-first teardown, and shall complete or reclaim the
address space only after both mappings are revoked and their extents released.
Construction placements shall not alias probe code, stack, transfer
candidates, or deliberate fault guards. Early teardown, mutation, missing or
early witness, or completion with either mapping live shall fail closed.

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

## TOPAL-REQ-SYSTEMS-CONTEXT-003 — Bounded affine runnable queue

Scheduler source shall be able to own suspended kernel contexts in a bounded
affine FIFO without transferring runnable selection to a machine provider.
Enqueue and dequeue shall consume their prior queue state, preserve exact FIFO
order, and return every ownership obligation on full or empty results. The
queue shall reject duplicate, terminal, completed, wrong-processor,
wrong-provider, or wrong-address-space entries and shall be consumed empty
before disposition.

The first executable queue shall have capacity two, enqueue the cooperative
then terminal workers, re-enqueue the cooperative suspended handoff at the
tail, and therefore select cooperative, terminal, cooperative. It shall reclaim
both retired stacks and consume the empty queue. Its storage shall be fixed
before dispatch and shall not imply dynamic allocation, preemption, blocking,
wakeup, priority, timeslicing, cancellation, migration, or SMP.

## TOPAL-REQ-SYSTEMS-CONTEXT-004 — Deadline-preemptible context transfer

Scheduler source shall be able to dispatch one selected suspended kernel
context until one matching absolute deadline. The operation shall establish
the exact dispatcher continuation and event binding before admitting delivery.
After typed event completion, a preemption disposition shall capture the
interrupted target as an opaque suspended context and resume only that
dispatcher. The returned affine outcome shall retain processor, provider,
address-space, clock, event, context, stack, and continuation identities.

The provider shall save and restore target-specific interrupted state but
shall neither inspect nor select the runnable queue. The first executable
profile shall preempt one closed worker, re-enqueue it at the source-owned FIFO
tail, run and reclaim the terminal worker, then restore, retire, and reclaim
the preempted worker. Its sealed preemption test region shall complete only
after matching preemption and later source-selected redispatch; it shall not be
a general wait or sleep operation. The profile shall remain single-processor
and single-address-space and shall not imply cancellation, rearming, periodic
timeslices, priority, blocking, wakeup, nested preemption, migration, SMP, user
contexts, or extended-state switching.

Completed deadline events shall not acquire another typed entry from a queued
raw target notification. The provider shall filter/acknowledge such delivery
before source dispatch, preserve the interrupted frame/registers and avoid
duplicate completion/preemption. Completion shall close its target route and
source. Qualification shall inject a completed event notification at suspended
worker resumption and retain full context/resource rollback evidence.

## TOPAL-REQ-SYSTEMS-CONTEXT-005 — One-shot affine kernel completion

Scheduler source shall be able to create one completion with matching affine
wait and signal endpoints, dispatch one worker which may block once, receive
that exact blocked context at its registered dispatcher, signal it, enqueue the
resulting suspended context, and later resume immediately after the block.
Signal-before-wait readiness shall persist so that no matching wakeup is lost.

Every transition shall retain processor, provider, address-space, completion,
context, stack, and continuation identity. Runnable selection shall remain
source policy. The initial executable profile shall run blocking, terminal,
awakened order and consume both contexts, the empty runnable queue, and the
one-shot completion before disposition. It shall not imply multiple waiters,
repeated signals, arbitrary predicate waits, timeouts, cancellation, process
signals, priorities, cross-processor wakeup, SMP, or user contexts.

## TOPAL-REQ-SYSTEMS-CONTEXT-006 — Affine user execution and syscall round trip

The systems profile shall represent validated user state as opaque affine
authority tied to processor, address-space, thread, and ABI identities. Entering
user execution shall consume the running kernel context and one matching
suspended user context. A user syscall shall create one affine typed entry that
exposes an untrusted semantic syscall number and six arguments without exposing
register slots or a target frame.

Kernel source shall own dispatch and consume each entry by returning one ABI
result to the exact revalidated user continuation or by terminating that user
context and restoring its exact registered kernel continuation. The provider
shall own privilege, stack, saved-state, security, result-placement, and return
mechanics but shall not select a syscall or invent a result.

The first executable x86-64 profile shall use one generated ring-3 probe and
the pinned Linux `read`, `write`, `getpid`, `uname`, `nanosleep`, and
`exit_group` numbers. It shall verify ordinary returns, generated output- and
input-direction user-copy fault recovery, and terminal restoration with status
zero. Fixed probe
mappings shall be qualification evidence only. ELF loading, pointer-bearing
syscalls other than the bounded standard-input `read`, bounded standard-output
`write`, `uname`, and zero-duration `nanosleep`
qualifications, general user-copy recovery, signals, restart, TLS, multiple
processes or threads, user preemption, migration, SMP, and the complete syscall
table remain unavailable.

The initial-process write adapter shall accept descriptor one, an executable
user return continuation, and one through 256 caller-selected bytes wholly
contained in a proved readable user mapping. It shall reject invalid arguments
before access or output, completely stage valid bytes in private kernel memory,
publish exactly the staged prefix through the checked console effect, return
the requested length, and retain no pointer. The provider shall not compare
against an embedded payload or single call site. Wrapping or cross-hole extents,
partial copy, early output, arbitrary descriptors, short result, duplicate
output, or output after terminal disposition shall fail closed.

The initial-process read adapter shall accept descriptor zero, an executable
user return continuation, and a destination of one through 256 bytes wholly
contained in a proved writable user mapping. It shall reject an invalid
descriptor, length, wrapped or cross-hole extent, or read-only destination
before consuming input. For a valid request it shall wait for the exact bounded
count, stage the complete input privately, perform one fault-contained copy of
the staged prefix to user memory, retain no pointer, and return the requested
length. It shall not compare against a provider-embedded input value or require
one exact candidate or call site. User mode shall validate the deterministic
qualification bytes and result before stack disposal. The exact
`TOPAL_KERNEL_USER_READ_OK` marker shall occur after terminal return and before
the constructed-image entry marker. Partial reception, early user publication,
wrong transfer or result, unbalanced disposal, or witness reordering shall fail
closed.

The initial-process descriptor adapter shall create descriptors zero, one, and
two as live process-private entries before user entry. Linux `close` shall
atomically change one live entry to closed and return zero; a negative, out-of-
range, or already closed descriptor shall return `-EBADF` without state change.
The bounded `read` and `write` adapters shall consult the same state before
input consumption, user-memory access, or output. The ring-3 probe shall close
standard input only after its final successful read, observe `-EBADF` from a
later read and repeated close without consuming input, and then exit normally.
Terminal completion shall clear every entry before restoring the kernel
continuation. Stateless success, I/O through a closed entry, retained state,
provider-selected policy, or a general VFS/descriptor-allocation model shall
fail closed or remain unavailable as applicable.

The initial descriptor adapter shall refine each live entry into a mapping to
one fixed access-mode-bearing open description. Linux `dup` shall install the
same description in the lowest closed slot, return `-EBADF` for an invalid
source, and return `-EMFILE` without state change when the three-slot table is
full. Linux `dup2` shall validate its source before atomically replacing the
destination, preserve an equal live source/destination, and return `-EBADF`
without replacement for invalid arguments. Closing either alias shall preserve
the other, and read/write shall authorize the shared description rather than
the slot number. Ring-3 qualification shall cover full-table rejection,
lowest-free selection, source-slot close survival, output-mode read rejection,
invalid-source nonreplacement, error-description replacement, exact returns,
and terminal mapping clearing. Live-bit-only copying, altered access mode,
non-lowest selection, premature replacement, or retained mappings shall fail
closed.

The initial descriptor adapter shall retain a separate `FD_CLOEXEC` bit per
slot, initially clear. Linux `fcntl` shall implement `F_GETFD`, masked
`F_SETFD`, and `F_GETFL`; it shall return shared open-description access modes,
look up the descriptor before command decoding, and return exact `EBADF` or
`EINVAL`. Close shall clear the slot bit, while `dup` and non-identity `dup2`
shall clear only the new destination bit. Ring-3 qualification shall cover
mode observation, set/get, unknown command, duplicate/source flag separation,
replacement clearing, closed-slot rejection, and terminal clearing. Flags on
open descriptions, slot-derived modes, copied duplication flags, wrong error
precedence, or retained state shall fail closed.

The initial descriptor adapter shall publish one immutable console object
through Linux `fstat`. Every initial description and alias shall share
character-device identity, root ownership, mode `0600`, link count one,
preferred I/O size 4096, zero size/blocks/timestamps, and ttyS0 special-device
identity. The x86-64 adapter shall stage and fault-contain the complete native
144-byte layout, including zero padding. It shall look up the descriptor before
candidate access and return exact `EBADF` or `EFAULT` without partial output.
Ring-3 qualification shall compare output and duplicate-alias metadata, inspect
every field and padding byte, and reject a closed slot. Slot-derived identity,
regular-file kind, access-mode permissions, partial publication, host time, or
retained pointers shall fail closed.

The initial descriptor adapter shall expose the shared console's immutable raw
terminal attributes through Linux `ioctl(TCGETS)`. The x86-64 adapter shall
publish the complete native 36-byte `termios` with raw input/output/local
flags, `B38400|CS8|CREAD|CLOCAL`, line discipline zero, and only `VMIN=1` among
the 19 control bytes. It shall enforce descriptor-before-command-before-
pointer precedence, privately stage and fault-contain the complete value, and
return exact `EBADF`, `ENOTTY`, or `EFAULT`. Ring-3 qualification shall inspect
every byte, compare input and output descriptions, reject an unknown live-
console request, and reject a closed slot. Fabricated cooked behavior, partial
publication, wrong precedence, or retained pointers shall fail closed.

The initial descriptor adapter shall expose the shared console's immutable
unspecified geometry through Linux `ioctl(TIOCGWINSZ)`. The x86-64 adapter
shall publish the complete native eight-byte `winsize` with zero rows, columns,
and pixel dimensions; retain descriptor-before-request-before-pointer
precedence; privately stage and fault-contain the complete value; and preserve
exact `EBADF`, `ENOTTY`, and `EFAULT`. Ring-3 qualification shall inspect every
byte, compare input and output descriptions, and reject a closed slot. Invented
or host-derived geometry, partial publication, wrong precedence, or retained
pointers shall fail closed.

The initial descriptor adapter shall classify Linux `lseek` for the shared
console without creating file-position state. It shall look up a descriptor
before seek-mode validation, return exact `EBADF` for invalid or closed slots,
return `ESPIPE` for modes zero through four on a live console independently of
offset, and return `EINVAL` for a larger unsigned mode on a live console.
Ring-3 qualification shall cover input and output `SEEK_CUR`, an unsupported
live mode, and the same unsupported mode on a closed slot. Fabricated success,
offset-dependent terminal results, wrong precedence, or retained position
state shall fail closed.

The initial-process program-break adapter shall derive its initial value from
the validated runtime ELF data-memory extent and bound it by the already-owned
4096-byte RW/NX data page. The provider shall zero the complete page tail from
the runtime file-data end before user entry and keep the current break in
private process state. A zero request shall query the current break. An in-
range byte-granular request shall atomically replace and return it; an out-of-
range request shall preserve and return the prior break as Linux `brk` does.
Dispatch shall require a live context and executable return continuation, and
teardown shall clear the private break before releasing the data mapping.
Compile-time data-size authority, allocator residue exposure, negative errno
results, additional page mapping, retained user pointers, or state surviving
teardown shall fail closed.

The initial-process scalar identity adapter shall accept `getpid`, `gettid`,
`getppid`, `getuid`, `geteuid`, `getgid`, and `getegid` only for the live
initial context and an executable user return continuation. It shall return
PID and TID one, parent PID zero, and coherent real/effective UID and GID zero.
Credential results shall agree with the corresponding initial auxiliary-vector
facts. Dispatch shall retain no argument, pointer, or mutable identity state.
A missing family member, wrong or inconsistent result, non-executable return,
wrong lifecycle state, or result after terminal disposition shall fail closed.

The bounded static-process credential tail shall, after the exact optional
configuration absence, observe the sealed real group and then admit exact
idempotent group and user selections. Both requested identities shall equal
the corresponding zero-valued sealed auxiliary-vector facts, and all three
operations shall revalidate the complete real/effective UID/GID group. The
adapter shall return zero, retain no request or mutable credential state, and
leave the credential group unchanged. Another value, order, continuation,
lifecycle phase, incomplete group, or state mutation shall fail closed.

The bounded static-process applet-output adapter shall, after the complete
credential tail, accept exactly one standard-output line-feed request. It shall
require descriptor one, length one, an executable continuation, and the
header-profile-bound candidate byte in the exact retained stack leaf or
anonymous program-break extent. The provider shall revalidate the active
hierarchy and backing, copy
the byte into private staging before output, require line-feed value, publish
it once, return one, and retain no pointer or staging state. Another phase,
range, mapping, byte, descriptor, length, continuation, result, or output before
complete validation shall fail closed.

The initial-process entry adapter shall publish a native zero-valued
`AT_SECURE` auxiliary-vector fact after the existing program-header facts and
before `AT_NULL`. It shall agree with the sealed credentials and absence of a
privilege-changing execution transition. Constructed code shall validate the
ordered pair, the moved terminator, and complete bounded-stack consumption.
Wrong order, value, termination, aliasing, or consumption shall fail closed.

The entry adapter shall publish native `AT_CLKTCK` with Linux `USER_HZ` value
100 after `AT_SECURE` and before `AT_NULL`. It shall remain distinct from
hardware and scheduler frequencies. Constructed code shall validate its exact
order and value, the moved terminator, preserved executable-name alias, and
complete bounded-stack consumption; disagreement shall fail closed.

The adapter shall publish native `AT_PLATFORM` after `AT_CLKTCK` and before
`AT_NULL`, pointing to a complete NUL-terminated `x86_64` string inside the
initialized stack block. The platform string shall be distinct from the fixed
`argv[0]`/`AT_EXECFN` executable-name object. Constructed code shall validate
the tag, pointer, bytes, termination, alias separation, and bounded consumption.

The initial x86-64 thread-pointer adapter shall accept only
`arch_prctl(ARCH_SET_FS)` and `arch_prctl(ARCH_GET_FS)` for the live initial
thread and an executable user return continuation. `SET_FS` shall require zero
or a canonical candidate inside an already-owned readable user mapping, retain the
value as private thread state, install it before return, and return zero.
`GET_FS` shall require a complete writable eight-byte user destination, copy
the retained value through fault-contained output, and return zero. Unsupported
operations, invalid bases, and invalid output candidates shall produce their
declared Linux errors without changing state. Teardown shall clear the private
value before mapping revocation. Exposed registers or MSRs, incorrect set/get
identity, retained user pointers, partial output, or live state after teardown
shall fail closed.

The initial-thread exit-word adapter shall accept `set_tid_address` only for
the live initial thread and an executable user return continuation. A nonnull
candidate shall name one complete writable four-byte user extent; null shall
disable registration. Successful dispatch shall retain the candidate
privately and return TID one without dereference. Terminal exit shall write
zero to a retained word under fault recovery, clear the private candidate, and
only then complete the user context. Invalid candidates, wrong results,
uncontained clear faults, terminal transition before clearing, or state that
survives user-mapping teardown shall fail closed.

The initial-thread robust-list adapter shall accept `set_robust_list` only for
the live initial thread, an executable user return continuation, exact native
length 24, and one complete readable 24-byte user extent. Registration shall
retain the candidate privately without dereference and return zero. Terminal
exit shall fault-contain a complete private copy of the head, require the list
link to name the head and the pending pointer to be null, clear private state,
and only then continue terminal disposition. Wrong size or extent, early
dereference, partial copy, nonempty terminal state, clear-before-validation,
or state surviving mapping teardown shall fail closed.

The bounded initial-resource adapter shall accept `getrlimit(RLIMIT_STACK)`
and query-only `prlimit64` for PID zero or the sealed self PID, selector
`RLIMIT_STACK`, a null new-limit candidate, and one complete writable 16-byte
old-limit extent. Both shall publish the same native 8 MiB soft and unlimited
hard pair under fault recovery and return zero. Other selectors, mutation,
other PIDs, invalid output, partial publication, wrong values, or retained
pointers shall return the declared Linux error or fail closed as applicable.

The bounded initial-signal-mask adapter shall accept native `rt_sigprocmask`
query, block, unblock, and replacement operations for the sole live thread and
an exact eight-byte set size. It shall stage complete input under fault
recovery, remove `SIGKILL` and `SIGSTOP`, atomically update private mask state,
and optionally publish the prior complete word under fault recovery after the
update. Wrong size or operation, invalid extents, partial transfer, wrong
update order or values, retained pointers, or state surviving teardown shall
return the declared Linux error or fail closed as applicable.

The prepared initial address space shall accept one additional affine frame as
an opaque non-present anonymous-page reservation without publishing a user
translation or exposing its address. The bounded initial-process adapter shall
activate it only for a supported private anonymous Linux `mmap`, zero the
complete frame before ordered RW/NX publication, revoke it for matching
`munmap`, reject overlapping activation, and revoke any live generation at
terminal completion. Source teardown shall consume the non-present reservation
and return the exact frame. Early access, stale contents, aliasing, duplicate
activation, incomplete invalidation, or wrong backing return shall fail closed.

The bounded initial-process adapter shall accept Linux `mprotect` for the exact
active anonymous page, a nonzero length no greater than one base page, and only
none, read-only, or read-write protection. It shall retain the active mapping
generation and contents while replacing its provider-private translation with
the requested non-executable state and completing target invalidation before
return. The inaccessible state shall remain active and restorable. Unsupported
bits, wrong ranges, absent mappings, executable publication, stale access,
content loss, or incomplete invalidation shall fail closed or return the
declared Linux error without changing the prior state.

The adapter shall accept Linux `madvise(MADV_DONTNEED)` for the exact active
anonymous page and a nonzero length no greater than one base page. It shall
eagerly zero the complete backing through kernel authority while preserving the
mapping generation and its current inaccessible, read-only, or read-write
non-executable protection. Unsupported advice or ranges, absent mappings,
partial zeroing, state change, or stale contents shall fail closed or return the
declared Linux error without changing the prior contents or state.

The initial-process adapter shall accept Linux
`clock_gettime(CLOCK_MONOTONIC)` with one complete writable native timespec
candidate. It shall obtain one qualified monotonic provider observation,
normalize it to nonnegative seconds and nanoseconds below one billion, publish
the complete pair under fault recovery, retain no pointer, and preserve
nondecreasing order across successful calls. Unsupported clocks, invalid
extents, partial output, unnormalized values, or time regression shall return
the declared Linux error or fail closed.

The adapter shall accept Linux `clock_getres(CLOCK_MONOTONIC)` with either a
null output or one complete writable native timespec candidate. It shall report
the exact normalized resolution of the same qualified provider source used by
`clock_gettime`; the initial x86-64 profile shall report zero seconds and ten
nanoseconds. Nonnull output shall be complete and fault-contained. Unsupported
clocks, inconsistent resolution, retained pointers, or partial output shall
return the declared Linux error or fail closed.

A separately qualified static process shall borrow the bootstrap owner's
monotonic clock for observation, resolution and relative sleep. The borrowed
identity and observation history shall survive exec and return to the kernel
owner after private process-binding erasure. Complete native records shall use
checked user transfers. Relative intervals shall be validated and rounded up,
deadlines shall retain the pinned ABI's saturating native time range without
wrapping, and success shall never precede the resulting relative deadline. Qualification shall cover normalized/nondecreasing
observations, copy boundaries, invalid intervals, zero and nonzero sleep,
resolution rounding, integer widths, resource preservation, exec lifetime and
terminal rollback, including unmodified standard utility runs. The sole-thread
qualification shall not imply signals, scheduling fairness or absolute sleep.

The initial-process syscall adapter shall return Linux `-ENOSYS` for every
syscall number not selected by an implemented dispatch arm. The fallback shall
use the ordinary executable return continuation without reading unused
arguments, copying user memory, or changing kernel or process state. Evidence
shall prove the exact result and successful continuation into a subsequent
implemented syscall. Success, process termination, or provider-owned syscall
selection shall fail closed. Separately qualified static processes shall retain
this behavior before and after exec, with unchanged startup phase, descriptors,
positions, cwd, signals and image authority. Native unknown-selector width,
invalid unused arguments, unchanged user-memory canaries and continued
implemented dispatch shall be physically qualified across replacement.

The adapter shall implement Linux `sched_yield` for the sole runnable initial
thread. It shall return zero through the executable continuation while
preserving the exact user context and all private process state. It shall use an
implemented dispatch arm rather than the unsupported fallback and shall not
invoke a provider context switch when no eligible peer exists.

The prepared user address space, suspended user context, completed execution,
and revoked address space shall retain one exact address-space identity through
their affine transitions. Preparation, context binding, revocation, and
reclaim shall remain distinct operations. The initial `initial-probe` layout
shall admit one live user address space and no source-selected mappings,
arbitrary permissions, ELF segments, dynamic stack construction, concurrent
edits, address-space switching, or recoverable partial construction.

## TOPAL-REQ-SYSTEMS-DEVICE-001 — Device and DMA protocol ownership

Device locations shall bind layouts to legal access widths, side effects,
reserved-bit policy, ordering, and a live device session. DMA shall transfer
buffer ownership through explicit CPU, prepared, device, completion/failure,
and reclamation states with mapping, cache, notification, and completion
evidence. Coherent hardware shall not erase the protocol or evidence boundary.

The initial PCI configuration refinement shall derive one transaction-scoped
session from bootstrap platform authority. It shall validate segment and BDF
coordinates plus an aligned dword register offset, perform exactly one
observable little-endian read, and either prove an expected value before
returning the authority or fail terminally. The provider shall select and
serialize the configuration mechanism without owning enumeration, matching,
resource assignment, capability walking, virtio negotiation, or driver binding.

The bounded virtio-PCI discovery refinement shall compose exactly 24 such
transactions into the sealed source-owned walk for `0000:00:05.0`. It shall
follow the conventional capability links through MSI-X and five virtio vendor
capabilities, validate every admitted virtio header and BAR-relative region,
and retain an ordered transaction identity for every read. The provider shall
remain unaware of capability and virtio meaning. Assigned BAR addresses, BAR
mapping, feature negotiation, device-register access, interrupts, DMA, and
entropy delivery shall remain unavailable.

A PCI region-register refinement shall derive a transaction-scoped device
session from validated function, configuration-access capability, BAR index,
BAR-relative extent, register offset, and exact-width protocol evidence. Source
shall own direction and value policy without receiving an assigned BAR address.
The provider shall serialize and realize the access through a qualified
configuration gateway or opaque mapping without interpreting virtio policy.

The bounded virtio negotiation refinement shall perform exactly 16 ordered
common-configuration accesses. It shall reset and read back status, observe both
pinned device-feature words, select only `VIRTIO_F_VERSION_1`, and read back
accepted `FEATURES_OK`. It shall leave `DRIVER_OK` clear until queue discovery,
setup, and population complete. Queue, interrupt, DMA, and entropy operations
shall remain unavailable.

The bounded virtio queue-discovery refinement shall perform exactly five
ordered common-configuration accesses after confirmed feature negotiation. It
shall observe one queue, select queue zero, establish capacity eight, prove the
queue remains disabled, and retain notification identity zero. The accesses
shall use exact two-byte semantics. DMA authority, queue enablement,
notification, `DRIVER_OK`, interrupts, and entropy delivery shall remain
unavailable.

The bounded PCI bus-master refinement shall perform one masked two-byte update
of the pinned function's command register after queue discovery. It shall prove
masked pre-state zero, set only command bit two, preserve every unmasked bit,
and prove the masked post-state by readback under one serialized transaction.
It shall not write the adjacent status field or grant DMA-memory authority.

The bounded virtio DMA-buffer refinement shall create exactly two fresh,
disjoint, 4 KiB-aligned 4 KiB buffers after bus-master authorization. One shall
be bidirectional shared-control memory and one shall be device-write payload
memory, both qualified by the pinned device and translation domain and retained
in `CpuOwned` state. It shall expose no address and grant no device ownership.

The bounded virtio split-ring refinement shall construct descriptor zero and
available entry zero in the CPU-owned control buffer through bounded integer
stores and one opaque same-domain reference to the payload buffer. It shall
publish available index one only after the descriptor and slot are complete,
retain CPU ownership, and expose no provider-selected DMA address.

The bounded DMA-preparation refinement shall perform exactly two affine
ownership transitions after split-ring construction: entropy payload identity
two followed by virtqueue-control identity one. It shall retain exact
device/domain, direction, visibility, role, and identity facts; require ordered
device visibility even when coherent hardware needs no cache instruction;
reject malformed, omitted, duplicated, reordered, substituted, or premature
preparation; and prohibit subsequent CPU access while device-owned.

The bounded queue-address refinement shall publish exactly three opaque
device-visible references after both DMA buffers become device-owned: control
offset zero to descriptor register offset 32, control offset 128 to driver
register offset 40, and control offset 148 to device register offset 48. It
shall retain the exact common-region, device/domain, buffer, offset, width, and
order facts; reject malformed, omitted, duplicated, reordered, substituted, or
premature publication; and expose no address to source.

The bounded virtio queue-activation refinement shall perform exactly four
generic common-region transactions after all queue addresses are published:
queue-enable write/readback followed by final `DRIVER_OK` status
write/readback. It shall retain exact region, width, offset, value, and order
facts; reject malformed, omitted, duplicated, reordered, substituted, or
premature activation; and leave device-owned buffers inaccessible to the CPU.

The bounded virtio queue-notification refinement shall perform exactly one
generic two-byte notification-region write after verified activation. It shall
retain the separate configuration-access capability, discovered notification
capability, BAR, region, multiplier-derived offset, queue-zero value, and
prerequisite ownership/order facts; reject malformed, omitted, duplicated,
reordered, substituted, or premature notification; and grant neither
completion nor CPU-access authority.

The bounded virtio completion-signal refinement shall perform exactly one
generic one-byte ISR-status read-eventually-equals-one operation after
notification. It shall retain the separate configuration-access capability,
discovered ISR capability, BAR, region, offset, width, expected queue-interrupt
bit, 1,000,000-attempt bound, read acknowledgement, and prerequisite order;
reject malformed, omitted, duplicated, reordered, substituted, premature, or
unbounded observation; fail closed when the bound is exhausted; and grant
neither used-ring validity nor CPU-access authority.

The bounded virtio used-ring refinement shall perform exactly one affine
device-to-CPU completion transition for the shared control buffer after the
acknowledged transport signal, followed by exact-width reads proving used
flags zero, index one, descriptor identity zero, and completed length 32. It
shall retain buffer, device, translation-domain, direction, visibility, and
role facts; require provider-completed device-to-CPU visibility before access;
reject malformed, omitted, duplicated, reordered, substituted, or premature
transitions and reads; and retain device ownership of the entropy payload.

The bounded virtio payload-completion refinement shall perform exactly one
affine device-to-CPU completion transition for entropy payload buffer identity
two after exact used-ring validation. It shall retain device,
translation-domain, device-to-memory direction, provider-managed visibility,
and payload-role facts; require provider-completed device-to-CPU visibility;
reject malformed, omitted, duplicated, reordered, substituted, or premature
completion; and assign no entropy credit or payload-byte meaning.

The bounded entropy-sample refinement shall capture exactly 32 bytes from
offset zero of CPU-owned entropy payload buffer identity two into fresh opaque
sample identity one while preserving payload ownership. It shall retain the
exact device, translation-domain, direction, visibility, and role facts;
record zero credited entropy bits; expose neither address nor byte values;
reject malformed, omitted, duplicated, reordered, substituted, premature, or
partially copied capture; and perform no conditioning or userspace delivery.

The bounded entropy-quality refinement shall assess exactly that captured raw
sample under one explicit initial-profile trust evidence resource. It shall
preserve sample identity and opaque bytes; require source identity one,
trust-evidence identity one, byte count 32, quality 1024 parts per 1024, and
the exactly derived 256 credited bits; reject malformed, omitted, duplicated,
reordered, substituted, or premature assessment; and create no provider call,
pool state, generator state, or random output.

The bounded entropy-pool refinement shall create exactly one affine empty pool
after that sample assessment. It shall require pool identity one, a 256-bit
credit capacity, and zero initial credited bits; expose no storage or byte
representation; create no provider call; reject malformed, omitted,
duplicated, reordered, substituted, or premature creation; and perform no
mixing, credit transfer, generator seeding, or random output.

The bounded entropy-mixing refinement shall consume that assessed sample and
empty pool through conditioner identity one, BLAKE2s-256 personalized by the
eight bytes `TopalRNG`. It shall produce the same pool identity with exactly 32
opaque mixed-state bytes and zero credited pool bits, consume and completely
wipe the 32-byte sample, and produce affine contribution identity one carrying
256 untransferred credited bits. It shall reject malformed, omitted,
duplicated, reordered, substituted, premature, partially written, or
incompletely wiped mixing and shall create no random output.

The bounded entropy-credit refinement shall consume the mixed-but-uncredited
pool and its affine contribution token after successful mixing. It shall
require pool identity one, contribution identity one, conditioner origin one,
capacity 256, prior credit zero, transferred credit 256, and resulting credit
256; preserve the provider-owned opaque pool bytes; consume the contribution;
create no provider call; reject malformed, omitted, duplicated, reordered,
substituted, overflowing, or premature transfer; and imply neither readiness
nor random output.

The bounded initial-CRNG refinement shall consume the fully credited pool into
fresh generator identity one after credit transfer. It shall require pool,
generator, and algorithm identity one; generation one; key extent 32; required
and observed credit 256; transfer the complete opaque conditioned state as the
ChaCha20 key; verify the copy; wipe and verify all pool bytes; and publish only
a ready affine generator. It shall reject malformed, omitted, duplicated,
reordered, substituted, premature, partially copied, or incompletely wiped
initialization and shall produce no random output.

The bounded CRNG-extraction refinement shall consume ready generator identity
one at generation one and publish only ready generation two plus fresh opaque
output identity one. It shall compute one standard 20-round ChaCha20 block from
the 32-byte key, zero counter, and zero nonce; replace and verify the persistent
key from the first 32 bytes before publishing and verifying the second 32
bytes; and wipe and verify all temporary cipher state. It shall reject
malformed, omitted, duplicated, reordered, substituted, premature, partially
published, or incompletely wiped extraction and shall perform no interface
delivery.

The bounded initial-`AT_RANDOM` refinement shall consume opaque output identity
one into prepared address-space identity one before its first context entry. It
shall require Linux interface identity one, source extent 32, selected offset
zero and extent 16, discarded offset 16 and extent 16, auxiliary tag 25, and
single-use delivery; copy and verify a distinct in-stack 16-byte object; wipe
and verify the complete provider output; and return only a randomized entry-
ready address space. It shall reject malformed, omitted, duplicated, reordered,
substituted, premature, partially copied, incompletely wiped, or reused
delivery and shall not expose random bytes or an address to source.

The bounded initial-`getrandom` refinement shall freshly extract generation two
to generation three as opaque output identity two and reserve that output for
process, Linux interface, and user-random service identity one before entry. It
shall require source extent 32, a maximum selected prefix of 16 bytes, discard
the unused selected suffix and bytes 16 through 31, accept only the current
Linux flag mask 7 while rejecting combinations containing mask 6, and permit
one successful nonempty request. Zero, flag-error, length-error, and complete-
range-fault results shall not consume the reservation. Success shall copy and
verify the requested prefix, wipe and verify all provider output, consume the
reservation, and expose neither random bytes, an address, nor a native syscall
number to source.

The reusable process-`getrandom` refinement shall consume the ready generation-
three generator and the live initial reservation into one affine process-
lifetime service before entry. The reserved output shall supply the first
successful request. Every later successful request shall validate its complete
arguments and writable range before a fresh extraction, replace the persistent
key before publication, return and verify at most 16 requested bytes, discard
the remainder, and wipe and verify the complete output. Zero and rejected
requests shall not advance the generator. Process exit shall destroy any live
output, generator key, and private service state. The refinement shall expose
no secret byte, address, generation representation, syscall number, register,
or target mechanism to source.

The full-output process-`getrandom` refinement shall raise both the initial
reservation and reusable service maximum request to the complete 32-byte
qualified provider output, with an empty reservation discard partition. It
shall preserve validation before state advance, use exactly one reserved or
freshly extracted output per success, copy and verify 1–32 requested bytes,
wipe and verify the complete output, retain no remainder, and leave the service
unchanged for zero and rejected calls. Qualification shall execute the existing
two 16-byte successes followed by one 32-byte success and observe every
returned qword without constraining secret-derived values.

The bounded small-request `getrandom` refinement shall consume the live service
into one service admitting complete requests through 256 bytes and at most
eight 32-byte outputs per call. It shall validate the complete writable range
before the first state transition; replace the persistent key before each fresh
output; copy ordered chunks; wipe and verify every complete output; retain no
remainder; return only the exact full length; and increment interface delivery
state once. Zero and rejected calls shall not change service state, and the
refinement shall expose no loop, key, byte, address, syscall, register, or
target fault mechanism to source.

The bounded large-request `getrandom` refinement shall consume that service
into one service admitting signal-free complete requests through 512 bytes
and at most 16 32-byte outputs per call. It shall preserve whole-range
validation before state change, ordered chunking, per-output key replacement,
complete copy verification and erasure, no retained remainder, exact return
length, and one interface delivery update. The numeric bound shall remain a
profile constant rather than a portable architecture page-size operation.

## TOPAL-REQ-SYSTEMS-ARTIFACT-001 — Host-independent kernel artifact

A systems artifact shall use a distinct qualified target profile, generated
special roots, semantic placement, explicit relocation/code-model policy, and
atomic validated publication. It shall have no implicit host syscall,
allocator, process startup, libc, dynamic loader, foreign runtime, or ordinary
process-termination path. Boot packaging shall be a separately qualified
artifact adapter.

A systems artifact may import a source-defined pure kernel library under a
versioned semantic interface. The shared ordinary frontend shall check the
selected public export and reachable helper graph. The first freestanding
profile shall admit only Boolean/nominal-Enum data and acyclic pure direct calls,
reject effects, allocation, foreign dependencies and unqualified forms, and
preserve ordinary language semantics. Target binding shall adapt private Enum
representations, preserve native calling convention, prohibit absent resource
rights, retain scoped ownership across exec and erase it at terminal return.
Publication shall bind source/interface/LLVM/object digests and checked stack
requirements, and shall include no hosted runtime or assembly from source.
Qualification shall execute all finite input combinations, normal helper frames,
source-body changes and Enum reordering, malformed/escape inputs, native syscall
boundaries and full rollback. Kernel policy shall reside in the kernel source
tree; native adapters shall translate semantic choices and enforce authority.

A separately qualified pure numeric source-library profile may use an explicitly
root-owned temporary arena and the shared ordinary numeric runtime. It shall
preserve arbitrary-precision semantics, bind only qualified kernel allocation
and fatal mechanisms, and prove conservative complete stack/allocation bounds
including runtime loops. The initial relative-sleep interface shall declare a
4096-byte stack and 65536-byte temporary-storage limit, admit no source resource
or address authority, prohibit allocated-value escape and reentrant activation,
and erase/read back invocation storage before return and binding at termination.
Publication shall reject missing proof, unsupported code, hosted dependencies
or insufficient resources. Runtime ownership, exhaustion and representation
failures shall fail closed. Qualification shall cover boundary arithmetic,
source/runtime execution, bounds, ABI preservation, retained ownership and full
reclamation; measured resource use alone shall not establish the bound.

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
