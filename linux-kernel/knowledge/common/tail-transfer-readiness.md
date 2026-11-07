# Tail-transfer implementation inventory

This source-backed inventory prepares Stage 2 of
[TK-DEC-282](../../design/repeated-source-tail-traversal.md). It is not a runtime
certificate or native admission. The source-plan inspector and
[clear-prefix source](../../kernel/algorithms/clear-prefix.t) establish an actual
ordinary traversal to lower; complete transfer, stack, allocation and cleanup
proofs remain required.

## Actual carried state

The selected source has one invariant borrowed region and three natural facts:
remaining bytes, current offset and accumulated success status. The caller
supplies status one. Each nonterminal step evaluates the decremented remaining
value, incremented next offset and `next-status` in ordinary argument order.
`next-status` performs the actual store at the old offset only while status is
one; any store refusal produces zero. Zero status remains zero without further
stores. The terminal step returns whether status is one. No source resource or
numeric storage escapes.

If the caller proves `offset + remaining <= owned byte count` mathematically,
then a step with positive remaining has a contained offset. Its successor
preserves that sum and decreases remaining by one. After exactly the original
remaining count, the offset reaches the admitted endpoint and the terminal
step performs no store. The model's back-edge bound does not replace that
interval invariant or charge the final guard/base action automatically.

With admitted native counts/offsets at most `u64::MAX`, that interval invariant
keeps actual offsets and remaining values within the same bound. A conservative
interval-only proof may charge 65-bit increment intermediates instead. Either
approach must retain arbitrary-precision source meaning; it cannot introduce
wrapping integer arithmetic or assume a relation it has not established.
Status transport must cover the actual initial input and its subsequent 0/1
results. The source-only selector currently supplies unconstrained native-width
Nat facts; it does not establish any of these caller invariants.

## Existing representation and refusal boundaries

The shared [numeric runtime](../../../src/topal-compiler/src/runtime/value_core.ll)
uses an immutable sign/length header with 32-bit limbs; zero may refer to a
read-only constant. The current x86-64 layout has a 16-byte header.
`int.from.u64` requests 20 or 24 bytes for a nonzero native word; the qualified
eight-byte-aligned arena charges 24 bytes in either case. A transfer operation
must handle constants, zero, negative Int values, multi-limb values and equal or
aliased arguments according to their semantic values. Raw address equality is
not an owning root or a valid lifetime proof. These are current runtime facts,
not a portable source representation or a complete bound for arithmetic calls.

The [region runtime](../../../src/topal-compiler/src/runtime/kernel_regions.ll)
currently uses 64-byte private view/control records. It checks invocation owner,
code, generation, rights, contained spans and issued-view facts. Its count cap
is 65536 bytes and its issued-view cap is 256. The
[allocation analysis](../../../src/topal-compiler/src/systems_region_bounds.rs)
also assumes that region count cap. Neither cap may silently be removed from
an existing profile. A later broader owned-region profile must bind actual
extent authority, representable target pointers and complete new evidence.

The numeric allocation analysis still rejects recursive closures. Native stack
inspection also rejects cyclic call graphs. The shared emitter disables tail
calls for qualified runtime functions. A plan therefore cannot become executable
by bypassing one of those checks or depending on LLVM to remove recursion.

## Required transfer construction

The initial selected recursive component is the source entry, with acyclic
ordinary helpers. Its source caller is the private bridge rather than an
arbitrary suspended source parent. Keep the borrowed root/control and immutable
input frame live throughout the invocation. Before any reset, transfer every
next-state scalar into disjoint owned storage and prove that no remaining live
numeric object, view, helper frame or cleanup obligation depends on the retiring
extent. The old and next state may coexist during transfer; both costs count.

The implementation must charge both transfer banks, pinned state, source work,
adapter/runtime allocations, alignment and checkpoint stack. Erase the complete
retired extent, including older high-water contents, before reuse. Retain source
evaluation and persistent-store order. On normal completion and runtime abort,
reclaim all domains before the owning caller resumes or reaches its fatal
boundary. The existing 65536/4096 reserves remain unchanged.

The source creates no derived Window geometry. This does not remove fresh loan
identities: the existing [call model](../../../src/topal-semantics/src/systems_region_library.rs)
issues a fresh callee view and frame when entering a helper. Logical tail/helper
loans still need correct fresh nonwrapping identities and admission of total
issuance before mutation; one-step helper depth is not that proof. Physical
descriptor reuse cannot reset that history or redefine the existing 256-view
native profile. Debug information must identify optimized traversal steps and retired
unavailable values accurately, while preserving all applicable native debugging
obligations. Ordinary values in an active outer caller cannot be reclaimed as
part of a later nested extension without separate liveness evidence.

Independent native oracles need guard pages, complete value/root comparisons,
real source stores and exact effect order, reused-bank poison/zero checks,
exhaustion at each transfer boundary, aliases/constants/multi-limb inputs,
unchanged live caller/control state, and normal/fatal register/stack cleanup.
The source-only plan tests provide none of that physical evidence. Subsequent
real allocator/metadata integration must act on the actual owned bytes and
extents, preserving reservations, provenance, disjointness and release lifetime.
