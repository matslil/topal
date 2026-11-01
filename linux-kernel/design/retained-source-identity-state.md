# Owned retention of an ordinary source state value

`TK-DEC-263`; systems-only refinement adopted under standing authorization.
Design adopted; implementation and physical qualification remain required.

The [source identity transition](source-process-identity-transition.md) computes
the next process-ID state in Topal, but the owning resource retains that state
only as native scalar facts. Larger source subsystems also need ordinary values
to survive a call. Qualify an explicitly owned retained-value boundary, first for
this existing one-field numeric state, without weakening temporary-only profiles
or claiming a general heap, arbitrary persistent values or effectful functions.

## Selected source and storage profile

Add interface `retained-process-identity-v1` in the existing optional
`process-identity-library` slot. Its ordinary declared signature and exact value
law remain `Record (next : Nat) -> Record (allocated : Nat, next : Nat)` from
`TK-DEC-262`. The source policy/helpers need no storage or machine vocabulary.
The interface additionally requires `state-limit-bytes is 4096`, alongside the
unchanged 4096-byte invocation stack and 65536-byte temporary-runtime limits.
Previous `process-identity-transition-v1` declarations have no retained-state
reserve and retain their existing copied-scalar contract. Missing, conflicting
or unqualified state metadata must fail before publication/use.

The new profile's owner retains an actual ordinary `Record (next : Nat)` value,
including its numeric object and ordinary order metadata, between successful
creation operations. A later source invocation borrows that published value;
it must not reconstruct its input by boxing a cached native cursor. The native
cursor may remain as a checked compatibility fact, but disagreement with the
retained source value is an invariant failure before source execution.

The uninitialized store denotes the existing zero sentinel. The first invocation
constructs the ordinary zero state through the shared runtime and source receives
that value. Every successful proposal thereafter retains its named `next` field
as the next ordinary state Record. The allocated PID still leaves the bridge only
as a validated scalar. The process resource retains all identity/lifetime authority.

This is an extension of systems qualification and kernel-owned storage lifetime.
It does not introduce a source `AllocationRegion` form, opt into core `v0.2`,
change ordinary immutable/Record/destructor meanings, admit resource-bearing
numeric fields, expose a pointer or add mutable-reference syntax. The first
retained schema is exactly one finite Nat field with the existing cursor range.
Other schemas and general persistent allocation require their own qualification.

## Portable ownership and version protocol

One explicitly reserved store belongs to the root-owned process-identity resource
and is borrowed for the prepared process lifetime. Its private identity, selected
code/type binding, published version, preparation state and complete storage
belong to that owner. A numeric PID, equal payload, address-shaped fact or source
Record never creates store authority. The root passes a qualified binding; the
runtime must not discover an ambient store.

The store has two private version banks. The published bank is immutable during
source evaluation. The inactive bank must be clean before preparation. The
generated adapter validates the complete selected input representation and passes
the ordinary Record through the unchanged shared function ABI. It checks both
actual named result fields and their ordinary order suffix. After validation it
performs a checked semantic copy of `next` into an ordinary Record/numeric object
in the inactive bank. No reference into the invocation arena or stack may remain
in that prepared graph. The ordinary value is preserved, including numeric meaning;
native layout is a private implementation/proof obligation.

Preparation does not publish a version or change the current cursor/child ID.
The source call ends and all temporary allocations, active arena state, retired
stack and temporary source pointers are erased/read back. Only then may the owner
commit the prepared version and the existing allocated-ID/cursor result as one
exclusive construction transition. The previous source version is erased/read
back after its last borrow and before resuming userspace or invoking source again.
Prepared state and its one-use publication obligation are cleared on completion.
No observer may see a partially committed version/PID combination.

Existing creation checks and temporary request staging precede preparation.
Existing native resource, child-snapshot, inheritance, readiness, wait/reap and
error rules remain in force. Recoverable prechecks leave the store unchanged.
Wrong owner/type/code, stale or overlapping preparation, malformed graph/result,
exhaustion, inconsistent cached facts, generation overflow or incomplete erasure
fails closed. An invariant failure is not a new Linux errno path. The new profile
must not fall back to copied-scalar/native policy when a declared store fails.

The initial realization retains single-owner, masked-interrupt invocation and
publication. It does not qualify concurrent state writers, lock-free publication,
SMP sharing or general snapshots. Another target must qualify its own private
representation, caller ABI, storage realization and ordering while preserving the
same immutable-version/owner protocol.

## Initial native realization and bounds

The first x86-64 store is a separate explicitly bound 4096-byte supervisor-owned
region, disjoint from the temporary arena, user mappings, existing backing and
other live resources. Its private 64-byte header leaves two 2016-byte banks.
For the selected ordinary schema a bank holds a 16-byte Record, including its
order suffix, and one aligned 24-byte numeric object. Publication must prove that
the complete 40-byte graph and all required metadata fit, with the remainder clean.
This representation is not a source ABI or a portable address convention.

The private adapter's native arguments may carry the qualified store binding,
expected owner/code and cached cursor facts. The caller must establish their
complete resource ownership before access. The adapter must check non-null,
non-overflowing extent and the selected private owner/code/version/graph shape
before using a graph reference. A stored numeric pointer must match its exact
owned bank node before dereference. Complete payload/range/order validation must
precede constructing the ordinary input. A malformed pointer cannot authorize a
load elsewhere.

The generated copy must follow the reviewed ordinary numeric/Record layout and
preserve finite values exactly, without truncating or inventing a different
integer algorithm. The first selected range is 2..2147483647 after initialization.
The source algorithm remains arbitrary-precision Topal. The bound and selected
protocol range restrict this interface, not the language's numeric meaning.

Complete artifact proofs must separately account for persistent header/banks,
maximum retained graph, preparation/retirement work, temporary input/source/
adapter work and actual aggregate/native call frames. All work must fit the
unchanged invocation reserves and the explicitly declared state reserve; measured
high-water marks do not establish these proofs. No hosted allocator, foreign
runtime or writable library global is introduced.

Exact source/module bytes, interface and state schema, checked projection, private
store layout, generated bridge, shared runtime, native objects, storage binding,
complete call/iteration/resource bounds and version protocol enter provenance.
Boot adaptation must reject missing, swapped, tampered, stale or undercharged
store evidence. The scalar-only identity interface and other numeric policies
retain their existing contracts and remain independently selectable.

## Lifetime, exec and termination

Retained identity state is shared resource history, not a restorable per-child
snapshot. Child execution or parent continuation restoration must not resurrect
an older version. Existing child-creation ownership checks continue to restrict
who can propose an allocation. Exec retains the selected store and its published
ordinary value while erasing invocation storage normally.

Terminal process return/cancellation must end any remaining borrow/preparation,
erase/read back both banks and all private header/version/ownership state, and
consume the store binding once. The root receives a clean complete region. All
selected identity/dispatch/header/deadline code/arena bindings must also be erased.
Only one identity interface version may occupy its semantic slot; selecting two
different versions must not create two independent allocator identities.

## Qualification and review

Risk is high at longer-lived source storage, alias/lifetime preservation, ordinary
aggregate representation, preparation versus publication, shared process history
and complete retirement. Review follows authoritative immutable/resource intent,
the existing ordinary lowering/layout, explicit owner state transitions, exact
native objects, conservative resource proofs and independent semantic/physical
oracles. Human code review remains required through the Step 4 PR.

Qualification must cover initialized/uninitialized and maximum/wrap state, repeated
version alternation, real source arguments pointing into the previously published
ordinary graph, exact copied next-state graphs, both scalar outputs and actual
ordinary Record order. It must prove the old graph survives preparation, temporary
storage is fully reclaimed before publication, the new graph survives invocation
cleanup, old storage is erased after last borrow, and later callbacks/child/exec/
parent restoration consume the correct version. Standalone retained-policy and
combined-policy configurations and terminal cancellation must be exercised.

An independent native client must execute the actual interface/store/calling
conventions and reject malformed facts, owner/code/version/order/pointer/payload
shapes, incomplete clean storage and invalid preparation/commit order. Model
tests must reject wrong ownership, duplicate semantic policy versions, overlap,
stale publication, resurrection and incomplete reclamation without mutation.

Controlled copied facts may exercise wrap, but must be identified separately
from normally reached persistent versions and must preserve the published state
until the qualified commit. A changed imported helper must produce a retained
proposal rejected by an independent oracle after temporary cleanup and before
owner publication, with root/provider objects unchanged. Existing scalar-only,
numeric/header/dispatch, process/pair/FIFO and complete userspace regressions remain
required on one primary kernel.

Alternatives retain only native scalar state, retain pointers into the erased
temporary arena, or introduce a general heap before its bounds/lifetimes are
qualified. The selected owned-copy/version boundary advances ordinary value
retention while keeping the existing temporary-only profiles intact. General
typed persistent allocation, effectful subsystems and the remaining
[initial acceptance](../step-4-status.md) remain required.
