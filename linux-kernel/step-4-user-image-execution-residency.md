# Step 4 user-image execution-residency increment

## Outcome

The finalized bounded code and data mappings now remain resident across the
existing sealed ring-3 qualification probe. After the probe completes its
Linux syscall and contained fault sequence and returns to the exact kernel
continuation, source publishes `TOPAL_KERNEL_USER_IMAGE_RETAINED`, tears down
data before code, and only then completes and reclaims the address space.

This is a systems-feature refinement only. It changes no core-language rule
and does not execute the constructed image.

## Model and x86-64 lowering

The checked source and architecture-independent model preserve both affine
role owners across probe preparation, entry, syscall dispatch, user-copy fault
recovery, exit, and return. They reject early teardown, a missing or early
post-return witness, code-first teardown, and address-space completion while
either mapping remains live.

Provider and provider-object revisions `/30` require the finalized mapping
state at probe preparation and keep it intact until post-return teardown. The
private code and data placements move to `0x604000` and `0x605000`, preserving
the probe's deliberate unmapped `0x602000` fault guard. Artifact revision
`topal.systems-artifact.x86_64-qemu-pc-q35/31` records 61 provider operations,
62 placements, and a 178-transition trace. Root revision
`topal.systems-root-object.x86_64/31` has 1,122 typed relocations, including
936 UART writes. The boot adapter remains `/6`.

## Physical evidence

Two pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical. Gate
schema `/33` records all 30 ordered markers, the unchanged Linux `-EFAULT`
probe outcomes, live fatal halt, and serial quiescence.

- linked kernel: `37ee53737aae9efc4e58d98869b00d2ce3bab11ec25d67d7a2dd2eab2d2f80ed`;
- boot image: `e34335b53fe0e6fb5d75a71a0bded5819a2c9d3d398bb4261068fc77b344aa6e`;
- artifact provenance: `72a5e62450c322361073b9597a4439eba8cb097350accbbb1ef1a3ebea3eeceb`;
- boot provenance: `64fd535fb0e71756f9afb9afa6df8185d59309018b2ce6e509402295cb4451a1`;
- serial: `84ff8c1e38a2b4b4441bd0220f3e8e2113d7250b1ce96a4173dd110ea4448431`;
- evidence record: `cd2570c282537701b565d6453544c54b55a288f9d1b07e4dc91359478d4f4530`.

## Remaining boundary

The sealed probe does not fetch from or access the constructed mappings.
Selecting an entry, executing constructed code, transferring image ownership,
arbitrary segment collections, aliases, relocation, ELF parsing, and demand
paging remain excluded.
