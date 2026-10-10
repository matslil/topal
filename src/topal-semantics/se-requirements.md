# Shared semantic-model requirements

## TOPAL-SEM-ASSURANCE-001 — Fail-closed evidence and resource models

The shared semantic library shall model evidence kind, status, producer,
subject, assumptions, language revision, and optional architecture identity.
It shall reject unauthorized producers and shall permit externally assumed
facts to discharge only ordinary semantic laws whose complete assumptions the
application admits. It shall provide conservative resource-bound composition,
closed progress ordering, invocation-exclusivity checking, consumption, region
escape checking, and compiler-only implementation-plan authority.

## TOPAL-SEM-PORTABLE-RUNTIME-001 — Deterministic portable reference behavior

The shared semantic library shall provide deterministic, architecture-neutral
reference behavior for synthesized channel selection, retained snapshots,
transaction commit and conflict, clock identity, monotonic observation,
periodic late-release policy, static-rate balance and scheduling, closed effect
protocols, and affine resumptions. Reference behavior shall not claim a
physical schedule, timing bound, nonblocking implementation, or compiler
lowering.

## TOPAL-SEM-LAYOUT-INFOFLOW-001 — Checked representation and policy models

The shared semantic library shall validate shape identities, bounded address
arithmetic, nonoverlapping and aligned strides, blocked storage size, canonical
sparse coordinates, zero-copy compatibility, finite information lattices,
implicit program-counter propagation, and scoped declassification or
endorsement authority. Validation shall fail closed on unknown identities,
overflow, overlap, or authority mismatch.

## TOPAL-SEM-ARCH-001 — Deterministic architecture reference model

The shared semantic library shall validate unique architecture component,
connection, cost, and provenance identities; reject dangling topology,
resources, provenance, feature dependencies, and reversed cost intervals; and
derive an order-independent SHA-256 identity from canonical effective facts.
This reference model shall perform no host detection and shall not itself
qualify a backend, platform, or empirical cost claim.

## TOPAL-SEM-SYSTEMS-001 — Authority-safe systems reference model

The shared semantic library shall model systems feature selection, sealed
authority identities, entry context/disposition compatibility, declared
observation and protocol linearization, address-family and mapping state,
fault-recovery scope, atomic modification/ordering relations, affine critical
tokens, linear context transfer, device-register protocols, DMA ownership,
semantic placement, and artifact obligations without executing a host machine
operation. It shall reject forgery, wrong context, escaped affine resources,
invalid state transitions, insufficient ordering, plain/atomic conflicts, and
unsupported providers deterministically.

The first implementation increment shall model the sealed bootstrap and
debug-break entry kinds, console-write and debug-break effects, resume and
fatal dispositions, stable semantic identities, and the exact initial target
profile. It shall model a fatal exception as terminal and a resumed exception
as returning to the interrupted bootstrap continuation. Unimplemented systems
families shall remain unavailable rather than receiving placeholder host
behavior.

The second implementation increment shall model one artifact-provided bounded
bootstrap pool, deterministic aligned monotonic allocation, sealed malformed
and exhaustion failures, affine region provenance and release, and whole-pool
reclamation only after bootstrap completes without live regions. Model offsets
shall remain abstract and shall not acquire machine-address meaning.

The boot-memory increment shall model consumption of the entered bootstrap
context; exhaustive success/failure refinement; normalized disjoint physical
range classes; conservative overlap precedence; complete-page alignment;
explicit live reservations; source provenance; and absence of allocatable
memory. Model range bounds shall be mathematical values for validation and
shall not grant machine-address authority.

The frame-allocation increment shall model consuming allocator creation,
exclusive allocator identity, deterministic aligned selection from normalized
allocatable ranges, explicit malformed and exhaustion failures, nonoverlapping
affine extent ownership and provenance, same-allocator consuming release, and
rejection of escape, cross-allocator release, double release, or completion
with live extents. Model frame indices shall not grant machine-address or
mapping authority.

The kernel-mapping increment shall model consuming frame ownership into one
opaque provider-qualified kernel mapping; explicit rights, execution, and
normal-memory policy; bounded retained byte contents; consuming unmap which
returns the original extent; and rejection of wrong-provider mapping,
out-of-bounds access, writable execution, frame release while mapped, use after
unmap, or completion with a live mapping. Model virtual identities shall not
grant numeric address or page-table authority.

The translation-space increment shall model the affine lifecycle from an
exclusive bootstrap-equivalent `TranslationUpdate`, through consuming commit
to an inactive `TranslationSpace`, through activation into a refined bootstrap
context. It shall retain provider and backing provenance, preserved coverage
and permission observations, and reject duplication, use after commit,
activation before commit, wrong-provider activation, backing reuse, and
completion with a live inactive update or space. Target table formats and
activation state shall remain outside the common model.

The active-translation-edit increment shall consume an active context into one
exclusive edit, consume a frame extent into a provisional inaccessible mapping,
and refine both mapping visibility and the active context only on successful
commit. Unmap shall consume the live mapping into a provisional unavailable
extent and restore releasable frame ownership only after committed target
invalidation. The model shall reject old-context use, nested edits, access
before map commit or after unmap, premature frame reuse, wrong-space commit,
and completion with a live edit while retaining opaque provider placement.

The first critical-scope increment shall model local maskable interrupts as an
affine context refinement bound to one processor and exact prior state. It
shall assign nesting identities, require last-in-first-out restoration,
consume restoration authority exactly once, and reject escape, disposition,
suspension, blocking, processor transfer, or completion with a live scope.
The model shall not expose target flags, registers, masks, or instructions and
shall not claim exclusion of non-maskable events, other processors, devices,
or DMA agents.

The first atomic-location increment shall model one provider-width unsigned
word whose construction consumes an exclusive ordinary region and whose end
transition returns that same region. It shall retain location identity,
alignment, domain, current value, and one modification order; distinguish
successful from observed-failure compare/exchange; validate success and
failure orders; and reject plain access, release, escape, or completion while
the location is live. Machine addresses, instruction strategies, registers,
and exclusive-monitor retries shall remain outside the common model.

The first external-interrupt increment shall model one affine local-
notification session with a monotonic event identity, consuming send, matching
observation and entry, mandatory completion, interrupted-context resumption,
wait completion, and exact prior local mask-state restoration. It shall reject
context use while pending, unmatched or duplicate events, resume before
completion, completion outside the handler, duplicate completion, and ordinary
completion with live notification authority. Vectors, controller state,
machine frames, acknowledgement encodings, wait instructions, and return
instructions shall remain outside the common model.

The first systems time increment shall model a provider-created monotonic clock
with exact clock identity, source-ordered observation identities, immutable
same-clock instants, and nondecreasing accepted values. It shall accept equal
successive observations and reject decreasing observations, cross-clock
comparison, invented or duplicate identities, and completion without the
required observations. Counter representation, target access, scale,
enablement, calibration, and wrap state shall remain outside the common model.

The first deadline-event increment shall model construction from one exact
same-clock instant and duration, one affine arm/wait lifecycle, one delivery
observation no earlier than the scheduled instant, mandatory matching entry
completion, resumption, and processor-context return. It shall retain both
scheduled and observed instants, admit late and already-expired delivery without
restarting the interval, and reject wrong-clock, early, duplicate, unmatched,
escaped, or incomplete lifecycles. Timer, comparator, routing, vector,
controller, acknowledgement, wait, frame, and return mechanics shall remain
outside the common model.

The third implementation increment shall model source-visible exhaustive
bootstrap allocation, borrowed bounds-checked plain byte store/load, and
consuming release. It shall retain byte contents without assigning a machine
address, reject out-of-bounds access and use after release, and distinguish
plain storage from volatile, atomic, device, DMA, firmware, and user-memory
protocols.

## TOPAL-SEM-INTEGRATION-001 — Explicit completion boundary

Reference-model unit tests establish the behavior of the shared algorithms but
shall not be reported as complete source-language, interpreter, debugger, or
compiler conformance. Each specification domain remains `planned` in
`se/core-language-coverage.md` until its source forms, tool integrations, and
applicable compiled evidence satisfy the terminal disposition recorded there.
