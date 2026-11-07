# Ordinary allocation actions for reusable RAM pages

`TK-DEC-278`; systems-only refinement under standing authorization.

## Purpose and preserved language meaning

Move reusable backing-slot selection into ordinary Topal over actual persistent
allocator state. The first owner is the existing RAM filesystem: eight resident
4096-byte buffers shared by its four regular-object slots. This gives the next
kernel component a real allocation/release lifecycle rather than another Linux
operation encoded as native policy. It does not complete general persistent
allocation, storage durability or distribution/container acceptance.

Use the existing `borrowed-byte-region-instance-action-v1` context, two Nat facts
and Boolean result. The instance is `ram-page-pool`; root and file-position
instances retain their identities and contracts. No new core syntax, reference,
allocator, function, Nat, effect or destruction meaning is introduced. In
particular, core `AllocationRegion` construction and core `Memory` semantics
remain as designed. Scoped `KernelByteRegion` views keep their existing nonescape,
alias, frame, rights, generation and complete reclamation laws.

This implements existing `TK-KERNEL-STATE-001`, `TK-KERNEL-MEMORY-001` and ordinary
kernel-library obligations. Review against [resources](../../docs/resources.md),
[effects](../../docs/effects.md), the [systems profile](../../docs/systems-profile.md),
[kernel architecture](topal-kernel-architecture.md), [borrowed regions](borrowed-kernel-regions.md),
[persistent actions](persistent-region-actions.md), [instances](instance-scoped-region-actions.md)
and [owned files](owned-volatile-files.md).

## Actual owner and controller

Add an eight-byte allocation map to the filesystem's existing owned store,
separate from its page associations and payloads. Byte `i` is zero for a free
slot and one for an allocated slot. The portable model must lend these actual
persistent bytes, and normal filesystem operations must use the same map.
A copied fixture or temporary arena is not allocator state.

The existing page associations retain object identity and logical page index;
the payload buffers retain their existing layout and contents. Outside private
controller preparation, the allocation map, associations and payload ownership
must agree. A free slot has a zero map byte, clean complete association and
zeroed/read-back payload. An allocated slot has one live object generation and
logical page association. Neither a bit nor an index grants memory authority.

The selected native owner places the map in the unused filesystem header space
after the identity counter and two directory timestamps, without overlapping
objects, associations, names, directory staging or payloads. Bind the actual
owner symbol, contained offset/extent, separate controller/arena/history,
source/module/code/data/hooks and complete resource bounds. Derive the instance
from this caller slot; an arbitrary valid offline object grants no caller access.

All operations remain inside the existing exclusive single-CPU filesystem
controller. Parent/child restoration, fork and exec cannot copy, roll back or
reset shared allocator state or invocation history. Position and page-pool loans
remain distinct resources even if a model binding label and numeric generation
happen to match. Their effects conservatively conflict in the filesystem's
synchronization domain. Concurrent allocation and SMP remain unqualified.

## Requests and ordinary source

The declared source entry is
`(KernelByteRegion, Nat, Nat) -> Boolean`. The facts are operation and slot:

| Operation | Facts | Source action |
| --- | --- | --- |
| Allocate | `0, 0` | Find a free map byte and store one there; succeed only after that store succeeds. |
| Release | `1, slot` | Validate `slot < 8` and the occupied byte, then store zero. |

Source reads and writes through ordinary checked byte helpers. Allocation may
choose any free slot; the initial source uses first fit. An independently
qualified alternate placement order must move the actual chosen backing slot,
without changing application-visible bytes or resource safety. No native code
may preselect a candidate and turn source into a redundant acknowledgement.

The canonical map and live-resource preconditions belong to the controller.
Wrong operation/slot data and failed byte accesses cannot manufacture authority.
An independently executed source function over a full canonical map returns
false without stores. Kernel calls occur only after the following admitted
operation protocol; an admitted false result is an invariant failure rather
than a new Linux error.

## Allocation and publication

Preserve the current Linux validation/error order. Descriptor/rights, full signed
range, zero count, effective append position, clipped transfer, complete user
prefix and capacity checks precede mutation. An existing resident page requires
no allocation action. A read-only capacity guard may establish that some free
slot exists before the existing fallible clock observation; it must not select
the backing slot. Exhaustion retains the current `ENOSPC` behavior with no source
entry, owner mutation, payload publication or position change.

After the clock observation and complete input staging have succeeded, suspend
the owner and invoke source on its actual map. Retain the actual Boolean while
reclaiming every source temporary and revoking all views. Before owner resumption,
validate the entire map delta against its admitted before-image: exactly one
previously free byte becomes one, and every other byte is unchanged. Derive the
selected slot from that delta, verify its clean payload and free association,
then publish the admitted object/logical-page association. Source selects the
slot; the controller validates and materializes the resource grant.

The association, payload, file size, timestamps and position remain private to
the same exclusive operation until publication. The existing transferred-prefix,
positional and append rules retain their meanings. A failure after admission
never resumes a partially prepared owner and promises no rollback of successful
source stores. A false or malformed delta must stop after complete temporary
cleanup; it cannot become an ordinary short write or capacity error.

## Release and reuse

Truncation or final-object destruction first proves that the selected resident
association may be released. Full caller/code/generation/context and clean
temporary-state refusal checks precede release preparation; a rejected entry
must not erase an owner's payload. Zero and read back the complete payload before
lending the allocation map for release. Supply the actual selected slot and
validate that the only map change is its one-to-zero transition. Require true
after complete temporary cleanup, erase the complete association and publish
the free slot before owner access resumes. No stale view survives this sequence.

Partial-page truncation retains its allocation bit and clears only the removed
suffix. Growth creates holes without allocating. Hole/EOF/zero reads, failed
admission and ordinary writes to resident pages issue no allocation action.
Alias/fork/exec, linked/unlinked lifetime and cancellation retain the same
shared page ownership. Terminal reclamation proves an empty allocation map and
clears complete controller/history/arena state. Separate model cleanup flags
must never be reported as physical erasure evidence.

## Portable model and native boundary

The model must connect action completion to the same filesystem state used by
read/write/seek/truncate/destruction laws. It must exclude other controller
access throughout the loan, reject foreign owner/binding/view/generation input
before mutation, and poison the controller after admitted false, bad delta,
abandonment or incomplete reclamation. Model indices and labels remain data.

Only the contained eight-byte map enters ordinary source. Native object pointers,
association addresses, payload addresses, user pointers, register state and page
table details remain outside the source interface. The allocation map is plain
memory, not MMIO, atomic or DMA storage. Its meaning is the same on x86-64,
AArch64 and RISC-V; filesystem chunk size and resource budget are distinct from
an architecture's hardware translation granule. Implement only x86-64 here.

Retain the existing frame ABI, ordinary source/object schemas and adapter family.
Compute complete source/caller/frame/stack/view/allocation bounds and prove
disjoint dedicated storage before selecting the new caller. Do not infer a
static bound from an observed stack high-water mark or reuse another instance's
temporary arena. If the source cannot fit existing qualified limits, expose the
concrete resource issue and refine the algorithm rather than silently weakening
admission. Native-only legacy selection must retain its explicit old contract.

## Commit stages and acceptance

1. Define this owner/protocol, update systems intent and exact normative rules,
   and implement the portable lifecycle model with traceable rejection tests.
2. Implement the ordinary source/module and exact source-object/code/data/hooks/
   resource/serialization binding. Independently execute the component through
   a checked native test caller, preserving earlier contexts.
3. Bind the actual kernel owner/controller/caller before enabling its artifact
   declaration, and integrate the x86-64 write/release paths and independent
   native/GDB/VM observations, including an alternate source placement order.
4. Qualify the complete workspace, strict source size, Linux/model controls and
   all affected VM cohorts; update reports/traceability and the existing Step 4 PR.

The primary native artifact changes when this caller is enabled, so affected
machine observations require fresh execution. Do not reuse the preceding
bit-identical maintenance exception for changed code. Validate allocation across
all canonical occupancy patterns, every release slot, full-width malformed facts,
all object identities/large sparse indices, exhaustion/reuse/zeroing, mixed file
lifetime, complete cleanup and inactive-child retention. Controlled pre-entry,
wrong source/delta, false-after-store and incomplete-reclamation cases must
observe the correct no-entry or no-owner-resumption boundary.

Risk is high at allocation/association consistency, mutation versus fallible
admission ordering, stale ownership, payload erasure, shared process lifetime,
caller preservation and source bounds. Self-review plus independent behavioral
oracles are required; no independent reviewer result is implied. No new Linux
syscall, general allocator or completed distribution/container/VM-host claim
follows from this increment. The [initial acceptance ledger](../step-4-status.md)
continues to apply.
