# Step 4 resident user-image mapping increment

## Outcome

The bounded code and data mappings now coexist during image construction.
Source retains generation-one `ExecutableCodeMapping` ownership while it
allocates a distinct frame and constructs generation-two
`ImmutableDataMapping`, then revokes and releases data before code.

This is a systems-feature refinement only. It changes no core-language rule
and exposes no address, page-table bit, descriptor, or target instruction.

## Model and x86-64 lowering

The checked source uses sealed `probe-code` and `probe-data` placements and two
separate one-frame allocations. The architecture-independent model preserves
both affine role owners concurrently, rejects code-first teardown and user
preparation with either mapping live, and returns each exact backing extent.

Provider revision `/29` uses separate private leaf and retained-frame slots at
sealed x86-64 user locations. The second allocation starts beyond resident
code while still excluding the three-frame address-space backing. Publication
proves all three extents are disjoint. Code follows RW/NX to RO/NX to RO/X;
data follows RW/NX to RO/NX and remains NX. Data and code receive separate
invalidation and frame-return completion.

Artifact revision `topal.systems-artifact.x86_64-qemu-pc-q35/29` records 61
planned provider operations, 62 placements, and a 176-transition trace. Root
revision `topal.systems-root-object.x86_64/29` has 1,061 typed relocations. The
boot adapter remains `/6`.

## Physical evidence

Two pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical. Gate
schema `/31` records all 28 markers, live fatal halt, and serial quiescence.

- linked kernel: `af923c073f2a542ccc97611a8e1c6dc5358e03377daa8f975916efd22ac88cdc`;
- boot image: `a9035d46fb471bdfb896d51801dd88ecfb49731a1e46c9129c169a63de4e0da6`;
- artifact provenance: `704c3211f7ea379ef100a3599bdb3e466e70777e11b18b69035f348ce9bddf12`;
- boot provenance: `784e94e174db164cb6428785f99ab652492873dca921b85134bec523a10e37d1`;
- serial: `968b9aaafe358dde292b500a8d4b997bcf274d212e77a1618638003932f3b0dc`;
- evidence record: `2dc65bde14d49b398d9b677e6b83cd509cb3ba35a7f98555d98e16994171c143`.

## Remaining boundary

Both construction mappings are still revoked before user entry. Retaining an
image into execution, invoking loaded code, arbitrary segment collections,
aliases, relocation, ELF parsing, and demand paging remain excluded.
