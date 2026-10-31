# Step 4 bounded static ELF `AT_ENTRY` fact

## Outcome

The qualified x86-64 path now derives one `AT_ENTRY` fact from the retained
opaque executable-entry authority, adds it after `AT_PAGESZ` and before
`AT_NULL`, publishes `TOPAL_KERNEL_STATIC_ELF_AUXV_ENTRY_OK`, restores the
exact page-size block, and publishes
`TOPAL_KERNEL_STATIC_ELF_AUXV_ENTRY_REVOKED_OK` before page-size removal.
Checked source selects only affine owners, the opaque
`authorized-static-elf-entry` value, and sealed placement and rollback
policies; it receives no tag, executable address, stack pointer, word,
alignment, or offset representation.

The x86-64 provider revalidates the resident ELF header, exact executable
leaf, retained entry authority, and inactive address space before clearing the
complete frame. It reconstructs a 16-byte-aligned block containing
`AT_PAGESZ=(6,4096)`, `AT_ENTRY=(9,0x4a6580)`, and `AT_NULL`, then reads back
every word, pointer relation, string byte, terminator, entry relation, and
otherwise-zero byte. Removal repeats the whole-frame process to restore the
exact page-size-qualified `busybox true` block. The RW/NX stack leaf, guard,
image leaves, inactive hierarchy, and bootstrap root remain unchanged.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-AUXV-ENTRY-001` adds no core-language element.
Source-negative tests reject substituted owners, backing, values, policies,
or markers. The semantic model rejects duplicate or out-of-order addition,
page-size removal, ordinary clearing, leaf revocation, or backing release
while the fact is live, and enforces page-size-entry to executable-entry-fact
to page-size-entry affine ownership.

Structural provider tests prove native tag/value encoding, resident-header
and executable-leaf agreement, aligned extended and restored entry positions,
complete 512-qword clearing before each reconstruction, complete readback, one
fence per transition, and absence of active-address invalidation. Artifact
`/127`, provider and provider-object `/117`, root object `/116`, boot adapter
`/69`, boot-resource `/2`, and gate `/129` identify this contract. The checked
source contains 316 operations; the provider plan contains 108 operations;
the root contains 170 fatal edges, a 377-event semantic trace, 4,351 typed
relocations, 3,915 UART calls, 70 required text symbols, and 111 ordered
observations.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions added and
removed the entry fact, reached both new witnesses in order, completed all 111
observations, remained in the terminal fatal halt, and produced identical
artifact and serial hashes. Recorded hashes are:

- static user image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `94a70e6661db1102a60bbe9920d8bf53aa2d01d5613cb87d8c6d3fb20ca472d4`;
- boot image: `e0489d2b570c9260dd97b76e37ee8515bd2d0c15a0781791807eb908a21d0106`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `a7d099cf801d986396a4489e5db92e58cd8d7fa050f2306cf29e447cb722b507`;
- boot provenance: `eff43663f4cb91ef5665a7186c7f3ebf3f63f49b28d0c7b45ddb4c7a5386acbd`;
- serial output: `2fd5d500a9abf0b3738013941f10fe29cd314d360291c9892d7eb09548b0d8fc`;
- evidence: `14f00f440d62fcee7c2c145e052cf50ce3ffbdbfe698b046e94bf470ee8ad8ce`.

## Risk and remaining boundary

Risk is high because an inconsistent executable-entry fact or incomplete
whole-object reconstruction could transfer execution to an unauthorized page
or corrupt process-entry parsing. Review covered header, entry-authority, and
executable-leaf agreement; exact ordering and alignment; complete byte
coverage; pointer/string preservation; inactive translation state; affine
rollback; negative source cases; structural machine-code inspection; and
duplicate physical execution.

This increment removes the fact before returning. It does not add program-
header, credential, platform, clock, secure-mode, or random facts, initialize
TLS, construct a user context, enter ring 3, execute BusyBox, or add a syscall.
