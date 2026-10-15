# Step 4 bounded runtime anonymous-page advice

## Outcome

The initial process now implements bounded Linux-compatible
`madvise(MADV_DONTNEED)` for its single runtime anonymous-page generation. The
x86-64 provider accepts only the exact selected address and a nonzero length
through 4096 bytes, then eagerly zeroes the complete backing frame through
kernel authority.

Advice preserves the active generation and its current inaccessible, R/NX, or
RW/NX protection. The constructed application applies advice while the page is
inaccessible, proves that overlapping `mmap` is still rejected, restores RW/NX,
observes zero, mutates the page again, and proves rejected advice preserves the
new byte. No core-language rule or source-visible target mechanism was added.

## Validation boundary

Rule `TOPAL-SYSTEMS-USER-ANON-ADVISE-001` requires exact range and advice
validation, complete eager zeroing, provider-authorized backing access,
generation and protection preservation, operation while inaccessible, and
rejection without mutation. Structural tests prove syscall dispatch, private
state bounds, the complete 4096-byte zero loop, two constructed calls, restored
zero observation, and continued `mprotect`/`munmap` behavior.

Artifact schema `/66`, provider `/63`, provider object `/63`, artifact root
`/64`, boot adapter `/26`, and gate `/68` identify the changed contract. The
boot-resource schema remains `/1`. Focused language, semantics, compiler, and
gate-fixture tests pass, as does the complete workspace suite.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 51 ordered observations. The evidence records:

- linked kernel: `ad03308e9c1b869d5ab0d991aa11d66aa05243023efea27c5a50bb62a43a1c5f`;
- boot image: `fdff8e7186307d7abb5ced374408967afe8695e75e617de742581fdb23f8284a`;
- boot resource: `ca76c935bfa59f124c947ed623aa17868a021088615c928bf6b2e6f52e60257e`;
- generated default image: `f3619e75581144a40d1584d53e1f0f21bc87dcf45e1952cf6649b8a5afc97ee3`;
- supplied image: `76f2d206f816bbf69750abe2201dd9a9e5c3f77598b42e428beac4dbd7a2946e`;
- serial input: `3316348dbadfb7b11c7c2ea235949419e23f9fa898ad2c198f999617912a9925`;
- artifact provenance: `0338bd020e8c4f49a625f97fdfad65a05ee503e6c838676db71546a16917027f`;
- boot provenance: `c566941957cae0de452d864ef6fd474313003476ce6c341adfd376aa564981f8`;
- serial output: `af426f6546f91c1f48d2b81d20c06bbf5de67aa7794bf51553738454320043ba`;
- evidence record: `52a8d3e21f38ae183d316500e821a5f1d1fbce1d07af1c35def3e13ddd95e8d3`.

## Risk and review

Risk is high because discard mutates live user backing even when its user PTE
is absent. Review covered validate-before-mutate ordering, complete-frame
zeroing, private state bounds, inaccessible-state operation, generation and
protection preservation, rejection without byte changes, later access,
structural inspection, full workspace validation, and two physical runs.

## Remaining boundary

There is no other advice class, partial-range behavior, lazy fault-backed
discard, multi-page mapping, caller-selected address, file backing, reclaim
pressure, sharing, overcommit, copy-on-write, fork, multiple processes,
concurrent VM edits, or general VMA subsystem.
