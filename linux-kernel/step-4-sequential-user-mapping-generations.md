# Step 4 sequential user-mapping generations increment

## Outcome

The bounded affine `probe-guard` lifecycle is now reusable for exactly two
explicit sequential generations. The same returned frame is consumed by a
fresh second mapping, and source receives it again only after the second
mapping is verified, revoked, and translation completion finishes.

This is a systems-feature refinement only. It changes no grammar, address or
pointer conversion, source-visible page-table operation, instruction name, or
core-language rule. Mapping identities, descriptor representation, placement,
and target completion remain opaque.

## Checked source and semantic model

The checked root performs map, store, load, and unmap twice before releasing
the frame. Generation two must consume the exact extent returned by generation
one. Negative cases reject another backing identity, a first-generation
mapping refinement in the second generation, a mismatched second unmap, early
release, missing transitions, and incorrect marker ordering.

The architecture-independent model assigns fresh mapping identities 1 and 2.
Its bootstrap authority state distinguishes prepared, mapped-once, and
verified-after-two states, so generation two cannot start before the first
unmap and a third generation cannot start after the second. The generic
protocol independently proves monotonically fresh identities and exact frame
return for both lifecycles.

## Initial x86-64 lowering

Provider, provider-object, root-object, and artifact revisions advance to
`/25`. The four generated mapping functions accept only the next legal
generation state. The first lifecycle progresses through states 4, 5, 6, and
7; the second through 8, 9, 10, and 11. Both publications require an empty
guard leaf and private mapping-frame slot. Both unmaps clear the leaf, complete
invalidation, clear the retained frame, and return that exact frame.

The generated root calls each mapping operation twice and prepares user
execution only after state 11 and final frame release. Structural inspection
requires both state sequences, both provider-call sites, dynamic generation
state publication, failure cleanup, and the existing disjoint state fields at
offsets `0x68`, `0x70`, and `0x78`.

Artifact revision `topal.systems-artifact.x86_64-qemu-pc-q35/25` retains 58
planned provider operations and 59 inspected placements while recording a
170-transition semantic trace. Root-object revision
`topal.systems-root-object.x86_64/25` has 1,046 typed relocations. Boot-adapter
revision `topal.boot-adapter.linux-x86-protocol-2.15-q35/6` is unchanged.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` builds and executions were
byte-identical. Gate schema `topal-kernel-toolchain-gate-qemu/27` records 28
ordered markers. `TOPAL_KERNEL_USER_MAPPING_OK` is emitted only after both
generations and final release. Later intentional guard-crossing user-copy
faults prove that neither generation left stale access. The evidence binds:

- linked kernel:
  `a6a66163a60c118c2382557fdd5fa9098d8c6cea49b8029df73360abed13d6d3`;
- boot image:
  `f5b33bf47f42ccb350067a72bdf9d83f6518635e72dad0cce38f433a9767de04`;
- artifact provenance:
  `e880cd649df2c49f81e2c293a3965b703bd5b0e418752c6f94a702fc881aace3`;
- boot provenance:
  `c36c16d2e0a3c7382af0b07376a46c3ff7a7fa23c1ed7322ad801ab2efb3dc2d`;
- serial observation:
  `968b9aaafe358dde292b500a8d4b997bcf274d212e77a1618638003932f3b0dc`;
  and
- complete evidence record:
  `bba9a19247851e068ff14e31d867ed230633bfbe2d11bb8a73aa92329c28ece4`.

QEMU remained live in the final fatal disposition and its serial output stayed
quiescent after all markers.

## Validation and review

This is high-risk mapping reuse, revocation, and affine-identity work. Review
covered authority-order conformance, systems-only scope, fresh identity,
cross-generation exclusion, exact frame return, provider-state reset,
publication and revocation ordering, failure cleanup, structural instruction
inspection, negative tests, two physical executions, and the complete
repository test surface. Final validation uses:

```console
scripts/run_bounded.py -- cargo fmt --all -- --check
scripts/run_bounded.py -- cargo check --workspace --all-targets
scripts/run_bounded.py -- cargo test --workspace -q -j1
scripts/run_bounded.py -- python3 \
  linux-kernel/labs/qemu/x86_64/topal_gate.py --write-evidence
scripts/run_bounded.py -- python3 -m unittest \
  linux-kernel/labs/qemu/x86_64/test_topal_gate.py -v
git -c core.whitespace=trailing-space,space-before-tab diff --check
```

## Remaining boundary

The profile still admits one frame, one sealed guard layout, fixed RW/NX
rights, a fixed byte, and no more than one live mapping. It does not admit
simultaneous mappings, aliases, arbitrary layout or permissions, persistent
image mappings, replacement, partial unmap, demand paging, copy-on-write, or
an ELF loader. Those remain later systems contracts.
