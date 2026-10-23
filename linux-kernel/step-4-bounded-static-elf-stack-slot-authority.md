# Step 4 bounded static ELF stack-slot authority

## Outcome

The qualified x86-64 path now derives one opaque affine stack-slot authority
from the entry-authorized static ELF construction, publishes
`TOPAL_KERNEL_STATIC_ELF_STACK_SLOT_AUTHORITY_OK`, revokes the slot authority,
and publishes `TOPAL_KERNEL_STATIC_ELF_STACK_SLOT_AUTHORITY_REVOKED_OK` before
entry revocation and teardown. Checked source receives neither a numeric stack
address nor a page-table index.

The provider revalidates the inactive static root, active bootstrap root,
complete hierarchy, all 503 final image leaves, executable entry authority,
and all nine absent leaves. It then specifically proves absent guard ordinal
510, absent slot ordinal 511, and the provider-private aligned initial top
`0x00600000`. Authorization and revocation are identical read-only checks;
neither installs backing, changes `CR3`, mutates a page-table byte, or enters
the image.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-STACK-SLOT-001` adds no core-language element.
Source-negative tests reject substituted owners, policies, markers, ordering,
entry revocation before slot revocation, and slot use outside the bounded
lifecycle. The semantic model enforces entry-authorized to slot-authorized to
entry-authorized ownership before the existing entry-revocation transition.

Structural provider tests prove identical authorization and revocation,
retained entry verification, exact absent slot and guard displacements, the
aligned private initial top, three `CR3` observations, no `CR3` write, and no
hierarchy mutation. Artifact `/121`, provider and provider-object `/112`, root
object `/110`, boot adapter `/69`, boot-resource `/2`, and gate `/123` identify
this executable contract. The checked root contains 160 fatal edges, a
356-event semantic trace, 3,883 typed relocations, 3,467 UART calls, and 101
ordered observations.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions derived and
revoked the slot authority, reached both new witnesses in order, completed all
101 observations, remained in the terminal fatal halt, and produced identical
artifact and serial hashes. Recorded hashes are:

- static user image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `1e12fa967d69d801626598705df5f3eb87904b362cc4a89dc8c0883ee61d436e`;
- boot image: `bfb7ff1f0c45ad7fc9ead9dd58949abe438f708c544cb8ffdf73180588d6ccc0`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `8e3e75c80ab7490c2f8ca20b14a63a13495be26d8ade2b32290c71a9c9997b84`;
- boot provenance: `739f1aa7b04cb6f8d623aea77bd8484049f1537bf70d15fa6cd758fa7f9dc8fe`;
- serial output: `3beb9f0be1d0af006eb5413881c805c9852aa2083b5effbf000d02be4d4e4e7f`;
- evidence: `edfd57cf20da16b39a3dc86fb6457e849b37073a47d121d1d5624ca2da404baa`.

## Risk and remaining boundary

Risk is high because selecting an overlapping or unguarded slot would make a
later writable stack capable of corrupting the loaded image. Review covered
exact leaf arithmetic, image/slot/guard disjointness, entry and root identity,
target alignment, affine transitions, failure edges, negative source tests,
structural machine-code inspection, and duplicate physical execution.

This increment deliberately revokes the slot authority before entry authority.
It does not allocate a stack frame, install a stack leaf, initialize
`argc`/`argv`/`envp` or auxiliary-vector contents, initialize TLS, construct a
user context, enter ring 3, or execute BusyBox.
