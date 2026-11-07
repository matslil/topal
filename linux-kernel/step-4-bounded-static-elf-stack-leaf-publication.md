# Step 4 bounded static ELF stack-leaf publication

## Outcome

The qualified x86-64 path now publishes the completely zeroed static ELF stack
backing into the reserved inactive user leaf, proves the exact read/write and
non-executable permissions, revokes the leaf, and only then releases the
backing. Checked source selects opaque owner and backing identities plus the
approved access and policy; it receives no virtual address, physical address,
page-table identity, leaf ordinal, or machine flag representation.

The provider writes only inactive leaf ordinal 511, retains ordinal 510 as the
guard, preserves all 503 image leaves and the seven other absent leaves,
validates the complete leaf value, and issues the required ordering fence. It
then erases and validates the same leaf before allowing whole-backing release.
The current bootstrap root and the inactive hierarchy remain distinct
throughout the operation.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-STACK-LEAF-001` adds no core-language element.
Source-negative tests reject substituted owners or backing, non-writable or
executable access, wrong policy or witnesses, early release, duplicate
publication, and out-of-order revocation. The semantic model enforces the
zeroed-backing to published-leaf to zeroed-backing ownership cycle.

Structural provider tests prove the exact ordinal write and erase, exact
read/write and non-executable user-leaf value, complete backing-zero checks,
preserved guard and spare leaves, ordering fence, and absence of active-address
invalidation. Artifact `/124`, provider and provider-object `/114`, root object
`/113`, boot adapter `/69`, boot-resource `/2`, and gate `/126` identify this
executable contract. The checked root contains 164 fatal edges, a 365-event
semantic trace, 4,077 typed relocations, 3,653 UART calls, and 105 ordered
observations.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions published and
revoked the inactive stack leaf, reached both new witnesses in order, completed
all 105 observations, remained in the terminal fatal halt, and produced
identical artifact and serial hashes. Recorded hashes are:

- static user image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `08be7a26bd461265324df71ff9004f9e3eed7af38a325829604d2f27087e647e`;
- boot image: `4a66efc049ffbbf5bcf1796cd703128a4c41220b8a20bb0e2650f2fe0c1c5f08`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `bbf0d03dd3ce3092d120c13f5501365aab28d4a45f5bc9a733b273b1cdf16a92`;
- boot provenance: `7cba181bb645ac52ec20a02b3ae50ecdb5b5467252ee09b12528034f1c37103f`;
- serial output: `256b73d788cc775512d772b450f5d8a6627310e2646b154a1fe76b9fa7440522`;
- evidence: `38b2e78e89441c1ab33f0b683e93d1045bfc93b09443c215f1e0bbacfc985924`.

## Risk and remaining boundary

Risk is high because an executable or stale leaf, a missing guard, publication
before zeroing, or release before revocation could violate userspace isolation.
Review covered exact permissions and ordinal, hierarchy and bootstrap-root
separation, zero preservation, guard and image-leaf preservation, affine
ordering, failure edges, negative source tests, structural machine-code
inspection, and duplicate physical execution.

This increment revokes the qualified mapping before returning. It does not
initialize `argc`/`argv`/`envp` or auxiliary-vector contents, initialize TLS,
construct a user stack pointer or context, enter ring 3, or execute BusyBox.
