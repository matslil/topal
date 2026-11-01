# Ordinary functions over borrowed kernel regions

`TK-DEC-269`; systems-only direction adopted under standing authorization.

The next compiler milestone is ordinary effectful kernel functions, rather than
another Linux operation implemented by a native adapter. Start with a borrowed
plain-memory region: the existing owning resource grants temporary access to
its bytes; ordinary functions operate through a sealed view; the owner resumes
with the actual resulting bytes. The first implementation stage is the portable
ownership/access model. Source checking, ordinary function lowering, artifact
binding and physical execution remain required before this interface is usable.

## Authority and ordinary language meaning

The view is systems vocabulary, not a new core reference or allocator. It can
only be derived from an existing, exclusively owned ordinary kernel region.
It does not authorize MMIO, atomics, DMA, firmware, userspace or arbitrary
numeric addresses. Ordinary function, immutable value, effect inference and
resource lifetime meanings remain unchanged. In particular, core `Memory`
locations are not redefined as shared mutable references, and core
`AllocationRegion` construction/failure policy is not invented here.

The owning resource is unavailable throughout the invocation. Initial execution
is synchronous on the same processor, with no suspension, concurrent writer,
callback, recursion or escaping view. A closed ordinary call graph may borrow
the region through parameters. Source cannot construct, serialize, return,
capture, persist or widen the view. A later compiler stage must check these
restrictions using typed identities; a numeric label cannot discharge them.

## Scoped views and calls

An invocation binds the exact owner, selected code identity, fresh nonwrapping
generation and resource bounds. A view records a contained byte window and
read or read/write rights. Derivation can narrow windows or rights, including
an empty window at the end; it cannot widen either. Aliases refer to the same
resource identity. Their effects conservatively conflict across the entire
owner, so an optimizer cannot reorder overlapping writes or turn an alias into
an independent resource. This stage introduces no concurrent alias semantics.

Entering a helper suspends caller access and lends selected views into a fresh
callee frame. Arguments are validated together before that transition. On
return, callee-local views are revoked before caller access resumes. Parent
views remain usable; views created in the callee do not escape. Generation,
frame and view identities are never reused within their lifetime. Complete
call depth and view budgets must be admitted before mutation, including aliases
and derived windows, rather than relying on observed high-water marks.

The initial model budgets at most 64 active frames and 256 live views per
invocation. These are systems qualification limits, not core function or type
limits. Native qualification must bind actual source/code identities and
conservative stack/allocation/access bounds; accepting a string in a reference
model does not constitute that evidence.

## Access, completion and failure

Byte load/store uses a checked view-relative offset. The resulting owner offset
must be representable and within the owned region. Reads require read rights;
writes additionally require write rights and values in 0..255. Validation of
the complete operation precedes access. Rejection must preserve bytes, active
frame, view registry, counters and underlying storage observations.

Successful stores update the same owned bytes; a later alias or caller reads
those bytes. Ending or cancelling an invocation revokes every view/frame and
returns the owner to ordinary use. It does not roll back earlier successful
stores or erase the owner's payload. If a subsystem needs transactional state,
it must prepare a distinct successor and publish it under its own protocol.
Source temporaries, adapter metadata, stack and code bindings still require
complete native reclamation before owner resumption; model revocation alone
does not prove physical erasure.

No source use after return/cancellation, owner release, stale generation,
inactive frame, wrong code binding, bounds overflow or rights escalation may
reach storage. A normal range/access refusal is a structured systems outcome;
forged ownership/code/lifetime evidence is an invariant failure, not a Linux
errno. General allocation failure and destructor policy retain their existing
core meaning and are not extended by this boundary.

## Integration and acceptance

The portable model borrows the existing bootstrap-storage owner, rather than
copying bytes into an unrelated simulator or inventing another allocator.
Rust borrowing prevents concurrent owner access in the model; explicit runtime
generation/frame checks model the obligations later source/native integration
must enforce. Tests must retain stale views across calls/invocations, compare
storage observations on rejection, exercise nested aliases and cancellation,
and test budget/identity overflow before publication.

Subsequent implementation must use the ordinary frontend/function/effect and
native lowering machinery with a closed module snapshot, qualified opaque
region transport and provider access. It must physically execute source helper
frames on owned bytes, preserve caller registers, reject changed/wrong bindings,
and reclaim all invocation storage before restoring the owner. Until then,
current pure interfaces and the primary kernel are unchanged and this milestone
is explicitly model-only. Broader subsystem state, general process/VFS/network
services, distribution replacement, containers and VM hosting remain obligations
in the [acceptance ledger](../step-4-status.md).

Risk is high at authority derivation, alias effects, hidden lifetimes, nested
frame revocation and recoverable refusal. Review follows the approved
[kernel state/ownership architecture](topal-kernel-architecture.md), core
[resources](../../docs/resources.md), [effects](../../docs/effects.md) and
[layout meanings](../../docs/layouts.md), then the normative rules and model.
Using integer addresses, pretending ordinary `Memory` values are mutable,
reusing a temporary arena as persistent state, or adding a native policy for
each Linux operation would not meet this milestone.
