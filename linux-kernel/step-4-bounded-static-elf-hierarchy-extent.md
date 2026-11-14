# Step 4 bounded static ELF hierarchy extent

## Outcome

The qualified x86-64 path now reserves one explicit four-frame extent for the
static image's future upper translation hierarchy while the exact sealed image
remains live. Checked source uses the typed
`static image construction reserve hierarchy` operation, publishes
`TOPAL_KERNEL_STATIC_ELF_HIERARCHY_EXTENT_OK`, and releases the complete extent
before the existing leaf-table then payload revocation sequence.

The provider selects the 16 KiB extent after the live leaf table, wholly inside
one normalized E820 RAM run, below the first-GiB bootstrap ceiling, and
disjoint from the 503-frame payload and complete retained boot resource. Root
lowering preserves the payload and leaf-table identities across allocation,
retains only the opaque hierarchy identity while it is live, restores the leaf
owner on whole release, and grants no byte or mapping access.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-HIERARCHY-EXTENT-001` adds no core-language
element. Source negative tests reject the wrong owner, count, alignment,
witness, or witness/release order. The semantic model admits the four-frame
request only from the sealed construction state, requires whole release back
to that exact state, and rejects revocation or disposition while the hierarchy
is live. Structural inspection proves the exact request, post-leaf allocation
floor, preserved leaf identity, whole-release restoration, and provider
admission of only the bounded count.

Artifact `/117`, provider and provider-object `/108`, root object `/106`, boot
adapter `/69`, boot-resource `/2`, and gate `/119` identify this executable
contract. The checked root contains 152 fatal edges, a 339-event semantic
trace, 3,488 typed relocations, 3,088 UART calls, and 93 ordered observations.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions reached the
hierarchy-extent witness in order, completed all 93 observations, remained in
the terminal fatal halt, and produced identical artifact and serial hashes.
Recorded hashes are:

- static user image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `40bc1039f75a1d21b3e748fb9f04c377ab12592867127084120d4f55d6947764`;
- boot image: `666acffbf6a3952115269fad44a4de4d283e750980744203345baf3ef5380ba0`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `2a6681187fe2f2a0d59f8338776b794db95617c80aef1ac3770cedb171da008b`;
- boot provenance: `d6e3e55ea00b8b3404946d0bea7b0c38198a1aecf8a43f8431b7562945660457`;
- serial output: `930b471566b6419a522d19eebc57179f0db2d9572957939e85bd7dbed5cd08a5`;
- evidence: `ab6ffde059b288f2c35de07bc95479a258ab02f87bb959f644b0652bb2fd3eb0`.

## Risk and remaining boundary

Risk is high because an overlap or ownership error could alias executable
payload, corrupt the leaf table, or return an extent while provider access
remains live. Review covered checked byte arithmetic, placement and exclusion,
register-held identities, exact state transitions, whole release, failure
edges, structural machine-code inspection, and duplicate physical execution.

This increment deliberately returns the hierarchy extent after its witness.
It does not initialize translation metadata, link the sealed leaf table,
attach or activate an address space, construct TLS or a process stack, publish
the static entry, or execute the image. The next increment can consume this
qualified extent and sealed construction to build and verify an inactive
address space while retaining a complete detach rollback.
