# Step 4 sealed user-mapping role increment

## Outcome

The two bounded mapping generations now finish in distinct affine image roles.
Generation one becomes an `ExecutableCodeMapping` with RO/X authority;
generation two becomes an `ImmutableDataMapping` that retains RO/NX authority.
Neither final refinement can substitute for the other, and unmap consumes the
exact role-specific owner.

This is a systems-feature refinement only. It changes no core-language rule
and exposes no page-table bit, address, cache operation, barrier, or
invalidation instruction.

## Model and x86-64 lowering

The checked first-generation sequence remains map, store, protect, load,
permit execute, and unmap. The second-generation sequence is map, store,
protect, load, seal immutable data, and unmap. The architecture-independent
model rejects execute promotion for generation two and immutable-data sealing
for generation one.

Provider revision `/28` accepts execute promotion only from verified state 6,
proves RO/NX, clears NX, completes publication and target invalidation, and
publishes executable state 14. Its separate immutable-data operation accepts
only verified state 10, proves writable is clear and NX remains set without
changing the leaf, and publishes state 16. Unmap accepts only states 14 and 16.
Structural tests inspect the role-specific input states, writable and NX
checks, execute-only NX clear and invalidation, final states, typed call sites,
and cleanup boundary.

Artifact revision `topal.systems-artifact.x86_64-qemu-pc-q35/28` records 61
planned operations, 62 placements, and a 174-transition trace. Root revision
`topal.systems-root-object.x86_64/28` retains 1,058 typed relocations. The boot
adapter remains `/6`.

## Physical evidence

Two pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical. Gate
schema `/30` records all 28 markers, live fatal halt, and serial quiescence.

- linked kernel: `e61443dc78089c1163f42491181bb26b4832c8331a3db9ac6ebe73259227669b`;
- boot image: `1899ad1460c5fbe4848bb82898847ced4a915fc77e8fc38a0641c6d8b7e90e55`;
- artifact provenance: `bd9dca849690d8f9f05feac4168563c57d25616e5c0ff8da4b24719ccf1737db`;
- boot provenance: `3b995302f32f9007a9aaa59ad5d45505e63bd3ea37d77042301f6e6fdeda9d84`;
- serial: `968b9aaafe358dde292b500a8d4b997bcf274d212e77a1618638003932f3b0dc`;
- evidence record: `f202547840fa12f828a699535905591b8772c652bb26e77e4b05c490957bc4e8`.

## Remaining boundary

Both mappings remain qualification-only and are unmapped before user entry.
Persistent simultaneous code and data mappings, arbitrary image layouts,
aliases, relocation, ELF loading, writable restoration, and application-code
execution remain excluded.
