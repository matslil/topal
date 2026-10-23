# Step 4 bounded static ELF final permissions

## Outcome

The qualified x86-64 path now consumes the populated 503-frame static-image
construction and seals its inactive leaf table with the approved W^X and RELRO
plan. Checked source names only the opaque populated owner and
`bounded-static-elf-wx-relro` plan. Success yields one affine sealed owner and
publishes `TOPAL_KERNEL_STATIC_ELF_PERMISSIONS_OK` before the existing complete
revocation and return protocol.

The provider revalidates both extents, every initial leaf, the retained boot
resource, all approved ELF metadata, all four copied intervals, and all four
preserved-zero intervals. It then publishes and reads back the exact ordered
roles: one header page RO/NX, 396 executable pages RO/X, 96 immutable and RELRO
pages RO/NX, and ten writable/BSS pages RW/NX. The nine tail entries remain
absent. Revocation independently authenticates those sealed roles before it
clears and verifies the complete 512-entry table.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-PERMISSIONS-001` adds no core-language element.
Source negative tests reject the wrong owner, plan, witness, or revocation
state. The semantic model rejects sealing before population, duplicate or
out-of-order ownership, and disposition with a live construction. Provider
inspection proves four complete content comparisons, four zero scans, the
four exact permission runs in both publication and readback, one publication
barrier, unchanged absent entries, complete metadata comparison, and sealed
role authentication before revocation.

Artifact `/116`, provider and provider-object `/107`, root object `/105`, boot
adapter `/69`, boot-resource `/2`, and gate `/118` identify this executable
contract. The checked root contains 151 fatal edges, a 336-event semantic
trace, 3,443 typed relocations, 3,045 UART calls, and 92 ordered observations.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions reached the
permission witness in order, completed all 92 observations, remained in the
terminal fatal halt, and produced identical artifact and serial hashes.
Recorded hashes are:

- static user image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `26f768b53cbd20193193de8ff89f9c91d688a535db90a5e97cdf7f3a40defb4c`;
- boot image: `7208bb2a11290602f153a163b052fa9d1fcd27ecfa28c3f1b9b08ce4d58cf4f9`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `4b1bbc09103c26abd2381050d891fba2dec8ca989d7f614084c1fc22281690a4`;
- boot provenance: `d3431508e626dc541743e0f4468e002d6482718f6e2a728ca528928c108d43c5`;
- serial output: `c40cb58c429787483c14224f70cfff31f2de4f4494709e67f4f38a1639b9f980`;
- evidence: `6f1cd78daebca015317833ffc6636eff5541b5293e97f51dae2a42694a63e0d8`.

## Risk and remaining boundary

Risk is high because an incorrect role could create writable executable
memory, leave RELRO mutable, substitute a frame, or make later revocation
accept a different layout. Review covered every interval boundary, exact role
counts and flags, frame-order preservation, complete readback, inactive
publication, sealed-state revocation, failure cleanup, structural machine-code
inspection, and duplicate physical execution.

This increment deliberately revokes and returns the verified payload. It does
not retain the sealed owner, construct upper translation levels, attach an
address space, initialize TLS or a process stack, publish the static entry
point, or execute the image. The next increment must attach the sealed image to
an inactive process address space without weakening these final roles.
