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

The cooperative context model shall retain two disjoint stack and context
identities, distinguish the first suspended handoff from terminal retirement,
enforce source-selected cooperative/terminal/cooperative FIFO order, and
require both terminal stacks to be reclaimed. It shall reject wrong entry
protocol, repeated handoff, retirement with live obligations, wrong order, and
completion with a live context.

The bounded runnable-queue model shall own suspended contexts in exact FIFO
order, retain queue and context obligations on full and empty results, and
require empty consumption before disposition. The initial profile shall prove
capacity two and cooperative/terminal/cooperative selection while negative
tests cover zero capacity, full, empty, duplicate or mismatched ownership, and
nonempty consumption.

The deadline-preemption model shall bind one selected suspended context, one
same-clock deadline event, and one exact dispatcher. It shall produce an affine
preempted-transfer result only after event completion and dispatcher
resumption, permit its suspended context to be taken once and re-enqueued, and
restore the exact interrupted continuation only when source selects it again.
It shall reject mismatched identities, wrong order, repeated take or restore,
and completion with any live transfer obligation.

The one-shot completion model shall create matching affine wait and signal
endpoints with one fresh identity. It shall distinguish block-first from
signal-first linearization, retain signal-first readiness, bind a blocked
worker to its exact suspended dispatcher, and return only the matching
awakened context for source-owned enqueue. It shall reject wrong completion,
context, dispatcher, provider, processor, or address-space identity, repeated
wait or signal, blocking with incompatible live obligations, and disposition
with any live endpoint, blocked context, dispatcher, or completed state.

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

The exact static-ELF frame refinement shall model the additional literal
503-frame unit-aligned request as one affine extent, require its complete
release, and retain the same normalized-range, reservation, non-overlap, and
no-address-authority invariants. The executable bootstrap model shall require
the request after the three-frame extent is returned, one witness while the
extent is live, and whole release before later operations.

The mapping-resource refinement shall model the exact nested one-frame
metadata allocation while the 503-frame payload owner remains live. It shall
retain two distinct affine identities, require provider-proved whole-interval
disjointness, require one observation while both are live, transition back to
payload-only ownership when metadata is released, and require final payload
release before continuation. Wrong nesting, overlap, reversed release,
combination, splitting, and completion with either resource live shall fail.

The initial-leaf refinement shall consume the exact two-resource state into one
opaque live construction, retain the 503 ordinal page-to-frame relation and
semantic RW/NX rights without target entry representation, and make success
atomic. Revocation shall refine that owner and return only the original
metadata extent; after its release, payload reclaim shall return only the
original 503-frame extent. The model shall reject partial construction, wrong
identity or ordering, early resource release, duplicate revoke/reclaim, active
attachment, and completion with live construction state.

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

The first context-transfer increment shall model consuming one checked
exclusive region into one suspended initial kernel context, consuming transfer
of the running bootstrap context to that worker, terminal retirement only with
no live worker obligation, exact caller resumption, and consuming reclamation
of the retired worker and stack. It shall retain processor, stack, thread,
entry, address-space, and extended-state-policy identities and reject wrong-
processor, wrong-entry, duplicate, escaped, use-after-transfer, incomplete, or
ordinary-return lifecycles. Register sets, stack pointers, frame layouts,
continuation addresses, and save/restore instructions shall remain outside the
common model.

The first user-execution increment shall model one opaque validated user state,
one suspended user context, exact kernel-continuation registration, affine
syscall entry, source-owned `getpid` result, return to the exact captured user
continuation, `exit_group` termination, and restoration of the exact kernel
continuation. It shall retain processor, address-space, thread, process, ABI,
probe, entry, and continuation identities and reject wrong order, syscall,
result, status, or identity. Registers, privilege encodings, entry frames,
stacks, security sequences, instructions, and return-path selection shall
remain outside the common model.

The third implementation increment shall model source-visible exhaustive
bootstrap allocation, borrowed bounds-checked plain byte store/load, and
consuming release. It shall retain byte contents without assigning a machine
address, reject out-of-bounds access and use after release, and distinguish
plain storage from volatile, atomic, device, DMA, firmware, and user-memory
protocols.

The qualified systems source-library refinement shall retain a typed optional
artifact import requirement under `TOPAL-SYSTEMS-SOURCE-LIBRARY-001` and
`TOPAL-SYSTEMS-KERNEL-POLICY-BINDING-001`. It shall identify the relative source
unit, public export, selected semantic interface and 1024-byte caller-stack
limit, reject missing/duplicate/unknown or escaping fields through shared
checking, preserve existing entry/lifetime validation and connect the import
to explicit publisher inputs and owned process binding. This metadata shall
not reinterpret core function syntax, grant numeric code authority or imply
unqualified general-library support.

The systems child-process model shall realize
`TOPAL-SYSTEMS-STATIC-PROCESS-CHILD-001`,
`TOPAL-SYSTEMS-STATIC-PROCESS-CHILD-WAIT-001` and
`TOPAL-SYSTEMS-STATIC-PROCESS-THREAD-REGISTRATION-001` through explicit process,
snapshot, shared-description and waitable-status lifetimes. Negative tests shall
cover missing/mismatched ownership, capacity, failed publication/retry, private
state restoration and once-only reaping. Modeled values shall grant no native
address/context authority or general scheduling guarantee.

The systems reference model shall retain the explicit numeric-library interface,
4096-byte stack and 65536-byte temporary-storage obligations, invocation owner
and exclusive activity, result bounds, exec-retained binding and terminal erasure
required by `TOPAL-SYSTEMS-NUMERIC-RUNTIME-001` and
`TOPAL-SYSTEMS-SLEEP-DEADLINE-LIBRARY-001`. It shall model arbitrary-precision
deadline arithmetic independently and reject missing, conflicting or out-of-scope
metadata. Machine allocation/frame bounds remain mandatory publisher/provider
evidence rather than invented numeric authority in the reference model.

Under `TK-DEC-260`, the model shall retain the exact image-library interface,
64-byte semantic snapshot and bounded machine/count facts from
`TOPAL-SYSTEMS-ELF64-HEADER-LIBRARY-001`, and compute header admission through
independent protocol decoding without assigning storage authority. The numeric
owner model shall bind the complete selected policy set, permit either policy
alone and reject overlapping, unbound or wrong-owner activations. Refusal/normal
return shall require clean invocation state; terminal return shall require every
selected binding erased. Machine copy, namespace, native ABI and complete resource
proofs remain publisher/provider obligations.

Under `TK-DEC-261`, the model shall retain the exact dispatch-library interface
and complete selected policy set from
`TOPAL-SYSTEMS-LINUX-DISPATCH-LIBRARY-001`, compute named operations independently
from normalized selector facts and preserve existing unknown-selector behavior.
Dispatch/header/deadline policies shall work independently and through sequential
exclusive borrowing; cross-owner, cross-policy and overlapping activation shall
fail without mutation. The model shall not assign authority to a selector or
operation name, invent native tag encodings, or replace required source/ABI,
resource-bound, before-handler and complete-erasure evidence.

Under `TK-DEC-262`, the systems model shall retain the exact process-identity
library obligation from `TOPAL-SYSTEMS-PROCESS-IDENTITY-LIBRARY-001`, model its
copied cursor and named numeric outputs independently, reject invalid cursors,
and cover initial, increment and wrap without granting ownership to numbers.
Exclusive policy binding shall include this fourth selected domain and preserve
independent/combined invocation, unchanged rejection and complete teardown.
Actual aggregate adaptation, native ABI, before-commit state and full resource
proofs shall remain publisher/provider obligations.

Under `TK-DEC-263`, the model shall realize
`TOPAL-SYSTEMS-RETAINED-NUMERIC-STATE-001` and
`TOPAL-SYSTEMS-RETAINED-PROCESS-IDENTITY-001` through explicit store ownership,
published/prepared versions, one-use commit and last-borrow retirement. Invalid
owner/binding/version/overlap, duplicate identity interface versions, stale
publication/resurrection and incomplete return shall reject without mutation.
Repeated transitions, actual value-history expectations, exec/parent restoration
and cancellation shall preserve the same semantic state law. These modeled
values shall grant no address authority or machine-layout/ABI evidence.

Under `TK-DEC-264`, the model shall implement
`TOPAL-SYSTEMS-RETAINED-STRUCTURED-STATE-001` and
`TOPAL-SYSTEMS-STRUCTURED-PROCESS-IDENTITY-001` and represent nested identity state with
next-ID, creation count and sticky wrapped-history facts. Ordinary facts shall
grant no store/process/generation authority. Exact initial/increment/wrap/count
and exhaustion behavior, one unique identity interface version, exclusive
preparation, cleanup before publication, last-borrow retirement and full return
shall be modeled independently. Invalid owners, stale/overlapping transitions,
malformed or exhausted state and invalid publication shall reject without
mutation. Native layout, pointer borrowing, aggregate ABI and complete machine
bounds remain separate qualification obligations; modeled values are not their
evidence.

Under `TK-DEC-265`, a volatile-file reference model shall distinguish owned
dispatch-version selection under `TOPAL-SYSTEMS-LINUX-DISPATCH-V2-001` and
resources under `TOPAL-SYSTEMS-VOLATILE-FILE-OWNERSHIP-001`,
`TOPAL-SYSTEMS-VOLATILE-FILE-TRANSFER-001` and
`TOPAL-SYSTEMS-VOLATILE-FILE-PERSONALITY-001`: specifically
filesystem/object/page identities from numeric slots, native descriptors and
offsets. It shall model sparse zero reads, actual prefix publication, append,
checked seek/truncate, page/object exhaustion, stable identity across aliases,
shared contents/positions through fork/exec and complete last-reference release.
Rejected owner, generation, rights, capacity or transfer operations shall leave
the model unchanged. Truncated or freed bytes shall not reappear. Process umask
shall be copied privately at fork and preserved by exec. Independent model
transitions shall not serve as native allocation, layout or physical evidence.

## TOPAL-SEM-INTEGRATION-001 — Explicit completion boundary

Reference-model unit tests establish the behavior of the shared algorithms but
shall not be reported as complete source-language, interpreter, debugger, or
compiler conformance. Each specification domain remains `planned` in
`se/core-language-coverage.md` until its source forms, tool integrations, and
applicable compiled evidence satisfy the terminal disposition recorded there.

## TOPAL-SEM-SYSTEMS-FILE-OPEN-PLAN-001 — Finite open disposition

Model `TOPAL-SYSTEMS-FILE-OPEN-PLAN-001` as eight copied Boolean facts and a
nominal plan, separate from filesystem authority and publication. Preserve the
older kernel-library interface and exact stack-only requirements; either
interface may occupy the single kernel-library slot. Document Linux error
precedence and absent/existing, exclusive/create, directory and truncate cases
without assigning native flags or code-pointer authority to ordinary values.

## TOPAL-SEM-SYSTEMS-POSITIONAL-FILE-001 — Explicit-position regular transfers

Model the approved [positional transfers](../../linux-kernel/design/positional-file-transfers.md)
using object-owned sparse pages, explicit positions distinct from description
positions, Linux append placement after supplied-interval admission, failure
atomicity and existing name/description lifetime. Preserve all earlier dispatch
versions and exclusive source-policy runtime ownership. Connect portable tests
to native, pinned-Linux and physical evidence without treating modeled numeric
offsets or descriptors as authority.

## TOPAL-SEM-SYSTEMS-NAMED-FILE-001 — Named volatile files

Model linked names, object identity, independent descriptions/positions and aliases separately. Page ownership shall follow objects even across independent reopening. Preserve data after unlink until the last independent description closes, retain closed linked objects under namespace ownership, issue fresh identities on name recreation and erase complete storage on final release. Cover exact names/metadata, capacities, error-state preservation, source-plan obligations and terminal reclamation with independent model cases.
See [design](../../linux-kernel/design/named-volatile-files.md).

## TOPAL-SEM-SYSTEMS-REGION-BORROW-001 — Scoped region calls

Model `TOPAL-SYSTEMS-REGION-LIBRARY-BINDING-001`,
`TOPAL-SYSTEMS-REGION-LIBRARY-CALL-001`,
`TOPAL-SYSTEMS-REGION-LIBRARY-ACCESS-001` and
`TOPAL-SYSTEMS-REGION-LIBRARY-LIFETIME-001` over the existing bootstrap-storage
resource. Cover exact owner/code/generation, contained/narrowed alias views,
current-frame access, all-argument prevalidation, nested revocation, budget and
identity exhaustion, bounds/value refusal, actual owner bytes and cancellation.
Rejected operations shall preserve underlying storage observations and model
state. Lifetime enforcement in a reference model shall not imply source/native
support or physical erasure. See [TK-DEC-269](../../linux-kernel/design/borrowed-kernel-regions.md).

## TOPAL-SEM-SYSTEMS-REGION-ROOT-001 — Root loan and byte knowledge

Model `TOPAL-SYSTEMS-REGION-ROOT-BINDING-001`,
`TOPAL-SYSTEMS-REGION-ROOT-CALL-001` and
`TOPAL-SYSTEMS-REGION-ROOT-RECLAIM-001` as explicit ownership/proof obligations.
Distinguish known bytes from unknown results of an opaque ordinary source call.
After such a call, keep extent/lifetime/rights checks while recording byte guards
as dynamic obligations, without claiming the model executed source or proved
unchanged contents. A store may restore knowledge of its overwritten byte.
Keep source/object and physical reclamation qualification distinct from the
resource model. See [TK-DEC-271](../../linux-kernel/design/root-borrowed-region-invocation.md).

## TOPAL-SEM-SYSTEMS-REGION-ACTION-001 — Persistent field loan obligations

Model `TOPAL-SYSTEMS-REGION-ACTION-OWNER-001`,
`TOPAL-SYSTEMS-REGION-ACTION-RETURN-001` and
`TOPAL-SYSTEMS-REGION-ACTION-POSITION-001` with actual controller-owned field
loans, same-owner alias effects, exclusive access, fresh code/generation and
complete reclamation before resumption. Retain the independent Linux position
reference laws, including append's effective base and positional nonmutation.
Do not claim opaque helper execution, unchanged source-written bytes or physical
qualification from a detached bootstrap buffer. Distinguish post-admission
invariant failure from ordinary Linux errors and rollback. See
[TK-DEC-272](../../linux-kernel/design/persistent-region-actions.md).

## TOPAL-SEM-SYSTEMS-REGION-INSTANCE-001 — Distinct framed caller domains

Retain independent root and persistent-field ownership domains when both select
the framed source context. Interface/export/instance labels shall not merge
owners or grant access. A root framed declaration shall retain the existing
single synchronous contained loan, exact zero-fact/Boolean expectation and
opaque source mutation. Source Boolean acceptance and resumed byte checks shall
remain runtime obligations rather than static predictions. Source false after
stores shall not imply rollback or owner resumption. Keep Unit and existing
field contexts usable with their original contracts. Source/object and physical
reclamation remain separate evidence. See
[TK-DEC-276](../../linux-kernel/design/instance-scoped-region-actions.md).

## TOPAL-SEM-SYSTEMS-REGION-PAGE-001 — Actual reusable-page owner loans

Model loans on the same persistent allocation bytes used by filesystem page
selection, release and normal reference laws. Keep live object/page authority,
bitmap/association/payload consistency, source views, generation and opaque
Boolean completion distinct. The exclusive controller shall validate complete
delta and reclamation before publishing an allocation or release; false,
abandonment or malformed completion shall poison the owner without rollback.
Before-entry rejection shall preserve storage and identity observations. Stale,
foreign and cross-position/page views shall reject even with equal labels or
numeric generations. Verify all occupancy patterns, release slots, reuse/zeroing,
shared lifetime and termination. Model flags and labels shall not constitute
compiled source, native authority or physical erasure evidence. See
[TK-DEC-278](../../linux-kernel/design/source-ram-page-allocation.md).

## TOPAL-SEM-SYSTEMS-REGION-UNSIGNED-001 — Actual contained field effects

Model explicit one-to-eight-byte unsigned formats over the same borrowed owner
bytes as existing access. Use arbitrary-precision range facts and complete
preflight; refuse invalid spans/rights/values before owner-payload observations
or mutation. Preserve surrounding bytes, alias effect identity, current-frame
exclusion, stale/foreign view refusal and successful stores across cancellation.
Exercise actual persistent field/map loans and their independent completion
guards. Format data and model cleanup flags shall not become native authority
or physical execution/erasure evidence. See
[TK-DEC-279](../../linux-kernel/design/unsigned-region-fields.md).

## TOPAL-SEM-SYSTEMS-INODE-MODE-001 — Owned shared inode mode transitions

Model `TOPAL-SYSTEMS-INODE-MODE-STATE-001` on the same actual metadata bytes
used by regular-file, mutable-directory and pipe observations and transfer/
lifetime laws. Keep mode/ctime shared across independent descriptions, aliases
and pipe endpoints; preserve type, atime/mtime, contents, positions, rights,
allocation and waits. Cover every permission pattern, native high-bit masking,
unchanged-mode ctime, read-only descriptions, retained unlinked objects, name
recreation, reuse, stale/foreign authority and complete failure-state preservation.
Directory namespace mutation shall retain its mode and pipe creation time shall
remain separate. Subsequent source-loan modeling shall retain independent
opaque-completion and cleanup obligations; direct model updates shall not claim
source/native execution or complete syscall error-order/physical evidence. See
[TK-DEC-280](../../linux-kernel/design/shared-inode-mode-actions.md).
