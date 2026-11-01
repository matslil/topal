# Source-produced structured retained state

`TK-DEC-264`; systems-only refinement adopted under standing authorization.
Design adopted; specification, implementation and qualification remain required.

The [first retained schema](retained-source-identity-state.md) borrows an actual
ordinary value but prepares a successor by constructing one fixed `next` Record
around a scalar result. Resource subsystems need source to produce their complete
next value, including nested fields and ordinary presentation order. Qualify that
boundary using a second, explicitly selected identity-state schema. This remains
bounded owned value retention; it adds no core mutation, allocation-region syntax,
resource-bearing stored field, concurrent writer or general persistent heap.

## Source contract and value law

Add `structured-process-identity-v1` to the existing optional
`process-identity-library` semantic slot. Preserve both earlier interface versions
and their source contracts. Exactly one version may occupy the slot. Keep explicit
state, invocation-stack and temporary-runtime reserves of 4096, 4096 and 65536
bytes. A separate source unit may implement the new signature; earlier source
files and imported value laws remain available for compatibility qualification.

The new ordinary source signature is:

```topal
Record (
  next : Nat,
  history : Record (creations : Nat, wrapped : Boolean)
) -> Record (
  allocated : Nat,
  state : Record (
    next : Nat,
    history : Record (creations : Nat, wrapped : Boolean)
  )
)
```

The existing next-ID law remains: zero initializes allocation to two, a legal
nonzero cursor allocates itself, and the successor increments or wraps from
2147483647 to two. Source additionally increments `history.creations` and retains
whether any source allocation has wrapped. Initial ordinary state is zero cursor,
zero creations and false wrapped flag. Initial ordinary presentation order
is `history,next`, with `creations,wrapped` inside history; later values preserve
the actual source-produced orders. The first successful state has at least
one creation and a legal nonzero successor. Initialized input has creations in
1..18446744073709551614; a successor may reach 18446744073709551615, at which
point another invocation fails before source execution. This bounded profile
does not change arbitrary-precision Nat meaning or admit truncation.

The creation count and wrapped flag are ordinary history facts. They mint no
process, generation, store or publication authority. Existing native resource
generation and private store epoch retain their ownership meanings. Neither
their numeric equality with source history nor a caller-supplied value creates
authority. History is internal state used to qualify structured retention; it
does not add a Linux UAPI capability or scalable PID namespace.

Source returns the whole `state` value. The adapter must preserve that actual
ordinary value, not reconstruct history or successor policy in native code. The
allocated ID and `state.next` may be projected to the existing private two-word
compatibility result after complete source/result validation. No other state
field need cross a scalar interface. The owning process alone publishes PID,
cursor and prepared state after complete invocation reclamation.

## Schema-directed projection and borrowing

Use a shared checked layout/projection description for retained ordinary
Records, nested Records, Boolean fields and bounded Nat nodes in the selected
schema. Derive canonical slots, nested aggregate types, alignment, padding,
per-value order suffix locations and numeric-node extents consistently with
ordinary lowering. A second handwritten function prototype or unrelated record
layout calculation is not an adequate compatibility argument.

Validate each stored reference against its exact schema-owned node before using
it. Check complete numeric sign/length/normalization, full finite payload and
range, Boolean representation, every Record order permutation, all padding and
clean unused bank storage. Initialized source must borrow the actual published
nested Record and both numeric nodes. It must not box cached scalar history.
Initial construction uses the shared ordinary runtime for zero values.

After validating the actual returned aggregate, project its `state` graph by
semantic field paths. Preserve both nested and outer state presentation order,
Boolean meaning and all numeric limbs. An equal immutable value may be copied
into independent owned nodes; source object addresses are not observable value
identity. Initialize private padding deterministically. No retained reference
may point into source temporaries, caller stack, an old bank or another store.
Malformed results must fail before any prepared graph/header mutation.

The first x86-64 realization keeps the 64-byte owner/version/preparation header
and two 2016-byte banks inside the existing 4096-byte store. The selected state
has two numeric nodes, each with a 24-byte reserved extent, and an ordinary
nested Record representation whose exact size must follow shared lowering.
The complete graph, all padding and clean tail must be independently checked.
Do not assume the flat one-field adapter or native result ABI describes the
larger ordinary aggregate. Qualify actual aggregate parameters/results and the
separate scalar native ABI through LLVM/object and independent native execution.
Other targets must qualify their own private layout and calling convention.

Use a layout description to generate projection and native prepared-graph
validation, so new field paths do not require unrelated ownership protocols.
The selected interface still fixes its semantic signature and finite domains;
this mechanism does not qualify arbitrary user-selected schemas or containers.
Require exact schema/layout and complete word/byte, copy/validation/retirement,
stack, temporary and persistent-capacity bounds in provenance. Separate graph
representation work from fixed context/binding bookkeeping. Measurements cannot
replace complete conservative bounds.

## Lifetime and compatibility

Retain the existing prepare-clean-publish-retire protocol. Preparation preserves
the old graph and published header/version. Complete arena and retired-stack
erasure/readback precedes exclusive owner publication. Reclaim the old graph
after its last borrow and before source reentry or userspace. The source history
survives exec and child/parent continuation restoration. Terminal cancellation
erases both banks and all header/code/store bindings exactly once.

Root arguments explicitly convey the selected store and code binding. Source
still receives only ordinary values, with no address, store, owner or native
layout facts. No additional identity domain, ambient store or fallback policy
may arise from adding another interface version. Keep one exclusive invocation
arena and the existing policy domain/tag. Wrong owner, schema, code, stale or
overlapping preparation, malformed state, generation exhaustion or incomplete
erasure fails closed. Existing recoverable creation checks precede preparation.

Scalar-only and first retained profiles must continue to execute independently
and alongside time/header/dispatch policies. Their old schemas, value laws and
limits remain intact. Metadata revisions must distinguish and reject missing,
cross-version, swapped or undercharged structured-state evidence.

## Qualification and risk

Continuing scalar reconstruction preserves the first profile but cannot qualify
source-produced nested values. Separate handwritten layout adapters duplicate
ordinary ABI knowledge and make drift harder to detect. Introducing a general
persistent heap immediately would also require broader ownership, allocation
and progress qualification. The selected schema-directed copy keeps existing
value/owner semantics while testing the additional representation obligations.

Risk is high at ordinary nested aggregate layout, per-value presentation order,
two-limb numeric preservation, pointer lifetimes, whole-state projection and
exclusive publication. Review against authoritative ordinary value/Record and
resource semantics, shared lowering, conservative evaluator bounds, generated
native objects and independent semantic/native/physical oracles. Human review
continues through the existing Step 4 PR.

Model and native tests must cover initialized/uninitialized state, repeated
version alternation, creation-count limb transitions and exhaustion, cursor wrap,
sticky wrapped history, both state Record orders, actual independent graph
ownership and malformed pointers, normalization, padding, Boolean/order metadata,
source results and publication sequencing. Rejection must preserve published
state and unprepared storage. Native tests must exercise the real ordinary
aggregate and private scalar conventions rather than a duplicate source model.

Physical evidence must observe source borrowing both published nodes and actual
history/Boolean/order values over successive creations, decode the actual whole
source result before projection, prove cleanup before publication, verify the
new graph survives temporary cleanup and observe complete previous-version
retirement and child publication. Reordered and standalone selections, controlled
wrap/count boundaries, changed-helper rejection before publication and full
termination are required. Identify controlled facts separately from naturally
reached history. Retain both earlier identity versions as compatibility controls
and refresh the complete existing runtime cohort on the new primary kernel.

General persistent allocation/effects and the remaining
[initial acceptance](../step-4-status.md) remain required. This selected structured
value is a foundation for larger source subsystems, not their implementation.
