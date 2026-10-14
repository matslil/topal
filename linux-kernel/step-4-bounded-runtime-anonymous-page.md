# Step 4 bounded runtime anonymous page

## Outcome

The initial process now implements one bounded Linux-compatible anonymous
`mmap`/`munmap` page. Topal source allocates the physical frame, transfers it
through an affine `user address space reserve anonymous` operation while its
provider-selected PTE remains absent, and receives that exact frame back only
through `user address space release anonymous` after user execution completes.

The x86-64 adapter accepts only address zero, lengths from one through 4096,
`PROT_READ | PROT_WRITE`, `MAP_PRIVATE | MAP_ANONYMOUS`, descriptor `-1`, and
offset zero. It zeroes the complete frame before every RW/NX publication,
rejects a second live mapping with `ENOMEM`, revokes the exact page through
`munmap`, and also revokes it during terminal cleanup. The constructed
application proves initial zero, mutation, active-map rejection, unmap,
zero-filled remap, and final unmap. No core-language rule changes.

## Validation boundary

Rule `TOPAL-SYSTEMS-USER-ANON-PAGE-001` requires source-owned allocation,
non-present reservation, complete zeroing before publication, Linux-compatible
argument validation, one live generation, exact revocation and invalidation,
terminal cleanup, repeated-map zeroing, and affine frame return after completed
execution. Structural tests prove the source lifecycle, provider-selected PTE,
reservation states, mmap/munmap dispatch, RW/NX publication, complete zeroing,
active-map `ENOMEM`, two successful map/unmap generations, terminal revocation,
and release-time absence checks.

Artifact schema `/64`, provider `/61`, provider object `/61`, artifact root
`/62`, boot adapter `/24`, and gate `/66` identify the changed contract. The
boot-resource schema remains `/1`. The artifact trace contains 202 semantic
events, 65 placements, and 1,827 relocation applications; the generated root
contains 1,633 source console-byte writes.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 51 ordered observations. Both mmap generations and their
intervening state and zero-fill checks complete before terminal cleanup. The
evidence records:

- linked kernel: `e0ea4bf211f7358829ebbcf8e3e12046227674e9ec3d9d63679e6acf3aa11e9f`;
- boot image: `6ec01662faec4c384b0ff4dfce9069ab2d8e7e53c17bfff7fc547830a23baeff`;
- boot resource: `64b222f8013d6a8c44818f24e786555f5bbff46cbb15a8eb4730beadffd82273`;
- generated default image: `aed63c92871ca2014469a16fceaa37901b1a8dccc9c005b826558877c8466faa`;
- supplied image: `1f52d597a3140890f183a5573aa21a28c7bc65a4619847d36e176b0d515b23a4`;
- serial input: `3316348dbadfb7b11c7c2ea235949419e23f9fa898ad2c198f999617912a9925`;
- artifact provenance: `cc8104d176d017ffcbbd0df99d6e6594037d6b5fb23b061126f9b5ac2bc69b13`;
- boot provenance: `e1932f692dbc5fe79a49e17d6a314fe90e8b50a0878a14c610f90f5575b11a15`;
- serial output: `af426f6546f91c1f48d2b81d20c06bbf5de67aa7794bf51553738454320043ba`;
- evidence record: `48a18de4d1457b7fa2ad65acd5c5e449cf7c0d9167b7726acd4d0f15cc4be3e2`.

## Risk and review

Risk is high because this adds runtime page-table mutation, reusable memory,
user-visible addresses, and terminal cleanup. Review covered affine ownership,
non-present reservation, exact argument and state checks, full-frame zeroing,
publish and revoke ordering, NX retention, invalidation, repeated-map behavior,
terminal revocation, release-time absence, structural inspection, full
workspace validation, and two independent physical runs.

## Remaining boundary

There is no general virtual-address selection, multi-page mapping, file-backed
mapping, additional protections or flags, partial unmap, `mprotect`, shared
memory, overcommit, page faults, copy-on-write, fork, multiple processes, or
general VM subsystem.
