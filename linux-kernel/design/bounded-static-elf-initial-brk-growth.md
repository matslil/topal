# Bounded static ELF initial `brk` growth

## Decision

The bounded static-process startup profile may extend the proven `brk(0)` query
with one exact successful growth request. The profile owns one opaque, page-
aligned, zero-initialized anonymous reserve for the live process. After the
query returns the initial break, the same process may request `0x005f7d80`;
the provider maps the reserve at the initial break, sets the byte-granular
current break to the request, and returns that request. The qualification
payload writes and reads the first admitted byte before terminal return.

The reserve is a distinct artifact-provided process resource. A later approved
[post-random growth](bounded-static-elf-post-random-brk-growth.md) may refine it
into a larger opaque extent while preserving this first-page behavior. It is
not part of the 503-frame ELF image, the stack, bootstrap storage, entropy DMA storage,
or source-selected physical memory. Process entry owns it exclusively from
pre-entry validation through terminal teardown. Exit removes the leaf, returns
the current break to its initial value, clears and verifies the complete page,
and restores the completed owner without retaining runtime heap state.

## Portable boundary

Source names one bounded initial program-break growth profile. The artifact
provides the profile's opaque reserve identity; source does not name it. Source
also does not name the requested or returned address, page size, mapping slot,
physical frame, PTE, invalidation instruction, syscall number, or register.

The portable contract is byte-granular break state backed by an exclusively
owned zero page. X86-64, AArch64, and RISC-V providers may realize the mapping
with different translation formats and invalidation mechanisms. Capability
architectures retain the mapping authority rather than reconstructing it from
the Linux integer result.

## X86-64 lowering

The pinned provider shall:

1. validate one distinct aligned 4 KiB zero reserve and an absent leaf for
   `0x005f7000` before entering the process;
2. preserve the existing one-shot `brk(0)` query result;
3. accept exactly one later syscall 12 request for `0x005f7d80` from the sealed
   executable mapping;
4. publish one user RW/NX leaf backed by the opaque reserve and invalidate that
   virtual page before returning the requested break through `SYSRETQ`;
5. permit the clean-room payload to store and reload the first admitted byte;
6. accept `exit_group(0)` only after successful growth;
7. restore the kernel address space, clear the runtime leaf, clear and verify
   the entire reserve, and clear private mapping authority; and
8. let the qualification inverse prove and clear the query/growth completion
   state before restoring the randomized-process owner.

## Rejected alternatives

- Extending the ELF payload extent to 504 frames would falsely classify
  anonymous heap backing as executable-file content.
- Reusing stack, bootstrap, or entropy storage would conflate independent
  ownership domains and make later teardown unsound.
- Returning success without mapping would expose an unusable Linux break and
  would fail the first-byte access proof.
- Adding a general VMA tree here would combine allocation, overlap, accounting,
  fault, and concurrency policy before the first bounded growth is qualified.

## Scope

This is a systems-profile refinement only and changes no core-language rule.
It admits one exact one-page-backed growth after the initial query. A selected
later profile may retain additional zeroed anonymous and provider-private
translation backing without changing this result. The approved
[post-random query](bounded-static-elf-post-random-brk-query.md) may
observe its current result without mutation. It does not support another
nonzero request unless that later profile is selected, shrink, lazy faults, overcommit,
`mmap`, `munmap`, `mprotect`, TLS initialization, BusyBox execution in the
qualification fixture, a prompt, or general memory-management compatibility.
