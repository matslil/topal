# Step 4 bounded static ELF `AT_PAGESZ` fact

## Outcome

The qualified x86-64 path now adds one provider-consistent `AT_PAGESZ` fact to
the guarded static ELF entry block, publishes
`TOPAL_KERNEL_STATIC_ELF_AUXV_PAGESZ_OK`, restores the exact minimal block, and
publishes `TOPAL_KERNEL_STATIC_ELF_AUXV_PAGESZ_REVOKED_OK` before ordinary
entry clearing. Checked source selects opaque ownership, `provider-base-page`,
and sealed placement/rollback policies; it receives no tag, numeric page size,
stack pointer, address, word, alignment, or offset representation.

The x86-64 provider completely validates and clears the prior frame,
reconstructs a 16-byte-aligned block with native tag 6 and value 4096 before
`AT_NULL`, and reads back every word, pointer relation, string byte,
terminator, and otherwise-zero byte. Removal repeats the whole-frame process
to restore the exact minimal `busybox true` block. The RW/NX leaf, guard, image
leaves, inactive hierarchy, and bootstrap root remain unchanged.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-AUXV-PAGESZ-001` adds no core-language element.
Source-negative tests reject substituted owners, backing, values, policies, or
markers. The semantic model rejects duplicate or out-of-order addition,
ordinary clearing, leaf revocation, or backing release while the fact is live,
and enforces initialized-entry to page-size-entry to initialized-entry affine
ownership.

Structural provider tests prove native tag/value encoding, aligned extended
and restored entry positions, complete 512-qword clearing before each
reconstruction, complete readback, one fence per transition, and absence of
active-address invalidation. Artifact `/126`, provider and provider-object
`/116`, root object `/115`, boot adapter `/69`, boot-resource `/2`, and gate
`/128` identify this contract. The checked source contains 312 operations; the
provider plan contains 106 operations; the root contains 168 fatal edges, a
373-event semantic trace, 4,265 typed relocations, 3,833 UART calls, 68
required text symbols, and 109 ordered observations.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions added and
removed the page-size fact, reached both new witnesses in order, completed all
109 observations, remained in the terminal fatal halt, and produced identical
artifact and serial hashes. Recorded hashes are:

- static user image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `c0476b3c506a9074b887ce7bd47f3ebd5319ac5ccfeef5b92c362eeee8651b49`;
- boot image: `2888c9d9a68435e0e3a9f929ee2497080e3e9eaa77e3dea42b22a776e2f1a752`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `36ae351446d0e3e8f57579100032e98c5eae1e83b1b93c02af319886532138bf`;
- boot provenance: `bea495748f15b4894338e366e1a2abd53024591e4a4100fc5b9602ad268489c1`;
- serial output: `b33be91929403fa88c505254c37feb4e895f8200bc3bbb3b9e31251271f88aca`;
- evidence: `c3f258cedaa7d157344724f10082d070b28efe75e29f54e8a18ea7d1066c7572`.

## Risk and remaining boundary

Risk is high because an inconsistent base-page fact or incomplete whole-object
reconstruction could mislead libc or corrupt process-entry parsing. Review
covered provider/address-space agreement, exact ordering and alignment,
complete byte coverage, pointer/string preservation, inactive translation
state, affine rollback, negative source cases, structural machine-code
inspection, and duplicate physical execution.

This increment removes the fact before returning. It does not add executable,
credential, platform, clock, secure-mode, or random facts, initialize TLS,
construct a user context, enter ring 3, execute BusyBox, or add a syscall.
