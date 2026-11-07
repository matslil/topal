# Step 4: whole source-produced structured state

`TK-DEC-264` implements the [structured-value design](design/structured-retained-source-state.md),
`TOPAL-SYSTEMS-RETAINED-STRUCTURED-STATE-001` and
`TOPAL-SYSTEMS-STRUCTURED-PROCESS-IDENTITY-001`. Source now returns its complete
nested successor value. The adapter retains that actual ordinary value through
a checked layout/projection description shared with ordinary lowering and native
validation. It preserves both numeric nodes, Boolean history and nested Record
presentation order. The [earlier retained interface](step-4-retained-source-identity.md)
and [scalar interface](step-4-source-process-identity.md) remain compatibility
controls. [Step 4](step-4-status.md) remains unfinished.

The primary kernel SHA-256 is:

```text
0780a77f1d5161d7703f953141cdb86b52b02233986f6832026838f48556a9dd
```

Artifact/root/provider/object/boot/gate revisions are
`240 / 226 / 227 / 219 / 140 / 240`. Numeric metadata, allocation certificate
and owned numeric machine profile use version eight. The new state certificate
is `topal.retained-structured-state/1`, with layout description
`topal.retained-value-layout.x86_64/1`. The earlier retained certificate advances
to version two to include additional explicit source-result work evidence.
Ordinary language meaning and the existing process model are unchanged.

## Complete ordinary value boundary

The new [policy](kernel/policy/structured-process-identity.t) selects
`structured-process-identity-v1` in the existing identity slot. Its state is
`Record(next:Nat, history:Record(creations:Nat, wrapped:Boolean))`; its result is
`Record(allocated:Nat, state:State)`. The existing
[allocation/successor helpers](kernel/support/process-identity.t) preserve the
old ID law. A new [history helper](kernel/support/identity-history.t) increments
the complete creation count and retains whether allocation has wrapped. These
are ordinary facts, independent of private owner generations/epochs; they grant
no authority and add no Linux UAPI capability.

The initial value has zero cursor/count and false wrapped history, with canonical
initial presentation order. Later values have legal next IDs and positive counts.
An input count may reach 18446744073709551614, producing a successor count of
18446744073709551615; another invocation rejects before source execution or
allocation. Nat retains arbitrary-precision meaning. The selected interface
bounds persisted values and checks the complete result instead of truncating it.

The ordinary x86-64 state Record occupies **40 bytes**, including a **24-byte**
nested history Record. Each numeric node reserves **24 bytes**; the complete
independent graph occupies **88 bytes**. The result Record occupies **56 bytes**.
An independent C aggregate probe agrees with the shared ordinary layout on
size, nested offsets, Boolean placement and all order-suffix offsets. Generated
LLVM uses shared ordinary type construction and actual aggregate transport;
the separate private native interface still returns only allocated/next words.
The physical witness decodes the complete actual ordinary result before any
retained projection or scalar adaptation.

Initialized source arguments contain pointers to both actual published numeric
nodes. The adapter validates the stored graph before using references and does
not box cached state. Its projection copies the complete actual successor,
preserving all numeric limbs, Boolean meaning and both state Record orders.
Private padding is deterministic and every retained pointer belongs to the
new bank. Source aggregate padding is not assumed to carry value meaning.
The owning process publishes only after full temporary cleanup, then retires
the old graph after its last borrow. Exec and child/parent restoration preserve
shared whole-value history. Terminal return clears the complete store and all
bindings. Source receives no store, pointer, layout or resource authority.

One explicitly bound **4096-byte** store still has a **64-byte** header and two
**2016-byte** banks. Scalar-only, first retained and structured interfaces occupy
one unique semantic domain; they cannot create separate allocator identities.
All retain independent/combined selection with time/header/dispatch policies.
The native encoder now owns generated label names so schema-generated checks
can use bounded field indexes; branch encoding and fixup behavior are unchanged.

## Conservative bounds and independent tests

The complete invocation stack bound is **1176 bytes** within **4096**, including
ordinary aggregate parameters/results, source/shared-runtime frames, returns,
native saves and alignment. Allocation is bounded by **592 bytes** within
**65536**: **512** source evaluation, **48** conservative initialization adaptation
and **32** arena header. Maximum intermediate operand width is **64 bits**.
Native measurements reach **88 bytes**. These supplement the conservative proofs.

State-region word operations are separately charged: bridge validation **512**,
native prepared validation **262**, projection **13**, preparation metadata
**3**, commit validation/metadata **6/6**, release
validation **2**, initialization cleanliness/binding **512/2**, old-bank erasure/
readback **504** and terminal erasure/readback **1024**. Projection charges each
partial-width store as a full word operation. The descriptor also charges
**10** maximum source numeric reads, **6** order values, **9** order checks and
**one** Boolean result. Input and result Record transport extents are **5** and
**7** words respectively; those extents are not counts of backend loads or
instructions. Context/binding bookkeeping has
its fixed native scope; store-work counts are not instruction-count claims.

The [native client](labs/qemu/x86_64/structured_identity_client.c) executes
**66584 transitions for each of eight order combinations**, **532672** total.
It varies ordinary outer/state/history declaration and construction order,
executes consecutive history, maximum/wrap and one-/two-limb count boundaries,
checks sticky wrap after actual publication, and uses deterministic dispersed
cursors/counts. Complete bank images independently verify pointer ownership,
normalization, Boolean/order/padding, old-version preservation and successor
survival after temporary erasure. **384** malformed input/store cases reject
before allocation or mutation. Six invalid source fields reject with a previously
published graph unchanged, including a multiword value whose legal low word
would conceal truncation. C qualification programs are never kernel-linked.

The semantic model shares the existing owner/version protocol and separately
retains complete structured facts. It checks repeated publication, other-policy
exclusion, exec retention, wrong owners, overlap, premature/stale commit,
unclean inactive values, count exhaustion and termination at five phases.
Controlled wide histories are identified separately and grant no owner authority.
All **31** legal nonempty numeric-policy selections retain one identity version.
The model does not substitute for actual graph/layout/ABI evidence.

Frontend tests require the exact reserves and slot. Boot adaptation additionally
rejects **24** altered structured layout/domain/transport/work certificates,
alongside the previous **12** retention and **10** identity-binding mutations.
The linked store must retain its complete aligned Data symbol in allocated
writable BSS without an executable ELF section flag; supervisor ownership uses
the existing machine mapping contract.

## Physical whole-state observations

The [harness](labs/qemu/x86_64/structured_gate.py),
[inspector](labs/qemu/x86_64/structured_inspect.py) and
[six retained observations](labs/qemu/x86_64/results/topal-structured-source-identity-gate.json)
prove the actual value boundary. Primary, all-orders-reversed and structured-only
configurations each complete nine creations: **27** complete source result/
projection/publication/retirement observations and **48** initialized numeric
node borrows. Three initial calls use ordinary zero construction. The witnesses
inspect actual parameter values/orders, returned whole aggregates, independent
prepared graphs, unchanged published state, complete temporary cleanup, subsequent
child IDs and full terminal reclamation.

Two controlled valid-domain version seeds separately exercise maximum PID wrap
and a two-limb creation count reaching 18446744073709551615 with true sticky
history. Their seeded owner/version state and native cursor agree. Each stops
after cleanup before the next publication and proves the seeded version remains
unchanged. These are controlled states, not naturally executed long histories.

A changed imported history helper increments by two. Its PID/cursor projection
still returns the correct `(2,3)`, but its complete state has count two instead
of one. The independent whole-value oracle rejects it after temporary cleanup
and before publication. Root/provider objects remain unchanged. Scalar-only
checking would miss this error. Expected rejection is not a successful userspace
run. Earlier scalar and one-field retained harnesses now explicitly label their
full-policy runs as compatibility variants of the new primary configuration.

## Regression, review and remaining acceptance

Retained evidence contains **89 VM observations**: **59** successful primary
runs bound to the kernel hash above, **13** successful configuration/compatibility
variants, **five** controlled boundary observations and **12** expected
rejections. The complete BusyBox command corpus, caller environments, finite
and numeric libraries, copied ELF headers, source dispatch, both earlier
identity interfaces, clocks, children, pair handoff, pipes and deadline entry
regressions pass. Expected rejections and controlled observations are counted
separately from successful primary userspace runs.

All **29** toolchain contracts and **four** reference-VM contracts pass. The
qualification also checks **263** unique normative systems rules, current
relative evidence/design links and consistent primary-kernel identities.
The full bounded serial Cargo workspace passes, including **238** compiler
library, **261** compiler CLI, **131** semantic-model and **609** language tests.
The independent packager snapshot includes the structured policy and both
imported modules. Formatting validation and whitespace checks pass.

Risk is high at nested ordinary layout, two-limb normalization, presentation
order, pointer lifetimes, exclusive publication and complete reclamation.
Single-agent review follows authoritative ordinary value/resource intent, the
semantics-preserving shared layout calculation, actual object/frame evidence,
conservative bounds and independent C/model/physical oracles. Human review
remains required through the Step 4 PR. No third-party implementation is added.

Future baseline changes should compare ID/history value laws with source and
creation/reuse/lifetime rules with the owner boundary. Another architecture must
qualify its own ordinary aggregate layout, numeric representation, private caller
ABI, placement, ordering and complete bounds while retaining the same value law.

This qualifies one selected structured schema. General persistent allocation,
resource-bearing/effectful source subsystems, scalable processes/scheduling,
signals, general/dynamic ELF, writable storage/devices, networking, namespaces/
cgroups, distribution replacement, Docker/Podman and VM hosting remain required
by the [initial acceptance](step-4-status.md).
