# Step 4 bounded static ELF address-space activation and restoration

## Outcome

The qualified x86-64 path now consumes the attached static image, activates
its exact translation root for one bounded supervisor continuation, publishes
`TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_ACTIVE_OK`, restores the retained
bootstrap root, and publishes
`TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_RESTORED_OK`. The existing detach,
hierarchy reclaim, metadata revocation, and payload reclaim sequence then runs
only with the restored affine owner.

The provider revalidates the complete 503-leaf image and four-frame hierarchy,
proves the bootstrap root active and the static root inactive, writes and reads
back the static `CR3`, and returns only to already mapped kernel code. Restore
proves the static root active and bootstrap root distinct, writes and reads
back the bootstrap `CR3`, then revalidates the unchanged hierarchy and leaves
through restored identity translation. No static-image byte executes and no
entry, stack, TLS, signal, syscall, process, or persistent address-space
authority is granted.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-ACTIVATION-001` adds no core-language element.
Source-negative tests reject substituted owners, activation/restoration
policies, witnesses, lifecycle order, and detach before restoration. The
semantic model enforces attached to active to restored to detached ownership.
The checked root retains both opaque root identities and admits teardown only
after successful restoration.

Structural provider tests prove one `CR3` write in each direction, exact
precondition and readback observations, restoration before construction-byte
dereference, complete leaf/hierarchy revalidation, and the absence of any
hierarchy clear or rebuild during the active interval. They also prove that
the hierarchy is constructed with provider-private x86 accessed state on
parents and accessed/dirty state on writable huge leaves. Pre-seeding those
architectural bits makes the hierarchy byte-stable across real page walks,
rather than hiding hardware mutation during post-restoration verification.

Artifact `/119`, provider and provider-object `/110`, root object `/108`, boot
adapter `/69`, boot-resource `/2`, and gate `/121` identify this executable
contract. The checked root contains 156 fatal edges, a 348-event semantic
trace, 3,681 typed relocations, 3,273 UART calls, and 97 ordered observations.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions completed the
bootstrap-to-static-to-bootstrap transition, reached both new witnesses in
order, completed all 97 observations, remained in the terminal fatal halt,
and produced identical artifact and serial hashes. Recorded hashes are:

- static user image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `60e93e3751977f734f51c4874565eca29c381d12bf3c044fde669c5539bc914b`;
- boot image: `1992f7d43f74f68b5efc745cf751c8816b76600b728975370baf7b99bed7b67d`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `dd730e48f8974cc6fc25777ddcdcdee1916c951526c116b1a899aa69c69e5874`;
- boot provenance: `3fa0ff355e07e099fe9388f2b9e8c6c6dacde7f7d770d921754dcc1ea95cf1d3`;
- serial output: `bcc4d933170d3820b0aad9cfea67027274f42b3b0453e4d2423559c160f06aaf`;
- evidence: `7219e49e26d294ab86a10678c255a333032357650ffcf2cca881a06201c49bdd`.

## Risk and remaining boundary

Risk is high because an incorrect root identity, missing synchronization,
unmapped continuation, hardware-mutated hierarchy, or premature teardown can
lose kernel execution or clear an active hierarchy. Review covered checked
root identities and arithmetic, exact page-table bytes, x86 accessed/dirty
behavior, provider placement, kernel/stack continuation, register-held affine
state, failure edges, negative source tests, structural machine-code
inspection, and duplicate physical execution.

Qualification first exposed two real assumptions. Construction resources in
the static image's 4–6 MiB virtual interval cannot be re-read safely until
bootstrap identity translation is restored, so restore now performs only root
checks before the switch and validates the resources immediately afterward.
Real x86 page walks also set accessed/dirty bits; the provider now initializes
those bits to their qualified final values and exact post-restoration readback
proves that activation changed no hierarchy byte.

This increment deliberately restores and destroys the address space after its
kernel-only witness. It does not retain it as a process address space, publish
the static entry point, allocate a process stack, initialize TLS, enter user
mode through the static image, or run BusyBox.
