# Readiness for general kernel allocation and storage

This inventory connects `TK-ELEMENT-STORAGE-001` and `TK-KERNEL-MEMORY-001`
to the qualified implementation at compiler/kernel commit
`9e7875eb9731f231d74ed45e6e4c5b34efefddee`. It identifies work required beyond
the current RAM-file and namespace profiles. New admissible systems profiles
need their own exact rules, source/native binding and execution evidence.

## Existing language meaning

Ordinary Topal already defines terminating recursion, including mutually
recursive declaration groups, constrained-integer and structural descent,
and checked `Decreases` evidence. Every recursive cycle must decrease a
well-founded measure. See [execution](../../../docs/execution.md) and
[capabilities](../../../docs/capabilities.md). A runtime iteration counter alone
does not discharge that source-language termination obligation.

The [physical-frame contract](../../design/physical-frame-allocation.md)
already defines nonzero counts, power-of-two alignment, distinct live affine
extents, non-overlap, allocator identity and whole-extent release. Allocation
policy and free-range structures belong in ordinary Topal libraries. Numeric
frame indexes and copied metadata remain facts; live allocator and extent
identities carry authority.

The [borrowed-region contract](../../design/borrowed-kernel-regions.md)
already permits ordinary source to operate on actual exclusively owned bytes.
It keeps owner access suspended, checks view/frame/generation identities and
rights, preserves admitted persistent writes on failure, and requires complete
temporary reclamation before owner resumption. Source cannot create authority
from numeric addresses or use a temporary arena as persistent storage.

## Qualified implementation limits

| Facility | Current qualification | Required generalization |
| --- | --- | --- |
| Physical extents | Selected static counts and separately qualified coexistence profiles | Requests derived from live workload needs, multiple independent live extents, reservations, exhaustion and reuse across the actual usable memory map |
| Ordinary source calls | Closed acyclic helper graphs; the numeric compiler rejects recursive closures | Qualified ordinary iteration/recursion with proven termination and complete native resource bounds |
| Source temporary storage | Exclusive 65536-byte arena and 4096-byte stack, with full invocation erasure | Proven bounded live temporary storage for repeated work; any reuse must preserve still-live values and borrowed authority |
| Persistent filesystem storage | Four independent regular inodes, four entries and eight reusable resident pages | Storage capacity derived from owned memory, hierarchy and object lifetimes, without fixed application-specific slot limits |
| Persistent source values | Separately qualified retained numeric/structured graphs and borrowed-byte actions | Allocation and reclamation for general live subsystem objects, with explicit ownership and publication |
| Shared use | Selected synchronous operations and explicit peer-loan exclusion | Qualified synchronization, preemption and multiprocessor ownership protocols |

The ordinary source restrictions are documented in the
[systems profile](../../../docs/systems-profile.md), and the recursive-closure
refusal is implemented in
[numeric library analysis](../../../src/topal-compiler/src/systems_numeric_library.rs).
The [kernel status](../../step-4-status.md) records the current executed
profiles and the full distribution/container/VM-host obligations.

## Evidence needed for the next storage implementation

First establish which existing ordinary termination, function/effect and
resource analyses can qualify the allocator's actual source. Preserve the
language's arbitrary-precision arithmetic, effect order and borrowed-region
meaning. A qualified iterative or recursive lowering must bind its complete
closed source graph, progress proof, actual inputs and native ABI. Its stack
and allocation certificate must cover every admitted execution; observed high
water marks alone are insufficient.

Reusing a physical frame or source temporary area requires proof that its old
contents and references have retired. Function-call optimization must preserve
logical view/frame lifetimes, fresh identities, generation monotonicity,
callee code binding and source-level behavior. The owning invocation resumes
only after its complete cleanup obligations hold. Failure after actual
persistent stores keeps those stores and reaches the existing fatal boundary.

Allocation must act on the real owner and return extents with provenance.
Tests need exhaustion, fragmentation, alignment and overflow cases; disjoint
simultaneous allocations; foreign, stale and duplicate release; reservation
preservation; zero-before-republication; repeated reuse and terminal cleanup.
Independent complete-state oracles and physical observations must distinguish
ordinary source effects from native admission and machine transitions.

The architecture boundary continues to expose opaque complete-frame extents.
X86-64, AArch64 and RISC-V providers retain their own granules, mapping rules,
boot-memory inputs and ordering obligations. Implementation remains x86-64;
those differences constrain the abstraction and its evidence.

General allocation is a prerequisite for expanding VFS, processes and virtual
memory toward the declared distribution and container workloads. It remains
part of the original implementation goal alongside drivers/Device Tree,
networking/isolation, full retained UAPI and VM hosting.

The subsequent [frame-owner model repair](../../step-4-frame-allocator-readiness.md)
reproduces and corrects same-label authority, identity-exhaustion mutation,
unchecked extent endpoints and malformed normalized-range admission. Its
independent fragmented-allocation oracle and complete semantics suite validate
the existing portable contract. Native general allocation remains required.

[TK-DEC-282](../../step-4-repeated-source-tail-traversal.md) now supplies the
source-plan stage for repeated ordinary work: exact natural progress, invariant
region transport, closed helpers and input/effect binding. The
[transfer inventory](tail-transfer-readiness.md) identifies actual representation,
authority and resource obligations for the next executable stage. No new native
profile is admitted by that source analysis.
