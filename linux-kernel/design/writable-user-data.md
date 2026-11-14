# Writable initial user-data contract

This record resolves `TK-DEC-068` as a systems-profile refinement. It adds a
typed mapping transition for ordinary writable ELF data while preserving W^X,
affine backing ownership, provider-private page-table representation, and the
existing verified-initialization boundary. It changes no core-language rule.

## Decision

`user mapping permit writable data` consumes a verified RO/NX mapping and
returns an opaque `WritableDataMapping` for the same provider, address space,
mapping identity, backing, placement, contents, and user accessibility. The
provider may restore write authority only for the sealed data role and must
prove executable authority remains absent. Target translation completion must
precede observable success.

The transition occurs after external initialized data and BSS have been copied,
zeroed, protected read-only, and verified. This preserves a stable loader
verification point before application mutation while making the resulting
mapping RW/NX. It is a typed role transition, not a general permission API.

The initial x86-64 ELF data segment is marked `PF_R | PF_W`. Constructed user
code must change one byte in the memory-only BSS extent, read it back, and
reach its existing terminal syscall path. The kernel emits the ordered
`TOPAL_KERNEL_USER_IMAGE_WRITABLE_DATA_OK` witness only after that user
execution returns successfully. Unmap consumes the exact writable-data owner
and returns its original frame extent.

## Rejected alternatives

Leaving the data mapping read-only cannot run ordinary programs with mutable
globals. Silently retaining RW permissions under the existing immutable-data
type would make the semantic authority false. A source-visible page-bit update
would expose target representation and permit W+X construction. Re-enabling
write before content verification would lose the stable initialization
boundary.

## Cross-architecture level

Writable, non-executable application data is portable semantic intent. X86-64
lowers it to a PTE write-authority update plus translation invalidation;
AArch64 and RISC-V would use their own permission and observation mechanisms.
No architecture exposes registers, instructions, page-table bits, addresses,
or barriers to portable source.

This increment does not add executable-write authority, arbitrary permission
selection, writable code, shared mappings, copy-on-write, demand paging,
dynamic linking, TLS, or a general process loader.
