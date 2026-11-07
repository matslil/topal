# Step 4 bounded static ELF address-space attachment

## Outcome

The qualified x86-64 path now consumes the sealed static image and its exact
four-frame hierarchy extent to construct one complete inactive address space.
Checked source publishes `TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_OK`, detaches
the same affine owner, publishes
`TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_DETACHED_OK`, returns and releases the
zeroed hierarchy, then performs the existing leaf-metadata and payload return.

The provider verifies the exact 503-frame payload, final 512-entry leaf table,
adjacent four-frame hierarchy, and inactive CR3 identity before construction.
It zeroes and reads back all 16 KiB, constructs and reads back the qualified
PML4, PDPT, low identity directory, static-image leaf link, and HPET/APIC
directory, and rechecks the sealed leaf roles. Detach repeats the hierarchy,
leaf, and inactive-root checks before clearing and reading back every hierarchy
byte. Neither operation activates the root or grants entry, stack, TLS, or
execution authority.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-ADDRESS-SPACE-001` adds no core-language
element. Source negative tests reject substituted owners, hierarchy, layout,
witnesses, reclaim role, and release authority. The semantic model enforces the
sealed-with-hierarchy to attached to detached to hierarchy-returned lifecycle.
The checked root preserves all three identities across provider calls and
returns the hierarchy only after successful detach.

Structural provider tests prove two inactive-CR3 observations per operation,
complete zero and readback, exact final leaf roles, exact qualified MMIO
leaves, full low-directory construction only during attach, and checked fatal
edges. Qualification also exposed and repaired a prior root-lowering bug that
computed the post-leaf hierarchy candidate before the generic allocation path
overwrote it; the runtime adjacency check failed closed until the exact
candidate was retained.

Artifact `/118`, provider and provider-object `/109`, root object `/107`, boot
adapter `/69`, boot-resource `/2`, and gate `/120` identify this executable
contract. The checked root contains 154 fatal edges, a 344-event semantic
trace, 3,581 typed relocations, 3,177 UART calls, and 95 ordered observations.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions reached both
address-space witnesses in order, completed all 95 observations, remained in
the terminal fatal halt, and produced identical artifact and serial hashes.
Recorded hashes are:

- static user image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `3691025d228a444fe3d31d3e451b1362a7c95a64dffae82372b8859e4707861a`;
- boot image: `19b7c18f0924620b9e411e5fc6a62d9b5fcb5f7c77e5f39602207ce0670fb364`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `42a5ce46030cf7bac09d8e743a68c202a562068c9d78c79c3a9d7435bc98b6b8`;
- boot provenance: `23f2a44f5161f1a17ddd57d07765f69a4b14d2a97b871ace7ef62777340f7d71`;
- serial output: `e3d90baa40a45bc6300d5bbd24abf54447f62175dbc7d567c8e74a1db9196325`;
- evidence: `f15a7e6c7bebbc6f3ef225f8eaf792cfe07dcad6b7fe530f85ba9320ce566f2e`.

## Risk and remaining boundary

Risk is high because a malformed entry, active-root alias, incomplete clear,
or premature affine return could grant or retain unintended memory authority.
Review covered checked arithmetic and adjacency, exact page-table bytes,
present and absent entry readback, root-register checks, publication fences,
register-held ownership, rollback order, failure edges, negative source tests,
structural machine-code inspection, and duplicate physical execution.

This increment deliberately detaches and destroys the address space after its
witness. It does not retain or activate the root, allocate a process stack,
initialize TLS, publish an entry point, enter user mode through this static
image, or run BusyBox. Those remain later bounded transitions.
