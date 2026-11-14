# Owned traversal sessions

This refines `TK-DEC-282` Stage 2 under the approved systems-only direction.
The portable model borrows actual existing storage, retains complete scalar
values and retires source lifetimes explicitly. It is a reference contract;
model labels, copied values and cleanup observations grant no native authority.

The selected entry has one invariant region loan and scalar state. Its helper
graph remains closed and acyclic within a step. The existing region-library
binding and its view/call behavior remain unchanged. A separate traversal
wrapper exposes no derived-window operation. Helpers get fresh frame/loan
identities and may access only their current scoped view.

Before borrowing or advancing generation/history, admission checks all initial
native-word facts, scalar layout, complete step bound and identity requirements.
Frame/loan counters belong to the actual storage allocation and persist across
sessions. Admission uses their actual next values; descriptor reuse cannot
reset history. Each issued identity reserves a representable successor counter.
Runtime checks also enforce the admitted per-step and invocation budgets.

Source execution has distinct ready, running and returned phases. Scalars use
opaque session/epoch/allocation references into owned banks, work or exact
bound constants. A reference is not a numeric address. Aliases denote the same
immutable value. Transfer checks provenance, classifier, complete magnitude and
sign, and copies every successor into disjoint owned storage. No source/helper
frame remains active when the returned step reaches checkpoint retirement.

Checkpoint validates the whole successor and its exact admitted natural
decrement before erasing work and the old bank. It retires the previous root
loan/frame, issues fresh identities and installs the complete successor.
Retired numeric/work references and old views cannot be reused. A terminal
step preserves its actual result separately and reclaims both state banks.
Completion requires all explicit reclamation obligations before returning a
copied scalar/Unit result and releasing the borrow.

Pre-entry refusal preserves storage, observations, generation and history.
An invalid operation after admission, cancellation or abandonment preserves
performed owner stores/history, erases modeled private buffers, revokes loans
and records a fatal disposition. The model never represents that disposition
as a successful owner resumption. Native fatal recovery, kernel context, full
code/data/hook binding and physical erasure remain independent obligations.

Ordinary byte bounds/rights/value refusal remains separate from protocol
failure: source load/store return None/false without any storage observation or
state/history change. Natural indices and byte values are checked completely,
including values above a target word. Invalid current-view provenance takes
precedence over a normal refusal and reaches fatal cleanup. This retains the
existing source member meaning and does not narrow arbitrary-precision values.

The model's canonical numeric buffers exercise complete transfer and retirement;
their heap representation is not a target stack or complete temporary-size
certificate. The target must independently account for banks, source work,
metadata, descriptors, recovery cells, alignment and every reached frame under
the existing 65536-byte temporary and 4096-byte source-stack limits. Future
allocator/metadata owners must lend their actual storage and independently
validate their completion invariants.
