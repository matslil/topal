# Freestanding systems-profile semantics

## Formal text

### TOPAL-SYSTEMS-FEATURE-001 — Feature and artifact boundary

`systems` SHALL be a revisioned language feature selected through
`TOPAL-SYN-CONTEXT-001`. It SHALL add the sealed `lang systems` namespace and
SHALL add no grammar production. A source context selecting it SHALL be valid
only within one qualified freestanding systems root artifact. The same root
SHALL NOT use an ordinary hosted-process artifact profile.

A dependency which does not select `systems` SHALL retain its portable
semantics and vocabulary when composed into a systems artifact. Selection,
import, name lookup, or architecture-model access SHALL NOT itself grant
runtime authority.

### TOPAL-SYSTEMS-AUTHORITY-001 — Unforgeable systems authority

An entry, processor, privilege, address-space, mapping, interrupt, preemption,
fault-recovery, device, DMA, context, placement, or artifact capability SHALL
be an opaque resource identity constructed only by a qualified artifact or
provider transition. Ordinary or generated source SHALL NOT construct, copy,
widen, serialize, deserialize, compare for underlying identity, or derive such
authority from a numeric value, model fact, layout, or diagnostic projection.

Every systems operation SHALL consume or borrow the exact live capability for
its subject and SHALL record its effects, context, lifetime, failure, and
provider evidence. A target model SHALL establish legality facts only and
SHALL NOT satisfy runtime authority.

### TOPAL-SYSTEMS-VOCABULARY-001 — Initial sealed root construction

The initial systems source profile SHALL use ordinary application, function,
product, and binding syntax under the sealed `lang systems` namespace. A root
SHALL construct exactly one `lang systems artifact` whose `bootstrap-storage`
member is one `lang systems bounded-bootstrap-storage`, whose `bootstrap`
member is one `lang systems bootstrap-entry`, and whose `debug-break` member is
one `lang systems synchronous-exception-entry`, and whose
`local-notification` member is one `lang systems external-interrupt-entry`.
It SHALL also contain exactly one `deadline-notification` member constructed as
one `lang systems external-interrupt-entry`.
It SHALL also contain exactly one `kernel-thread` member constructed as one
`lang systems resumed-thread-entry`.
The storage constructor SHALL
contain exactly positive `capacity-bytes` and `alignment-bytes` natural-number
fields. Alignment SHALL be a power of two and SHALL NOT exceed capacity. The
bootstrap entry SHALL have one handler from `BootstrapContext` to
`BootstrapDisposition`; the debug-break entry SHALL have one handler from
`DebugBreakContext` to `DebugBreakDisposition`; and the local-notification
entry SHALL have one handler from `LocalNotificationInterruptContext` to
`LocalNotificationInterruptDisposition`; and the deadline-notification entry
SHALL have one handler from `DeadlineInterruptContext InitialMonotonicClock` to
`DeadlineInterruptDisposition InitialMonotonicClock`; and the kernel-thread
entry SHALL have one handler accepting `KernelThreadContext InitialProcessor`
and `SuspendedKernelContext InitialProcessor` and producing
`KernelThreadDisposition InitialProcessor`.

The only admitted initial handler operations SHALL be context-qualified
`console write`, `debug break`, local-notification completion,
deadline-notification completion, kernel-context terminal retirement, `resume`,
and `fatal`. Console write SHALL
borrow the live board console capability and this increment SHALL accept only a
static text value. Debug break SHALL borrow the bootstrap context and declare
one synchronous debug-break observation. Resume SHALL consume one live
completed entry context and resume exactly its interrupted continuation. Fatal
SHALL consume its live entry context and SHALL NOT return.

Every handler SHALL end in one disposition admitted by its context. The
context SHALL occur only as the receiver of an admitted operation, SHALL be
consumed exactly once by the final disposition, and SHALL NOT be rebound,
captured, stored, passed, returned, or used after consumption. Unknown
namespace members, extra artifact entries, mismatched handler classifiers,
duplicate roots, and ordinary-source use of any systems classifier or member
SHALL be rejected before lowering.

The bounded bootstrap pool SHALL be monotonic and SHALL NOT acquire a host
service. Each allocation request SHALL name a positive byte count, a
power-of-two alignment no greater than the pool alignment, and a closed
semantic placement. It SHALL return either one affine `BootstrapRegion` or one
sealed `BootstrapStorageErrorCode`: `invalid-request` for a malformed or
phase-invalid request and `exhausted` when aligned capacity is insufficient.
Releasing a region SHALL consume it without making its bytes available for a
later bootstrap allocation. Bootstrap completion SHALL require no live region
and SHALL make the complete pool reclaimable. No source operation SHALL expose
the model offset as a machine or physical address.

A live `BootstrapRegion` SHALL admit plain byte store and load only while
borrowed by its originating bootstrap execution. An access SHALL use a
zero-based offset strictly below the region byte count; a stored value SHALL be
a `Nat` no greater than 255, and a load SHALL return such a `Nat`. The initial
executable slice SHALL accept only statically known requests and offsets whose
alignment, capacity, value range, and bounds are proven before lowering.
Dynamic access SHALL remain unavailable until it carries a proof or an
explicitly specified failure result. It SHALL NOT be unchecked.

Allocation SHALL use the ordinary exhaustive `Result` decision, with an affine
region bound only in the `Ok` action and a sealed storage error bound only in
the `Error` action. Every reachable success action SHALL release its region
exactly once before the bootstrap context is consumed. Load and store SHALL
borrow rather than consume the region; release SHALL consume it. A region SHALL
NOT escape through return, storage, capture, message, task, foreign,
serialization, another entry, or an allocation-error action.

Plain region access SHALL NOT acquire volatile, atomic, device, DMA, firmware,
or user-memory semantics. Those domains SHALL require their separately typed
protocols. A provider MAY choose a target-specific concrete location and
instruction sequence only when it preserves the region's bounds, byte
contents, ownership, lifetime, and failure behavior without exposing its
address or allocation metadata to source.

The first semantic target profile SHALL record target
`x86_64-unknown-none`, board `topal-qemu-pc-q35-10.2`, and profile
`topal.systems.x86_64-qemu-pc-q35-10.2/1`. Executable qualification SHALL apply
only to the closed operation, provider, artifact, option, board, and physical-
evidence set recorded for that profile. Any unqualified extension SHALL fail
before producing an artifact.

### TOPAL-SYSTEMS-ENTRY-001 — Static special entry

`SystemEntry K` SHALL classify a static artifact declaration rather than an
ordinary callable function. `K` SHALL be one of `Bootstrap`, `SecondaryCpu`,
`SynchronousException`, `ExternalInterrupt`, `UserSyscall`,
`MachineCritical`, or `ResumeThread` in the initial profile.

An entry declaration SHALL identify one target profile, handler, opaque
context classifier, legal capability bundle, execution restrictions, and
closed disposition set. The context and capabilities SHALL be affine, SHALL
exist only for the dynamic entry extent, and SHALL NOT escape through return,
storage, capture, message, task, foreign, serialization, or another entry.

The backend SHALL own physical entry symbols, sections, alignment, machine
frames, stack transitions, saved state, security transitions, unwind/debug
information, and return sequences. Source SHALL NOT select or observe those
representations.

### TOPAL-SYSTEMS-DISPOSITION-001 — Complete entry disposition

An entry handler SHALL end in exactly one disposition admitted by its entry
kind. A disposition SHALL name the semantic continuation, such as resume,
return to validated user state, schedule, deliver a user exception, or fatal
transition. It SHALL consume the entry context and every outstanding affine
entry obligation.

The checker SHALL reject a disposition unless required interrupt
acknowledgement, masking/preemption restoration or transfer, address-space and
user-state validation, resource release/transfer, and recovery obligations are
proved. A disposition SHALL NOT be represented as an ordinary function return
in checked semantics.

### TOPAL-SYSTEMS-OBSERVATION-001 — Declared observation and linearization

A systems external observation SHALL have event
`Observe(s,q,p)`, where `s` is one live source capability, `q` is that source's
monotonic event identity, and `p` is a value or declared error satisfying its
selected layout. Its contract SHALL state ordering, duplication, loss,
acknowledgement, and completion rules.

A concurrent protocol MAY expose `Linearize(d,t)` for transition `t` in domain
`d` when several invariant-preserving transitions are enabled without an
ordering edge. Every permitted linearization SHALL preserve the protocol
invariant and external contract. The resulting events SHALL participate in the
observable trace and replay evidence.

Absent `Observe` or `Linearize` input/effects, systems code SHALL retain the
portable schedule-equivalence requirement. These events SHALL NOT authorize a
data race, arbitrary random choice, invented event, or removal, duplication,
merging, or speculation of a required observation.

### TOPAL-SYSTEMS-LOCAL-INTERRUPT-001 — Local-notification interrupt lifecycle

`local notification send` SHALL consume one admitted processor context and
produce one affine pending session with source identity `s`, the source's next
monotonic event identity `q`, and the exact prior local maskable-interrupt
state. The consumed context SHALL be invalid until the matching wait completes.
A second send, ordinary use of the consumed context, escape, duplication, or
ordinary completion while the session is live SHALL be rejected.

`local notification wait` SHALL consume that pending session. Successful wait
completion SHALL require, in order, `Observe(s,q)`, entry through the declared
local-notification external-interrupt entry, consumption of its completion
obligation, resumption of the interrupted continuation, and restoration of the
recorded prior maskable-interrupt state. Another event SHALL NOT satisfy the
wait. The provider MAY temporarily change local mask state inside the sealed
wait transition but source SHALL NOT observe or retain that state.

The external entry SHALL receive one affine
`LocalNotificationInterruptContext(s,q)`. `local notification complete` SHALL
consume it and produce one completed context admitting only the declared
resume or fatal disposition. Resume before completion, completion outside the
matching entry, duplicate completion, context escape, or ordinary return SHALL
be rejected. Completion SHALL discharge the target acknowledgement or pending-
state obligation; a zero-instruction completion SHALL require target evidence
that the complete obligation is otherwise satisfied.

The initial profile SHALL admit exactly one local-notification handler and one
send/wait event. It SHALL make no timer, bounded-latency, fairness, nested-entry,
SMP-delivery, shared-device-routing, or scheduler-disposition guarantee.

### TOPAL-SYSTEMS-MONOTONIC-CLOCK-001 — Monotonic-clock observation

The initial systems profile SHALL provide one opaque provider-created
`Clock InitialMonotonicClock` borrowed through each admitted processor context.
`monotonic clock now` SHALL borrow that context and clock and SHALL return one
immutable `Instant InitialMonotonicClock`. Neither authority SHALL be consumed
or widened by observation.

For clock identity `c`, its next observation identity `q`, and provider value
`t`, a successful observation SHALL record
`Observe(c,q,Instant(c,t))`. Observation identities SHALL increase in source
order. An accepted value SHALL be greater than or equal to the preceding value
accepted from `c`; equality SHALL be permitted. A decreasing value SHALL fail
closed and SHALL NOT be clamped, replaced, or exposed as an accepted instant.

Instants SHALL retain exact clock identity and SHALL NOT compare, subtract, or
substitute across clocks. The provider SHALL own counter selection, scale,
resolution, enablement, access, wrap extension, regression detection, and the
target state required for monotonicity. Source SHALL NOT observe a counter
address, register, instruction, width, frequency, calibration mechanism, or
wrap state. The compiler SHALL NOT invent, predict, merge, duplicate, remove,
or speculate a required clock observation.

The initial executable slice SHALL admit exactly two observations after its
local-notification wait and before its time-success marker. It SHALL make no
wall-clock, periodic-release, sleep, timeout,
bounded-latency, rate-accuracy, SMP, suspend, migration, userspace-ABI, or
real-time guarantee.

### TOPAL-SYSTEMS-DEADLINE-EVENT-001 — One-shot deadline event

Constructing `Deadline c` from `Instant c` and an exact positive `Duration`
SHALL preserve the absolute same-clock instant and SHALL fail on range
overflow. It SHALL NOT observe the clock. A deadline from another clock SHALL
NOT substitute, arm, or satisfy the event.

`deadline notification arm` SHALL borrow one admitted processor context,
consume one deadline, and produce one affine `ArmedDeadline c` with one new
monotonic event identity `q`. The deadline, armed event, and temporarily
consumed context SHALL NOT be duplicated, escaped, or used by another event.
`deadline notification wait` SHALL consume the armed event and return the
processor context only after the matching observation, typed entry, consuming
completion, and resumption have occurred.

Delivery SHALL record `DeadlineEvent(c,q,scheduled,observed)`, where scheduled
is the deadline's instant and observed is one newly accepted `Instant c`.
Observed SHALL be greater than or equal to scheduled. Late delivery SHALL
retain both values. If the deadline is already expired when armed, the event
SHALL become immediately deliverable against the original scheduled instant;
the provider SHALL NOT restart the relative duration.

The entry SHALL receive one affine `DeadlineInterruptContext(c,q,scheduled,
observed)`. `deadline notification complete` SHALL consume that context and
produce one completed context admitting only resume or fatal disposition.
Resume before completion, completion outside the matching entry, duplicate
completion, an early observation, or ordinary completion with a live deadline
authority SHALL be rejected.

The initial executable slice SHALL admit exactly one deadline constructed from
the second qualified monotonic observation and `1[ms]`, one arm/wait lifecycle,
and one declared handler. Comparator, timer, route, vector, controller,
acknowledgement, wait, frame, and return representation SHALL remain private to
the provider. Cancellation, rearming, periodic release, scheduler integration,
multiple outstanding events, bounded latency, rate accuracy, SMP delivery,
suspend/migration guarantees, and userspace timer ABI SHALL remain unavailable.

### TOPAL-SYSTEMS-MACHINE-001 — Closed machine-provider transition

A machine-provider operation SHALL have a stable closed identity and specify:
consumed and produced semantic state; privilege and resource identities;
legal execution contexts; fault and recovery behavior; ordering and visibility
relations; required architecture evidence; and an abstract model transition or
explicit unavailable-model result.

Only a language revision SHALL introduce a common machine-operation identity.
A common identity SHALL have one architecture-independent contract. A facility
without that contract SHALL use a target-qualified identity. No provider or
source package SHALL introduce an arbitrary instruction, register, clobber,
intrinsic, or opaque privileged transition.

### TOPAL-SYSTEMS-ADDRESS-001 — Disjoint address families

Systems address families SHALL include `Physical`, `KernelVirtual`,
`UserVirtual A`, `Device D`, `Dma I`, and `Firmware F`, each qualified by its
owning resource identity where shown. Numeric equality across families or
owners SHALL establish neither aliasing nor authority.

An address range SHALL record a target-qualified width and half-open
mathematical-natural interval whose endpoint calculation does not overflow
that width. Checked subrange and offset derivation SHALL preserve family,
owner, bounds, and no stronger authority. Decoding an external number SHALL
produce only an untrusted candidate until validated against one live range.
There SHALL be no general integer-to-address or cross-family cast.

### TOPAL-SYSTEMS-BOOT-MEMORY-001 — Affine boot-memory refinement

`describe memory` SHALL consume exactly one entered bootstrap context and its
opaque provider-private handoff. It SHALL be handled through an exhaustive
`Result` decision. The `Ok` action SHALL bind one `MemoryDescribedContext`; the
`Error` action SHALL bind one `BootMemoryFailureContext` which admits only a
fatal disposition. The consumed entered context SHALL be invalid on both
paths, and neither result context SHALL escape its entry extent.

A successful provider SHALL produce disjoint, nonempty, nonwrapping,
page-qualified `Physical` ranges classified as `Allocatable`, `Reclaimable`,
`Persistent`, `Reserved`, `Unusable`, or `Unknown`. Each range SHALL retain its
source classification and provider provenance. An unknown classification
SHALL NOT be allocatable. Overlapping claims SHALL be split and resolved so
that any non-allocatable claim prevents allocation and `Unusable` dominates.
Incomplete page-edge fragments SHALL remain reserved.

Before an `Allocatable` capability is produced, the provider SHALL subtract
every live bootstrap reservation, including applicable adapter, transition,
stack, kernel-image, bootstrap-storage, handoff, command-line, initramfs, and
firmware ranges. Each reservation SHALL retain an owner and reclamation
condition. Portable source SHALL NOT inspect a native handoff record, derive a
numeric physical address, or regain the entered context.

Malformed, overflowing, cyclic, contradictory, unsupported, or allocation-
empty input SHALL select the failure action. A provider SHALL NOT approximate
such input with host memory, fabricated RAM, or an unchecked range.

### TOPAL-SYSTEMS-FRAMES-001 — Affine physical-frame allocation

`create frame allocator` SHALL consume exactly one memory-described bootstrap
context through an exhaustive `Result` decision. Its `Ok` action SHALL bind one
`FrameAllocatorContext` retaining the preceding bootstrap capabilities and
exclusively owning the normalized allocatable-frame authority. Its `Error`
action SHALL bind one `FrameAllocatorFailureContext` which admits only a fatal
disposition. The consumed context SHALL be invalid on both paths, and neither
result context SHALL escape its entry extent.

An allocation SHALL borrow one live allocator and request a nonzero frame count
and power-of-two alignment in target-profile base-frame units. Success SHALL
produce one affine `PhysicalFrameExtent` carrying allocator identity, extent
identity, count, alignment, and provider provenance. Its base SHALL remain
opaque. The extent SHALL grant neither byte access nor mapping, device, DMA,
firmware, or user-memory authority. Malformed or exhausted allocation SHALL
select an explicit failure without invalidating the allocator.

Live extents SHALL NOT overlap each other, non-allocatable ranges, or retained
reservations. `release` SHALL consume an extent into exactly its originating
allocator. Cross-allocator release, double release, use after release, extent
escape, and a bootstrap disposition with a live extent SHALL be invalid.
Allocation choice and coalescing MAY vary only when success, failure, alignment,
non-overlap, ownership, and provenance observations remain equivalent.

The initial x86-64 executable profile SHALL admit only the static allocation
shapes `(frame-count: 1, alignment-frames: 1)` and `(frame-count: 3,
alignment-frames: 1)`. Either successful request SHALL produce one indivisible
extent and SHALL require release of that complete extent. The profile SHALL
reject every other count or alignment, a dynamic request, extent splitting,
and more than one simultaneously live extent before artifact publication.

The disjoint-live-extent refinement SHALL permit one exception to that last
restriction: after one `PreparedUserAddressSpace` owns the exact `(3, 1)`
extent, the same allocator MAY return one additional `(1, 1)` extent with fresh
identity. The complete returned interval SHALL be disjoint from the prepared
space's complete backing interval, all reservations, and all non-allocatable
ranges. The additional extent SHALL be released before user-state preparation.
No third live extent, alternative shape, split, address observation, or
recoverable disposition with the additional extent live SHALL be admitted.

The initial `PreparedUserAddressSpace` MAY consume that additional extent in
one `probe-guard` verification transaction. The transaction SHALL derive only
RW/NX user mapping authority, SHALL confirm one fixed byte store/load through
that mapping, SHALL revoke all derived access, SHALL complete the target
translation-invalidation contract, and SHALL return the exact input extent.
Success SHALL NOT be observable before revocation and completion. The prepared
address-space identity and backing SHALL remain unchanged, and user-state
preparation SHALL remain invalid until source releases the returned extent.

The affine refinement SHALL represent that transaction with four closed
operations. `user address space map` SHALL consume the additional extent and
produce one fresh opaque `LiveUserMapping`; `user mapping store byte` SHALL
consume it and produce `StoredUserMapping`; `user mapping load byte` SHALL
consume that refinement, compare the fixed byte, and produce
`VerifiedUserMapping`; and `user address space unmap` SHALL consume the
verified mapping and return exactly the original extent. Publication SHALL
complete before map success, and revocation plus target translation completion
SHALL precede unmap success. The prepared address-space identity and original
backing SHALL be invariant across all four operations.

Skipping, reordering, duplicating, or using a consumed refinement; releasing
the extent or preparing user state while a mapping is live; using another
address space, extent, layout, right, or byte; and recoverable disposition
after publication failure SHALL be invalid. The initial refinement SHALL admit
only the sealed `probe-guard` layout, RW/NX rights, and one fixed byte. A
bounded reuse refinement MAY admit exactly two explicit sequential lifecycles
over the same returned extent. Each map SHALL create a fresh opaque mapping
identity, and the second map SHALL begin only after first-generation revocation
and target translation completion. No first-generation access or refinement
SHALL survive or be accepted by the second generation. Release SHALL follow
the second unmap. Identity reuse, cross-generation substitution, release
between generations, a third generation, simultaneous mappings, and user-state
preparation before final release SHALL be invalid. The refinement SHALL expose
no address or descriptor value and SHALL NOT constitute a general user-mapping
or image-construction API.

The permission-tightening refinement SHALL insert `user mapping protect read
only` between store and load in each admitted generation. The operation SHALL
consume `StoredUserMapping` and SHALL produce a `ProtectedUserMapping` with the
same provider, address-space, mapping, backing, placement, memory-kind,
user-access, non-executable, and content identities but without write
authority. Revocation of writable authority and target translation completion
SHALL precede observable success. Load SHALL consume only that protected
refinement.

Store through protected authority, load or unmap through a stale stored
refinement, skipped or duplicate protection, permission widening, executable
authority, identity substitution, and recoverable failure after permission
publication begins SHALL be invalid. The initial profile SHALL admit only the
sealed RW/NX-to-RO/NX transition and SHALL expose no permission bit or
invalidation mechanism.

The executable-promotion refinement SHALL insert `user mapping permit execute`
between load verification and unmap in each admitted generation. The operation
SHALL consume `VerifiedUserMapping` and SHALL produce an
`ExecutableUserMapping` with the same provider, address-space, mapping,
backing, placement, memory-kind, user-access, content, and read-only identities
but with executable authority replacing non-executable authority. Executable
publication and target translation and instruction-observation completion
SHALL precede observable success. Unmap SHALL consume only that executable
refinement.

Promotion from writable, stored, protected, or otherwise unverified authority;
write-authority restoration; W+X authority; skipped or duplicate promotion;
stale refinement use; identity substitution; and recoverable failure after
publication begins SHALL be invalid. The initial profile SHALL admit only the
sealed RO/NX-to-RO/X transition and SHALL expose no permission bit, cache,
barrier, or invalidation mechanism.

The sealed image-role refinement SHALL classify generation one as
`ExecutableCodeMapping` through executable promotion. For generation two,
`user mapping seal immutable data` SHALL consume `VerifiedUserMapping` and
produce `ImmutableDataMapping` while preserving provider, address-space,
mapping, backing, placement, memory-kind, user-access, content, read-only, and
non-executable identities. The provider SHALL prove that no permission or
content changed. Unmap SHALL consume the exact role-specific final refinement.

Executable promotion of generation two, data sealing of generation one,
cross-role substitution, unclassified unmap, unauthorized write-authority
restoration, and W+X authority SHALL be invalid. The refinement SHALL expose no role encoding,
permission representation, or target validation mechanism.

The bounded resident-image refinement SHALL consume two separately allocated
one-frame extents into distinct sealed `probe-code` and `probe-data` mappings.
`ExecutableCodeMapping` SHALL remain live while the data mapping advances to
`ImmutableDataMapping` or `WritableDataMapping`; the provider SHALL prove the address-space backing and
both mapping extents are pairwise disjoint. The two finalized mappings SHALL
coexist with RO/X and role-appropriate RO/NX or RW/NX authority respectively.

Source SHALL unmap and release finalized data before unmapping and releasing
executable code. Each unmap SHALL revoke only its exact mapping, complete the
target-required observation boundary, and return its exact original extent.
User-context preparation SHALL require both mappings revoked and both extents
released. A third mapping, backing reuse, swapped placement or role, cross-role
unmap, code-first teardown, release while mapped, or recoverable exit with live
construction authority SHALL be invalid. This refinement SHALL NOT constitute
a general mapping collection, layout API, application image, or loader.

The resident-image readiness qualification SHALL emit exactly one
`TOPAL_KERNEL_USER_IMAGE_READY` console observation after
the generation-two data role is produced and before either finalized mapping is
unmapped. At emission, the exact generation-one `ExecutableCodeMapping` and
generation-two final data mapping SHALL both remain live. Emission SHALL
preserve their identities, backing, placements, contents, permissions, and
affine ownership and SHALL produce no mapping or access authority.

Emission before both mappings are finalized, after either unmap begins, with
substituted roles or identities, or with a missing, duplicate, or renamed
marker SHALL be invalid. The observation SHALL expose no address, permission
encoding, page-table representation, or target completion mechanism and SHALL
NOT imply readiness for user entry or constitute an application-image or
loader interface.

The bounded execution-residency refinement SHALL preserve the exact
generation-one `ExecutableCodeMapping` and generation-two
final data mapping across preparation, entry, syscall dispatch, contained
user-copy faults, terminal exit, and kernel-continuation restoration of the
sealed initial probe. Probe execution SHALL borrow but SHALL NOT consume,
duplicate, transfer, mutate, access, or execute either mapping.

After the probe returns, source SHALL emit exactly one
`TOPAL_KERNEL_USER_IMAGE_RETAINED` console observation while both finalized
mappings remain live. Source SHALL then unmap and release immutable data before
unmapping and releasing executable code. User-execution completion and address-
space reclaim SHALL require both mappings absent and both returned extents
released.

The provider SHALL prove that construction placements do not alias probe code,
stack, user-transfer candidates, or the deliberate fault-guard range. Entry
without both roles, teardown before exact return, mutation or substitution
during execution, missing or early retained-image observation, completion with
either mapping live, and recoverable exit with live construction authority
SHALL be invalid. This refinement SHALL NOT authorize execution of constructed
contents or define a loader or general process-image interface.

### TOPAL-SYSTEMS-WRITABLE-DATA-001 — Verified writable user-data role

`user mapping permit writable data` SHALL consume only the generation-two
`VerifiedUserMapping` and SHALL produce one `WritableDataMapping`. Provider,
address-space, mapping, backing, placement, memory-kind, user-access, content,
and non-executable identities SHALL be unchanged. The transition SHALL restore
write authority only for the declared data role, and target translation
completion SHALL precede observable success.

The operation SHALL be invalid for generation one, a mapping which has not
passed the read-only verification boundary, a stale or substituted identity,
an executable mapping, or a transition that can produce W+X authority. Unmap
SHALL consume the exact writable-data refinement and return its original
backing only after revocation and target completion. Source SHALL observe no
permission bit, address, page-table representation, invalidation instruction,
or barrier.

The initial x86-64 qualification SHALL use `WritableDataMapping` for the ELF
`PF_R | PF_W` data/BSS segment. Constructed user code SHALL write and read back
one byte inside the memory-only BSS extent before terminal success. The exact
`TOPAL_KERNEL_USER_IMAGE_WRITABLE_DATA_OK` observation SHALL occur once after
successful user return and before retained-image teardown. A missing, renamed,
duplicate, early, or late observation SHALL be invalid.

### TOPAL-SYSTEMS-USER-WRITE-001 — Bounded initial standard output

The initial-process profile MAY dispatch Linux-compatible `write` for
descriptor one when the return continuation lies in the finalized executable
mapping, length is from one through 256 bytes, and the complete overflow-free
candidate extent lies in a provider-proved readable user mapping. Dispatch
SHALL reject invalid descriptor, length, or candidate with the personality's
Linux-compatible error before dereference or output.

For a valid request, dispatch SHALL copy the complete caller-selected extent
into private kernel staging before any byte becomes externally observable and
SHALL retain no user pointer after the copy. It SHALL publish exactly the staged
prefix through the target's checked console effect and return the requested
length to the same revalidated user continuation. The provider SHALL NOT
require a single embedded payload or call-site address. A wrong entry state,
non-executable return, cross-hole or wrapping extent, partial or faulted copy,
short result, early output, retained pointer, duplicate output, or output after
terminal disposition SHALL be invalid.

This rule SHALL NOT expose a syscall number, register convention, instruction,
address, serial mechanism, arbitrary descriptor, descriptor table, VFS object,
partial-write policy, or unbounded output interface to source.

### TOPAL-SYSTEMS-USER-READ-001 — Bounded initial standard input

The initial-process profile MAY dispatch Linux-compatible `read` for descriptor
zero when the return continuation lies in the finalized executable mapping,
length is from one through 256 bytes, and the complete overflow-free
destination extent lies in a provider-proved writable user mapping. Dispatch
SHALL reject an invalid descriptor, length, wrapped or cross-hole extent, or
read-only destination with the personality's Linux-compatible error before
input consumption or user-memory access.

For a valid request, dispatch SHALL receive exactly the requested bounded count
into private kernel staging before beginning user publication. It SHALL then
perform one fault-contained copy of the complete staged prefix, retain no user
pointer, and return the requested length to the same revalidated continuation.
The provider SHALL NOT require a single input value, candidate, or call-site
address. Constructed qualification code SHALL validate its deterministic bytes
and result before releasing its bounded stack region. Source SHALL emit exactly
one `TOPAL_KERNEL_USER_READ_OK` observation after exact terminal return and
before the constructed-image entry witness. Wrong state, non-executable return,
partial reception, early publication, faulted or partial transfer, short
result, retained pointer, unbalanced stack, or missing, duplicate, early,
renamed, or late witness SHALL be invalid.

This rule SHALL NOT expose a syscall number, register convention, instruction,
address, serial mechanism, arbitrary descriptor, end-of-file, partial-read or
signal policy, line discipline, terminal interface, or unbounded input buffer
interface to source.

### TOPAL-SYSTEMS-USER-BRK-001 — Bounded initial program break

The initial-process profile MAY dispatch Linux-compatible `brk` when the user
context is live and the return continuation lies in the finalized executable
mapping. The provider SHALL derive the immutable initial break from the
validated runtime ELF data-memory extent and SHALL prove it lies within the
owned 4096-byte writable, non-executable data page. It SHALL zero the complete
page tail beginning at the runtime file-data end before user entry and SHALL
retain one current break in private process state.

A zero request SHALL return the current break without changing it. A byte-
granular request in the closed interval from initial break through data-page
end SHALL atomically become and return the new break. Any other request SHALL
leave state unchanged and return the prior break. Dispatch SHALL NOT return a
negative errno for an unsupported break. Data-role teardown SHALL clear the
private break before mapping revocation and backing release.

Compile-time initial-size authority, arithmetic wrap, page-tail allocator
residue, state publication before successful image construction, wrong entry
state, non-executable return, a result other than requested or prior break as
applicable, or live break state after data teardown SHALL be invalid. This rule
SHALL NOT expose a virtual address, page-table representation, state location,
syscall number, register convention, instruction, VMA collection, additional
page allocation, resource-limit policy, overcommit, `mmap`, `mprotect`, or
`munmap` interface to source.

### TOPAL-SYSTEMS-USER-IDENTITY-001 — Initial scalar identity coherence

The initial-process profile MAY dispatch Linux-compatible scalar process,
thread, parent, user, and group identity queries only while the user context is
live and the return continuation lies in the finalized executable mapping.
`getpid` and `gettid` SHALL return one, `getppid` SHALL return zero, and
`getuid`, `geteuid`, `getgid`, and `getegid` SHALL return zero. The credential
results SHALL agree with the initial `AT_UID`, `AT_EUID`, `AT_GID`, and
`AT_EGID` facts.

Dispatch SHALL retain no argument, pointer, or mutable identity state and SHALL
return to the same revalidated continuation. A missing family member, wrong or
incoherent result, wrong entry state, non-executable return, duplicate result,
or return after terminal disposition SHALL be invalid. This rule SHALL NOT
expose syscall numbers, register conventions, instructions, task structures,
namespace representations, credential storage, mutation, lookup, or general
process-management authority to source.

### TOPAL-SYSTEMS-X64-FS-BASE-001 — Initial x86-64 user FS base

The initial x86-64 profile MAY dispatch Linux-compatible
`arch_prctl(ARCH_SET_FS)` and `arch_prctl(ARCH_GET_FS)` only while the initial
user context is live and the return continuation lies in the finalized
executable mapping. `ARCH_SET_FS` SHALL accept zero or a canonical candidate whose
complete one-byte extent lies in an already-owned readable user mapping. On
success the provider SHALL retain that exact value as private thread state,
install it as the user FS base before return, and return zero.

`ARCH_GET_FS` SHALL accept only a complete writable eight-byte destination in
an already-owned user mapping, copy the retained value through a fault-
contained user-output scope, and return zero. Unsupported operation codes SHALL
return `-EINVAL`, an invalid base SHALL return `-EPERM`, and an invalid output
candidate SHALL return `-EFAULT`; these failures SHALL NOT change the retained
base. Data-role teardown SHALL clear private FS-base state before mapping
revocation and backing release.

Wrong set/get identity, noncanonical or unmapped bases, partial output, wrong
entry state, non-executable return, retained user pointers, duplicate result,
or live private state after teardown SHALL be invalid. This rule SHALL NOT
expose a syscall number, operation-code representation, instruction, register,
MSR, saved-frame layout, private state address, TLS layout, allocator, cloning,
or general architecture-control authority to source.

### TOPAL-SYSTEMS-USER-CLEAR-TID-001 — Initial thread exit word

The initial-process profile MAY dispatch Linux-compatible `set_tid_address`
only while the initial user context is live and its return continuation lies
in the finalized executable mapping. A nonnull candidate SHALL name a complete
writable four-byte extent in an already-owned user mapping; null SHALL disable
registration. Successful dispatch SHALL retain the candidate as private
thread state without dereference and SHALL return TID one.

Terminal user exit SHALL, for a nonnull retained candidate, write zero under a
fault-contained user-output scope. It SHALL clear private candidate state
after that store and before completing the user context or revoking any user
mapping. A clear fault SHALL fail closed. The one-thread profile SHALL NOT emit
a futex wake because no waiter can exist.

Invalid candidates, wrong results, retained user access outside terminal
clearing, partial stores, terminal transition before clear, duplicate result,
or live candidate state at mapping teardown SHALL be invalid. This rule SHALL
NOT expose a syscall number, user address, task structure, futex key or queue,
clone service, or general thread-lifecycle authority to source.

### TOPAL-SYSTEMS-USER-ROBUST-EMPTY-001 — Initial empty robust list

The initial-process profile MAY dispatch Linux-compatible `set_robust_list`
only while the initial user context is live and its return continuation lies
in the finalized executable mapping. The declared length SHALL equal the
native 24-byte head size and the candidate SHALL name a complete readable
24-byte extent in an already-owned user mapping. Successful dispatch SHALL
retain the candidate as private thread state without dereference and return
zero.

Terminal user exit SHALL copy the complete head into private staging under a
fault-contained user-input scope. The native list link SHALL equal the head
candidate and the pending-operation pointer SHALL be null. The provider MAY
ignore the opaque futex offset only because no entry is visited. It SHALL clear
private candidate state after validation and before completing the user context
or revoking a user mapping. A copy fault or nonempty state SHALL fail closed.

Wrong size, incomplete extent, registration-time access, partial copy,
nonempty terminal state, clearing before validation, duplicate result, or live
candidate state at mapping teardown SHALL be invalid. This rule SHALL NOT
expose a syscall number, user address, native field offset, futex word, list
walker, wake queue, clone service, or general robust-list authority to source.

### TOPAL-SYSTEMS-USER-AUXV-SECURE-001 — Non-secure initial execution

The sole initial process MAY receive one native `AT_SECURE` auxiliary-vector
entry. It SHALL follow the validated program-header fact group, SHALL precede
the unique `AT_NULL` terminator, and SHALL contain value zero. The value SHALL
agree with the sealed real/effective credential facts and the absence of any
set-user-ID, set-group-ID, file-capability, or security-module execution
transition.

The provider SHALL own the native tag, word layout, placement, and initial
stack construction. Constructed user code SHALL validate the exact ordered tag
and value, the following terminator, and complete bounded-block consumption
before continuing. Existing `argv[0]` and `AT_EXECFN` SHALL retain exact alias
identity after the stack layout grows.

A nonzero or missing value, wrong ordering, duplicate fact or terminator,
credential disagreement, broken alias, incomplete consumption, or unbounded
stack access SHALL be invalid. This rule SHALL NOT add privilege transitions,
capabilities, security modules, environment filtering, `AT_RANDOM`, or a
general auxiliary-vector builder.

### TOPAL-SYSTEMS-USER-AUXV-CLKTCK-001 — Userspace accounting tick rate

The sole initial process MAY receive one native `AT_CLKTCK` auxiliary-vector
entry. It SHALL immediately follow `AT_SECURE`, SHALL precede the unique
`AT_NULL` terminator, and SHALL contain Linux `USER_HZ` value 100. The value
SHALL NOT be derived from or interpreted as the qualified hardware clock period
or scheduler timer frequency.

The provider SHALL own the native tag, word layout, placement, and initial
stack construction. Constructed user code SHALL validate the exact ordered tag
and value, the following terminator, preserved `argv[0]`/`AT_EXECFN` alias, and
complete bounded-block consumption before continuing.

A missing, duplicate, reordered, or different value, confused clock identity,
broken alias, incomplete consumption, or unbounded access SHALL be invalid.
This rule SHALL NOT add process accounting, CPU-time clocks, interval timers,
scheduler ticks, `times`, or a general auxiliary-vector builder.

### TOPAL-SYSTEMS-USER-AUXV-PLATFORM-001 — Native platform string

The sole initial process MAY receive one native `AT_PLATFORM` auxiliary-vector
entry. It SHALL immediately follow `AT_CLKTCK`, SHALL precede the unique
`AT_NULL`, and SHALL point to a complete NUL-terminated platform string inside
the initialized bounded stack block. The initial x86-64 profile SHALL publish
exactly `x86_64`.

The platform string SHALL be a distinct object from the executable-name string.
`argv[0]` and `AT_EXECFN` SHALL retain exact alias identity only with the latter.
Constructed user code SHALL validate the tag, exact pointer, all platform bytes,
termination, following `AT_NULL`, alias separation, and complete bounded-block
consumption before continuing.

An invalid, cross-boundary, unterminated, aliased, missing, duplicate, or
reordered string, broken executable-name alias, incomplete consumption, or
using the fact to select a provider SHALL be invalid. This rule SHALL NOT add
hardware-capability masks, personalities, compatibility ABIs, runtime target
selection, or a general auxiliary-vector builder.

### TOPAL-SYSTEMS-USER-RLIMIT-STACK-001 — Initial stack-limit query

The initial-process profile MAY dispatch Linux-compatible
`getrlimit(RLIMIT_STACK)` and query-only
`prlimit64(self, RLIMIT_STACK, null, old-limit)` while the user context is live
and the return continuation lies in the finalized executable mapping. Self
SHALL admit PID zero or the sealed PID one. The old-limit candidate SHALL name
a complete writable 16-byte extent in an already-owned user mapping.

Both forms SHALL copy one native pair containing an 8 MiB soft limit and
`RLIM_INFINITY` hard limit through a fault-contained user-output scope, retain
no pointer, and return zero. Another process SHALL return `-ESRCH`; another
resource selector or a nonnull new-limit candidate SHALL return `-EINVAL`; an
invalid output candidate SHALL return `-EFAULT`, all without partial output.

Wrong values, partial publication, wrong entry state, non-executable return,
mutation acceptance, retained candidates, or duplicate result SHALL be
invalid. This rule SHALL NOT expose syscall numbers, selector encodings,
native field offsets, resource storage, accounting, enforcement, mutation,
inheritance, cgroups, or general resource-limit authority to source.

### TOPAL-SYSTEMS-USER-SIGNAL-MASK-001 — Initial thread signal mask

The initial-thread profile MAY dispatch Linux-compatible
`rt_sigprocmask(how, set, old-set, set-size)` while the user context is live
and the return continuation lies in the finalized executable mapping. Set size
SHALL be the native eight bytes. A nonnull set candidate SHALL name a complete
readable eight-byte extent and a nonnull old-set candidate SHALL name a
complete writable eight-byte extent in already-owned user mappings.

The private blocked-signal word SHALL begin at zero. For nonnull input the
adapter SHALL copy the complete word under fault recovery, remove the native
`SIGKILL` and `SIGSTOP` bits, and admit only block, unblock, or replacement.
It SHALL retain the prior word, commit the resulting private word atomically,
then publish the complete prior word through a nonnull old-set candidate under
fault recovery. Null input SHALL leave state unchanged and ignore `how`; null
output SHALL perform no user write. Terminal teardown SHALL clear private mask
state.

Wrong size or operation SHALL return `-EINVAL`; an invalid user extent SHALL
return `-EFAULT`. Partial input, partial output, wrong update ordering, failure
to remove either unblockable bit, state change after rejected input, wrong
entry state, non-executable return, retained candidates, or nonzero state at
mapping teardown SHALL be invalid. This rule SHALL NOT expose the syscall
number, operation encodings, bit positions, native layout, pending queues,
actions, delivery, signal frames, restart policy, or general signal authority
to source.

### TOPAL-SYSTEMS-USER-ANON-PAGE-001 — Bounded runtime anonymous page

`user address space reserve anonymous` SHALL consume one owned single-frame
extent and a matching prepared address-space authority, SHALL produce one
opaque affine reservation tied to both identities, and SHALL leave its
provider-selected user translation non-present. It SHALL NOT expose a numeric
address, target entry, page-table representation, or retained source copy of
the frame authority.

While the sole initial user context is live, the profile MAY admit
Linux-compatible `mmap` only for a null address, nonzero length no greater than
one base page, read-write protection, private anonymous flags, descriptor
minus one, and offset zero. If the reservation is non-present, the provider
SHALL zero the complete frame before ordered RW/NX publication, create one
fresh active generation, and return its page-aligned provider-selected user
address. Activation while already active SHALL return `-ENOMEM`; unsupported
arguments SHALL return `-EINVAL` without changing reservation state.

Matching Linux-compatible `munmap` with the selected address and nonzero
length no greater than one base page SHALL completely revoke an active
translation with ordered target completion and return zero. The same valid
call while non-present SHALL be a successful no-op. Terminal user completion
SHALL revoke any active generation. `user address space release anonymous`
SHALL require and consume the terminally non-present reservation, SHALL return
the exact original single-frame extent, and SHALL preserve the separately
owned address-space authority.

Pre-publication user access, incomplete zeroing, writable-and-executable
publication, overlapping active generations, stale access after revocation,
identity change, duplicate release, active release, or wrong frame return
SHALL be invalid. This rule SHALL NOT expose syscall numbers, native flags,
selected addresses, translation formats or instructions, VMA collections,
multiple pages, file backing, page-fault policy, sharing, overcommit,
concurrent edits, or general mapping authority to source.

### TOPAL-SYSTEMS-USER-ANON-PROTECT-001 — Bounded anonymous-page protection

While the sole initial user context is live and its bounded anonymous mapping
generation is active, the profile MAY admit Linux-compatible `mprotect` only
for the exact provider-selected page address, a nonzero length no greater than
one base page, and protection equal to none, read-only, or read-write. Any
executable or unsupported protection bit, different address, excessive or zero
length, or non-active reservation SHALL return the declared Linux error without
changing reservation, protection, or contents.

On success the provider SHALL replace the active translation with the requested
abstract permission, SHALL preserve the mapping generation and every backing
byte, SHALL retain non-executability, SHALL complete the target-qualified
publication and invalidation protocol before user return, SHALL record the new
private protection state, and SHALL return zero. None SHALL make the user
translation non-present while the reservation remains active; a later admitted
protection change SHALL restore access to the same generation and contents.

`munmap` and terminal cleanup SHALL revoke an active generation from any of the
three protection states. A second `mmap` while protection is none SHALL still
return `-ENOMEM`; only complete unmap makes the reservation available for a
fresh zero-filled generation.

Executable publication, writable-and-executable state, state or content change
after rejected input, access contradicting the recorded permission, generation
change, incomplete invalidation, duplicate result, or stale access SHALL be
invalid. This rule SHALL NOT expose syscall numbers, native protection bits,
selected addresses, page-table formats or instructions, protection keys,
multiple pages, partial splitting, concurrent edits, or general VMA authority
to source.

### TOPAL-SYSTEMS-USER-ANON-ADVISE-001 — Bounded anonymous-page advice

While the sole initial user context is live and its bounded anonymous mapping
generation is active, the profile MAY admit Linux-compatible `madvise` only
for the exact provider-selected page address, a nonzero length no greater than
one base page, and `MADV_DONTNEED`. A different address, excessive or zero
length, unsupported advice, or non-active reservation SHALL return the declared
Linux error without changing reservation, protection, or contents.

On success the provider SHALL zero every byte of the backing frame through
kernel authority, SHALL preserve the active generation and its current none,
read-only, or read-write non-executable protection, and SHALL return zero. The
operation SHALL remain valid while protection is none. A later admitted access
after restoring permission SHALL observe zero throughout the advised page.

Partial zeroing, generation or protection change, executable publication,
state or content change after rejected input, stale contents after success,
duplicate result, or dependence on user access SHALL be invalid. This rule
SHALL NOT expose syscall or advice numbers, backing addresses, zeroing
instructions, page-table formats, lazy faults, other advice classes, reclaim
pressure, or general VMA authority to source.

### TOPAL-SYSTEMS-USER-CLOCK-MONOTONIC-001 — Bounded monotonic clock query

While the sole initial user context is live and its return continuation lies in
the finalized executable mapping, the profile MAY admit Linux-compatible
`clock_gettime` only for `CLOCK_MONOTONIC` and one complete writable native
16-byte timespec in an already-owned user mapping.

The provider SHALL obtain exactly one observation from its qualified monotonic
source, convert it to nonnegative seconds and nanoseconds less than one billion,
publish the complete pair under fault recovery, retain no candidate, and return
zero. Successive successfully published pairs SHALL be lexicographically
nondecreasing. Unsupported clock identity SHALL return `-EINVAL`; an invalid
candidate SHALL return `-EFAULT` without reporting partial output as success.

Constant fabricated time, negative or unnormalized fields, regression, partial
publication, retained pointers, duplicate results, wrong lifecycle state, or
non-executable return SHALL be invalid. This rule SHALL NOT expose syscall or
clock encodings, counters, frequencies, MMIO, rollover state, realtime epochs,
vDSO, timers, adjustment, namespaces, or general clock authority to source.

### TOPAL-SYSTEMS-USER-CLOCK-RESOLUTION-001 — Bounded monotonic resolution

While the sole initial user context is live and its return continuation lies in
the finalized executable mapping, the profile MAY admit Linux-compatible
`clock_getres` only for `CLOCK_MONOTONIC`. A null result candidate SHALL return
zero without output. A nonnull candidate SHALL name a complete writable native
16-byte timespec in an already-owned user mapping.

The provider SHALL publish the exact normalized resolution of the same
qualified source used for monotonic observations, retain no candidate, and
return zero. The initial x86-64 Q35 profile SHALL publish zero seconds and ten
nanoseconds under fault recovery. Unsupported clock identity SHALL return
`-EINVAL`; invalid output SHALL return `-EFAULT` without reporting partial
output as success.

Inconsistent, negative, or unnormalized resolution, partial publication,
retained pointers, duplicate results, wrong lifecycle state, or non-executable
return SHALL be invalid. This rule SHALL NOT expose clock encodings, counters,
frequencies, MMIO, realtime policy, vDSO, timers, or adjustment to source.

### TOPAL-SYSTEMS-USER-ENOSYS-001 — Unsupported syscall fallback

While the sole initial user context is live and its return continuation lies in
the finalized executable mapping, any syscall number not selected by an
implemented dispatch arm SHALL return the native Linux `-ENOSYS` result through
the ordinary syscall return path.

The fallback SHALL NOT read or validate unused arguments, access user memory,
mutate kernel or process state, consume an observation, or terminate the user
context. After receiving the result, the same context SHALL remain able to
invoke an implemented syscall. Adding an implementation for a syscall SHALL
replace fallback selection for that number with the implementation's own
validated contract.

Success, a fabricated result, argument-dependent behavior, state mutation,
provider-owned syscall selection, non-executable return, or terminal
disposition SHALL be invalid. This rule SHALL NOT add syscall reflection,
seccomp, audit, tracing, or a general Topal error mechanism.

### TOPAL-SYSTEMS-USER-SCHED-YIELD-001 — Sole-runnable scheduling point

While the sole initial user context is live and its return continuation lies in
the finalized executable mapping, the profile MAY dispatch Linux-compatible
`sched_yield`. With no other eligible user context, dispatch SHALL preserve the
same context, address-space and thread identities and all private process state,
and SHALL return zero through the ordinary syscall continuation.

The syscall SHALL have an implemented dispatch arm and SHALL NOT be selected by
the unsupported-call fallback. The kernel SHALL own the no-peer scheduling
decision; the provider SHALL NOT fabricate a context transfer or result.

A nonzero result, state change, provider callback, terminal disposition,
fallback selection, wrong context identity, or non-executable return SHALL be
invalid. This rule SHALL NOT add runnable collections, priorities, affinity,
multiple user threads, preemption, migration, or SMP scheduling.

### TOPAL-SYSTEMS-MAPPING-001 — Linear mapping state

A location SHALL combine one live address range with layout, rights, lifetime,
supported access sizes/alignment, cache policy, and ordering domain. A mapping
SHALL be a linear resource relating source and destination ranges under page
layout, permissions, owner, lifetime, and translation-provider evidence.

Page-table editing SHALL require one exclusive builder. Activation,
permission change, unmapping, translation invalidation, and reuse SHALL follow
one declared state protocol and SHALL name every affected processor/scope.
Storage or address reuse SHALL be invalid until the provider's completion
contract establishes that no stale translation can authorize access.

`kernel map` SHALL borrow one live frame-allocator context and consume one
affine `PhysicalFrameExtent`. Success SHALL produce one affine
`KernelMapping` recording opaque source and provider-selected kernel-virtual
extents, rights, execution policy, memory kind, owner, lifetime, and provider
evidence without exposing either numeric base. Failure SHALL admit only the
declared context-preserving failure action or a consuming disposition; it
SHALL NOT duplicate or silently release the extent.

A live read-write, non-executable normal-memory mapping MAY authorize bounded
plain byte load and store. Such access SHALL NOT imply physical, user, device,
DMA, firmware, volatile, atomic, or executable authority. Writable execution,
out-of-bounds access, wrong-provider use, frame release while mapped, mapping
escape, use after unmap, and a disposition with a live mapping SHALL be
invalid.

`kernel unmap` SHALL consume the mapping, revoke its access authority, satisfy
the provider completion contract, and return exactly the original opaque frame
extent. Physical storage reuse SHALL remain invalid until this transition
completes. A provider MAY adopt a pre-existing sealed translation only when it
proves the same rights, lifetime, inaccessibility-after-unmap, and reuse
observations as a constructed translation.

Bootstrap translation replacement SHALL begin by borrowing one live allocator
and consuming provider-selected backing authority into one exclusive affine
`TranslationUpdate`. The initial request SHALL be exactly template
`bootstrap-equivalent` with page policy `provider-selected`. The update SHALL
retain a snapshot of the current qualified bootstrap coverage and access
observations without exposing backing addresses, table levels, entries, or
target state.

`translation commit` SHALL consume one live validated update into one inactive
affine `TranslationSpace`. `translation activate` SHALL consume that inactive
space and the current translation authority, perform the provider-required
publication, synchronization, activation, and completion relations, and
produce a refined bootstrap context owning the new active space. On activation
success, subsequent operations SHALL use only the refined context. On an
indeterminate activation result, the initial slice SHALL admit only fatal
termination.

The replacement SHALL preserve the existing bootstrap coverage and permissions
and SHALL NOT add user authority, widen access, or make physical and virtual
families interchangeable. Duplicate or escaped updates, use after commit,
activation before commit, wrong-provider activation, backing release while
owned by an update or space, reuse of the prior translation by ordinary source,
and disposition with a live inactive update or space SHALL be rejected before
lowering or fail closed.

An active translation edit SHALL consume one active context into one exclusive
affine `TranslationEdit`. While the edit is live, the prior context SHALL admit
no operation or disposition. The initial map request SHALL be exactly normal,
read-write, non-executable kernel memory with placement `provider-selected`.
Map SHALL consume one live frame extent into one provisional opaque
`KernelMapping`; that mapping SHALL admit no access before commit.

`translation commit` SHALL consume the edit, validate the complete candidate
state, publish the provider-private translation changes, perform the required
ordering and invalidation completion relations, and return a refined active
context. Commit failure with indeterminate target visibility SHALL admit only
fatal termination. On success, the provisional mapping SHALL become live and
bounded by its consumed extent.

Unmap in an exclusive edit SHALL consume the live mapping into a provisional
returned extent. That extent SHALL NOT be accessed, released, or remapped until
commit has removed the translation and completed the required invalidation
scope. After successful commit, the old mapping SHALL be unusable and the
returned extent SHALL regain ordinary frame ownership. Abandonment, duplicate
edit or mapping authority, access before map commit, access after unmap,
release before unmap commit, nested or concurrent edits, wrong-space commit,
and disposition with a live edit SHALL be rejected before lowering or fail
closed.

Source SHALL NOT observe or select a virtual address, granule, table level,
entry, address-space identifier, invalidation address, processor mask,
register, or instruction. The initial slice SHALL admit one live dynamic
mapping and SHALL leave user mappings, device memory, permission changes,
executable mappings, remote-processor shootdown, and provider-metadata
reclamation unsupported.

### TOPAL-SYSTEMS-RECOVERY-001 — Closed fault recovery and user transfer

A recovery scope SHALL name a finite fault class, admitted generated
operations, resource identities, result mapping, and cleanup. The backend
SHALL bind only generated fault sites in that scope to generated recovery
continuations. Source SHALL NOT observe, construct, or supply a faulting or
recovery instruction address.

A user transfer SHALL require one live `UserVirtual A` validation capability,
untrusted candidate range and maximum extent, direction, layout or byte policy,
kernel-owned storage, and partial-progress/interruption policy. It SHALL return
validated data, permitted partial progress, or a structured fault. Direct
source access through a user candidate SHALL be invalid.

The initial `copy_to_user` form SHALL derive exactly one opaque affine
`UserTransferScope P A T B I` from a live `UserSyscallContext P A T B I`.
Opening the scope SHALL suspend disposition of that entry. Each transfer SHALL
consume one scope and return exactly one refined successor with either
`Complete(copied = extent)` or `Fault(copied-prefix)`; reunion with the exact
successor SHALL be required before return or termination. Candidate overflow,
wrong identity, duplicate scope use, direct candidate access, and disposition
with a live scope SHALL be rejected before lowering or fail closed.

The initial executable x86-64 profile SHALL extend its generated probe with
native Linux syscall 63 (`uname`). One valid call SHALL receive a source-owned,
six-field, 65-byte-per-field `new_utsname` and result zero. One call whose
390-byte destination crosses from its writable user page into an unmapped
guard page SHALL enter only the generated page-fault recovery continuation and
return Linux `-EFAULT`. Prefix stores before fault MAY remain observable.
Unscoped or mismatched page faults SHALL remain fatal.

The initial `copy_from_user` form SHALL consume and return the same exact
entry-bound affine scope. Its destination staging SHALL remain inaccessible to
source unless the selected layout was copied completely and validated.
`Complete(value, copied = extent)` SHALL expose that value;
`Fault(copied-prefix)` SHALL expose no staging bytes or partial value.

The executable x86-64 profile SHALL then invoke native Linux syscall 35
(`nanosleep`) with a null remainder candidate. One call SHALL copy and validate
a native 16-byte `{ tv_sec = 0, tv_nsec = 0 }` and return zero without arming a
deadline. One call SHALL copy its first eight bytes before the final field
crosses into the unmapped guard page, recover only at the generated supervisor
read site, discard the staging value, and return Linux `-EFAULT`. A direction,
fault class, site, address, entry, extent, layout, or prefix mismatch SHALL
remain fatal or fail closed.

These initial forms SHALL NOT admit arbitrary extents or input layouts,
concurrent mapping changes, nonzero sleep, remainder output, interruption,
restart, general UTS namespaces, or core-language exception handling.

An unlisted fault, nested fault outside its own admitted scope,
machine-critical event, or corrupted entry/stack state SHALL NOT be caught by
this rule and SHALL follow its owning entry or fatal protocol.

### TOPAL-SYSTEMS-ATOMIC-001 — Atomic location and modification order

`AtomicLocation T D` SHALL be a live, aligned fixed-layout location for one
provider-supported scalar or tagged state `T` in synchronization domain `D`.
Its construction SHALL consume exclusive storage ownership; its destruction
SHALL prove that no atomic operation remains before returning storage to
non-atomic use or release.

For each atomic location `l`, all successful modifications SHALL have one
strict total modification order `mo_l` consistent with happens-before. Atomic
load, store, exchange, compare/exchange, and admitted read-modify-write SHALL
be indivisible and a read SHALL observe one value permitted by `mo_l` and the
operation's order. An atomic operation SHALL have an effect on `l` and `D`.

Plain conflicting access to live atomic storage SHALL be invalid. Atomicity
SHALL NOT grant access to overlapping storage, repair an invalid lifetime, or
imply device/DMA atomicity.

For the initial `atomic word` form, construction SHALL consume one live
ordinary region and select one naturally aligned provider-width unsigned word
wholly contained at the requested byte offset. The only admitted domain SHALL
be `cpu-shared`. The whole region SHALL remain unavailable for plain access or
release while the atomic location is live. `atomic end` SHALL consume the
location only when no operation remains in flight, retain its final word
contents, and return the same region identity and extent to plain ownership.

Initial compare/exchange SHALL return `Exchanged previous` exactly when the
observed value equals `expected`; that successful transition SHALL replace the
value with `desired` at one point in `mo_l`. Otherwise it SHALL return
`Observed actual` without modifying `l`. The source result SHALL be exhaustive,
and every ordinary continuation SHALL end the location before region release.
Source SHALL NOT observe a machine address, provider word spelling, register,
instruction, exclusive-monitor state, or retry attempt.

### TOPAL-SYSTEMS-ORDER-001 — Atomic and visibility relations

Atomic orders SHALL be:

- `AtomicOnly`: `mo_l` participation without an unrelated visibility edge;
- `Acquire`: `AtomicOnly` plus an edge from an observed release sequence to
  observations sequenced after the acquire;
- `Release`: `AtomicOnly` plus an edge from observations sequenced before it to
  an acquire which observes its release sequence;
- `AcquireRelease`: both relations for a modifying operation; and
- `Sequential`: `AcquireRelease` plus one strict total order over all
  sequential operations consistent with happens-before and each `mo_l`.

Compare/exchange SHALL declare success and failure orders separately; a
failure order SHALL be `AtomicOnly` or `Acquire`. A provider MAY strengthen an
implementation order but SHALL NOT expose a result forbidden by the selected
order.

The initial atomic-word increment SHALL admit `AcquireRelease` success and
`Acquire` failure for compare/exchange and `Acquire` for the following load.
These orders apply only to CPU-shared normal memory. Target implementations MAY
use one locked instruction, an architectural atomic extension, or a qualified
exclusive-reservation loop provided the observable result and progress
contract are preserved.

CPU memory, device/MMIO, DMA ownership, cache maintenance, instruction
synchronization, and translation maintenance SHALL be distinct relation
families. Each operation SHALL name direction, observations, agents, topology
scope, and completion. A zero-instruction lowering SHALL require evidence that
the target already guarantees the complete relation.

### TOPAL-SYSTEMS-CRITICAL-001 — Affine critical-state restoration

Entering a preemption or interrupt critical scope SHALL produce one affine
token containing processor identity, domain, nesting identity, and exact prior
state. The token SHALL be consumed exactly once by the matching restoration or
an admitted disposition transfer on every exit path. It SHALL NOT escape,
suspend, cross a processor, or be restored out of nesting order.

For the initial `local-maskable-interrupts` domain, `critical enter` SHALL
consume the current execution context and produce a context refined to that
domain plus opaque restoration authority for the exact immediately preceding
state. `critical restore` SHALL consume that refined context and authority and
produce the preceding context. A nested entry, where supported, SHALL allocate
a distinct nesting identity, and restoration SHALL consume identities in
last-in-first-out order. Completion, disposition, suspension, blocking, and
processor transfer with a live critical context SHALL be rejected.

Target interrupt flags, registers, controller masks, and instructions SHALL
NOT be source-observable values. A provider SHALL restore exactly the state
captured by the matching entry; it SHALL NOT unconditionally enable a domain
which was disabled before entry.

The exclusion established by a critical scope SHALL contain only the producers
named by its domain. Masking local interrupts SHALL NOT establish exclusion
from another processor, non-maskable event, device, or DMA agent. An operation
which may block SHALL be invalid in interrupt or machine-critical context.

### TOPAL-SYSTEMS-CONTEXT-001 — Linear execution-context transfer

A suspended execution context SHALL be an opaque linear resource owning its
kernel stack, saved target state, extended-state policy, thread identity, and
address-space relationship. It SHALL arise only from special entry,
initial-context construction, or prior qualified transfer.

Context transfer SHALL consume the running-context capability and exactly one
validated suspended context, perform its declared per-processor and
address-space transition, and resume exactly one continuation. It SHALL be
nonordinary control flow and SHALL NOT expose register slots, stack pointers,
or continuation instruction addresses to source. User/debug state SHALL use a
separate validated semantic view.

The initial executable slice SHALL construct exactly one
`SuspendedKernelContext InitialProcessor` by borrowing the admitted running
context and consuming one 16 KiB, 16-byte-aligned, bootstrap-reclaimable
ordinary region plus the static `kernel-thread` entry. Construction SHALL NOT
run the entry. The suspended context SHALL retain the originating pool,
region, processor, thread, address-space, entry, and extended-state-policy
identities without exposing their target representation.

The initial `kernel context transfer` SHALL consume the bootstrap running
context and that suspended context, suspend the caller at one typed
continuation, retain the active address-space identity, and resume the worker
on `InitialProcessor`. Local maskable interrupts SHALL be disabled at the
transfer boundary. The worker entry SHALL receive its running context and the
affine suspended caller.

`kernel context retire to caller` SHALL be a terminal worker disposition. It
SHALL be legal only with no live region, mapping, frame, atomic, critical,
interrupt, recovery, or cleanup obligation. It SHALL consume the worker and
suspended caller, mark the worker terminal, and resume exactly that caller.
The caller SHALL receive one affine `CompletedKernelContextTransfer
InitialProcessor` owning its restored running authority and the retired
worker's stack. `kernel context reclaim` SHALL consume that completion, return
the stack region to its originating monotonic pool, and produce the resumed
caller context. Use after transfer, duplicate transfer, wrong-processor or
wrong-entry use, retirement with a live obligation, ordinary worker return,
and disposition with a live suspended or completed context SHALL be rejected.

This slice SHALL NOT admit involuntary preemption, scheduler policy, run
queues, priority, timeslicing, blocking, general yield, arbitrary cancellation,
multiple threads, migration, SMP, address-space switching, user contexts,
floating-point or vector ownership, TLS/per-CPU switching, stack growth, or
unwinding across transfer. X86-64, AArch64, and RISC-V register and frame sets
SHALL remain provider evidence rather than portable source meaning.

### TOPAL-SYSTEMS-CONTEXT-002 — Cooperative context handoff

A qualified cooperative transfer SHALL consume one running context and one
matching suspended target. If the old continuation is later resumed, it SHALL
receive exactly one affine outcome which restores its running authority and
either returns the peer as suspended or owns the peer as terminal until
reclamation. An outcome SHALL retain provider, processor, address-space,
context, stack, entry, and continuation identities. Source SHALL NOT copy,
forge, widen, or reinterpret an outcome.

A closed entry protocol MAY statically refine the outcome when the checker can
prove whether that selected continuation hands off or retires. A mismatch
between the declared result classifier and the selected entry protocol SHALL
be rejected. `kernel context retire to target` SHALL consume the running
context and one matching suspended target at a transfer-safe point, mark the
running context terminal, and resume exactly the target with a
`CompletedKernelContextTransfer InitialProcessor`. Reclamation SHALL consume
the completion and return the retired stack to its originating pool.

The initial cooperative slice SHALL construct exactly two suspended contexts
from disjoint 16 KiB, 16-byte-aligned, bootstrap-reclaimable regions. Its
checked source order SHALL be cooperative-worker transfer and suspended
handoff, terminal-worker transfer and retirement/reclamation, cooperative-
worker resumption and retirement/reclamation, then bootstrap disposition with
no live context. The cooperative worker SHALL hand off exactly once before
retirement; the terminal worker SHALL retire on first entry. Target reuse,
wrong order, overlapping stack ownership, wrong result refinement, retirement
with a live obligation, and disposition with a live or unreclaimed context
SHALL be rejected.

Runnable selection SHALL remain source policy. The provider SHALL NOT own a
run queue or select the next context. This slice SHALL retain one processor,
one active address space, and disabled local maskable interrupts and SHALL NOT
admit involuntary preemption, blocking, dynamic run queues, priority,
timeslicing, cancellation, migration, SMP, user contexts, floating-point or
vector ownership, TLS/per-CPU switching, stack growth, guard pages, or
cross-transfer unwinding.

### TOPAL-SYSTEMS-CONTEXT-003 — Bounded affine runnable queue

A `BoundedKernelRunnableQueue P` SHALL be an opaque affine FIFO with retained
positive fixed capacity. It SHALL own each enqueued `SuspendedKernelContext P` and
retain the queue, provider, processor, active-address-space, context, and stack
identities needed for later selection. It SHALL NOT expose or confer access to
target context representation.

Enqueue SHALL consume the prior queue and one suspended context. Success SHALL
place that context at the tail; a full result SHALL return the unchanged queue
and context without copying or loss. Dequeue SHALL consume the prior queue. A
ready result SHALL return the oldest context and remaining queue; an empty
result SHALL return the unchanged queue. Static refinement of either result
SHALL require proof of the exact capacity and occupancy history.

A queue SHALL reject a running, retired, completed, duplicate,
wrong-processor, wrong-provider, or wrong-address-space context. Copying,
serialization, use after consumption, dropping a nonempty queue, and final
disposition with a live queue or queued context SHALL be rejected. Consuming
an empty queue SHALL discharge its final obligation.

The initial executable queue SHALL have capacity two. It SHALL enqueue the
cooperative context followed by the terminal context, dequeue the cooperative
context, enqueue its suspended handoff at the tail, then dequeue terminal and
cooperative in that order. Both terminal outcomes SHALL be reclaimed and the
empty queue SHALL be consumed before bootstrap disposition. The queue SHALL be
ordinary source policy and SHALL introduce no provider queue operation. This
slice SHALL NOT admit unbounded or dynamically allocated runnable storage,
preemption, blocking, wakeup, priorities, timeslicing, cancellation, migration,
SMP, or multiple address spaces.

### TOPAL-SYSTEMS-CONTEXT-004 — Deadline-preemptible context transfer

`kernel context transfer until deadline` SHALL consume one running dispatcher
context, one matching suspended kernel context, and one same-provider
`Deadline c`. Before interrupt delivery is admitted, it SHALL atomically bind
the dispatcher's exact suspended continuation as the sole preemption target
and arm one fresh event. The binding SHALL retain processor, provider,
address-space, clock, event, context, stack, and continuation identities. An
already-expired deadline SHALL become deliverable only after the target is
bound.

After consuming the matching deadline completion obligation,
`preempt current kernel context` SHALL consume the running selected context,
capture its exact interrupted continuation as one opaque suspended context,
and resume only the registered dispatcher. The dispatcher SHALL receive one
affine `PreemptedKernelContextTransfer P C` which restores its running
authority and owns that suspended context plus scheduled and observed delivery
evidence. Consuming the outcome SHALL return the suspended context for
source-owned enqueue. The outcome SHALL NOT be copied, dropped, forged,
widened, or reinterpreted as cooperative or terminal completion.

Preemption without a matching active transfer, before completion, for a wrong
clock, event, processor, provider, address space, context, or continuation, or
after resolution SHALL be rejected. The handler SHALL NOT inspect the
interrupted frame, access a runnable queue, choose another context, redirect
the disposition, or resume normally after selecting preemption. Later transfer
to the suspended target SHALL restore the exact interrupted machine
continuation and frozen affine source obligations. Before that restoration,
the provider SHALL bind the newly suspended dispatcher as the target's current
return continuation; terminal retirement SHALL NOT refer to the dispatcher
binding consumed by the earlier preemption.

The initial executable profile SHALL use one one-shot deadline to preempt the
first context selected from the capacity-two queue, enqueue that suspended
context at the tail, retire and reclaim the terminal context, then restore,
retire, and reclaim the preempted context before consuming the empty queue.
The first worker's sealed `kernel context await deadline preemption` operation
SHALL NOT return on initial entry. Matching deadline preemption SHALL suspend
it, and only later source-selected transfer to that exact context SHALL restore
the interrupted continuation and complete the operation. This operation SHALL
NOT define a general wait, sleep, blocking, or wakeup protocol.
It SHALL retain one processor and active address space and SHALL NOT admit
cancellation, rearming, periodic ticks, general timeslices, priority,
blocking, wakeup, nested preemption, migration, SMP, user contexts,
floating-point or vector ownership, TLS/per-CPU switching, stack growth, or
cross-transfer unwinding. Target interrupt frames, register sets, mask state,
stack-switch instructions, and interrupt-return mechanics SHALL remain
provider evidence.

### TOPAL-SYSTEMS-CONTEXT-005 — One-shot affine kernel completion

A one-shot `KernelCompletion P I` SHALL create exactly one affine wait endpoint
and one affine signal endpoint with matching fresh identity `I`. Neither
endpoint SHALL be copied, forged, serialized, substituted across processor,
provider, address-space, or completion identities, or survive completion.

A completion-bound transfer SHALL register the dispatcher's exact suspended
continuation before its selected worker may consume the wait endpoint. The
block and signal transitions SHALL have one declared linearization order. If
blocking linearizes first, the provider SHALL capture the exact worker as one
opaque `BlockedKernelContextTransfer P I` and resume only the registered
dispatcher. Consuming that outcome with the matching signal endpoint SHALL
produce one `SuspendedKernelContext P` for source-owned enqueue. Later transfer
to that context SHALL resume immediately after the block operation.

If signalling linearizes first, readiness SHALL persist and the matching block
operation SHALL complete without sleeping. Repeated wait, repeated signal,
wrong identity, wrong dispatcher, dropped blocked state, enqueue before wake,
and disposition with a live endpoint or blocked context SHALL be rejected.
The provider SHALL NOT select a runnable context or own a run queue.

The initial executable profile SHALL statically refine the block-first branch,
run and reclaim the terminal context, signal and enqueue the blocked worker,
then resume, retire, and reclaim it before consuming the empty runnable queue
and completion. It SHALL retain one processor, one active address space, one
waiter, and one signal and SHALL NOT admit multiple waiters, repeated signals,
predicate wait queues, spurious wakeups, timeouts, cancellation, process
signals, priority ordering, cross-processor wakeup, SMP, or user contexts.

### TOPAL-SYSTEMS-CONTEXT-006 — Affine user execution and syscall round trip

`ValidatedUserState A B` SHALL be opaque affine authority for one legal user
continuation in address space `A` under ABI personality `B`. A
`SuspendedUserContext P A T B` SHALL additionally retain processor `P` and
thread `T`. Source SHALL NOT inspect, construct, copy, serialize, or reinterpret
their instruction, stack, status, register, descriptor, privilege, frame, or
return representation.

Entering user execution SHALL consume one running kernel context and one
matching suspended user context. A syscall SHALL produce exactly one affine
`UserSyscallContext P A T B I` with fresh entry identity `I`. Its accessors
SHALL expose the ABI syscall number and six arguments only as untrusted
semantic machine words. The handler SHALL consume it through exactly one
closed disposition: `ReturnToUser(result)` SHALL encode one ABI result,
revalidate, and resume only the captured user continuation;
`TerminateUser(status)` SHALL consume that continuation and restore only its
registered kernel continuation with terminal status.

The backend SHALL own privilege transition, kernel entry stack, saved target
state, speculation/security sequence, result placement, return validation, and
legal fast or general return mechanics. A provider SHALL NOT select a syscall,
invent a result, expose its frame, or own Linux dispatch policy. Wrong
processor, address-space, thread, ABI, entry, or continuation identity;
duplicate entry or disposition; return after termination; and disposition with
a live syscall context SHALL be rejected.

Initial user-address-space preparation SHALL consume exactly one live
three-frame `PhysicalFrameExtent` and the sealed `initial-probe` layout into a
fresh opaque `PreparedUserAddressSpace A`. Initial user-state preparation SHALL
consume that exact space into its `SuspendedUserContext P A T B`. The user
context SHALL exclusively own the space and complete extent through every
entry, syscall return, and terminal transition.

Terminal completion SHALL revoke all mappings derived from the extent, satisfy
the provider translation-invalidation contract, and return exactly one opaque
`RevokedUserAddressSpace A`. Reclaim SHALL consume that revoked state and
return exactly the original indivisible extent. Wrong identity, reclaim before
revocation, duplicate transition, release, splitting, access, reuse, or
disposition while either address-space or user-execution state owns the extent
SHALL be invalid. Failure after publication begins SHALL be fatal-only in the
initial profile.

The initial executable x86-64 profile SHALL create one process/thread and one
generated ring-3 probe in allocator-backed validated RX and RW/NX user
mappings. The probe
SHALL invoke native Linux syscall 39 (`getpid`), verify the source-owned PID
result after one return to user mode, exercise syscall 63 (`uname`) through the
separately specified initial `copy_to_user` success and fault paths, exercise
syscall 35 (`nanosleep`) through the initial `copy_from_user` success and fault
paths, and invoke syscall 231 (`exit_group`) with status zero. Termination SHALL
restore the exact registered kernel continuation. The probe SHALL be
qualification evidence, not an executable format or loader. ELF, dynamic
linking, pointer-bearing syscalls other than sealed `uname` and zero-duration
`nanosleep`, general user copy, signals, restart, seccomp, audit, ptrace, TLS,
extended state, fork/clone, multiple processes or threads, user preemption,
migration, SMP, and the complete syscall table SHALL remain unavailable.

The initial profile SHALL admit one live `initial-probe` address space. It
SHALL NOT expose or admit source-selected virtual addresses, page-table levels
or entries, arbitrary permissions, ELF segments, dynamic stack construction,
concurrent address-space edits, multiple live spaces, or address-space
switching.

### TOPAL-SYSTEMS-DEVICE-001 — Register protocol access

A device location SHALL bind its layout to one live device session, permitted
access directions and widths, alignment, read/write side effects, reserved-bit
policy, ordering domain, and failure behavior. An access SHALL be an observable
effect and SHALL occur exactly as required by its protocol: it SHALL NOT be
invented, removed, duplicated, combined, or moved across a conflicting event.

Target-specific addressed-I/O facilities SHALL satisfy the same resource,
effect, context, and lifetime rules even when they are not MMIO. Knowledge of
an address or port number SHALL grant no session or access capability.

### TOPAL-SYSTEMS-PCI-CONFIG-001 — Bounded configuration observation

The bootstrap platform authority MAY derive one transaction-scoped PCI
configuration session for a checked dword observation. Its request SHALL carry
a segment in `0..65535`, bus in `0..255`, device in `0..31`, function in
`0..7`, register offset in `0..252` divisible by four, and expected
little-endian 32-bit value. Invalid or target-unrepresentable requests SHALL be
rejected before artifact generation.

The operation SHALL perform exactly one observable 32-bit configuration read,
consume the transaction session, and return the same bootstrap authority only
after equality with the expected value is proved. Access failure or mismatch
SHALL enter the fatal disposition. The read SHALL NOT be invented, removed,
duplicated, widened, combined, or reordered across a conflicting device event.

The provider SHALL own configuration-mechanism selection, access ordering,
serialization, privilege, and target fault behavior. Source and ordinary
kernel/library code SHALL own the PCI coordinates, register selection,
comparison, enumeration, matching, resource interpretation, and driver policy.
An x86 provider MAY use configuration mechanism 1; an ECAM provider MAY use a
qualified mapped window. Neither representation SHALL expose port or physical
addresses to source or grant device-register, BAR, interrupt, DMA, or virtio
authority.

The initial x86-64 Q35 profile SHALL admit only segment zero and the sealed
discovery sequence defined by `TOPAL-SYSTEMS-VIRTIO-PCI-DISCOVERY-001`. Its
first observation SHALL read register zero at `00:05.0` and expect the combined
modern virtio-rng device/vendor value `0x10441af4`.

### TOPAL-SYSTEMS-VIRTIO-PCI-DISCOVERY-001 — Bounded capability discovery

The initial x86-64 Q35 profile SHALL admit exactly the following ordered
configuration-dword observations for `0000:00:05.0`. Each tuple is
`(register-offset-bytes, expected-little-endian-value)`:

| Purpose | Required tuples |
| --- | --- |
| function identity | `(0x00, 0x10441af4)` |
| capability head and MSI-X link | `(0x34, 0x00000098)`, `(0x98, 0x00018411)` |
| virtio PCI-configuration-access capability | `(0x84, 0x05147009)`, `(0x88, 0x00000000)`, `(0x8c, 0x00000000)`, `(0x90, 0x00000000)` |
| virtio notification capability | `(0x70, 0x02146009)`, `(0x74, 0x00000004)`, `(0x78, 0x00003000)`, `(0x7c, 0x00001000)`, `(0x80, 0x00000004)` |
| virtio device-configuration capability | `(0x60, 0x04105009)`, `(0x64, 0x00000004)`, `(0x68, 0x00002000)`, `(0x6c, 0x00001000)` |
| virtio ISR capability | `(0x50, 0x03104009)`, `(0x54, 0x00000004)`, `(0x58, 0x00001000)`, `(0x5c, 0x00001000)` |
| virtio common-configuration capability | `(0x40, 0x01100009)`, `(0x44, 0x00000004)`, `(0x48, 0x00000000)`, `(0x4c, 0x00001000)` |

The model SHALL assign transaction identities 1 through 24 in source order.
It SHALL reject any omitted, duplicated, reordered, or substituted request.
The source/library interpretation SHALL own capability-link traversal,
vendor/type recognition, preferred-instance choice, capability-length checks,
BAR association, relative-region checks, and notification-multiplier policy.
The provider SHALL observe only independent checked PCI dword requests.

The sequence SHALL establish only the pinned transport layout. It SHALL NOT
grant or claim general PCI enumeration, assigned-BAR discovery or mapping,
bus mastering, device-register access, virtio feature negotiation, interrupts,
DMA, or entropy delivery.

### TOPAL-SYSTEMS-PCI-REGION-001 — Checked region-register access

The bootstrap platform authority MAY derive one transaction-scoped PCI region
session for one checked register read or write. A request SHALL carry a PCI
segment and BDF, selected PCI configuration-access capability offset, BAR index
in `0..5`, BAR-relative region offset and length, register-relative offset,
access width, direction, and expected or written value. The complete access
extent SHALL fit within the region without overflow. The value SHALL fit the
access width. Invalid or target-unrepresentable requests SHALL be rejected
before artifact generation.

Each operation SHALL perform exactly one observable register access of the
specified width and direction, consume the transaction session, and return the
bootstrap authority only after completion. A read SHALL prove equality or
enter the fatal disposition. A write SHALL publish exactly the supplied value.
Neither access may be invented, removed, duplicated, widened, combined, or
reordered across a conflicting event.

Source/library code SHALL own device identity, capability selection, BAR and
relative-region selection, register protocol, width, direction, value and
driver policy. The provider SHALL own configuration-gateway or opaque-mapping
realization, assigned-address resolution, ordering, serialization, privilege,
memory attributes, and fault behavior. Source SHALL NOT receive an assigned
BAR address, addressed-I/O port, or physical mapping token from this operation.

### TOPAL-SYSTEMS-VIRTIO-PCI-NEGOTIATION-001 — Bounded feature negotiation

The initial profile SHALL admit only region offset zero, length `0x1000`, BAR
4, configuration-access capability offset `0x84`, and function
`0000:00:05.0`. It SHALL perform exactly the following ordered accesses to the
virtio common configuration region. Tuples are
`(direction, register-offset-bytes, width-bytes, value)`:

| Phase | Required tuples |
| --- | --- |
| reset | `(write, 0x14, 1, 0)`, `(read, 0x14, 1, 0)` |
| acknowledge driver | `(write, 0x14, 1, 1)`, `(read, 0x14, 1, 1)`, `(write, 0x14, 1, 3)`, `(read, 0x14, 1, 3)` |
| observe device features | `(write, 0x00, 4, 0)`, `(read, 0x04, 4, 0x30000000)`, `(write, 0x00, 4, 1)`, `(read, 0x04, 4, 0x00000101)` |
| publish driver features | `(write, 0x08, 4, 0)`, `(write, 0x0c, 4, 0)`, `(write, 0x08, 4, 1)`, `(write, 0x0c, 4, 1)` |
| accept features | `(write, 0x14, 1, 11)`, `(read, 0x14, 1, 11)` |

The model SHALL assign transaction identities 1 through 16 in source order.
The selected driver feature words SHALL accept only
`VIRTIO_F_VERSION_1` from the exact observed device words. The read after
status value 11 SHALL establish that the device retained `FEATURES_OK`. The
profile SHALL leave `DRIVER_OK` clear until queue discovery, setup, and
population are complete. Any mismatch SHALL be fatal.

The profile SHALL NOT claim arbitrary register access, alternate features,
queue selection or setup, interrupt configuration, bus mastering, DMA,
notification, completion, entropy delivery, or failed-device recovery.

### TOPAL-SYSTEMS-VIRTIO-PCI-QUEUE-DISCOVERY-001 — Bounded queue discovery

After successful `TOPAL-SYSTEMS-VIRTIO-PCI-NEGOTIATION-001`, the initial
profile SHALL perform exactly the following ordered common-configuration
accesses. Tuples are `(direction, register-offset-bytes, width-bytes, value)`:

| Phase | Required tuples |
| --- | --- |
| observe queue count | `(read, 0x12, 2, 1)` |
| select queue zero | `(write, 0x16, 2, 0)` |
| observe selected queue | `(read, 0x18, 2, 8)`, `(read, 0x1c, 2, 0)`, `(read, 0x1e, 2, 0)` |

The model SHALL continue the PCI region transaction sequence with identities
17 through 21 in source order. The observations SHALL prove that queue zero exists, admits at
most eight descriptors, is disabled, and has notification identity zero. Each
access SHALL preserve exact two-byte semantics. Any mismatch SHALL be fatal.

The profile SHALL NOT derive a CPU or device-visible address, grant DMA
authority, resize or enable the queue, configure interrupts or bus mastering,
publish descriptors, notify or poll the device, deliver entropy, or set
`DRIVER_OK` from these facts.

### TOPAL-SYSTEMS-PCI-CONFIG-UPDATE-001 — Checked masked word update

The bootstrap platform authority MAY derive one transaction-scoped PCI
configuration-word update. A request SHALL identify segment and BDF, a
two-byte-aligned register offset in `0..254`, a nonzero 16-bit mask, expected
masked pre-state, and desired masked post-state. Expected and desired values
SHALL contain no bits outside the mask.

The provider SHALL serialize one exact-width read, pre-state comparison, one
exact-width write preserving every unmasked bit, and one exact-width readback
comparison. Failure of either comparison SHALL enter the fatal disposition.
The provider SHALL NOT widen the operation into an access that writes an
adjacent field.

After `TOPAL-SYSTEMS-VIRTIO-PCI-QUEUE-DISCOVERY-001`, the initial profile SHALL
admit exactly one request: segment zero, `00:05.0`, offset `0x04`, mask
`0x0004`, expected masked pre-state `0x0000`, and desired masked post-state
`0x0004`. This authorizes PCI bus mastering but SHALL NOT grant a DMA mapping,
device-visible address, buffer ownership, queue enablement, notification, or
`DRIVER_OK`.

### TOPAL-SYSTEMS-DMA-001 — DMA ownership protocol

A DMA buffer SHALL have exactly one of `CpuOwned`, `Prepared`, `DeviceOwned`,
`Completed`, `Failed`, or terminal reclaimed states. A transition SHALL name
direction, device and translation domains, mapping, cache/visibility work,
descriptor publication, notification, completion source, and failure/cleanup.

CPU access SHALL be invalid while ownership is `DeviceOwned`, except for an
explicitly modeled coherent shared protocol. Device notification SHALL require
completed preparation and descriptor visibility. Unmapping or reclamation
SHALL require completed/cancelled device ownership and every provider-specific
completion obligation. A coherent provider MAY perform no cache instruction
only with evidence preserving these same transitions.

### TOPAL-SYSTEMS-DMA-BUFFER-001 — Bounded CPU-owned DMA buffers

DMA-buffer creation SHALL identify a validated device, translation domain,
nonzero byte count, power-of-two alignment, transfer direction, visibility
policy, and semantic role. It SHALL derive a fresh opaque affine identity in
the `CpuOwned` state. Every live DMA buffer SHALL be disjoint from all other
live DMA, ordinary, user, bootstrap, and firmware allocations. Source SHALL
NOT observe its physical, virtual, bus, or I/O-virtual address.

After `TOPAL-SYSTEMS-PCI-CONFIG-UPDATE-001`, the initial profile SHALL create
exactly two buffers for device identity one and translation-domain identity
one, in order: identity one SHALL be a 4,096-byte, 4,096-byte-aligned
bidirectional `VirtqueueControl` buffer; identity two SHALL be a distinct
4,096-byte, 4,096-byte-aligned device-to-memory `EntropyPayload` buffer. Both
SHALL use provider-managed visibility and remain `CpuOwned` in this increment.

The provider SHALL prove storage capacity, alignment, disjointness, zeroed
initial contents, and compatibility with the selected device/translation
domain. Creation SHALL NOT prepare cache visibility, publish an address or
descriptor, grant device ownership, enable a queue, set `DRIVER_OK`, notify a
device, or report completion.

### TOPAL-SYSTEMS-STORAGE-001 — Allocation and semantic placement

A systems allocation SHALL be fallible and SHALL name one live pool/region,
alignment, address family, context legality, reclaim policy, and physical/DMA
constraints. Bootstrap allocation SHALL remain within its bounded artifact-
provided resource. No allocation or cleanup path SHALL acquire a host service
implicitly.

An ordinary kernel-owned region SHALL support bounds-checked plain byte load
and store while borrowed. Release SHALL consume the region, after which no
access is valid. Region access SHALL expose neither a machine address nor
provider allocation metadata and SHALL NOT be used for volatile, atomic,
device, DMA, firmware, or user memory.

Static placement SHALL use closed semantic requirements, including special
entry text, read-only or mutable data, per-processor template,
bootstrap-reclaimable data, page-aligned table, and boot-adapter input. Source
SHALL NOT name an object section, linker directive, or physical register. The
artifact provider SHALL prove that its concrete placement satisfies every
requirement.

### TOPAL-SYSTEMS-ARTIFACT-001 — Validated freestanding publication

A systems artifact SHALL record a distinct target profile, architecture model,
provider revisions, data layout, object format, relocation/code model, entry
set, placement plan, link inputs, debug/unwind identity, and provenance. It
SHALL contain no implicit host syscall, allocator, process startup, libc,
dynamic loader, foreign runtime, or ordinary process-termination path.

Linking and structural inspection SHALL complete before atomic publication of
the kernel, debug, map, and provenance outputs. Boot packaging SHALL be a
separately qualified adapter which records its boot/firmware contract and input
artifact digest. A fatal disposition SHALL enter a nonreturning provider whose
minimal path requires no general allocation, blocking lock, scheduler,
filesystem, or userspace service.

### TOPAL-SYSTEMS-QUALIFY-001 — Qualification and fail-closed support

Each systems element SHALL have semantic, context, authority, lifetime,
ordering, target, and unsupported-provider tests; an abstract model transition
or explicit model limitation; artifact inspection where lowering is physical;
and emulator or hardware evidence for each executable-qualified provider.

A common element SHALL be reviewed against x86-64, AArch64, and RISC-V before
admission. This review SHALL NOT executable-qualify those targets. A compiler,
interpreter, or provider missing applicable qualification SHALL reject the
feature, operation, target, or artifact before producing partial output and
SHALL NOT substitute a host mechanism.

## Graphical presentation

```mermaid
flowchart LR
    S[systems source] --> C[checked semantic element]
    M[qualified architecture model] --> P[sealed provider]
    A[live runtime capability] --> P
    C --> P
    P --> T[target transition or artifact]
    T --> E[trace and qualification evidence]
```

## Explanatory notes

The systems profile exposes machine interaction without making representation
the source model. A target may use an instruction sequence, runtime table,
generated entry stub, or no instruction at all when each refines the same
semantic transition.

Atomic and external-observation rules extend the execution relation only in a
selected systems context. They do not weaken ordinary Topal race rejection or
permit an optimizer to choose an application-visible interleaving.
