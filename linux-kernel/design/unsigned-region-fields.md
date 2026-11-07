# Checked unsigned fields in ordinary kernel regions

`TK-DEC-279`; systems-only refinement under standing chat authorization.

## Purpose and authority

Ordinary subsystem code needs several metadata fields in one callback. The
current [eight-byte codec](../step-4-persistent-region-actions.md) has a complete
60152-byte temporary bound in the existing 65536-byte arena. Two such operations
cannot share that certificate. Merely increasing the arena would retain the
cost and give ordinary source no reusable field-access contract.

Add checked unsigned-field operations to the existing scoped
`KernelByteRegion`. Width and byte order describe a byte encoding, independently
of CPU register size, pointer size or the ABI's integer layout. Keep ordinary
Nat, Optional, Boolean, function evaluation, effects, destruction, Memory and
AllocationRegion meanings unchanged. No source pointer, cast, instruction,
register or new core integer type is introduced.

Review against [resources](../../docs/resources.md),
[effects](../../docs/effects.md), the [systems profile](../../docs/systems-profile.md),
[kernel architecture](topal-kernel-architecture.md),
[scoped regions](borrowed-kernel-regions.md),
[source qualification](source-kernel-region-functions.md) and
[persistent actions](persistent-region-actions.md). Existing
`TK-KERNEL-STATE-001` and `TK-KERNEL-MEMORY-001` remain the goals.

## Source interface and portable semantics

Within a qualified region source context:

```topal
observed is region unsigned load (
  offset-bytes is offset,
  byte-count is 4,
  byte-order is big-endian
)
stored is region unsigned store (
  offset-bytes is offset,
  byte-count is 8,
  byte-order is little-endian,
  value is value
)
```

Load returns `Optional Nat`; store returns `Boolean`. `offset-bytes` and store
`value` are ordinary arbitrary-precision Nat expressions. `byte-count` is a
static integer literal from one through eight; `byte-order` is the sealed static
fact `little-endian` or `big-endian`. Invalid, unknown, missing or duplicated
format facts reject source qualification. They are not runtime values carrying
authority. Runtime fact expressions retain ordinary named-field evaluation
order; the implementation must not reorder them into native argument slots
before evaluating their effects.

For width `w`, little-endian decoding is the sum of byte `i` times `256^i`;
big-endian decoding uses `256^(w-1-i)`. A valid load returns exactly that Nat,
including values above the signed 64-bit range. Store admits precisely
`0 <= value < 256^w`, then writes the complete encoding. Values and offsets
must never be truncated into range. Arbitrarily large Nat facts refuse just
as any other out-of-range fact does.

Resolve the existing live view first. The entire field must fit inside that
view and its actual owner, using checked, nonwrapping interval calculations.
Load refusal returns None before reading owner payload. Store additionally
requires write rights and the full value-range check before its first payload
access; refusal returns false without reading or writing payload. Guard reads
of the private view/control are distinct from owner-payload access. These
operations perform no hidden payload read on a write-only algorithm path.
Successful store writes every selected byte and preserves surrounding bytes.
Earlier effects evaluating source arguments are not undone by refusal.

The same view may be unaligned. There is no alignment request or guarantee of
atomicity, one hardware transaction or synchronization. The operation concerns
owned plain memory; MMIO, atomic, DMA, firmware and user-memory protocols retain
their domains. An x86-64, AArch64 or RISC-V provider may use byte accesses or a
proved equivalent lowering. Only x86-64 implementation is qualified here.

## Ownership and effects

Use the existing owner, binding, generation, active-frame, contained-window and
rights checks. Format/offset/value data cannot create or widen a view. Narrow
aliases retain the same owner-relative Read or Write identity, including across
ordinary helper calls. Suspended callers, retired callee views, foreign owners,
cross-domain views and ended invocations remain unavailable. No extra view,
generation, retained history or resource owner is created by a field operation.

Load is an owner-relative Read and store an owner-relative Write; aggregate
width does not split the effect domain. Complete preflight provides refusal
atomicity for this primitive, not transactional rollback of successful stores
or earlier source effects. The owning root/field/page controller retains its
existing opaque-source completion, fatal-failure and complete temporary
reclamation obligations. Invariant or authority failure cannot become an
ordinary None/false path that resumes an invalid owner.

The portable model operates on the same actual backing as byte access, including
persistent file-position and page-map loans. It uses arbitrary-precision facts
to expose high-limb truncation mistakes. Its model format constructor validates
encoding data only; it grants no native authority. Model operations and cleanup
flags are not executed source bodies or physical erasure evidence.

## Source/native qualification

Retain existing ordinary signatures, framed facts, instance identities, native
entry ABI and controller resources. Prove the complete closed call graph,
resource costs and code/read-only data bindings before admitting source objects.
An additional separately digested runtime is preferred so byte-only source
objects retain their reviewed base primitive implementation and canonical
metadata. New runtime evidence must be present exactly when needed; reject
null, missing, orphan, changed or undercharged evidence. Preserve explicit
Unit/action/unscoped-frame/instance controls, including empty immutable data.

Review actual algorithms before certifying costs. A field load can construct one
Nat and one Optional, and store needs no numeric temporary allocation after
input transport; those expectations are not certificates. Include every guard,
conversion, byte loop and shared helper in allocation/stack proofs. Bounded
primitive loops do not authorize recursive or unbounded ordinary source.
Keep the existing 65536-byte temporary and 4096-byte source-stack limits.

Native component qualification must use an independent byte oracle and actual
source calls. Cover every width/order, signed-boundary and maximum unsigned
values, large Nat refusals, unaligned and boundary-adjacent windows, guarded
payload/frame mappings, read-only aliases, nested calls and fact evaluation
order. Debugger observations must distinguish actual payload operations from
control/view checks and independently observe arguments, values, effects,
caller preservation and complete temporary cleanup.

## First owning application and alternatives

Replace the implementation of the ordinary `store-u64-le` file-position helper
with an explicit eight-byte little-endian store. Preserve its public signature,
source arithmetic and signed-file-offset check. The real controller still lends
the shared position only after admitted transfer; native code must not duplicate
the arithmetic or codec. Derive new complete certificates rather than copying
the old 60152-byte bound or an observed high-water mark.

Retain bytewise source fixtures for independent earlier-context controls where
their helper-level observations require the old implementation. Current tests
must execute the current selected codec. Freeze old fixtures explicitly, without
presenting their old source/runtime hashes as current qualification.

The alternatives are continued ordinary bytewise arithmetic, larger arenas and
architecture-sized machine-word access. The selected operation makes byte
encoding explicit and reusable while retaining existing core values and owned
region checks. Wider and signed encodings, dynamic format selection and
concurrent/atomic fields require separate qualification; this initial width
limit is not a restriction on core Nat.

## Four commit stages

1. Define this systems interface, exact rules and requirements; implement the
   actual-owner portable model with independent encoding/refusal/lifetime tests.
2. Implement typed source selection, effects, lowering, reviewed complete costs
   and strict runtime/object binding; execute independent C/GDB component tests
   while retaining byte-only and earlier callable controls.
3. Integrate the current file-position helper and actual native owning caller;
   update binding/boot refusal and physical source/owner/cleanup observations.
4. Qualify the whole Rust workspace, source ceiling, lab contracts and every
   affected fresh VM cohort; update traceability and the existing Step 4 PR.

Changing the selected codec changes native inputs, so affected machine evidence
requires fresh execution. The old TK278 observations remain their own checkpoint.
Do not use metadata labels or an unchanged ABI as evidence of unchanged code.

Risk is high at whole-field preflight, endian/unsigned conversion, alias rights,
owner exclusion, source evaluation order, malformed large facts and complete
costs. Self-review and independent behavioral oracles are required; no independent
reviewer result is implied. The [initial acceptance ledger](../step-4-status.md)
still requires general allocation/subsystems and distribution/container/VM-host
qualification. This increment alone completes none of those obligations.
