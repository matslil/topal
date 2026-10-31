# Bounded static ELF entry authority

This record resolves `TK-DEC-138` as the boundary between a qualified static
ELF address space and later user-context construction. It is a systems-profile
refinement only: checked source owns an opaque affine entry authority, while
providers own numeric entry addresses, leaf representation, and target
instruction-entry mechanics.

## Decision

`static image construction authorize entry` shall consume the exact restored
`AttachedStaticImageConstruction` from `TK-DEC-137` and accept only policy
`validated-static-elf-entry`. Before success, the provider shall revalidate
the complete inactive hierarchy, all final leaves and absent entries, the
copied ELF entry identity, and containment of that entry in the immutable
executable load interval. It shall produce one affine
`EntryAuthorizedStaticImageConstruction` owning the unchanged payload, leaf
metadata, hierarchy, and an opaque entry authority.

Source shall emit exactly one `TOPAL_KERNEL_STATIC_ELF_ENTRY_AUTHORITY_OK`
observation. It shall then invoke `static image construction revoke entry`
with that exact owner and policy `bounded-entry-qualification-rollback`.
Revocation shall revalidate the same inactive entry mapping, consume the entry
authority, reproduce the restored attached construction, and emit exactly one
`TOPAL_KERNEL_STATIC_ELF_ENTRY_AUTHORITY_REVOKED_OK` observation before the
existing detach and resource-return sequence.

No numeric entry address, page-table entry, physical address, instruction
pointer, or architecture register shall become source-visible. Authorization
does not create a thread, process, stack, TLS block, signal state, syscall
continuation, or user-mode transition and shall not branch to or fetch an
instruction from the static image.

## Exact x86-64 realization

For the approved static ELF plan, the provider shall require:

- inactive static and active bootstrap root identities;
- the exact five-frame hierarchy and 503 final leaves from `TK-DEC-136`;
- ELF entry `0x004a6580` in the copied header;
- entry-page ordinal 166 in the payload extent;
- final entry leaf equal to the corresponding payload frame with user
  read/execute and no write authority; and
- entry offset `0x580` inside that executable page and inside the copied
  executable interval.

Authorization and revocation shall each perform complete hierarchy and leaf
readback and shall modify no hierarchy, leaf, payload, or root-register byte.
The checked root retains only the provider-private resource identities and an
affine semantic state; it does not materialize the numeric entry.

## Affine lifecycle and rollback

Only the restored attached owner may authorize entry. While entry authority is
live, activation, detach, hierarchy reclaim, metadata revocation, payload
reclaim, duplicate authorization, or recoverable disposition is invalid.
Only exact revocation may return to the restored attached state admitted by
the existing detach path. Any identity, root, hierarchy, leaf, ELF-entry,
containment, order, or readback failure is fatal.

## Architecture pressure test

An opaque entry authority and proof that it names immutable executable bytes
are portable policy. AArch64 and RISC-V providers may validate different leaf
representations and instruction-coherence obligations. Capability systems may
derive a sealed executable capability rather than a numeric program counter.
The authority therefore exposes neither x86 page bits nor a universal integer
entry address.

## Alternatives rejected

Publishing the ELF entry as an integer was rejected because source could forge
or reuse target addresses. Entering user mode during authorization was rejected
because stack, TLS, context, and return authority do not yet exist. Treating
the prior ELF-plan check as sufficient was rejected because runtime payload,
leaf, hierarchy, and root identities must still agree after construction.
Keeping authority live through teardown was rejected because an executable
capability must not outlive its mappings. Re-deriving the entry from an
untrusted boot-resource pointer was rejected because copy verification already
established the owned runtime image.

## Deliberate bounds

This increment derives, witnesses, and revokes one opaque entry authority. It
does not retain a process address space, allocate or initialize a stack,
initialize TLS, enter user mode, execute BusyBox, or add any syscall.
