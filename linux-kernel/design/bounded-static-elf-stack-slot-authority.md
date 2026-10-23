# Bounded static ELF stack-slot authority

This record resolves `TK-DEC-139` as the virtual-layout boundary between an
entry-authorized static ELF construction and later stack backing. It is a
systems-profile refinement only: checked source owns an opaque affine stack
slot authority, while providers own virtual addresses, page-table indices,
stack direction, and target alignment.

## Decision

`static image construction authorize stack slot` shall consume the exact
`EntryAuthorizedStaticImageConstruction` from `TK-DEC-138` and accept only
policy `bounded-linux-initial-stack-slot`. Before success, the provider shall
revalidate the complete inactive hierarchy, all final image leaves, the
executable entry authority, and one page-aligned absent stack slot separated
from every image mapping by at least one absent guard page. It shall produce
one affine `StackSlotAuthorizedStaticImageConstruction` owning the unchanged
payload, leaf metadata, hierarchy, entry authority, and opaque stack-slot
authority.

Source shall emit exactly one
`TOPAL_KERNEL_STATIC_ELF_STACK_SLOT_AUTHORITY_OK` observation. It shall then
invoke `static image construction revoke stack slot` with that exact owner and
policy `bounded-stack-slot-qualification-rollback`. Revocation shall
revalidate the same inactive slot, consume only the stack-slot authority,
reproduce the entry-authorized construction, and emit exactly one
`TOPAL_KERNEL_STATIC_ELF_STACK_SLOT_AUTHORITY_REVOKED_OK` observation before
entry revocation.

No virtual address, page-table index, stack pointer, stack-growth direction,
pointer width, word layout, or physical address shall become source-visible.
Authorization neither allocates a frame nor installs a leaf, initializes stack
bytes, constructs `argc`/`argv`/`envp`/auxiliary-vector state, changes a root,
or enters the image.

## Exact x86-64 realization

For the approved static ELF plan, the provider shall require:

- the inactive static root and active qualified bootstrap root;
- the exact four-frame hierarchy and 503 final image leaves;
- unchanged entry authority for ELF entry `0x004a6580`;
- absent leaf ordinal 511, denoting the page `[0x005ff000, 0x00600000)`;
- absent leaf ordinal 510 as the immediate guard below the downward-growing
  stack slot; and
- an initial top of `0x00600000`, aligned to both the 4096-byte page boundary
  and the x86-64 16-byte process-entry convention.

All nine non-image leaves 503 through 511 remain absent during authorization
and revocation. The checked root retains only an affine semantic state; the
numeric slot and initial top remain provider-private.

## Affine lifecycle and rollback

Only the entry-authorized owner may authorize the stack slot. While slot
authority is live, entry revocation, activation, detach, hierarchy reclaim,
metadata revocation, payload reclaim, duplicate authorization, or recoverable
disposition is invalid. Only exact stack-slot revocation may reproduce the
entry-authorized state admitted by `TK-DEC-138`. Any identity, root,
hierarchy, leaf, guard, alignment, order, or readback failure is fatal.

## Architecture pressure test

An opaque aligned stack slot, disjoint image mappings, and an absent guard are
portable policy. AArch64 and RISC-V providers may choose different virtual
placement and process-entry alignment while retaining downward stack growth.
A target with upward-growing or capability-bounded stacks may choose its native
growth direction and produce a bounded stack capability, provided the guard
and non-overlap obligations remain semantic facts. The authority therefore
exposes neither the x86 ordinal nor a universal integer stack pointer.

## Alternatives rejected

Publishing a stack address was rejected because source could forge or widen
memory authority. Reusing the static image's writable data was rejected because
stack lifetime and mutability must remain independent of ELF segments. Mapping
a frame during slot selection was rejected because virtual-layout authority,
physical backing, initialization, and process-entry construction need separate
rollback boundaries. Omitting a guard was rejected because an immediately
adjacent writable image page would turn bounded stack underflow into silent
image corruption. Keeping slot authority live through entry revocation was
rejected because the slot derives from that entry-authorized address space.

## Deliberate bounds

This increment derives, witnesses, and revokes one opaque absent stack slot.
It does not allocate or map stack backing, initialize stack contents or TLS,
construct a user context, enter user mode, execute BusyBox, or add a syscall.
