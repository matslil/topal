# Step 4 bounded static ELF initial `brk` growth

## Outcome

The bounded static-process startup path now extends its proven `brk(0)` query
with one exact successful growth. The clean-room executable requests
`0x005f7d80`, checks the complete returned value, writes `0x5a` to the first
newly admitted byte at `0x005f7000`, reads it back, and only then requests
`exit_group(0)`.

The artifact owns a distinct aligned zero 4 KiB reserve. The x86-64 provider
validates that reserve and the absent destination leaf before entry, maps it
user RW/NX for the exact growth request, invalidates the virtual page, and
returns through `SYSRETQ`. Terminal teardown restores the kernel address space,
removes the leaf, clears and verifies all 4 KiB, clears private mapping
authority, and leaves the completed process owner with no runtime heap state.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-INITIAL-BRK-GROWTH-001` refines the existing
query and process-entry lifecycle. Source selects only the bounded growth
profile; the provider owns addresses, syscall selectors, page-table format,
invalidation, and the artifact reserve identity. Checked source emits
`TOPAL_KERNEL_STATIC_ELF_INITIAL_BRK_GROWTH_OK` only after the completed owner
proves query/growth/access/terminal ordering. Negative tests reject a different
ABI, startup profile, terminal result, policy, query marker, growth marker, or
exit marker.

Structural qualification proves the clean-room query/growth/compare/access/
exit sequence, native `brk` and `exit_group` selectors, exact requested and
returned break, an initially absent heap leaf, user RW/NX publication, page
invalidation, two completion states, terminal gating, leaf revocation, complete
reserve zeroing and verification, cleared private authority, and sixteen-qword
qualification-inverse clearing.

Artifact schema `/159`, provider `/149`, provider object `/141`, root `/148`,
boot adapter `/69`, boot resource `/2`, and gate `/161` identify the changed
contract. The generated root has 406 operations, a 467-event semantic trace,
8,706 relocations, and 6,084 console-writer relocations. The formal systems
profile now contains 151 rules.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` executions produced identical
canonical evidence with SHA-256
`35db5a35d5ee18a4cd6ea7a20e59b1c034a7bcb4c1c35458595aa03b54e66a07`.
Both reached all 157 ordered observations, including initial-break query,
growth and byte access, terminal return, process-entry restoration, complete
static-process rollback, and the unchanged later kernel/user probes. The
evidence records:

- linked kernel: `4e33ed093f99496df5707f22759639bf5478e680e2392725bc177e6d6d3b6721`;
- boot image: `5d25a68718a9901f6fd7d7ba33da6fcc4ddac16fd5475004b9ab4782f8bde118`;
- boot resource: `e2c5de7d95a7dc614fb5a975b1b9d1f09fc4d3d3fcc1ad2a7d02050970e04317`;
- static clean-room image: `66424737eb424f031d6163119d533514211446c65573a86ecc86db1d3a3d7f6a`;
- artifact provenance: `4d9adfede91f3a29e7e033539c22e0e3336991b4fe6420b7c216d4416def03b5`;
- boot provenance: `accfa7cd23f6702c2c4c07f56fc4d3173dcc5c0bbf5fed755ccb8c0c95ed1de0`;
- source: `f1c910da4ac8e1429137f634e60f4c7578ed17cde6f92c52c651e0d3625675ba`;
- serial output: `3370d9abfbe444f1595649ad62cdac0ab5bc8c2eff9a0440f68d4c7ba7ce51ed`.

## Risk and review

Risk is high because this mutates a live userspace page table and makes a new
user-writable mapping visible across resumable syscall return. Review covered
reserve separation and zero state, exact PTE slot and flags, checked request
ordering, `SWAPGS`/`SYSRETQ` pairing, TLB invalidation, current-break state,
kernel address-space restoration before teardown, leaf removal, full-page
erasure, private-authority clearing, fail-closed paths, structural code
inspection, full workspace validation, and duplicate physical execution.

## Remaining boundary

This increment admits only one exact one-page-backed growth after `brk(0)`. It
does not support another request, shrink, multiple pages, lazy faults,
overcommit, `mmap`, `munmap`, `mprotect`, TLS initialization, redistributed
BusyBox bytes in qualification, a prompt, or general memory-management and
application compatibility. The next static-process increment can follow the
pinned executable's observed `arch_prctl(ARCH_SET_FS, ...)` request from this
proven query-and-growth boundary.
