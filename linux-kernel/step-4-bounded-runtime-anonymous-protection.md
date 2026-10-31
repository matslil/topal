# Step 4 bounded runtime anonymous-page protection

## Outcome

The initial process now implements bounded Linux-compatible `mprotect` for its
single runtime anonymous page. The x86-64 provider accepts the exact selected
address, a nonzero length through 4096 bytes, and `PROT_NONE`, `PROT_READ`, or
`PROT_READ | PROT_WRITE`. Every successful state remains non-executable and
completes page-table replacement and target invalidation before user return.

The provider retains one active generation across inaccessible, R/NX, and
RW/NX states. `PROT_NONE` installs a non-present PTE without releasing the
reservation, so another `mmap` still returns `-ENOMEM`; restoring access
preserves the backing bytes. `munmap` and terminal cleanup revoke every active
protection state. No core-language rule or source-visible target mechanism was
added.

## Validation boundary

Rule `TOPAL-SYSTEMS-USER-ANON-PROTECT-001` requires exact range and protection
validation, preserved generation identity and contents, NX in all accessible
states, a restorable inaccessible state, ordered target completion, rejection
without state change, and revocation from every protection. The constructed
application mutates an RW page, reads it after R/NX transition, makes it
inaccessible, proves overlapping `mmap` rejection, restores RW/NX and observes
the retained byte, rejects executable protection while retaining state, and
then completes the existing unmap/remap zero-fill sequence.

Artifact schema `/65`, provider `/62`, provider object `/62`, artifact root
`/63`, boot adapter `/25`, and gate `/67` identify the changed contract. The
boot-resource schema remains `/1`. Focused language, semantics, compiler, and
gate-fixture tests pass, as does the complete workspace suite.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 51 ordered observations. The evidence records:

- linked kernel: `328aa68677db5fe881b928995c8c6eceb0f06288007a2b5e7d205d983c8f1234`;
- boot image: `320b27811a9d713c9929e00afd90d9eb3327838f9ac22a3e375f51911b0e0b5e`;
- boot resource: `c251a5c3f6db4d8b106b17b2c28d7b50cce0241529af8bdffd92723082f41675`;
- generated default image: `8fc6e439859bb03b544b0178db650376d5ee77f696cb14e7d5ab327192925bc7`;
- supplied image: `fb610856550f86cfc7cc7d6f485c50c14f4616b2cfdf199fd02d94490ea623bf`;
- serial input: `3316348dbadfb7b11c7c2ea235949419e23f9fa898ad2c198f999617912a9925`;
- artifact provenance: `5bd92c6c3bd7d9fc7a06187cb62dfadbc3e97d581f1ce6113bfcedde31aa709f`;
- boot provenance: `e73f32ad087ac56bd92e285b447b939deb43d4566db9d2ff03f306bce0410f0a`;
- serial output: `af426f6546f91c1f48d2b81d20c06bbf5de67aa7794bf51553738454320043ba`;
- evidence record: `05d4750b241d849a0d15fde082c0c59a01f0b47fbbb6a8288eedd3b2c2cacb57`.

## Risk and review

Risk is high because protection changes alter live user authorization and must
remain coherent with syscall state, later unmap, and terminal cleanup. Review
covered validate-before-mutate ordering, exact state bounds, PTE permission
bits, W^X, `PROT_NONE` generation retention, content preservation, rejected
executable protection, invalidation ordering, cleanup from all live states,
structural inspection, full workspace validation, and two physical runs.

## Remaining boundary

There is no executable protection, multi-page mapping, caller-selected address,
partial range splitting, file-backed mapping, protection key, `mremap`,
`madvise`, shared memory, overcommit, page-fault recovery, copy-on-write, fork,
multiple processes, concurrent VM edits, or general VMA subsystem.
