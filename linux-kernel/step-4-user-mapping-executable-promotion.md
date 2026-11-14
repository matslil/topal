# Step 4 user-mapping executable-promotion increment

## Outcome

Both bounded mapping generations now promote verified RO/NX contents to RO/X
before unmap. The transition consumes the exact verified refinement, never
restores write authority, completes target translation and instruction-
observation requirements, and returns executable affine ownership.

This is a systems-feature refinement only. It changes no core-language rule
and exposes no page-table bit, address, cache operation, barrier, or
invalidation instruction.

## Model and x86-64 lowering

The checked sequence is now map, store, protect, load, permit execute, and
unmap for both generations. The architecture-independent model distinguishes
executable ownership and rejects writable, unverified, stale, skipped, or
duplicate promotion.

Provider revision `/27` accepts only verified states 6 and 10, proves the leaf
is read-only and NX, clears NX, fences publication, invalidates the sealed
guard address, and publishes executable states 14 and 15. Unmap accepts only
those states. Structural tests inspect the NX test/clear, fence, invalidation,
states, typed call sites, and cleanup boundary.

Artifact revision `topal.systems-artifact.x86_64-qemu-pc-q35/27` records 60
planned operations, 61 placements, and a 174-transition trace. Root revision
`topal.systems-root-object.x86_64/27` has 1,058 typed relocations. The boot
adapter remains `/6`.

## Physical evidence

Two pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical. Gate
schema `/29` records all 28 markers, live fatal halt, and serial quiescence.

- linked kernel: `4dbca84efd0a6b4228a6cce2950b965077eed95d87ef43b0b18d4c9d3d99188e`;
- boot image: `590157b016bf59042e97a8552fa28bc4d31bb0981c5eb90806cd88c175829c0a`;
- artifact provenance: `08416054b9db714dd7d90a562753e5400e341e98f812a73b458774a37c9ed16a`;
- boot provenance: `3211199ad114561b66e0acacafa0d2c66aa23de671de63fe4da043fab03522bf`;
- serial: `968b9aaafe358dde292b500a8d4b997bcf274d212e77a1618638003932f3b0dc`;
- evidence record: `d57b31806a7ca3624641742c562f0396aa90dcbeebddd0cb28a945f2d4f5d1cf`.

## Remaining boundary

The executable mapping remains a sealed qualification and is not invoked as
application code. Arbitrary rights, writable restoration, simultaneous or
persistent mappings, aliases, image layouts, and ELF loading remain excluded.
